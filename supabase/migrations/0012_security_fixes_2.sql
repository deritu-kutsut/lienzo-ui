-- Lienzo UI · Community. Second round of security fixes from the server audit (migration 0012). Builds on 0011.
--
-- WHAT IT FIXES (numbers match the audit)
--   1. HIGH   Storage paths with encoded dot-segments. 0011 only rejected a literal «..». Now an ALLOWLIST (same rules as _shared/paths.ts):
--             quarantine  <uid>/<item id>/<file>   and   <uid>/perfil/avatar.png | banner.png
--             public      <kind>/<item id>/<file>  and   perfiles/<uid>/<avatar|banner>[-<16 hex>].png
--             <file> = 1-100 chars of [A-Za-z0-9._ -], starting and ending with [A-Za-z0-9_-] (no leading dot, no trailing dot/space, no «..»).
--             Enforced in: legacy.owns_path (upload policy), the quarantine STORAGE insert policy (cuarentena_subir) and a BEFORE trigger on
--             community_items / public_profiles that validates every path column when it is inserted or CHANGED (rows that were already stored
--             keep working: the trigger only looks at what is being written; legacy.bad_storage_paths() lists old rows that do not match).
--   2. MEDIUM Avatar / banner TOCTOU. New server-only table profile_image_pins (sha256 of the scanned bytes) + claim_profile() / release_profile():
--             the `profile` function claims a run (distinct kinds «image» / «save», hourly cap per person) before it downloads or scans anything.
--   3. MEDIUM Takedown. Server-only table storage_cleanup (the queue) + claim_cleanup() / finish_cleanup(). Triggers queue the public files of an
--             item that is deleted by its author, rejected or withdrawn («delete»), or hidden by reports for malware / copyright («to_quarantine»:
--             the function moves the file back to quarantine so a moderator can still re-approve it). The Edge Functions drain the queue.
--   4. MEDIUM Tags: at most 8, each <= 30 chars, <= 200 in total (constraint community_items_tags_shape; existing rows are trimmed first so the
--             constraint can be validated). The «content longer than the review window → human review» rule is in the functions (decide.ts).
--   5. LOW-MEDIUM
--      · public names: profiles.name_reviewed (the last REVIEWED name). Only a reviewed name is copied to community_items.author_name and
--        public_profiles.public_name. Reserved / impersonation names (table reserved_names, server-only and configurable) are refused for people.
--      · upload cap: server-only upload_log, fed by an insert trigger and never deleted by item deletes (retention 7 days) → limit_uploads counts it.
--      · community_items.updated_at (bumped when title / description / tags / author_name change) = optimistic lock for human approvals.
--      · claim lease 10 min (was 5) > the 5 min author edit lock; heartbeat_moderation() extends it while a run is working.
--      · hide_on_reports counts only reports since the last review/approval (reviewed_at).
--      · record_download ignores items that are not public, and 18+ items for people who did not opt in to 18+.
--      · public_profiles pending flags / updated_at: PARTIALLY fixed (see «NOT FIXED» below).
--
-- NOT FIXED HERE (needs a client change)
--   · Other signed-in people can still read avatar_pending / banner_pending / updated_at of a creator through the REAL table public_profiles
--     (client 1.5 reads `public_profiles?select=*` for its own row, and `select=*` fails if any column grant is missing). The legacy view
--     perfiles_publicos already hides them from everybody else. Once the client reads its own row from a view, revoke those columns.
--
-- WHAT STAYS EXACTLY AS IT WAS (old app versions 1.0 – 1.5)
--   · No legacy view, column, grant or function name is changed. `items`, `perfiles`, `corazones`… keep their definitions, the INSTEAD OF triggers
--     still write through the real table so every new trigger / constraint applies to them too. Nothing is granted to anon / authenticated except
--     execute on the three pure path helpers used by policies. New tables and functions are service_role only.
--   · Behaviour changes old clients can notice: a NEW public name appears on the public pages only after it was reviewed (the first upload/profile
--     save triggers the review; until then public pages show the previous reviewed name, or an empty name for a brand-new profile); names that
--     are reserved are refused with «That public name is not available.»; file names with «..», a trailing dot/space or characters outside
--     [A-Za-z0-9._ -] are refused on upload (the client's safe_name() never produces them); more than 8 tags / tags over 30 chars are refused.
--
-- HOW TO APPLY  (order matters)
--   1. Back up (Dashboard → Database → Backups).  `npx supabase db push`  (or paste this file in the SQL editor). One transaction; idempotent.
--   2. THEN deploy the Edge Functions (they call claim_profile, claim_cleanup, review_public_name, heartbeat_moderation… and read updated_at).
--      The functions of 0011 keep working against this schema (they do not know the new objects).
--   3. THEN publish web/moderacion (it sends updated_at with approve; against old functions the extra field is ignored).
--   4. Run `select * from legacy.bad_storage_paths();` (SQL editor, service role) and fix / reject any row it lists.
--   5. Dashboard settings: see supabase/MIGRATION.md → «Dashboard settings to change».
--
-- HOW TO ROLL BACK
--   Drain the queue first (call review `cleanup` until it answers done: 0 – the rollback DROPS storage_cleanup), redeploy the previous Edge
--   Functions, run supabase/rollbacks/0012_security_fixes_2.down.sql, then `supabase migration repair --status reverted 0012`.
--   NOT reversible: the tag trimming of existing rows and the grandfathering of existing public names (both recorded below).

begin;

-- ═════════════════════════ 1. Strict path allowlist ═════════════════════════
-- The UUID and file-name patterns are written out in every function on purpose (immutable, no dependencies, easy to diff against paths.ts).

-- Quarantine path of a listing: NULL, or exactly <author>/<item id>/<file>. (Same name and signature as 0011: the upload policy keeps using it.)
create or replace function legacy.owns_path(p_author uuid, p_id uuid, p_path text) returns boolean language sql immutable as $$
  select p_path is null or (
    char_length(p_path) <= 250
    and position('..' in p_path) = 0
    and p_path ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/[A-Za-z0-9_-]([A-Za-z0-9._ -]{0,98}[A-Za-z0-9_-])?$'
    and starts_with(p_path, p_author::text || '/' || p_id::text || '/')
  )
$$;

-- Public path of a listing: NULL, or <kind>/<item id>/<file>. `kind` is the English directory or the Spanish one that pre-0010 servers wrote.
create or replace function legacy.public_path_ok(p_kind public.item_kind, p_id uuid, p_path text) returns boolean language sql stable as $$
  select p_path is null or (
    char_length(p_path) <= 250
    and position('..' in p_path) = 0
    and p_path ~ '^(plugin|image|font|template|element|theme|imagen|fuente|plantilla|elemento|tema)/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/[A-Za-z0-9_-]([A-Za-z0-9._ -]{0,98}[A-Za-z0-9_-])?$'
    and (starts_with(p_path, p_kind::text || '/' || p_id::text || '/') or starts_with(p_path, legacy.kind_es(p_kind) || '/' || p_id::text || '/'))
  )
$$;

-- Public avatar / banner: NULL, or perfiles/<uid>/<avatar|banner>[-<16 hex>].png
create or replace function legacy.profile_public_path_ok(p_uid uuid, p_path text) returns boolean language sql immutable as $$
  select p_path is null or (
    p_path ~ '^perfiles/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/(avatar|banner)(-[0-9a-f]{16})?\.png$'
    and starts_with(p_path, 'perfiles/' || p_uid::text || '/')
  )
$$;

-- Object name the caller may CREATE in the quarantine bucket: their own <uid>/<item id>/<file> or <uid>/perfil/avatar.png|banner.png.
create or replace function legacy.quarantine_object_ok(p_name text, p_uid uuid) returns boolean language sql immutable as $$
  select p_uid is not null and p_name is not null
    and char_length(p_name) <= 250
    and position('..' in p_name) = 0
    and starts_with(p_name, p_uid::text || '/')
    and (
      p_name ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/[A-Za-z0-9_-]([A-Za-z0-9._ -]{0,98}[A-Za-z0-9_-])?$'
      or p_name ~ '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/perfil/(avatar|banner)\.png$'
    )
$$;

revoke all on function legacy.owns_path(uuid, uuid, text) from public;
revoke all on function legacy.public_path_ok(public.item_kind, uuid, text) from public;
revoke all on function legacy.profile_public_path_ok(uuid, text) from public;
revoke all on function legacy.quarantine_object_ok(text, uuid) from public;
grant execute on function legacy.owns_path(uuid, uuid, text) to anon, authenticated, service_role;
grant execute on function legacy.public_path_ok(public.item_kind, uuid, text) to anon, authenticated, service_role;
grant execute on function legacy.profile_public_path_ok(uuid, text) to anon, authenticated, service_role;
grant execute on function legacy.quarantine_object_ok(text, uuid) to anon, authenticated, service_role;

-- Validating trigger (NOT a CHECK, on purpose): it only inspects a path column when it is inserted or CHANGED, so a legacy row that does not
-- match can still be updated (rejected, hidden, counted…) by the server without blocking. Applies to every role, service_role included.
create or replace function public.enforce_storage_paths() returns trigger language plpgsql set search_path = public as $$
declare bad boolean;
begin
  if tg_op = 'INSERT' then
    bad := not (legacy.owns_path(new.author, new.id, new.quarantine_path) and legacy.owns_path(new.author, new.id, new.quarantine_icon)
                and legacy.owns_path(new.author, new.id, new.quarantine_preview) and legacy.public_path_ok(new.kind, new.id, new.public_path)
                and legacy.public_path_ok(new.kind, new.id, new.public_icon) and legacy.public_path_ok(new.kind, new.id, new.public_preview));
  else
    bad := (new.quarantine_path is distinct from old.quarantine_path and not legacy.owns_path(new.author, new.id, new.quarantine_path))
        or (new.quarantine_icon is distinct from old.quarantine_icon and not legacy.owns_path(new.author, new.id, new.quarantine_icon))
        or (new.quarantine_preview is distinct from old.quarantine_preview and not legacy.owns_path(new.author, new.id, new.quarantine_preview))
        or (new.public_path is distinct from old.public_path and not legacy.public_path_ok(new.kind, new.id, new.public_path))
        or (new.public_icon is distinct from old.public_icon and not legacy.public_path_ok(new.kind, new.id, new.public_icon))
        or (new.public_preview is distinct from old.public_preview and not legacy.public_path_ok(new.kind, new.id, new.public_preview));
  end if;
  if coalesce(bad, true) then
    raise exception 'Invalid storage path.' using errcode = '23514';
  end if;
  return new;
end $$;
revoke all on function public.enforce_storage_paths() from public, anon, authenticated;
drop trigger if exists items_storage_paths on public.community_items;
create trigger items_storage_paths before insert or update of quarantine_path, quarantine_icon, quarantine_preview, public_path, public_icon, public_preview
  on public.community_items for each row execute function public.enforce_storage_paths();

create or replace function public.enforce_profile_paths() returns trigger language plpgsql set search_path = public as $$
declare bad boolean;
begin
  if tg_op = 'INSERT' then
    bad := not (legacy.profile_public_path_ok(new.id, new.public_avatar) and legacy.profile_public_path_ok(new.id, new.public_banner));
  else
    bad := (new.public_avatar is distinct from old.public_avatar and not legacy.profile_public_path_ok(new.id, new.public_avatar))
        or (new.public_banner is distinct from old.public_banner and not legacy.profile_public_path_ok(new.id, new.public_banner));
  end if;
  if coalesce(bad, true) then
    raise exception 'Invalid storage path.' using errcode = '23514';
  end if;
  return new;
end $$;
revoke all on function public.enforce_profile_paths() from public, anon, authenticated;
drop trigger if exists public_profiles_paths on public.public_profiles;
create trigger public_profiles_paths before insert or update of public_avatar, public_banner on public.public_profiles
  for each row execute function public.enforce_profile_paths();

-- Quarantine uploads: the object NAME must be one of the two shapes the clients write. (Existing objects are untouched: this is an INSERT policy.
-- The select / delete policies of 0001 are kept so people can still read and delete their own files.)
drop policy if exists cuarentena_subir on storage.objects;
create policy cuarentena_subir on storage.objects for insert to authenticated
  with check (bucket_id = 'cuarentena' and legacy.quarantine_object_ok(name, (select auth.uid())));

-- Operator report: stored rows whose paths do not match the allowlist (the server treats them as «not available»). service_role only.
create or replace function legacy.bad_storage_paths() returns table (item uuid, col text, path text) language sql stable security definer set search_path = public as $$
  select id, 'quarantine_path', quarantine_path from public.community_items where not legacy.owns_path(author, id, quarantine_path)
  union all select id, 'quarantine_icon', quarantine_icon from public.community_items where not legacy.owns_path(author, id, quarantine_icon)
  union all select id, 'quarantine_preview', quarantine_preview from public.community_items where not legacy.owns_path(author, id, quarantine_preview)
  union all select id, 'public_path', public_path from public.community_items where not legacy.public_path_ok(kind, id, public_path)
  union all select id, 'public_icon', public_icon from public.community_items where not legacy.public_path_ok(kind, id, public_icon)
  union all select id, 'public_preview', public_preview from public.community_items where not legacy.public_path_ok(kind, id, public_preview)
  union all select id, 'public_avatar', public_avatar from public.public_profiles where not legacy.profile_public_path_ok(id, public_avatar)
  union all select id, 'public_banner', public_banner from public.public_profiles where not legacy.profile_public_path_ok(id, public_banner)
$$;
revoke all on function legacy.bad_storage_paths() from public, anon, authenticated;
grant execute on function legacy.bad_storage_paths() to service_role;

do $do$
declare n integer;
begin
  select count(*) into n from legacy.bad_storage_paths();
  if n > 0 then raise notice '0012: % stored storage path(s) do not match the allowlist: select * from legacy.bad_storage_paths();', n; end if;
end $do$;

-- ═════════════════════════ 2. New columns and server-only tables ═════════════════════════
alter table public.community_items add column if not exists updated_at timestamptz not null default now();
alter table public.moderation_calls add column if not exists kind text not null default 'item';
create index if not exists moderation_calls_author_kind_idx on public.moderation_calls (author, kind, at desc);

-- Hash of the profile image the server SCANNED (the human approval publishes only bytes with this hash).
create table if not exists public.profile_image_pins (
  person uuid not null references public.profiles (id) on delete cascade,
  class text not null check (class in ('avatar', 'banner')),
  sha256 text not null check (sha256 ~ '^[0-9a-f]{64}$'),
  created_at timestamptz not null default now(),
  primary key (person, class)
);
-- Claim of a profile action (one run at a time per person and kind).
create table if not exists public.profile_runs (
  person uuid not null references public.profiles (id) on delete cascade,
  kind text not null check (kind in ('image', 'save')),
  processing_at timestamptz,
  primary key (person, kind)
);
-- Uploads ever made (survives deleting the item): the daily cap counts THIS, not the items that still exist.
create table if not exists public.upload_log (
  item uuid primary key,
  author uuid not null,
  created_at timestamptz not null default now()
);
create index if not exists upload_log_author_idx on public.upload_log (author, created_at desc);
-- Names nobody can pick (impersonation). mode: exact = the whole normalised name; contains = anywhere; affix = at the start or the end.
create table if not exists public.reserved_names (
  pattern text primary key check (pattern ~ '^[a-z0-9]{2,32}$'),
  mode text not null default 'contains' check (mode in ('exact', 'contains', 'affix'))
);
-- The takedown queue.
create table if not exists public.storage_cleanup (
  id bigserial primary key,
  bucket text not null check (bucket in ('publico', 'cuarentena')),
  path text not null check (char_length(path) between 3 and 400),
  action text not null default 'delete' check (action in ('delete', 'to_quarantine')),
  item uuid,
  reason text,
  created_at timestamptz not null default now(),
  locked_at timestamptz,
  attempts integer not null default 0,
  done_at timestamptz
);
create unique index if not exists storage_cleanup_open_idx on public.storage_cleanup (bucket, path, action) where done_at is null;
create index if not exists storage_cleanup_todo_idx on public.storage_cleanup (id) where done_at is null;

alter table public.profile_image_pins enable row level security;
alter table public.profile_runs enable row level security;
alter table public.upload_log enable row level security;
alter table public.reserved_names enable row level security;
alter table public.storage_cleanup enable row level security;
revoke all on public.profile_image_pins, public.profile_runs, public.upload_log, public.reserved_names, public.storage_cleanup from public, anon, authenticated;
grant all on public.profile_image_pins, public.profile_runs, public.upload_log, public.reserved_names, public.storage_cleanup to service_role;
grant usage, select on all sequences in schema public to service_role;

insert into public.reserved_names (pattern, mode) values
  ('lienzo', 'contains'), ('deritu', 'contains'), ('moderator', 'contains'),
  ('admin', 'affix'), ('official', 'affix'), ('staff', 'affix'), ('support', 'affix'),
  ('administrator', 'exact'), ('mod', 'exact'), ('root', 'exact'), ('system', 'exact'), ('owner', 'exact'), ('team', 'exact')
on conflict (pattern) do nothing;

insert into public.upload_log (item, author, created_at)
  select id, author, created_at from public.community_items where created_at > now() - interval '7 days'
on conflict (item) do nothing;

-- updated_at: moves only when what a moderator reads changes (not for counters, status or hashes).
create or replace function public.touch_item() returns trigger language plpgsql as $$
begin
  if new.title is distinct from old.title or new.description is distinct from old.description
     or new.tags is distinct from old.tags or new.author_name is distinct from old.author_name then
    new.updated_at := clock_timestamp();
  else
    new.updated_at := old.updated_at; -- nobody (not even service_role by accident) rewrites it
  end if;
  return new;
end $$;
revoke all on function public.touch_item() from public, anon, authenticated;
drop trigger if exists items_touch on public.community_items;
create trigger items_touch before update on public.community_items for each row execute function public.touch_item();

-- ═════════════════════════ 3. Tags ═════════════════════════
create or replace function legacy.tags_ok(p_tags text[]) returns boolean language sql immutable as $$
  select p_tags is null or (
    coalesce(cardinality(p_tags), 0) <= 8
    and not exists (select 1 from unnest(p_tags) t where t is null or char_length(t) > 30 or t ~ '[[:cntrl:]]')
    and coalesce((select sum(char_length(t)) from unnest(p_tags) t), 0) <= 200
  )
$$;
-- Trims what already exists so the constraint can be validated: first 8 tags, control characters removed, 30 chars each, 200 in total.
-- (NOT reversible: the original long tags are not kept.)
create or replace function legacy.clean_tags(p_tags text[]) returns text[] language plpgsql immutable as $$
declare r text[] := '{}'; t text; total integer := 0;
begin
  foreach t in array coalesce(p_tags, '{}') loop
    exit when cardinality(r) >= 8;
    t := left(regexp_replace(coalesce(t, ''), '[[:cntrl:]]', '', 'g'), 30);
    continue when t = '';
    continue when total + char_length(t) > 200;
    r := r || t;
    total := total + char_length(t);
  end loop;
  return r;
end $$;
revoke all on function legacy.tags_ok(text[]) from public;
revoke all on function legacy.clean_tags(text[]) from public;
grant execute on function legacy.tags_ok(text[]) to anon, authenticated, service_role;
grant execute on function legacy.clean_tags(text[]) to service_role;

update public.community_items set tags = legacy.clean_tags(tags) where not legacy.tags_ok(tags);
alter table public.community_items drop constraint if exists community_items_tags_shape;
alter table public.community_items add constraint community_items_tags_shape check (legacy.tags_ok(tags)) not valid;
alter table public.community_items validate constraint community_items_tags_shape;

-- ═════════════════════════ 4. Public names ═════════════════════════
create or replace function legacy.name_key(p text) returns text language sql immutable as $$
  select translate(lower(regexp_replace(coalesce(p, ''), '[._-]+', '', 'g')), '013457', 'oieast')
$$;
-- SECURITY DEFINER: people cannot read reserved_names, but the triggers that call this (also definer) must.
create or replace function legacy.name_is_reserved(p_name text) returns boolean language sql stable security definer set search_path = public as $$
  select exists (
    select 1 from public.reserved_names r, (select legacy.name_key(p_name) as k) x
    where (r.mode = 'exact' and x.k = r.pattern)
       or (r.mode = 'contains' and position(r.pattern in x.k) > 0)
       or (r.mode = 'affix' and (x.k like r.pattern || '%' or x.k like '%' || r.pattern))
  )
$$;
revoke all on function legacy.name_key(text) from public;
revoke all on function legacy.name_is_reserved(text) from public;
grant execute on function legacy.name_key(text) to service_role;
grant execute on function legacy.name_is_reserved(text) to service_role;

-- name_reviewed = the last public name that passed review. Existing names are GRANDFATHERED once (not reversible), except reserved ones.
do $do$
begin
  if not exists (select 1 from information_schema.columns where table_schema = 'public' and table_name = 'profiles' and column_name = 'name_reviewed') then
    alter table public.profiles add column name_reviewed text;
    update public.profiles set name_reviewed = public_name where not legacy.name_is_reserved(public_name);
  end if;
end $do$;

-- People cannot choose a reserved name and cannot write name_reviewed (not even on INSERT through the base table).
create or replace function public.guard_public_name() returns trigger language plpgsql security definer set search_path = public as $$
begin
  if coalesce(auth.role(), '') in ('anon', 'authenticated') then
    if tg_op = 'INSERT' then
      new.name_reviewed := null;
      if legacy.name_is_reserved(new.public_name) then raise exception 'That public name is not available.' using errcode = '23514'; end if;
    else
      new.name_reviewed := old.name_reviewed;
      if new.public_name is distinct from old.public_name and legacy.name_is_reserved(new.public_name) then
        raise exception 'That public name is not available.' using errcode = '23514';
      end if;
    end if;
  end if;
  return new;
end $$;
revoke all on function public.guard_public_name() from public, anon, authenticated;
drop trigger if exists profiles_guard_name on public.profiles;
create trigger profiles_guard_name before insert or update on public.profiles for each row execute function public.guard_public_name();

-- Only a REVIEWED name is copied to the public places.
create or replace function public.set_author_name() returns trigger language plpgsql security definer set search_path = public as $$
begin
  select coalesce(name_reviewed, '') into new.author_name from public.profiles where id = new.author;
  return new;
end $$;
create or replace function public.create_public_profile() returns trigger language plpgsql security definer set search_path = public as $$
begin
  insert into public.public_profiles (id, public_name) values (new.id, coalesce(new.name_reviewed, '')) on conflict (id) do nothing;
  return new;
end $$;
create or replace function public.propagate_author_name() returns trigger language plpgsql security definer set search_path = public as $$
begin
  if new.name_reviewed is null then return new; end if; -- nothing reviewed yet: nothing is copied
  update public.community_items set author_name = new.name_reviewed where author = new.id and author_name is distinct from new.name_reviewed;
  update public.public_profiles set public_name = new.name_reviewed, updated_at = now() where id = new.id and public_name is distinct from new.name_reviewed;
  return new;
end $$;
revoke all on function public.set_author_name() from public, anon, authenticated;
revoke all on function public.create_public_profile() from public, anon, authenticated;
revoke all on function public.propagate_author_name() from public, anon, authenticated;
drop trigger if exists profiles_propagate_name on public.profiles;
create trigger profiles_propagate_name after update of public_name, name_reviewed on public.profiles for each row execute function public.propagate_author_name();

-- Called by the functions (service_role) once the name passed the text review. Refuses reserved names; false = nothing was marked.
create or replace function public.review_public_name(p_user uuid, p_name text) returns boolean language plpgsql security definer set search_path = public as $$
begin
  if legacy.name_is_reserved(p_name) then return false; end if;
  update public.profiles set name_reviewed = public_name where id = p_user and public_name = p_name;
  return found;
end $$;
revoke all on function public.review_public_name(uuid, text) from public, anon, authenticated;
grant execute on function public.review_public_name(uuid, text) to service_role;

-- ═════════════════════════ 5. Upload cap that deleting cannot dodge ═════════════════════════
create or replace function public.limit_uploads() returns trigger language plpgsql security definer set search_path = public as $$
begin
  if coalesce(auth.role(), '') in ('anon', 'authenticated') then
    new.created_at := now();
  end if;
  delete from public.upload_log where author = new.author and created_at < now() - interval '7 days'; -- retention
  if (select count(*) from public.community_items where author = new.author and status = 'pending') >= 10 then
    raise exception 'Too many uploads pending review (maximum 10).';
  end if;
  if (select count(*) from public.upload_log where author = new.author and created_at > now() - interval '1 day') >= 20 then
    raise exception 'Daily upload limit reached (20).';
  end if;
  return new;
end $$;
revoke all on function public.limit_uploads() from public, anon, authenticated;

create or replace function public.log_upload() returns trigger language plpgsql security definer set search_path = public as $$
begin
  insert into public.upload_log (item, author) values (new.id, new.author) on conflict (item) do nothing;
  return new;
end $$;
revoke all on function public.log_upload() from public, anon, authenticated;
drop trigger if exists items_log_upload on public.community_items;
create trigger items_log_upload after insert on public.community_items for each row execute function public.log_upload();

-- ═════════════════════════ 6. Moderation claims: longer lease, heartbeat, profile claims ═════════════════════════
-- Same signature as 0011 (the lease default goes from 300 to 600 seconds: it must outlive the 5 minute author edit lock). The hourly rate
-- only counts calls of kind «item», so profile actions have their own cap.
create or replace function public.claim_moderation(
  p_item uuid, p_max_attempts integer, p_max_failures integer,
  p_user uuid default null, p_user_per_hour integer default 0, p_lock_seconds integer default 600
) returns text language plpgsql set search_path = public as $$
begin
  if p_user is not null and p_user_per_hour > 0 then
    delete from public.moderation_calls where author = p_user and at < now() - interval '1 day';
    if (select count(*) from public.moderation_calls where author = p_user and kind = 'item' and at > now() - interval '1 hour') >= p_user_per_hour then
      return 'rate';
    end if;
  end if;
  insert into public.moderation_runs (item) values (p_item) on conflict (item) do nothing;
  update public.moderation_runs
     set attempts = attempts + 1, processing_at = now(), last_run_at = now()
   where item = p_item and attempts < p_max_attempts and failures < p_max_failures
     and (processing_at is null or processing_at < now() - make_interval(secs => p_lock_seconds));
  if not found then
    if exists (select 1 from public.moderation_runs where item = p_item and processing_at >= now() - make_interval(secs => p_lock_seconds)) then
      return 'busy';
    end if;
    return 'limit';
  end if;
  if p_user is not null then insert into public.moderation_calls (author, kind) values (p_user, 'item'); end if;
  return 'ok';
end $$;
revoke all on function public.claim_moderation(uuid, integer, integer, uuid, integer, integer) from public, anon, authenticated;
grant execute on function public.claim_moderation(uuid, integer, integer, uuid, integer, integer) to service_role;

-- A running review calls this between its steps: the claim (and the author edit lock, which reads the same timestamp) stays alive.
create or replace function public.heartbeat_moderation(p_item uuid) returns void language sql set search_path = public as $$
  update public.moderation_runs set processing_at = now() where item = p_item and processing_at is not null;
$$;
revoke all on function public.heartbeat_moderation(uuid) from public, anon, authenticated;
grant execute on function public.heartbeat_moderation(uuid) to service_role;

-- Profile actions: 'ok' (you hold the claim), 'busy' (another run of the same kind is alive) or 'rate' (too many this hour). Nothing is spent on a refusal.
create or replace function public.claim_profile(p_user uuid, p_kind text, p_per_hour integer, p_lock_seconds integer default 600)
returns text language plpgsql set search_path = public as $$
begin
  if p_kind not in ('image', 'save') then return 'busy'; end if;
  delete from public.moderation_calls where author = p_user and at < now() - interval '1 day';
  if p_per_hour > 0 and (select count(*) from public.moderation_calls where author = p_user and kind = p_kind and at > now() - interval '1 hour') >= p_per_hour then
    return 'rate';
  end if;
  insert into public.profile_runs (person, kind) values (p_user, p_kind) on conflict (person, kind) do nothing;
  update public.profile_runs set processing_at = now()
   where person = p_user and kind = p_kind and (processing_at is null or processing_at < now() - make_interval(secs => p_lock_seconds));
  if not found then return 'busy'; end if;
  insert into public.moderation_calls (author, kind) values (p_user, p_kind);
  return 'ok';
end $$;
create or replace function public.release_profile(p_user uuid, p_kind text) returns void language sql set search_path = public as $$
  update public.profile_runs set processing_at = null where person = p_user and kind = p_kind;
$$;
revoke all on function public.claim_profile(uuid, text, integer, integer) from public, anon, authenticated;
revoke all on function public.release_profile(uuid, text) from public, anon, authenticated;
grant execute on function public.claim_profile(uuid, text, integer, integer) to service_role;
grant execute on function public.release_profile(uuid, text) to service_role;

-- ═════════════════════════ 7. Takedown queue ═════════════════════════
create or replace function legacy.enqueue_cleanup(p_bucket text, p_path text, p_action text, p_item uuid, p_reason text) returns void
language sql security definer set search_path = public as $$
  insert into public.storage_cleanup (bucket, path, action, item, reason) values (p_bucket, p_path, p_action, p_item, p_reason)
  on conflict (bucket, path, action) where done_at is null do nothing
$$;
revoke all on function legacy.enqueue_cleanup(text, text, text, uuid, text) from public;
grant execute on function legacy.enqueue_cleanup(text, text, text, uuid, text) to service_role;

-- An item that stops being public for good (rejected / withdrawn): its public files are queued for deletion.
create or replace function public.queue_files_on_status() returns trigger language plpgsql security definer set search_path = public as $$
declare p text;
begin
  if new.status in ('rejected', 'withdrawn') and old.status is distinct from new.status then
    foreach p in array array[old.public_path, old.public_icon, old.public_preview] loop
      if p is not null and legacy.public_path_ok(old.kind, old.id, p) then
        perform legacy.enqueue_cleanup('publico', p, 'delete', old.id, 'status:' || new.status::text);
      end if;
    end loop;
  end if;
  return new;
end $$;
revoke all on function public.queue_files_on_status() from public, anon, authenticated;
drop trigger if exists items_queue_status on public.community_items;
create trigger items_queue_status after update of status on public.community_items for each row execute function public.queue_files_on_status();

-- The author (or an account deletion) deletes the item: every file it still has goes to the queue. (It also fires for ON DELETE CASCADE.)
create or replace function public.queue_files_on_delete() returns trigger language plpgsql security definer set search_path = public as $$
declare p text;
begin
  foreach p in array array[old.public_path, old.public_icon, old.public_preview] loop
    if p is not null and legacy.public_path_ok(old.kind, old.id, p) then
      perform legacy.enqueue_cleanup('publico', p, 'delete', old.id, 'deleted');
    end if;
  end loop;
  foreach p in array array[old.quarantine_path, old.quarantine_icon, old.quarantine_preview] loop
    if p is not null and legacy.owns_path(old.author, old.id, p) then
      perform legacy.enqueue_cleanup('cuarentena', p, 'delete', old.id, 'deleted');
    end if;
  end loop;
  return old;
end $$;
revoke all on function public.queue_files_on_delete() from public, anon, authenticated;
drop trigger if exists items_queue_delete on public.community_items;
create trigger items_queue_delete after delete on public.community_items for each row execute function public.queue_files_on_delete();

-- Atomic claim of a batch (several functions may drain at once). Rows that keep failing stop at 8 attempts. Finished rows are purged after 7 days.
create or replace function public.claim_cleanup(p_limit integer default 10, p_lock_seconds integer default 300)
returns setof public.storage_cleanup language plpgsql security definer set search_path = public as $$
begin
  delete from public.storage_cleanup where done_at is not null and done_at < now() - interval '7 days';
  return query
    update public.storage_cleanup c set attempts = c.attempts + 1, locked_at = now()
     where c.id in (
       select s.id from public.storage_cleanup s
        where s.done_at is null and s.attempts < 8 and (s.locked_at is null or s.locked_at < now() - make_interval(secs => p_lock_seconds))
        order by s.id limit least(greatest(p_limit, 1), 50) for update skip locked)
    returning c.*;
end $$;
create or replace function public.finish_cleanup(p_ids bigint[], p_ok boolean) returns void language sql security definer set search_path = public as $$
  update public.storage_cleanup
     set done_at = case when p_ok then now() else null end, locked_at = null
   where id = any(p_ids) and done_at is null;
$$;
revoke all on function public.claim_cleanup(integer, integer) from public, anon, authenticated;
revoke all on function public.finish_cleanup(bigint[], boolean) from public, anon, authenticated;
grant execute on function public.claim_cleanup(integer, integer) to service_role;
grant execute on function public.finish_cleanup(bigint[], boolean) to service_role;

-- Reports: only those since the last review / approval count, and an item hidden for malware / copyright (2+ such reports) has its public files
-- queued to be MOVED back to quarantine (a moderator can still re-approve it; the pinned hashes are verified again by publish()).
create or replace function public.hide_on_reports() returns trigger language plpgsql security definer set search_path = public as $$
declare it record; since timestamptz; n integer; bad integer; p text;
begin
  select id, kind, status, reviewed_at, public_path, public_icon, public_preview into it from public.community_items where id = new.item;
  if not found then return new; end if;
  since := coalesce(it.reviewed_at, '-infinity'::timestamptz);
  select count(*), count(*) filter (where reason in ('malware', 'copyright')) into n, bad
    from public.reports where item = new.item and created_at > since;
  if n >= 3 then
    update public.community_items set status = 'pending', rejection_reason = 'Hidden by reports: pending human review'
      where id = new.item and status = 'approved';
    if found and bad >= 2 then
      foreach p in array array[it.public_path, it.public_icon, it.public_preview] loop
        if p is not null and legacy.public_path_ok(it.kind, it.id, p) then
          perform legacy.enqueue_cleanup('publico', p, 'to_quarantine', it.id, 'hidden: malware/copyright reports');
        end if;
      end loop;
    end if;
  end if;
  return new;
end $$;
revoke all on function public.hide_on_reports() from public, anon, authenticated;

-- ═════════════════════════ 8. Downloads: only public items, 18+ only for people who opted in ═════════════════════════
create or replace function public.record_download(p_item uuid) returns void language plpgsql security definer set search_path = public as $$
declare v_uid uuid := auth.uid();
begin
  if v_uid is null then return; end if; -- signed-in people only, as before
  if not exists (
    select 1 from public.community_items i
     where i.id = p_item and i.status = 'approved'
       and (not i.is_18 or exists (select 1 from public.profiles p where p.id = v_uid and p.show_18))
  ) then return; end if;
  insert into public.download_log (person, item) values (v_uid, p_item) on conflict do nothing;
  if found then
    update public.community_items set downloads = downloads + 1 where id = p_item;
  end if;
end $$;
revoke all on function public.record_download(uuid) from public, anon;
grant execute on function public.record_download(uuid) to authenticated;
-- registrar_descarga(p_item) (legacy wrapper, security invoker) is untouched.

notify pgrst, 'reload schema';
commit;

-- ═════════════════════════ Checks to run after applying (read-only) ═════════════════════════
--  select * from legacy.bad_storage_paths();                       -- old rows that do not match the allowlist (fix or reject them)
--  select count(*) from storage_cleanup where done_at is null;     -- the queue; call review {action:"cleanup"} to drain it
--  select pattern, mode from reserved_names order by 2, 1;         -- edit this table to change the reserved / impersonation names
--  select policyname, cmd, with_check from pg_policies where schemaname = 'storage' and policyname = 'cuarentena_subir';
--  -- as an author: insert into community_items (..., quarantine_path) values (..., '<uid>/<id>/..%2f..%2fx.json');   -- ERROR 42501 (policy)
