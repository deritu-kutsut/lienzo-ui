-- 0014: your own profile through a view, and the pending-review flags / updated_at of public_profiles stop being readable by other people.
-- Problem (audit, "NOT FIXED" in 0012): signed-in people could read avatar_pending / banner_pending / updated_at of EVERY creator through the real table
--   public.public_profiles, because the 1.5 client read its own row with `public_profiles?select=*` (a `select=*` fails when a column grant is missing).
-- Fix:
--   1. View public.my_public_profile (security_invoker): only the CALLER's row, all columns including the pending flags. The three columns that are about to
--      lose their grant come from the same definer helpers the legacy view perfiles_publicos already uses (legacy.pp_pending / legacy.pp_updated, which
--      reveal a value only to its owner or to service_role) — a security_invoker view cannot name a column that the caller cannot read.
--   2. REVOKE SELECT on the table from anon and authenticated, then GRANT it back on the columns people may see (the same list that anon had since 0006/0007).
-- What does NOT change (old apps 1.0 – 1.4.1): they read the legacy views perfiles_publicos / perfiles. Those views only name the public columns of the table
--   (their pending flags and actualizado_en come from the definer helpers), so they keep working. service_role keeps full access (Edge Functions, triggers).
-- DEPLOY ORDER — apply AFTER (or together with) releasing the 1.5.x client:
--   · the 1.5.x client reads my_public_profile and falls back to explicit columns of the table while the view does not exist (so applying this file a bit
--     after the release is safe: pending badges keep working);
--   · NO released client may read `public_profiles?select=*` once this is applied (it would get a permission error). 1.5.0 pre-release builds did; the
--     released 1.0 – 1.4.x never touched this table directly.
-- Apply: `npx supabase db push` (one transaction, idempotent). Rollback: supabase/rollbacks/0014_own_profile_view.down.sql.
begin;

-- The helpers were created in 0010 with execute for anon / authenticated; restated so this file does not depend on that.
grant execute on function legacy.pp_pending(uuid, text), legacy.pp_updated(uuid) to anon, authenticated, service_role;

create or replace view public.my_public_profile with (security_invoker = true) as
  select id, public_name, bio, github, links, public_avatar, public_banner,
         legacy.pp_pending(id, 'avatar') as avatar_pending, legacy.pp_pending(id, 'banner') as banner_pending,
         legacy.pp_updated(id) as updated_at, followers, hearts_received
  from public.public_profiles
  where id = (select auth.uid());
revoke all on public.my_public_profile from public, anon, authenticated;
grant select on public.my_public_profile to authenticated;
grant all on public.my_public_profile to service_role;

-- Table: people read the public columns only. (Table-level REVOKE also drops any column grant, hence the explicit re-grant below.)
revoke select on public.public_profiles from anon, authenticated;
grant select (id, public_name, bio, github, links, public_avatar, public_banner, followers, hearts_received) on public.public_profiles to anon, authenticated;
grant all on public.public_profiles to service_role;

notify pgrst, 'reload schema';
commit;
