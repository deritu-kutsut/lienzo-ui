-- Lienzo UI · Community. Security fixes from the server audit (migration 0011).
--
-- WHAT IT FIXES (numbers match the audit)
--   1. HIGH  Authors could edit title / description / tags of already APPROVED (or in-review) items and the change went live without
--            moderation. Now a BEFORE UPDATE trigger puts an edited approved item back to «pending» (the public catalog only shows
--            `approved`, so it disappears until a moderator approves it again); edits of rejected / withdrawn items, and edits of an item
--            that is being processed right now, are refused. Only people (JWT role anon/authenticated) are affected: service_role and
--            SQL-editor sessions (moderators, Edge Functions) edit freely. The update policy is narrowed too.
--   2. HIGH  TOCTOU on icon / preview: new columns icon_sha256 / preview_sha256 (+ the existing sha256 for the main file) are pinned by
--            `moderate`; `publish()` refuses anything that no longer matches. (Edge Function change, see _shared/server.ts.)
--   3. HIGH  Anyone could LIST the whole «publico» bucket (storage policy publico_leer). The policy is dropped and a RESTRICTIVE policy
--            makes sure no anon/authenticated query can read «publico» rows through the Storage API (listing). Public buckets still serve
--            direct object URLs (/storage/v1/object/public/publico/…) because that route does not go through RLS.
--   4. Insert hardening: the upload policy now pins rejection_reason / sha256 / bytes / hashes null, version '1.0.0', created_at within ±1 min
--            and quarantine paths to `<uid>/<item id>/…` (no «..»). limit_uploads() / limit_hearts() stamp created_at := now() on the server
--            (so the daily limits cannot be dodged with a forged date). hearts.created_at is pinned too.
--   5. (function side) validatePlugin no longer throws on `"commands": 5`; rescan skips failing items. See the TypeScript changes.
--   6. Moderation quota protection: server-only tables moderation_runs / moderation_calls and the functions claim_moderation() /
--            release_moderation() (atomic claim, cap of runs per item, failure cap, per-user hourly rate limit). service_role only.
--   +  record_download() now counts ONCE per (person, item) (table download_log). The legacy wrapper registrar_descarga() is unchanged.
--
-- WHAT STAYS EXACTLY AS IT WAS (old app versions 1.0 – 1.4.1)
--   · No legacy view, column, grant or function name is changed. `items`, `perfiles`, `corazones`… keep their definitions; the INSTEAD OF
--     triggers still write through the real table, so the new trigger / policies apply to them too.
--   · No GRANT is added to anon / authenticated. The new tables and functions are service_role only. The two new columns on community_items
--     are readable like the rest of the table (they are plain hashes, same as sha256) and NOT writable by people (UPDATE is column-granted
--     to title/description/tags only; INSERT must leave them null by policy).
--   · Public file URLs (avatars, banners, icons, previews, downloads) keep working: only the Storage LIST/SELECT policy goes away.
--
-- HOW TO APPLY  (order matters)
--   1. Back up (Dashboard → Database → Backups).  `npx supabase db push`  (or paste this file in the SQL editor). One transaction:
--      it applies fully or not at all. Safe to re-run (idempotent).
--   2. Deploy the Edge Functions (moderate / moderar, review / revisar, delete-account / borrar-cuenta). They need this migration FIRST
--      (they call claim_moderation / release_moderation and read the new columns). Old functions keep working against the new schema,
--      except that nobody pins the hashes yet.
--   3. Items that were pending BEFORE the migration have no pinned icon/preview hash: a moderator must press «rescan» on them once
--      (publish refuses to approve an item that was never hash-pinned).
--   4. Verify: see the checks at the bottom of this file.
--
-- HOW TO ROLL BACK
--   Run supabase/rollbacks/0011_security_fixes.down.sql (one transaction), then `supabase migration repair --status reverted 0011`, and
--   redeploy the previous Edge Functions. NOTE: rolling back re-opens the «publico» listing hole and the unmoderated-edit hole.

begin;

-- ═════════════════════════ 0. Helper (schema `legacy`, not exposed through the API) ═════════════════════════
-- True when `p_path` is NULL or lives under `<author>/<item id>/` and has no «..». Used by the upload policy.
create or replace function legacy.owns_path(p_author uuid, p_id uuid, p_path text) returns boolean language sql immutable as $$
  select p_path is null or (
    char_length(p_path) between 1 and 300
    and position('..' in p_path) = 0
    and starts_with(p_path, p_author::text || '/' || p_id::text || '/')
  )
$$;
revoke all on function legacy.owns_path(uuid, uuid, text) from public;
grant execute on function legacy.owns_path(uuid, uuid, text) to anon, authenticated, service_role;

-- ═════════════════════════ 1. New columns and server-only tables ═════════════════════════
-- Hashes of the exact bytes `moderate` scanned (the main file's one is the existing `sha256`).
alter table public.community_items
  add column if not exists icon_sha256 text check (icon_sha256 is null or icon_sha256 ~ '^[0-9a-f]{64}$'),
  add column if not exists preview_sha256 text check (preview_sha256 is null or preview_sha256 ~ '^[0-9a-f]{64}$');

-- Moderation runs: claim / attempts / failures per item. Server only (no policies, no grants for people).
create table if not exists public.moderation_runs (
  item uuid primary key references public.community_items (id) on delete cascade,
  attempts integer not null default 0,
  failures integer not null default 0,
  processing_at timestamptz,
  last_run_at timestamptz
);
-- One row per accepted `moderate` call, to rate-limit each author.
create table if not exists public.moderation_calls (
  author uuid not null,
  at timestamptz not null default now()
);
create index if not exists moderation_calls_author_idx on public.moderation_calls (author, at desc);
-- Downloads already counted: one per (person, item).
create table if not exists public.download_log (
  person uuid not null references auth.users (id) on delete cascade,
  item uuid not null references public.community_items (id) on delete cascade,
  created_at timestamptz not null default now(),
  primary key (person, item)
);
create index if not exists download_log_item_idx on public.download_log (item);

alter table public.moderation_runs enable row level security;
alter table public.moderation_calls enable row level security;
alter table public.download_log enable row level security;
revoke all on public.moderation_runs, public.moderation_calls, public.download_log from public, anon, authenticated;
grant all on public.moderation_runs, public.moderation_calls, public.download_log to service_role;

-- ═════════════════════════ 2. Atomic claim of a moderation run (item 6) ═════════════════════════
-- Returns 'ok' (you hold the claim), 'busy' (another run holds it and it is not stale), 'limit' (runs or failures used up) or 'rate'
-- (the author called too often this hour). A stale claim (older than p_lock_seconds) can be taken over. The UPDATE ... WHERE is atomic:
-- two concurrent callers cannot both get 'ok'.
create or replace function public.claim_moderation(
  p_item uuid, p_max_attempts integer, p_max_failures integer,
  p_user uuid default null, p_user_per_hour integer default 0, p_lock_seconds integer default 300
) returns text language plpgsql set search_path = public as $$
begin
  if p_user is not null and p_user_per_hour > 0 then
    delete from public.moderation_calls where author = p_user and at < now() - interval '1 day';
    if (select count(*) from public.moderation_calls where author = p_user and at > now() - interval '1 hour') >= p_user_per_hour then
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
  if p_user is not null then insert into public.moderation_calls (author) values (p_user); end if;
  return 'ok';
end $$;

create or replace function public.release_moderation(p_item uuid, p_failed boolean) returns void language sql set search_path = public as $$
  update public.moderation_runs set processing_at = null, failures = case when p_failed then failures + 1 else 0 end where item = p_item;
$$;

revoke all on function public.claim_moderation(uuid, integer, integer, uuid, integer, integer) from public, anon, authenticated;
revoke all on function public.release_moderation(uuid, boolean) from public, anon, authenticated;
grant execute on function public.claim_moderation(uuid, integer, integer, uuid, integer, integer) to service_role;
grant execute on function public.release_moderation(uuid, boolean) to service_role;

-- ═════════════════════════ 3. Edits after approval go back to moderation (item 1) ═════════════════════════
-- Fires for title / description / tags only (the only columns people can update) and only for people: JWT role anon/authenticated.
-- service_role (Edge Functions, moderators) and SQL-editor sessions have no such role claim and pass through untouched.
-- SECURITY DEFINER only to read moderation_runs; it never writes outside NEW.
create or replace function public.reset_review_on_edit() returns trigger language plpgsql security definer set search_path = public as $$
begin
  if coalesce(auth.role(), '') not in ('anon', 'authenticated') then
    return new;
  end if;
  if new.title is not distinct from old.title and new.description is not distinct from old.description and new.tags is not distinct from old.tags then
    return new; -- nothing that needs a new review changed
  end if;
  if old.status = 'approved' then
    -- Hidden from the catalog (it only lists status = 'approved') until a moderator approves the new text. The public files stay in place
    -- so the re-approval needs no re-upload (publish() handles «pending but already public»).
    new.status := 'pending';
    new.rejection_reason := 'Edited after approval: pending human review';
  elsif old.status = 'pending' then
    -- Do not let the text change under a review that is running right now (it would be published unreviewed).
    if exists (select 1 from public.moderation_runs r where r.item = old.id and r.processing_at is not null and r.processing_at > now() - interval '5 minutes') then
      raise exception 'This content is being reviewed right now: try again in a few minutes.' using errcode = '42501';
    end if;
  else
    raise exception 'This content can no longer be edited.' using errcode = '42501';
  end if;
  return new;
end $$;
revoke all on function public.reset_review_on_edit() from public, anon, authenticated;

drop trigger if exists items_reset_review on public.community_items;
create trigger items_reset_review before update of title, description, tags on public.community_items
  for each row execute function public.reset_review_on_edit();

-- Update policy: only own items that are pending or approved, and the row must end up pending or approved. (People can only write
-- title / description / tags by column grant, so status cannot be changed by them; an edited approved item is flipped by the trigger.)
drop policy if exists items_update_own on public.community_items;
drop policy if exists items_retirar_propio on public.community_items;
create policy items_update_own on public.community_items for update to authenticated
  using (author = (select auth.uid()) and status in ('pending', 'approved'))
  with check (author = (select auth.uid()) and status in ('pending', 'approved'));

-- ═════════════════════════ 4. Insert hardening (item 4) ═════════════════════════
drop policy if exists items_upload on public.community_items;
drop policy if exists items_subir on public.community_items;
create policy items_upload on public.community_items for insert to authenticated
  with check (
    author = (select auth.uid()) and status = 'pending'
    and public_path is null and public_icon is null and public_preview is null
    and reviewed_at is null and downloads = 0 and hearts = 0
    and rejection_reason is null and sha256 is null and bytes is null and icon_sha256 is null and preview_sha256 is null
    and version = '1.0.0'
    and created_at between now() - interval '1 minute' and now() + interval '1 minute'
    and legacy.owns_path(author, id, quarantine_path)
    and legacy.owns_path(author, id, quarantine_icon)
    and legacy.owns_path(author, id, quarantine_preview)
  );

-- The server stamps the creation time of what people insert (a trigger runs BEFORE the policy check): daily limits cannot be dodged.
-- Same bodies as 0010 plus the stamp. Only for JWT role anon/authenticated: imports / service_role keep their own dates.
create or replace function public.limit_uploads() returns trigger language plpgsql security definer set search_path = public as $$
begin
  if coalesce(auth.role(), '') in ('anon', 'authenticated') then
    new.created_at := now();
  end if;
  if (select count(*) from public.community_items where author = new.author and status = 'pending') >= 10 then
    raise exception 'Too many uploads pending review (maximum 10).';
  end if;
  if (select count(*) from public.community_items where author = new.author and created_at > now() - interval '1 day') >= 20 then
    raise exception 'Daily upload limit reached (20).';
  end if;
  return new;
end $$;

create or replace function public.limit_hearts() returns trigger language plpgsql security definer set search_path = public as $$
begin
  if coalesce(auth.role(), '') in ('anon', 'authenticated') then
    new.created_at := now();
  end if;
  if (select count(*) from public.hearts where person = new.person and created_at > now() - interval '1 day') >= 200 then
    raise exception 'Too many hearts today (maximum 200).';
  end if;
  return new;
end $$;
revoke all on function public.limit_uploads() from public, anon, authenticated;
revoke all on function public.limit_hearts() from public, anon, authenticated;

drop policy if exists hearts_give on public.hearts;
drop policy if exists corazones_dar on public.hearts;
create policy hearts_give on public.hearts for insert to authenticated
  with check (
    person = (select auth.uid())
    and exists (select 1 from public.community_items i where i.id = item and i.status = 'approved')
    and created_at between now() - interval '1 minute' and now() + interval '1 minute'
  );

-- ═════════════════════════ 5. Storage: nobody lists «publico» (item 3) ═════════════════════════
-- Direct object URLs of a PUBLIC bucket do not need a policy; only the Storage API's list / authenticated-read routes did.
drop policy if exists publico_leer on storage.objects;
-- Backstop: even if some other permissive SELECT policy exists (e.g. created by hand in the dashboard), anon and authenticated can never
-- read «publico» rows through the Storage API. service_role (Edge Functions) ignores RLS and is not affected.
drop policy if exists publico_no_listar on storage.objects;
create policy publico_no_listar on storage.objects as restrictive for select to anon, authenticated
  using (bucket_id <> 'publico');

-- ═════════════════════════ 6. Downloads count once per person and item ═════════════════════════
create or replace function public.record_download(p_item uuid) returns void language plpgsql security definer set search_path = public as $$
declare v_uid uuid := auth.uid();
begin
  if v_uid is null then return; end if; -- signed-in people only, as before
  if not exists (select 1 from public.community_items where id = p_item and status = 'approved') then return; end if;
  insert into public.download_log (person, item) values (v_uid, p_item) on conflict do nothing;
  if found then
    update public.community_items set downloads = downloads + 1 where id = p_item;
  end if;
end $$;
revoke all on function public.record_download(uuid) from public, anon;
grant execute on function public.record_download(uuid) to authenticated;
-- The legacy wrapper registrar_descarga(p_item) (security invoker, authenticated only) already calls record_download and is left untouched.

notify pgrst, 'reload schema';
commit;

-- ═════════════════════════ Checks to run after applying (read-only) ═════════════════════════
--  -- as an author (SQL editor with `set local role authenticated; set local request.jwt.claims = '{"sub":"<uid>","role":"authenticated"}'`):
--  --   update community_items set title = title || ' x' where id = '<an approved item of that author>';   -- status becomes 'pending'
--  --   update community_items set title = 'y' where id = '<a rejected item of that author>';              -- ERROR 42501
--  select policyname, permissive, cmd, roles from pg_policies where schemaname = 'storage' and tablename = 'objects' order by 1;
--  --   no publico_leer; publico_no_listar RESTRICTIVE.   And from outside:
--  --   curl -s -X POST "$SUPABASE_URL/storage/v1/object/list/publico" -H "apikey: $ANON" -H "Authorization: Bearer $ANON" \
--  --        -H 'content-type: application/json' -d '{"prefix":"","limit":10}'      -> [] (empty)
--  --   curl -sI "$SUPABASE_URL/storage/v1/object/public/publico/<a real public_path>"                -> 200
--  select claim_moderation(gen_random_uuid(), 5, 3);    -- as service_role: FK error for an unknown item (the function fails closed)
