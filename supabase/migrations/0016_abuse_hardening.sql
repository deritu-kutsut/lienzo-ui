-- Lienzo UI · Community. Abuse hardening (migration 0016). Builds on 0015 (0001-0015 are already applied in production; none of them is edited).
--
-- WHAT IT FIXES (each item has a test that reproduces the attack against 0015 and shows it fails after this file: tests/0016_abuse_hardening_pglite.mjs)
--   M1  profiles.created_at / terms_accepted_at were client-writable on INSERT, so a person could back-date their profile and pass the «profile >= 24 h
--       old» rule of file_copyright_claim. Now guard_profile_moderation (INSERT and UPDATE) forces created_at = now() on INSERT and pins it on UPDATE;
--       terms_accepted_at is STAMPED by the server (any client-supplied value becomes now(), and once set it can not be changed by the person). Old clients
--       (<= 1.6.0) keep working: they still create a profile with `acepto_terminos_en` (it is stored as now()) and the legacy views are untouched.
--       The «age of an account» is now legacy.account_since(uid) = greatest(profiles.created_at, auth.users.created_at): the real sign-up time of the
--       auth account can not be forged. Existing profiles whose created_at is OLDER than their auth account (only possible by forging it) are repaired.
--   M2  hide_on_reports: 3 throw-away accounts hid any approved item and 2 malware/copyright reports retired the public files. Now
--       · reports_create (RLS) needs legacy.can_report(): profile >= 24 h old (real age, see M1), not suspended, not upload-banned and, only when
--         app_settings.terms_enforced_from is set and in force, the terms accepted through accept_terms (NULL flag = not required: old clients keep reporting);
--       · a trigger caps reports at 10 per account per day and stamps created_at = now() for people (the legacy view let the client choose it);
--       · hide_on_reports counts WEIGHTED reports: account >= 7 days old = 1.0, 24 h .. 7 days = 0.5, younger / suspended / the author themself = 0.
--         The item is hidden (status pending, «pending human review») when the score reaches 3. Public files are NEVER retired automatically by reports any
--         more: hiding is enough until a person looks (a moderator rejects it, or re-approves it). Direct file URLs stay reachable until then.
--   M3  set_suspension now takes the creator's approved items down (status pending, reason «Hidden: author suspended») and queues their public files
--       with the existing `to_quarantine` action (no Edge Function change: the drain moves them back to quarantine). Lifting the suspension does NOT
--       republish: the items stay pending with the reason «Pending re-review: suspension lifted» and a moderator approves them again from the moderation
--       web (publish() re-verifies the pinned hashes). They do not count in the 10-pending upload cap.
--   B3  A person with a ban in force (upload_banned_until in the future) can neither file copyright claims nor reports.
--   P13 EXECUTE of legacy.raise_code, legacy.license_ok and legacy.source_url_ok is revoked from anon (nothing anonymous calls them: they run inside
--       SECURITY DEFINER code or a CHECK on a table anon can not write). legacy.is_suspended keeps anon on purpose: the RESTRICTIVE select policies
--       items_hide_suspended / pp_hide_suspended apply to anon and call it as the caller. The schema `legacy` must never be exposed by the Data API.
--   P6  Ban evasion by deleting the account: server-only table `profile_tombstones` (HMAC-SHA256 of the OAuth identity `provider:sub`, keyed with a secret
--       that lives only in `tombstone_secret`) with the strikes / ban / suspension of an account that had any. It is written by a BEFORE DELETE trigger on
--       auth.users (and on profiles) and read by a BEFORE INSERT trigger on profiles, which gives the new profile the old strikes / ban / suspension.
--       No Edge Function changes. Tombstones expire after 3 years (permanent bans never). Needs the Privacy Policy to say so. The snapshot never blocks a deletion.
--   P10 export_my_data no longer returns the claimant's free text (`description`) of claims against your items.
--
-- WHAT STAYS EXACTLY AS IT WAS: no column, table grant, view or function signature is removed; the legacy Spanish views are untouched.
-- HOW TO APPLY: back up, `npx supabase db push` (one transaction, idempotent). No Edge Function has to be redeployed for it.
-- HOW TO ROLL BACK: supabase/rollbacks/0016_abuse_hardening.down.sql.
begin;

-- ═════════════════════════ 0. Helpers ═════════════════════════
-- Age of an account: the later of the profile row and the auth account (auth.users.created_at can not be written by people).
create or replace function legacy.account_since(p_uid uuid) returns timestamptz language plpgsql stable security definer set search_path = public as $$
declare v_p timestamptz; v_a timestamptz;
begin
  select created_at into v_p from public.profiles where id = p_uid;
  begin
    execute 'select created_at from auth.users where id = $1' into v_a using p_uid;
  exception when undefined_table or undefined_column then v_a := null; end;
  return greatest(v_p, v_a);
end $$;
revoke all on function legacy.account_since(uuid) from public, anon, authenticated;
grant execute on function legacy.account_since(uuid) to service_role;

-- HMAC-SHA256 with the built-in sha256() (no pgcrypto needed: its schema differs between environments). RFC 2104 / RFC 4231.
create or replace function legacy.hmac_sha256(p_key bytea, p_msg bytea) returns bytea language plpgsql immutable as $$
declare k bytea := p_key; ik bytea; ok bytea; i integer;
begin
  if length(k) > 64 then k := sha256(k); end if;
  k := k || decode(repeat('00', 64 - length(k)), 'hex');
  ik := k; ok := k;
  for i in 0..63 loop
    ik := set_byte(ik, i, get_byte(k, i) # 54);   -- 0x36
    ok := set_byte(ok, i, get_byte(k, i) # 92);   -- 0x5c
  end loop;
  return sha256(ok || sha256(ik || p_msg));
end $$;
revoke all on function legacy.hmac_sha256(bytea, bytea) from public, anon, authenticated;
grant execute on function legacy.hmac_sha256(bytea, bytea) to service_role;

-- ═════════════════════════ 1. M1: created_at / terms_accepted_at are server-owned ═════════════════════════
-- Repair rows whose created_at was forged to be older than the auth account itself (impossible legitimately).
do $$
begin
  execute 'update public.profiles p set created_at = u.created_at from auth.users u where u.id = p.id and p.created_at < u.created_at';
exception when undefined_table or undefined_column then null;
end $$;

-- SECURITY INVOKER on purpose (see 0015): current_user is the caller's role; inside the SECURITY DEFINER functions it is their owner.
create or replace function public.guard_profile_moderation() returns trigger language plpgsql set search_path = public as $$
begin
  if current_user in ('anon', 'authenticated') then
    if tg_op = 'INSERT' then
      new.terms_version := null; new.strikes := 0; new.upload_banned_until := null; new.suspended := false;
      new.created_at := now();                                                         -- M1: the server decides when the profile began
      new.terms_accepted_at := case when new.terms_accepted_at is null then null else now() end;   -- old clients send it: stamped, never back-dated
    else
      new.terms_version := old.terms_version; new.strikes := old.strikes;
      new.upload_banned_until := old.upload_banned_until; new.suspended := old.suspended;
      new.created_at := old.created_at;
      -- once accepted it can not be changed or cleared by the person; a first acceptance (old clients, view perfiles) is stamped now()
      new.terms_accepted_at := case when old.terms_accepted_at is not null then old.terms_accepted_at
                                    when new.terms_accepted_at is null then null else now() end;
    end if;
  end if;
  return new;
end $$;
revoke all on function public.guard_profile_moderation() from public, anon, authenticated;

-- ═════════════════════════ 2. P6: tombstones against ban evasion by deleting the account ═════════════════════════
create table if not exists public.tombstone_secret (
  singleton boolean primary key default true check (singleton),
  secret bytea not null
);
insert into public.tombstone_secret (singleton, secret)
  values (true, decode(replace(gen_random_uuid()::text || gen_random_uuid()::text, '-', ''), 'hex')) on conflict (singleton) do nothing;

create table if not exists public.profile_tombstones (
  key_hash text primary key check (key_hash ~ '^[0-9a-f]{64}$'),   -- HMAC-SHA256 of «provider:sub»; the identity itself is never stored
  strikes smallint not null default 0 check (strikes >= 0),
  upload_banned_until timestamptz,
  suspended boolean not null default false,
  created_at timestamptz not null default now(),
  expires_at timestamptz not null default now() + interval '3 years'   -- 'infinity' for permanent bans
);
alter table public.tombstone_secret enable row level security;
alter table public.profile_tombstones enable row level security;
revoke all on public.tombstone_secret, public.profile_tombstones from public, anon, authenticated;
grant all on public.tombstone_secret, public.profile_tombstones to service_role;

-- Hashes of the OAuth identities of an auth user. auth.identities may not exist (tests) or have other columns: never fails.
create or replace function legacy.identity_hashes(p_uid uuid) returns text[] language plpgsql stable security definer set search_path = public as $$
declare v_keys text[]; v_secret bytea; k text; out text[] := '{}';
begin
  if to_regclass('auth.identities') is null then return out; end if;
  begin
    execute 'select coalesce(array_agg(distinct provider || '':'' || (identity_data ->> ''sub'')), ''{}'') from auth.identities where user_id = $1 and identity_data ->> ''sub'' is not null'
      into v_keys using p_uid;
  exception when others then return out;
  end;
  select secret into v_secret from public.tombstone_secret where singleton;
  if v_secret is null then return out; end if;
  foreach k in array coalesce(v_keys, '{}') loop
    out := out || encode(legacy.hmac_sha256(v_secret, convert_to(k, 'UTF8')), 'hex');
  end loop;
  return out;
end $$;
revoke all on function legacy.identity_hashes(uuid) from public, anon, authenticated;
grant execute on function legacy.identity_hashes(uuid) to service_role;

-- Snapshot of an account that has strikes, a ban in force or a suspension. Never raises (a deletion must always go through).
create or replace function legacy.snapshot_tombstone(p_uid uuid) returns void language plpgsql security definer set search_path = public as $$
declare p record; h text;
begin
  select strikes, upload_banned_until, suspended into p from public.profiles where id = p_uid;
  if not found then return; end if;
  if p.strikes = 0 and not p.suspended and (p.upload_banned_until is null or p.upload_banned_until <= now()) then return; end if;
  delete from public.profile_tombstones where expires_at < now();
  foreach h in array legacy.identity_hashes(p_uid) loop
    insert into public.profile_tombstones (key_hash, strikes, upload_banned_until, suspended, expires_at)
    values (h, p.strikes, p.upload_banned_until, p.suspended,
            case when p.upload_banned_until >= '9999-01-01'::timestamptz then 'infinity'::timestamptz
                 else greatest(now() + interval '3 years', coalesce(p.upload_banned_until, now())) end)
    on conflict (key_hash) do update set
      strikes = greatest(public.profile_tombstones.strikes, excluded.strikes),
      upload_banned_until = greatest(public.profile_tombstones.upload_banned_until, excluded.upload_banned_until),
      suspended = public.profile_tombstones.suspended or excluded.suspended,
      expires_at = greatest(public.profile_tombstones.expires_at, excluded.expires_at);
  end loop;
exception when others then
  raise warning 'tombstone snapshot skipped: %', sqlerrm;
end $$;
revoke all on function legacy.snapshot_tombstone(uuid) from public, anon, authenticated;
grant execute on function legacy.snapshot_tombstone(uuid) to service_role;

create or replace function legacy.tombstone_on_delete() returns trigger language plpgsql security definer set search_path = public as $$
begin
  perform legacy.snapshot_tombstone(old.id);
  return old;
exception when others then
  return old;
end $$;
revoke all on function legacy.tombstone_on_delete() from public, anon, authenticated;

-- Applies a tombstone to a NEW profile (runs after profiles_guard_moderation: «t» sorts after «g»; the guard zeroes the columns for people first).
create or replace function legacy.tombstone_restore() returns trigger language plpgsql security definer set search_path = public as $$
declare t record;
begin
  select max(strikes) as strikes, max(upload_banned_until) as until, coalesce(bool_or(suspended), false) as susp into t
    from public.profile_tombstones where key_hash = any (legacy.identity_hashes(new.id)) and expires_at > now();
  if t.strikes is not null then
    new.strikes := greatest(new.strikes, t.strikes)::smallint;
    new.upload_banned_until := greatest(new.upload_banned_until, t.until);
    new.suspended := new.suspended or t.susp;
  end if;
  return new;
exception when others then
  raise warning 'tombstone restore skipped: %', sqlerrm;
  return new;
end $$;
revoke all on function legacy.tombstone_restore() from public, anon, authenticated;

drop trigger if exists profiles_tombstone_snapshot on public.profiles;
create trigger profiles_tombstone_snapshot before delete on public.profiles for each row execute function legacy.tombstone_on_delete();
drop trigger if exists profiles_tombstone_restore on public.profiles;
create trigger profiles_tombstone_restore before insert on public.profiles for each row execute function legacy.tombstone_restore();

-- The identities still exist when auth.users is deleted (the cascade to auth.identities / profiles comes after a BEFORE trigger), so the hook is on auth.users.
-- If this role may not create it (it normally can), the profiles trigger above still covers every deletion that keeps the identities.
do $$
begin
  execute 'drop trigger if exists lienzo_tombstone_on_user_delete on auth.users';
  execute 'create trigger lienzo_tombstone_on_user_delete before delete on auth.users for each row execute function legacy.tombstone_on_delete()';
exception when insufficient_privilege then
  raise warning 'could not create the tombstone trigger on auth.users (insufficient privilege): ban evasion by account deletion is NOT covered. See MIGRATION.md.';
end $$;

-- ═════════════════════════ 3. Copyright claims: real age, strict terms, no claims while banned (M1, B3) ═════════════════════════
create or replace function public.file_copyright_claim(p_item uuid, p_description text, p_claimant text default null, p_contact text default null)
returns jsonb language plpgsql security definer set search_path = public as $$
declare
  v_uid uuid := auth.uid(); v_name text; v_susp boolean; v_ban timestamptz; v_since timestamptz; v_terms boolean; it record; v_id uuid; p text; v_flag text; v_hide boolean := true;
  v_desc text := btrim(coalesce(p_description, '')); v_claimant text; v_contact text := nullif(btrim(coalesce(p_contact, '')), '');
begin
  if v_uid is null then perform legacy.raise_code('sign_in_required', 'Sign in first.', '28000'); end if;
  -- FOR UPDATE: two parallel calls of the same person queue here, so the counts below are exact.
  select public_name, suspended, upload_banned_until, (terms_version is not null)
    into v_name, v_susp, v_ban, v_terms from public.profiles where id = v_uid for update;
  if not found then perform legacy.raise_code('profile_required', 'Create your profile first.', 'P0002'); end if;
  v_since := legacy.account_since(v_uid);
  if v_susp then perform legacy.raise_code('account_suspended', 'This account is suspended.', '42501'); end if;
  if v_ban is not null and v_ban > now() then
    perform legacy.raise_code('upload_banned', 'Accounts with an upload ban can not file claims.', '42501');
  end if;
  -- terms_version is only ever set by accept_terms(): the legacy timestamp (which old clients write) is not enough to file a claim.
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

-- ═════════════════════════ 4. Reports (M2, B3) ═════════════════════════
-- Who may report. NULL terms flag = the terms are not required (old clients keep reporting); the age and the ban are always checked.
create or replace function legacy.can_report() returns boolean language plpgsql stable security definer set search_path = public as $$
declare v_uid uuid := auth.uid(); p record; s public.app_settings%rowtype;
begin
  if v_uid is null then return false; end if;
  select suspended, upload_banned_until, terms_version into p from public.profiles where id = v_uid;
  if not found or p.suspended then return false; end if;
  if p.upload_banned_until is not null and p.upload_banned_until > now() then return false; end if;
  if legacy.account_since(v_uid) > now() - interval '24 hours' then return false; end if;
  select * into s from public.app_settings where singleton;
  if s.terms_enforced_from is not null and s.terms_enforced_from <= now() and p.terms_version is null then return false; end if;
  return true;
end $$;
revoke all on function legacy.can_report() from public, anon;
grant execute on function legacy.can_report() to authenticated, service_role;

drop policy if exists reports_create on public.reports;
drop policy if exists reportes_crear on public.reports;
create policy reports_create on public.reports for insert to authenticated
  with check (reporter = (select auth.uid()) and legacy.can_report());

-- Weight of one report = how much the reporter's account is worth. Old accounts count fully; fresh ones half; the rest nothing.
create or replace function legacy.report_weight(p_uid uuid) returns numeric language sql stable security definer set search_path = public as $$
  select coalesce((select case when p.suspended then 0
                               when legacy.account_since(p.id) <= now() - interval '7 days' then 1.0
                               when legacy.account_since(p.id) <= now() - interval '24 hours' then 0.5
                               else 0 end
                     from public.profiles p where p.id = p_uid), 0)
$$;
revoke all on function legacy.report_weight(uuid) from public, anon, authenticated;
grant execute on function legacy.report_weight(uuid) to service_role;

-- Per-account daily cap and a server-side timestamp (people only; imports and service_role pass through).
create or replace function public.limit_reports() returns trigger language plpgsql security definer set search_path = public as $$
begin
  if coalesce(auth.role(), '') in ('anon', 'authenticated') then
    new.created_at := now();
    if new.reporter is not distinct from auth.uid() then
      perform 1 from public.profiles where id = new.reporter for update;   -- serializes one person's parallel reports
      if (select count(*) from public.reports where reporter = new.reporter and created_at > now() - interval '1 day') >= 10 then
        perform legacy.raise_code('report_rate_limited', 'Too many reports today: try again tomorrow.', '54000');
      end if;
    end if;
  end if;
  return new;
end $$;
revoke all on function public.limit_reports() from public, anon, authenticated;
drop trigger if exists reports_limit on public.reports;
create trigger reports_limit before insert on public.reports for each row execute function public.limit_reports();

-- Hides (never retires): a weighted score of 3 since the last review hides the item until a person decides. The public files stay where they are.
create or replace function public.hide_on_reports() returns trigger language plpgsql security definer set search_path = public as $$
declare it record; since timestamptz; score numeric;
begin
  select id, author, reviewed_at into it from public.community_items where id = new.item;
  if not found then return new; end if;
  since := coalesce(it.reviewed_at, '-infinity'::timestamptz);
  select coalesce(sum(legacy.report_weight(r.reporter)), 0) into score
    from public.reports r where r.item = new.item and r.created_at > since and r.reporter is distinct from it.author;
  if score >= 3 then
    update public.community_items set status = 'pending', rejection_reason = 'Hidden by reports: pending human review'
      where id = new.item and status = 'approved';
  end if;
  return new;
end $$;
revoke all on function public.hide_on_reports() from public, anon, authenticated;

-- ═════════════════════════ 5. M3: suspension takes the creator's files down ═════════════════════════
create or replace function public.set_suspension(p_user uuid, p_suspended boolean) returns void language plpgsql security definer set search_path = public as $$
declare v_prev boolean; v_new boolean := coalesce(p_suspended, false); it record; p text;
begin
  select suspended into v_prev from public.profiles where id = p_user for update;
  if not found then return; end if;
  update public.profiles set suspended = v_new where id = p_user;
  if v_new then
    -- (also when it was already suspended: idempotent, it catches anything approved in between)
    for it in select id, kind, public_path, public_icon, public_preview from public.community_items where author = p_user and status = 'approved' for update loop
      update public.community_items set status = 'pending', rejection_reason = 'Hidden: author suspended' where id = it.id;
      foreach p in array array[it.public_path, it.public_icon, it.public_preview] loop
        if p is not null and legacy.public_path_ok(it.kind, it.id, p) then
          perform legacy.enqueue_cleanup('publico', p, 'to_quarantine', it.id, 'hidden: author suspended');
        end if;
      end loop;
      insert into public.moderation_log (item, step, result, detail) values (it.id, 'human', 'review', 'author suspended: item hidden, public files queued for quarantine');
    end loop;
  elsif v_prev then
    -- Not republished automatically: a moderator approves them again (the files are in quarantine, publish() re-verifies the hashes).
    for it in select id from public.community_items where author = p_user and status = 'pending' and rejection_reason = 'Hidden: author suspended' for update loop
      update public.community_items set rejection_reason = 'Pending re-review: suspension lifted' where id = it.id;
      insert into public.moderation_log (item, step, result, detail) values (it.id, 'human', 'review', 'suspension lifted: item waits for re-approval');
    end loop;
  end if;
end $$;
revoke all on function public.set_suspension(uuid, boolean) from public, anon, authenticated;
grant execute on function public.set_suspension(uuid, boolean) to service_role;

-- limit_uploads: items hidden because of a suspension do not count towards the 10-pending cap either (same body as 0015 + that condition).
create or replace function public.limit_uploads() returns trigger language plpgsql security definer set search_path = public as $$
declare v_until timestamptz; v_susp boolean;
begin
  if coalesce(auth.role(), '') in ('anon', 'authenticated') then
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
  if (select count(*) from public.community_items ci where ci.author = new.author and ci.status = 'pending'
        and coalesce(ci.rejection_reason, '') not in ('Hidden: author suspended', 'Pending re-review: suspension lifted')
        and not exists (select 1 from public.copyright_claims cc where cc.item = ci.id and cc.status in ('open', 'counter'))) >= 10 then
    raise exception 'Too many uploads pending review (maximum 10).';
  end if;
  if (select count(*) from public.upload_log where author = new.author and created_at > now() - interval '1 day') >= 20 then
    raise exception 'Daily upload limit reached (20).';
  end if;
  return new;
end $$;
revoke all on function public.limit_uploads() from public, anon, authenticated;

-- ═════════════════════════ 6. P13: anon does not need these ═════════════════════════
revoke execute on function legacy.raise_code(text, text, text) from anon;
revoke execute on function legacy.license_ok(text) from anon;
revoke execute on function legacy.source_url_ok(text) from anon;
-- legacy.is_suspended keeps anon ON PURPOSE: the restrictive select policies call it as the caller (anon browses the catalog).

-- ═════════════════════════ 7. P10: export without the claimant's free text ═════════════════════════
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
    -- Claims AGAINST you: no claimant name / contact and no free-text description (it can contain the claimant's own email or phone).
    -- The substance of a claim reaches you through the moderator who handles it, not through this export.
    'copyright_claims_against_my_items', (select coalesce(jsonb_agg(jsonb_build_object('id', c.id, 'item', c.item, 'status', c.status,
        'created_at', c.created_at, 'resolved_at', c.resolved_at, 'restore_after', c.restore_after) order by c.created_at), '[]'::jsonb)
        from public.copyright_claims c where c.item_author = v_uid)
  );
end $$;
revoke all on function public.export_my_data() from public, anon;
grant execute on function public.export_my_data() to authenticated, service_role;

notify pgrst, 'reload schema';
commit;

-- ═════════════════════════ Checks to run after applying (read-only) ═════════════════════════
--  select policyname, with_check from pg_policies where schemaname = 'public' and tablename = 'reports';           -- reports_create mentions legacy.can_report()
--  select tgname from pg_trigger where tgrelid = 'auth.users'::regclass and tgname = 'lienzo_tombstone_on_user_delete';   -- one row (else see the warning)
--  select count(*) from public.profile_tombstones;                                                                   -- 0 until a flagged account is deleted
--  select id, title, rejection_reason from public.community_items where rejection_reason in ('Hidden: author suspended', 'Pending re-review: suspension lifted');
