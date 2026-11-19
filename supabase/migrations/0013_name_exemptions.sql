-- 0013: name exemptions for the project owner / trusted accounts.
-- Problem found while testing 0012 in production: the reserved-name list contains «deritu» and «lienzo» (impersonation protection), so the
-- OWNER'S OWN public name («Deritu_Kutsut») was never grandfathered (name_reviewed stayed NULL) and could not be approved either
-- (review_public_name refuses reserved names): new uploads showed an EMPTY author name. A server-only table of exempt accounts fixes it
-- without weakening the list for everybody else.
-- Apply: `npx supabase db push` (one transaction, idempotent). Rollback: supabase/rollbacks/0013_name_exemptions.down.sql.
begin;

create table if not exists public.name_exemptions (
  user_id    uuid primary key references public.profiles(id) on delete cascade,
  note       text,
  created_at timestamptz not null default now()
);
alter table public.name_exemptions enable row level security;
revoke all on public.name_exemptions from public, anon, authenticated;
grant all on public.name_exemptions to service_role;

create or replace function legacy.name_exempt(p_user uuid) returns boolean language sql stable security definer set search_path = public as $$
  select exists (select 1 from public.name_exemptions where user_id = p_user)
$$;
revoke all on function legacy.name_exempt(uuid) from public;
grant execute on function legacy.name_exempt(uuid) to service_role;

create or replace function public.guard_public_name() returns trigger language plpgsql security definer set search_path = public as $$
begin
  if coalesce(auth.role(), '') in ('anon', 'authenticated') then
    if tg_op = 'INSERT' then
      new.name_reviewed := null;
      if legacy.name_is_reserved(new.public_name) and not legacy.name_exempt(new.id) then raise exception 'That public name is not available.' using errcode = '23514'; end if;
    else
      new.name_reviewed := old.name_reviewed;
      if new.public_name is distinct from old.public_name and legacy.name_is_reserved(new.public_name) and not legacy.name_exempt(new.id) then
        raise exception 'That public name is not available.' using errcode = '23514';
      end if;
    end if;
  end if;
  return new;
end $$;
revoke all on function public.guard_public_name() from public, anon, authenticated;

create or replace function public.review_public_name(p_user uuid, p_name text) returns boolean language plpgsql security definer set search_path = public as $$
begin
  if legacy.name_is_reserved(p_name) and not legacy.name_exempt(p_user) then return false; end if;
  update public.profiles set name_reviewed = public_name where id = p_user and public_name = p_name;
  return found;
end $$;
revoke all on function public.review_public_name(uuid, text) from public, anon, authenticated;
grant execute on function public.review_public_name(uuid, text) to service_role;

-- The project owner's account (the one that publishes the official examples): exempt, and its current name counts as reviewed.
insert into public.name_exemptions (user_id, note)
  select id, 'project owner' from public.profiles where public_name = 'Deritu_Kutsut'
on conflict do nothing;
-- (the after-update trigger copies the reviewed name to the owner's items and public profile)
update public.profiles set name_reviewed = public_name
 where name_reviewed is null and id in (select user_id from public.name_exemptions);

commit;
