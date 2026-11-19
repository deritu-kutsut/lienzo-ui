-- Lienzo UI · Community. Terms acceptance, content declarations, copyright claims, strikes and account suspension (migration 0015). Builds on 0014.
--
-- WHAT IT ADDS (everything is ADDITIVE: no existing column, grant, view or function signature is removed or changed)
--   1. Terms of Use versions. Server-only table `terms_versions` (the version ids only; the TEXTS live in the repository docs) and the public RPC
--      `current_terms()`. `profiles.terms_version` can ONLY be set by the SECURITY DEFINER function `accept_terms(p_version)` (authenticated;
--      the user id comes from auth.uid()). It refuses unknown or outdated versions and appends to the append-only audit table `terms_log`.
--      The legacy timestamp `terms_accepted_at` (old clients write it through the view `perfiles.acepto_terminos_en` at profile creation) keeps
--      working and counts as «accepted version v0».
--   2. Grace flag instead of a hard switch. Table `app_settings` (single row): `terms_enforced_from` and `license_required_from` are NULL = NOT
--      enforced (the default). While off, uploads are accepted whatever the account accepted and the version in force is only RECORDED in
--      `community_items.uploaded_under_terms`. The maintainer flips the flag later (see supabase/MIGRATION.md). When on, an upload by an account
--      whose accepted version is not the current one is refused with `terms_required`.
--   3. Per-item declarations on `community_items`: `license` (allowlist, see legacy.license_ok), `source_url` (http/https, <= 300 chars) and
--      `rights_confirmed_at` (stamped by the server: a client-supplied value is replaced by now()). NULL is allowed for legacy items and old clients.
--      Errors carry a machine code in the message prefix AND in the `hint` field of the PostgREST error: license_invalid, source_url_invalid,
--      rights_required, terms_required, upload_banned, account_suspended.
--   4. Strikes and bans. `profiles.strikes`, `profiles.upload_banned_until`, `profiles.suspended`: people can neither insert nor update them (guard
--      trigger). `limit_uploads()` refuses uploads while banned or suspended. A suspended creator's items and public profile disappear from the catalog
--      (restrictive SELECT policies; the author still sees their own). Server-only functions: set_upload_ban, set_suspension, reverse_strike.
--   5. Copyright claims. Server-only table `copyright_claims`. `file_copyright_claim()` (authenticated) HIDES the item at once (status pending, public
--      files queued with the existing `to_quarantine` action of storage_cleanup), rate-limited per user (serialized), only for profiles >= 24 h old that accepted the terms, and records the claim. More than 3 open claims in 24 h against
--      one author are recorded and FLAGGED but do not hide the item. Items hidden by a claim do not count towards the 10-pending upload cap. A trigger refuses to
--      approve an item that has an open claim. Moderators resolve claims with resolve_claim() (service_role; the `review` function exposes it as
--      claim_uphold / claim_reject): upheld = a strike for the author, 3 strikes = permanent upload ban; rejected = the item can be restored.
--   6. Reports: the reason allowlist grows (copyright, adult, malware, hate, spam, impersonation, personal_data, other). The old values stay valid.
--   7. Data export: `export_my_data()` returns the caller's own rows as JSON. Account deletion keeps working (terms_log cascades with the profile).
--
-- WHAT STAYS EXACTLY AS IT WAS (old app versions 1.0 - 1.5)
--   · The legacy views (items, perfiles, reportes, ...) are not redefined. They write through INSTEAD OF triggers that call the base tables as the
--     caller, so the new triggers apply to old clients too — but with the flags OFF (default) they only RECORD, they refuse nothing, except
--     upload_banned / account_suspended (a banned person must not upload through any client).
--   · No GRANT is added to anon / authenticated except EXECUTE on the RPCs documented in supabase/CLIENT_CONTRACT.md (current_terms, accept_terms,
--     file_copyright_claim, export_my_data). New tables are service_role only.
--
-- HOW TO APPLY (order matters, see supabase/MIGRATION.md)
--   1. Back up. `npx supabase db push` (or paste this file in the SQL editor). One transaction; idempotent (safe to re-run).
--   2. THEN deploy the Edge Functions (profile: accept_terms; review: claim_* actions). 3. THEN publish web/moderacion. 4. THEN release the client.
--   Nothing is enforced after this migration. To enforce the terms later: update public.app_settings set terms_enforced_from = '<timestamp>';
-- HOW TO ROLL BACK
--   supabase/rollbacks/0015_terms_and_copyright.down.sql (read its header: the new columns and tables, and their data, are dropped).
begin;

-- ═════════════════════════ 0. Helpers (schema `legacy`, not exposed through the API) ═════════════════════════
-- Raises an error whose message starts with a machine code and whose HINT is the same code (PostgREST returns both).
create or replace function legacy.raise_code(p_code text, p_text text, p_errcode text default 'P0001') returns void language plpgsql as $$
begin
  raise exception '%: %', p_code, p_text using errcode = p_errcode, hint = p_code;
end $$;
revoke all on function legacy.raise_code(text, text, text) from public;
grant execute on function legacy.raise_code(text, text, text) to anon, authenticated, service_role;

-- Licence allowlist. Everything here lets OTHER users use the file in their own projects (that is what the community is for);
-- «all rights reserved» is deliberately not an option. Change the list by redefining this function (the CHECK constraint calls it).
--   cc0 / public-domain: no restrictions · cc-by: credit · cc-by-sa: credit + share alike · cc-by-nc: credit, no commercial use
--   mit / apache-2: software licences (plugins, templates with scripts) · ofl: SIL Open Font License (fonts)
--   proprietary-free-use: the author's own terms, free to use in any project (state them in the description)
create or replace function legacy.license_ok(p text) returns boolean language sql immutable as $$
  select p is null or p in ('cc0', 'public-domain', 'cc-by', 'cc-by-sa', 'cc-by-nc', 'mit', 'apache-2', 'ofl', 'proprietary-free-use')
$$;
-- Source link: http(s) only, a plain host (no user:password@, no spaces / control characters), at most 300 characters.
create or replace function legacy.source_url_ok(p text) returns boolean language sql immutable as $$
  select p is null or (
    char_length(p) <= 300
    and p ~* '^https?://[a-z0-9][a-z0-9.-]*(:[0-9]{1,5})?([/?#][^[:space:][:cntrl:]]*)?$'
  )
$$;
revoke all on function legacy.license_ok(text) from public;
revoke all on function legacy.source_url_ok(text) from public;
grant execute on function legacy.license_ok(text) to anon, authenticated, service_role;
grant execute on function legacy.source_url_ok(text) to anon, authenticated, service_role;

-- ═════════════════════════ 1. Settings, terms versions and the audit log ═════════════════════════
-- One row. NULL = not enforced (the default). A timestamp = enforced from then on (a future one schedules it).
create table if not exists public.app_settings (
  singleton boolean primary key default true check (singleton),
  terms_enforced_from timestamptz,
  license_required_from timestamptz,
  updated_at timestamptz not null default now()
);
insert into public.app_settings (singleton) values (true) on conflict (singleton) do nothing;

create table if not exists public.terms_versions (
  version text primary key check (version ~ '^[A-Za-z0-9._-]{1,32}$' and version <> 'v0'),  -- «v0» is reserved for the legacy timestamp
  published_at timestamptz not null default now(),
  summary text not null default '' check (char_length(summary) <= 500)
);
-- Order of publication = insertion order (id), NOT the text of the version («10.0» < «9.0» as text) and not published_at (equal inside one transaction).
alter table public.terms_versions add column if not exists id bigint generated always as identity;
create unique index if not exists terms_versions_id_key on public.terms_versions (id);
insert into public.terms_versions (version, summary) values
  ('1.0', 'First version: Terms of Use, Community Guidelines, Privacy Policy and Copyright policy (see docs/TERMS.md in the repository).')
on conflict (version) do nothing;

-- Append-only audit of acceptances. Deleted only by the cascade of an account deletion (right to erasure).
create table if not exists public.terms_log (
  id bigserial primary key,
  user_id uuid not null references public.profiles (id) on delete cascade,
  version text not null references public.terms_versions (version),
  accepted_at timestamptz not null default now()
);
create index if not exists terms_log_user_idx on public.terms_log (user_id, accepted_at desc);

alter table public.app_settings enable row level security;
alter table public.terms_versions enable row level security;
alter table public.terms_log enable row level security;
revoke all on public.app_settings, public.terms_versions, public.terms_log from public, anon, authenticated;
grant all on public.app_settings, public.terms_versions, public.terms_log to service_role;
grant usage, select on all sequences in schema public to service_role;

create or replace function legacy.current_terms_version() returns text language sql stable security definer set search_path = public as $$
  select version from public.terms_versions order by id desc limit 1
$$;
revoke all on function legacy.current_terms_version() from public;
grant execute on function legacy.current_terms_version() to service_role;

create or replace function public.guard_terms_log() returns trigger language plpgsql as $$
begin
  if tg_op = 'UPDATE' then
    raise exception 'terms_log is append-only.' using errcode = '42501';
  end if;
  -- DELETE: only from inside the ON DELETE CASCADE of a profile (trigger depth > 1). A direct DELETE is refused for everybody.
  if pg_trigger_depth() < 2 then
    raise exception 'terms_log is append-only.' using errcode = '42501';
  end if;
  return old;
end $$;
revoke all on function public.guard_terms_log() from public, anon, authenticated;
drop trigger if exists terms_log_append_only on public.terms_log;
create trigger terms_log_append_only before update or delete on public.terms_log for each row execute function public.guard_terms_log();

-- The current version, for the client («do I have to ask again?»). Public: it holds version ids only.
create or replace function public.current_terms() returns jsonb language sql stable security definer set search_path = public as $$
  select coalesce((select jsonb_build_object('version', version, 'published_at', published_at, 'summary', summary)
                   from public.terms_versions order by id desc limit 1), 'null'::jsonb)
$$;
revoke all on function public.current_terms() from public;
grant execute on function public.current_terms() to anon, authenticated, service_role;

-- ═════════════════════════ 2. Profile columns and who may write them ═════════════════════════
alter table public.profiles
  add column if not exists terms_version text references public.terms_versions (version),
  add column if not exists strikes smallint not null default 0 check (strikes >= 0),
  add column if not exists upload_banned_until timestamptz,
  add column if not exists suspended boolean not null default false;

-- People cannot write these columns: UPDATE has column grants (none of these is granted) but INSERT is table-wide, so a trigger pins them.
-- SECURITY INVOKER on purpose: current_user is the caller's role, and inside the SECURITY DEFINER functions below it is their owner.
create or replace function public.guard_profile_moderation() returns trigger language plpgsql set search_path = public as $$
begin
  if current_user in ('anon', 'authenticated') then
    if tg_op = 'INSERT' then
      new.terms_version := null; new.strikes := 0; new.upload_banned_until := null; new.suspended := false;
    else
      new.terms_version := old.terms_version; new.strikes := old.strikes;
      new.upload_banned_until := old.upload_banned_until; new.suspended := old.suspended;
    end if;
  end if;
  return new;
end $$;
revoke all on function public.guard_profile_moderation() from public, anon, authenticated;
drop trigger if exists profiles_guard_moderation on public.profiles;
create trigger profiles_guard_moderation before insert or update on public.profiles for each row execute function public.guard_profile_moderation();

-- accept_terms: the ONLY way to set profiles.terms_version.
create or replace function public.accept_terms(p_version text) returns jsonb language plpgsql security definer set search_path = public as $$
declare v_uid uuid := auth.uid(); v_cur text := legacy.current_terms_version(); v_prev text; v_at timestamptz := now(); v_acc timestamptz;
begin
  if v_uid is null then perform legacy.raise_code('sign_in_required', 'Sign in first.', '28000'); end if;
  if p_version is null or not exists (select 1 from public.terms_versions where version = p_version) then
    perform legacy.raise_code('terms_unknown_version', 'Unknown terms version.', '22023');
  end if;
  if p_version is distinct from v_cur then
    perform legacy.raise_code('terms_outdated', 'These terms are no longer current: update the app and accept the current version.', '22023');
  end if;
  select terms_version, terms_accepted_at into v_prev, v_acc from public.profiles where id = v_uid for update;
  if not found then perform legacy.raise_code('profile_required', 'Create your profile first.', 'P0002'); end if;
  if v_prev is distinct from p_version then
    update public.profiles set terms_version = p_version, terms_accepted_at = v_at where id = v_uid;
    insert into public.terms_log (user_id, version, accepted_at) values (v_uid, p_version, v_at);
    v_acc := v_at;
  end if;
  return jsonb_build_object('version', p_version, 'accepted_at', v_acc, 'current', v_cur);
end $$;
revoke all on function public.accept_terms(text) from public, anon;
grant execute on function public.accept_terms(text) to authenticated, service_role;

-- ═════════════════════════ 3. Declarations on community_items ═════════════════════════
alter table public.community_items
  add column if not exists license text,
  add column if not exists source_url text,
  add column if not exists rights_confirmed_at timestamptz,
  add column if not exists uploaded_under_terms text;       -- version the author had accepted when uploading (null = nothing, 'v0' = legacy timestamp)
alter table public.community_items drop constraint if exists community_items_license_check;
alter table public.community_items add constraint community_items_license_check check (legacy.license_ok(license));
alter table public.community_items drop constraint if exists community_items_source_url_check;
alter table public.community_items add constraint community_items_source_url_check check (legacy.source_url_ok(source_url));
-- (People cannot UPDATE these columns: UPDATE is column-granted to title / description / tags only, since 0001.)

-- Runs for people only (JWT role anon / authenticated). service_role, imports and the SQL editor pass through untouched.
-- Named so that it fires AFTER limit_uploads (ban / suspension are reported first).
create or replace function public.enforce_upload_declarations() returns trigger language plpgsql security definer set search_path = public as $$
declare s public.app_settings%rowtype; v_cur text := legacy.current_terms_version(); v_eff text;
begin
  if coalesce(auth.role(), '') not in ('anon', 'authenticated') then return new; end if;
  -- BEFORE reading anything about new.author: a person can only insert as themselves. Otherwise the codes below (terms_required, ...) would tell
  -- whether ANOTHER account accepted the terms. The message is the one RLS would give, so nothing is revealed (RLS runs after BEFORE triggers).
  if new.author is distinct from auth.uid() then
    raise exception 'new row violates row-level security policy for table "community_items"' using errcode = '42501';
  end if;
  select * into s from public.app_settings where singleton;
  select coalesce(terms_version, case when terms_accepted_at is not null then 'v0' end) into v_eff from public.profiles where id = new.author;
  new.uploaded_under_terms := v_eff;                                            -- recorded even while nothing is enforced
  new.rights_confirmed_at := case when new.rights_confirmed_at is null then null else now() end;   -- the server stamps it
  if not legacy.license_ok(new.license) then perform legacy.raise_code('license_invalid', 'That licence is not allowed.', '23514'); end if;
  if not legacy.source_url_ok(new.source_url) then perform legacy.raise_code('source_url_invalid', 'The source link must be an http(s) address of at most 300 characters.', '23514'); end if;
  if new.license is not null and new.rights_confirmed_at is null then
    perform legacy.raise_code('rights_required', 'Confirm that you own the content or have the right to share it.', '23514');
  end if;
  if s.terms_enforced_from is not null and s.terms_enforced_from <= now() and v_eff is distinct from v_cur then
    perform legacy.raise_code('terms_required', 'Accept the current Terms of Use (version ' || coalesce(v_cur, '?') || ') to upload.', '42501');
  end if;
  if s.license_required_from is not null and s.license_required_from <= now() then
    if new.license is null then perform legacy.raise_code('license_invalid', 'Choose a licence for your content.', '23514'); end if;
    if new.rights_confirmed_at is null then perform legacy.raise_code('rights_required', 'Confirm that you own the content or have the right to share it.', '23514'); end if;
  end if;
  return new;
end $$;
revoke all on function public.enforce_upload_declarations() from public, anon, authenticated;
drop trigger if exists items_validate_declarations on public.community_items;
create trigger items_validate_declarations before insert on public.community_items for each row execute function public.enforce_upload_declarations();

-- Uploads are refused while the person is banned or suspended (same body as 0012 + the checks).
create or replace function public.limit_uploads() returns trigger language plpgsql security definer set search_path = public as $$
declare v_until timestamptz; v_susp boolean;
begin
  if coalesce(auth.role(), '') in ('anon', 'authenticated') then
    -- Same as enforce_upload_declarations: no state of another account (ban, suspension, pending count) before the ownership check.
    if new.author is distinct from auth.uid() then
      raise exception 'new row violates row-level security policy for table "community_items"' using errcode = '42501';
    end if;
    new.created_at := now();
    select upload_banned_until, suspended into v_until, v_susp from public.profiles where id = new.author;
    if coalesce(v_susp, false) then
      perform legacy.raise_code('account_suspended', 'This account is suspended.', '42501');
    end if;
    if v_until is not null and v_until > now() then
      perform legacy.raise_code('upload_banned', case when v_until >= '9999-01-01'::timestamptz then 'Uploads are disabled for this account.'
        else 'Uploads are disabled for this account until ' || to_char(v_until at time zone 'UTC', 'YYYY-MM-DD HH24:MI') || ' UTC.' end, '42501');
    end if;
  end if;
  delete from public.upload_log where author = new.author and created_at < now() - interval '7 days'; -- retention
  -- Items hidden by a copyright claim do not count: a claimant must not be able to lock an author out of uploading by claiming their items.
  if (select count(*) from public.community_items ci where ci.author = new.author and ci.status = 'pending'
        and not exists (select 1 from public.copyright_claims cc where cc.item = ci.id and cc.status in ('open', 'counter'))) >= 10 then
    raise exception 'Too many uploads pending review (maximum 10).';
  end if;
  if (select count(*) from public.upload_log where author = new.author and created_at > now() - interval '1 day') >= 20 then
    raise exception 'Daily upload limit reached (20).';
  end if;
  return new;
end $$;
revoke all on function public.limit_uploads() from public, anon, authenticated;

-- ═════════════════════════ 4. Suspension hides a creator's catalog ═════════════════════════
create or replace function legacy.is_suspended(p_uid uuid) returns boolean language sql stable security definer set search_path = public as $$
  select coalesce((select suspended from public.profiles where id = p_uid), false)
$$;
revoke all on function legacy.is_suspended(uuid) from public;
grant execute on function legacy.is_suspended(uuid) to anon, authenticated, service_role;

-- RESTRICTIVE: ANDed with the existing permissive policies. The owner still sees their own rows; service_role bypasses RLS.
drop policy if exists items_hide_suspended on public.community_items;
create policy items_hide_suspended on public.community_items as restrictive for select to anon, authenticated
  using (author = (select auth.uid()) or not legacy.is_suspended(author));
drop policy if exists pp_hide_suspended on public.public_profiles;
create policy pp_hide_suspended on public.public_profiles as restrictive for select to anon, authenticated
  using (id = (select auth.uid()) or not legacy.is_suspended(id));

-- Downloads of a suspended creator's items are not counted (same body as 0012 + the check).
create or replace function public.record_download(p_item uuid) returns void language plpgsql security definer set search_path = public as $$
declare v_uid uuid := auth.uid();
begin
  if v_uid is null then return; end if; -- signed-in people only, as before
  if not exists (
    select 1 from public.community_items i
     where i.id = p_item and i.status = 'approved' and not legacy.is_suspended(i.author)
       and (not i.is_18 or exists (select 1 from public.profiles p where p.id = v_uid and p.show_18))
  ) then return; end if;
  insert into public.download_log (person, item) values (v_uid, p_item) on conflict do nothing;
  if found then
    update public.community_items set downloads = downloads + 1 where id = p_item;
  end if;
end $$;
revoke all on function public.record_download(uuid) from public, anon;
grant execute on function public.record_download(uuid) to authenticated;

-- ═════════════════════════ 5. Copyright claims ═════════════════════════
create table if not exists public.copyright_claims (
  id uuid primary key default gen_random_uuid(),
  item uuid references public.community_items (id) on delete set null,
  item_author uuid,                                   -- snapshot, so a strike still lands if the author deletes the item
  claimant text not null check (char_length(claimant) between 1 and 200),
  claimant_contact text check (claimant_contact is null or char_length(claimant_contact) <= 200),
  description text not null check (char_length(description) between 1 and 2000),
  status text not null default 'open' check (status in ('open', 'upheld', 'rejected', 'counter')),
  reporter uuid references public.profiles (id) on delete set null,   -- who filed it in the app (null for claims entered by a moderator)
  counter_notice text check (counter_notice is null or char_length(counter_notice) <= 2000),
  counter_at timestamptz,
  restore_after timestamptz,                          -- earliest restore date after a counter-notice (14 calendar days)
  flag text check (flag is null or char_length(flag) <= 100),   -- set when the claim was filed but the item was NOT hidden (see file_copyright_claim)
  created_at timestamptz not null default now(),
  resolved_at timestamptz,
  resolver text check (resolver is null or char_length(resolver) <= 100)
);
create index if not exists copyright_claims_item_idx on public.copyright_claims (item) where status in ('open', 'counter');
alter table public.copyright_claims add column if not exists flag text check (flag is null or char_length(flag) <= 100);
create index if not exists copyright_claims_author_idx on public.copyright_claims (item_author, created_at desc);
create index if not exists copyright_claims_reporter_idx on public.copyright_claims (reporter, created_at desc);
alter table public.copyright_claims enable row level security;
revoke all on public.copyright_claims from public, anon, authenticated;
grant all on public.copyright_claims to service_role;

-- An item with an open (or counter-noticed) claim cannot be approved again by accident: resolve the claim first.
create or replace function public.block_approval_during_claim() returns trigger language plpgsql security definer set search_path = public as $$
begin
  if new.status = 'approved' and old.status is distinct from 'approved'
     and exists (select 1 from public.copyright_claims c where c.item = new.id and c.status in ('open', 'counter')) then
    perform legacy.raise_code('copyright_claim_open', 'This item has an open copyright claim: resolve the claim first.', '42501');
  end if;
  return new;
end $$;
revoke all on function public.block_approval_during_claim() from public, anon, authenticated;
drop trigger if exists items_block_claim_approval on public.community_items;
create trigger items_block_claim_approval before update of status on public.community_items for each row execute function public.block_approval_during_claim();

-- Reporting from the app: Report › Copyright. HIDES the item at once; a moderator decides. Notifies nobody.
-- Anti-abuse (a claim hides content before anybody looks at it, so it is a weapon for harassment):
--   · the reporter's profile must be at least 24 h old and have accepted the terms (any version, legacy timestamp included);
--   · calls of one reporter are serialized (profile row lock) so the daily limit cannot be beaten by parallel requests;
--   · more than 3 open claims against the SAME author in 24 h: the claim is recorded and flagged (flag = 'author_flood') but the item is NOT hidden;
--     a moderator sees the flag in claim_list and decides. The first 3 still hide, so a real rights holder with a few works is not blocked.
create or replace function public.file_copyright_claim(p_item uuid, p_description text, p_claimant text default null, p_contact text default null)
returns jsonb language plpgsql security definer set search_path = public as $$
declare
  v_uid uuid := auth.uid(); v_name text; v_susp boolean; v_since timestamptz; v_terms boolean; it record; v_id uuid; p text; v_flag text; v_hide boolean := true;
  v_desc text := btrim(coalesce(p_description, '')); v_claimant text; v_contact text := nullif(btrim(coalesce(p_contact, '')), '');
begin
  if v_uid is null then perform legacy.raise_code('sign_in_required', 'Sign in first.', '28000'); end if;
  -- FOR UPDATE: two parallel calls of the same person queue here, so the counts below are exact.
  select public_name, suspended, created_at, (terms_version is not null or terms_accepted_at is not null)
    into v_name, v_susp, v_since, v_terms from public.profiles where id = v_uid for update;
  if not found then perform legacy.raise_code('profile_required', 'Create your profile first.', 'P0002'); end if;
  if v_susp then perform legacy.raise_code('account_suspended', 'This account is suspended.', '42501'); end if;
  if not v_terms then perform legacy.raise_code('terms_required', 'Accept the Terms of Use before filing a copyright claim.', '42501'); end if;
  if v_since > now() - interval '24 hours' then
    perform legacy.raise_code('profile_too_new', 'Your profile must be at least 24 hours old to file a copyright claim: try later or open a GitHub issue.', '42501');
  end if;
  v_claimant := btrim(coalesce(nullif(btrim(coalesce(p_claimant, '')), ''), v_name, ''));
  if char_length(v_desc) < 10 or char_length(v_desc) > 2000 or char_length(v_claimant) < 1 or char_length(v_claimant) > 200
     or (v_contact is not null and char_length(v_contact) > 200) then
    perform legacy.raise_code('claim_invalid', 'Describe the work and why you believe it is infringed (10 to 2000 characters); the claimant name allows 200.', '22023');
  end if;
  if (select count(*) from public.copyright_claims where reporter = v_uid and created_at > now() - interval '1 day') >= 5 then
    perform legacy.raise_code('claim_rate_limited', 'Too many claims today: try again tomorrow or open a GitHub issue.', '54000');
  end if;
  if (select count(*) from public.copyright_claims where reporter = v_uid and status = 'rejected' and resolved_at > now() - interval '30 days') >= 3 then
    perform legacy.raise_code('claims_blocked', 'Several of your recent claims were rejected: open a GitHub issue instead.', '42501');
  end if;
  select id, author, kind, status, public_path, public_icon, public_preview into it from public.community_items where id = p_item for update;
  if not found or it.status <> 'approved' then perform legacy.raise_code('item_not_found', 'That item is not available.', 'P0002'); end if;
  if it.author = v_uid then perform legacy.raise_code('claim_invalid', 'You cannot file a claim against your own content: withdraw it instead.', '22023'); end if;

  -- Per-victim limit. The advisory lock serializes claims against the same author (different reporters), so the count is exact too.
  perform pg_advisory_xact_lock(hashtextextended('copyright_claim_author:' || it.author::text, 0));
  if (select count(*) from public.copyright_claims where item_author = it.author and status in ('open', 'counter') and created_at > now() - interval '1 day') >= 3 then
    v_hide := false; v_flag := 'author_flood';
  end if;

  if v_hide then
    update public.community_items set status = 'pending', rejection_reason = 'Hidden: copyright claim pending review' where id = it.id;
    foreach p in array array[it.public_path, it.public_icon, it.public_preview] loop
      if p is not null and legacy.public_path_ok(it.kind, it.id, p) then
        perform legacy.enqueue_cleanup('publico', p, 'to_quarantine', it.id, 'hidden: copyright claim');
      end if;
    end loop;
  end if;
  insert into public.copyright_claims (item, item_author, claimant, claimant_contact, description, reporter, flag)
    values (it.id, it.author, v_claimant, v_contact, v_desc, v_uid, v_flag) returning id into v_id;
  insert into public.moderation_log (item, step, result, detail)
    values (it.id, 'human', 'review', 'copyright claim ' || v_id::text || ' filed: ' || case when v_hide then 'item hidden' else 'FLAGGED (' || v_flag || '), item NOT hidden' end);
  return jsonb_build_object('claim', v_id, 'status', 'open', 'hidden', v_hide);
end $$;
revoke all on function public.file_copyright_claim(uuid, text, text, text) from public, anon;
grant execute on function public.file_copyright_claim(uuid, text, text, text) to authenticated, service_role;

-- Moderator decisions (service_role only; the review function calls them). Idempotent guards: only open / counter claims can be resolved.
-- upheld   -> a strike for the author of the item; 3 strikes = permanent upload ban (upload_banned_until = 9999-12-31).
-- rejected -> no strike. (Restoring the item is done by the review function: publish() re-verifies the pinned hashes.)
create or replace function public.resolve_claim(p_claim uuid, p_outcome text, p_resolver text default null) returns jsonb
language plpgsql security definer set search_path = public as $$
declare c public.copyright_claims%rowtype; v_strikes smallint; v_until timestamptz;
begin
  if p_outcome not in ('upheld', 'rejected') then perform legacy.raise_code('claim_invalid', 'Outcome must be upheld or rejected.', '22023'); end if;
  select * into c from public.copyright_claims where id = p_claim for update;
  if not found then perform legacy.raise_code('claim_not_found', 'Claim not found.', 'P0002'); end if;
  if c.status not in ('open', 'counter') then perform legacy.raise_code('claim_not_open', 'This claim was already resolved.', '22023'); end if;
  update public.copyright_claims set status = p_outcome, resolved_at = now(), resolver = left(coalesce(p_resolver, ''), 100) where id = p_claim;
  if p_outcome = 'upheld' and c.item_author is not null then
    update public.profiles
       set strikes = least(strikes + 1, 1000)::smallint,
           upload_banned_until = case when strikes + 1 >= 3 then '9999-12-31 00:00:00+00'::timestamptz else upload_banned_until end
     where id = c.item_author
     returning strikes, upload_banned_until into v_strikes, v_until;
  end if;
  insert into public.moderation_log (item, step, result, detail)
    values (c.item, 'human', case when p_outcome = 'upheld' then 'rejected' else 'approved' end, left('copyright claim ' || p_claim::text || ' ' || p_outcome, 300));
  return jsonb_build_object('claim', p_claim, 'status', p_outcome, 'item', c.item, 'author', c.item_author, 'strikes', v_strikes, 'upload_banned_until', v_until);
end $$;

-- The uploader answered with a counter-notice: the item stays hidden; it may be restored once restore_after has passed (14 calendar days).
create or replace function public.mark_claim_counter(p_claim uuid, p_note text) returns jsonb language plpgsql security definer set search_path = public as $$
declare c public.copyright_claims%rowtype;
begin
  select * into c from public.copyright_claims where id = p_claim for update;
  if not found then perform legacy.raise_code('claim_not_found', 'Claim not found.', 'P0002'); end if;
  if c.status <> 'open' then perform legacy.raise_code('claim_not_open', 'Only an open claim can receive a counter-notice.', '22023'); end if;
  if char_length(btrim(coalesce(p_note, ''))) not between 1 and 2000 then perform legacy.raise_code('claim_invalid', 'The counter-notice text must be 1 to 2000 characters.', '22023'); end if;
  update public.copyright_claims set status = 'counter', counter_notice = btrim(p_note), counter_at = now(), restore_after = now() + interval '14 days' where id = p_claim;
  return jsonb_build_object('claim', p_claim, 'status', 'counter', 'restore_after', now() + interval '14 days');
end $$;

-- Other moderator tools (service_role only).
create or replace function public.set_upload_ban(p_user uuid, p_until timestamptz) returns void language sql security definer set search_path = public as $$
  update public.profiles set upload_banned_until = p_until where id = p_user
$$;
create or replace function public.set_suspension(p_user uuid, p_suspended boolean) returns void language sql security definer set search_path = public as $$
  update public.profiles set suspended = coalesce(p_suspended, false) where id = p_user
$$;
-- Appeals: takes one strike back and lifts the automatic ban when the person is below 3 strikes again.
create or replace function public.reverse_strike(p_user uuid) returns jsonb language plpgsql security definer set search_path = public as $$
declare v_strikes smallint; v_until timestamptz;
begin
  update public.profiles
     set strikes = greatest(strikes - 1, 0)::smallint,
         upload_banned_until = case when greatest(strikes - 1, 0) < 3 and upload_banned_until >= '9999-01-01'::timestamptz then null else upload_banned_until end
   where id = p_user returning strikes, upload_banned_until into v_strikes, v_until;
  return jsonb_build_object('strikes', v_strikes, 'upload_banned_until', v_until);
end $$;
revoke all on function public.resolve_claim(uuid, text, text) from public, anon, authenticated;
revoke all on function public.mark_claim_counter(uuid, text) from public, anon, authenticated;
revoke all on function public.set_upload_ban(uuid, timestamptz) from public, anon, authenticated;
revoke all on function public.set_suspension(uuid, boolean) from public, anon, authenticated;
revoke all on function public.reverse_strike(uuid) from public, anon, authenticated;
grant execute on function public.resolve_claim(uuid, text, text) to service_role;
grant execute on function public.mark_claim_counter(uuid, text) to service_role;
grant execute on function public.set_upload_ban(uuid, timestamptz) to service_role;
grant execute on function public.set_suspension(uuid, boolean) to service_role;
grant execute on function public.reverse_strike(uuid) to service_role;

-- ═════════════════════════ 6. Report reasons ═════════════════════════
-- Old values stay valid (adult_content, hate_or_harassment are what the legacy view `reportes` writes); the new ones are what the new client sends.
alter table public.reports drop constraint if exists reports_reason_check;
alter table public.reports add constraint reports_reason_check check (reason in (
  'malware', 'adult_content', 'hate_or_harassment', 'copyright', 'other',
  'adult', 'hate', 'spam', 'impersonation', 'personal_data'));

-- ═════════════════════════ 7. Data export (right of access / portability) ═════════════════════════
-- The caller's OWN rows only. Nothing about other people (claimants' names and contacts are left out of the claims against you).
create or replace function public.export_my_data() returns jsonb language plpgsql stable security definer set search_path = public as $$
declare v_uid uuid := auth.uid();
begin
  if v_uid is null then perform legacy.raise_code('sign_in_required', 'Sign in first.', '28000'); end if;
  return jsonb_build_object(
    'exported_at', now(),
    'profile', (select to_jsonb(p) from public.profiles p where p.id = v_uid),
    'public_profile', (select to_jsonb(pp) from public.public_profiles pp where pp.id = v_uid),
    'items', (select coalesce(jsonb_agg(to_jsonb(i) order by i.created_at), '[]'::jsonb) from (select * from public.community_items where author = v_uid order by created_at limit 2000) i),
    'reports_filed', (select coalesce(jsonb_agg(to_jsonb(r) order by r.created_at), '[]'::jsonb) from public.reports r where r.reporter = v_uid),
    'hearts', (select coalesce(jsonb_agg(to_jsonb(h) order by h.created_at), '[]'::jsonb) from public.hearts h where h.person = v_uid),
    'follows', (select coalesce(jsonb_agg(to_jsonb(f) order by f.created_at), '[]'::jsonb) from public.follows f where f.follower = v_uid),
    'downloads', (select coalesce(jsonb_agg(to_jsonb(d) order by d.created_at), '[]'::jsonb) from public.download_log d where d.person = v_uid),
    'terms_log', (select coalesce(jsonb_agg(to_jsonb(t) - 'user_id' order by t.accepted_at), '[]'::jsonb) from public.terms_log t where t.user_id = v_uid),
    'copyright_claims_filed', (select coalesce(jsonb_agg(jsonb_build_object('id', c.id, 'item', c.item, 'claimant', c.claimant, 'claimant_contact', c.claimant_contact,
        'description', c.description, 'status', c.status, 'created_at', c.created_at, 'resolved_at', c.resolved_at) order by c.created_at), '[]'::jsonb)
        from public.copyright_claims c where c.reporter = v_uid),
    'copyright_claims_against_my_items', (select coalesce(jsonb_agg(jsonb_build_object('id', c.id, 'item', c.item, 'description', c.description, 'status', c.status,
        'created_at', c.created_at, 'resolved_at', c.resolved_at, 'restore_after', c.restore_after) order by c.created_at), '[]'::jsonb)
        from public.copyright_claims c where c.item_author = v_uid)
  );
end $$;
revoke all on function public.export_my_data() from public, anon;
grant execute on function public.export_my_data() to authenticated, service_role;

notify pgrst, 'reload schema';
commit;

-- ═════════════════════════ Checks to run after applying (read-only) ═════════════════════════
--  select public.current_terms();                                                      -- {"version":"1.0",...}
--  select terms_enforced_from, license_required_from from public.app_settings;         -- both NULL = nothing enforced yet
--  select count(*) filter (where terms_version is not null) as accepted_new, count(*) filter (where terms_accepted_at is not null) as accepted_legacy from public.profiles;
--  select id, status, created_at from public.copyright_claims where status in ('open','counter') order by created_at;
