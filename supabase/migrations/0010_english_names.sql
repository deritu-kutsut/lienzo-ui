-- Lienzo UI · Community. English names for tables, columns, enums and functions, WITHOUT breaking the Spanish API.
--
-- WHY
--   The desktop app (1.0 – 1.3.3) talks to PostgREST with the Spanish names (items.tipo, perfiles_publicos.nombre_publico,
--   rpc/registrar_descarga, …). Those versions are in the wild and must keep working. The new client (wire format v2) and the
--   Edge Functions use the English names. So this migration is ADDITIVE: it renames the real objects to English and puts a
--   compatibility layer under the old Spanish names.
--
-- WHAT CHANGES
--   real tables (English)            legacy name, now a VIEW (security_invoker) with the old Spanish columns
--   ─────────────────────────────    ───────────────────────────────────────────────────────────────────────
--   community_items  (was items)     items
--   profiles         (was perfiles)  perfiles
--   public_profiles                  perfiles_publicos
--   reports                          reportes
--   hearts                           corazones
--   follows                          seguidores
--   moderation_log                   moderacion_log
--   moderation_tokens                tokens_moderacion
--   enums: item_kind (image, font, template, element, theme, plugin), item_status (pending, approved, rejected, withdrawn)
--   rpc registrar_descarga(p_item)   is now a wrapper of record_download(p_item)
--   Storage buckets «cuarentena» and «publico» are NOT renamed on purpose: their names are inside the URLs that old clients
--   call (/storage/v1/object/cuarentena/…, /object/public/publico/…) and Storage cannot alias a bucket.
--
-- HOW THE COMPAT LAYER STAYS SAFE (no access is widened)
--   · Every legacy view is `security_invoker = true`: the caller's own grants and the base table's RLS policies apply, exactly as before.
--   · Views that only rename columns (perfiles, corazones, seguidores) are auto-updatable; the base RLS checks (WITH CHECK / USING)
--     run on the base table.
--   · Views that translate values (items, reportes, moderacion_log, tokens_moderacion) use INSTEAD OF triggers that are SECURITY INVOKER
--     (never definer) and update ONLY the columns that actually changed, so column-level grants on the base table still reject
--     what they rejected before (e.g. an author PATCHing `estado` still gets 42501).
--   · Grants on each legacy view replicate the old table grants one to one.
--   · perfiles_publicos: anon only has column-level SELECT on the base table (no *_pendiente, no actualizado_en). A security_invoker view needs
--     the invoker to be able to read every column it references, so those three columns are served by SECURITY DEFINER helpers in
--     the non-exposed schema `legacy`, which only return the real value to the profile's owner or to service_role (false/null for anybody else).
--   · The `legacy` schema is not in PostgREST's exposed schemas: nothing in it is reachable through /rest/v1 (rpc included).
--   · One hole is closed on the way: the upload policy now also requires hearts = 0 (a client could insert an item with an arbitrary counter).
--
-- HOW TO APPLY  (order matters, see supabase/MIGRATION.md)
--   1. `npx supabase db push` (or paste this file in the SQL editor). It is a single transaction: it either applies fully or not at all.
--   2. Deploy the Edge Functions (they already talk English to the database).
--   3. Release the new client.
--   Safe to re-run: every rename is guarded, data updates are idempotent, views/functions use CREATE OR REPLACE.
--   Take a backup (Dashboard → Database → Backups) first; the data itself is not rewritten except the enum labels,
--   report reasons, moderation_log labels and token scopes (all converted to English; the legacy views translate them back).
--
-- HOW TO ROLL BACK
--   Run supabase/rollbacks/0010_english_names.down.sql: it drops the compat views and the English helper functions and renames
--   everything back to the Spanish names (data labels are converted back too). Then redeploy the PREVIOUS version of the Edge Functions
--   (git tag/commit before this change). Because old clients keep working through the views, you normally do NOT need to roll back
--   when only the new client has a problem: just stop distributing it.

-- ═════════════════════════ 0. Helper schema (not exposed through the API) ═════════════════════════
create schema if not exists legacy;
grant usage on schema legacy to anon, authenticated, service_role;

-- ═════════════════════════ 1. Real objects: rename types, tables, columns, policies, triggers, functions ═════════════════════════
do $do$
declare r record;
begin
  -- Enum types and labels.
  if to_regtype('public.tipo_item') is not null and to_regtype('public.item_kind') is null then
    alter type public.tipo_item rename to item_kind;
  end if;
  if to_regtype('public.estado_item') is not null and to_regtype('public.item_status') is null then
    alter type public.estado_item rename to item_status;
  end if;
  for r in select * from (values
    ('item_kind', 'imagen', 'image'), ('item_kind', 'fuente', 'font'), ('item_kind', 'plantilla', 'template'),
    ('item_kind', 'elemento', 'element'), ('item_kind', 'tema', 'theme'),
    ('item_status', 'pendiente', 'pending'), ('item_status', 'aprobado', 'approved'),
    ('item_status', 'rechazado', 'rejected'), ('item_status', 'retirado', 'withdrawn')
  ) as v(t, o, n) loop
    if exists (select 1 from pg_enum e where e.enumtypid = to_regtype('public.' || r.t) and e.enumlabel = r.o) then
      execute format('alter type public.%I rename value %L to %L', r.t, r.o, r.n);
    end if;
  end loop;

  -- Tables («items» becomes community_items to free the name for the legacy view).
  for r in select * from (values
    ('items', 'community_items'), ('perfiles', 'profiles'), ('perfiles_publicos', 'public_profiles'), ('reportes', 'reports'),
    ('corazones', 'hearts'), ('seguidores', 'follows'), ('moderacion_log', 'moderation_log'), ('tokens_moderacion', 'moderation_tokens')
  ) as v(o, n) loop
    if exists (select 1 from pg_class c where c.relnamespace = 'public'::regnamespace and c.relname = r.o and c.relkind = 'r')
       and to_regclass('public.' || r.n) is null then
      execute format('alter table public.%I rename to %I', r.o, r.n);
    end if;
  end loop;

  -- Columns.
  for r in select * from (values
    ('community_items', 'autor', 'author'), ('community_items', 'tipo', 'kind'), ('community_items', 'titulo', 'title'),
    ('community_items', 'descripcion', 'description'), ('community_items', 'etiquetas', 'tags'), ('community_items', 'es_18', 'is_18'),
    ('community_items', 'estado', 'status'), ('community_items', 'motivo_rechazo', 'rejection_reason'),
    ('community_items', 'ruta_cuarentena', 'quarantine_path'), ('community_items', 'ruta_publica', 'public_path'),
    ('community_items', 'descargas', 'downloads'), ('community_items', 'creado_en', 'created_at'),
    ('community_items', 'revisado_en', 'reviewed_at'), ('community_items', 'autor_nombre', 'author_name'),
    ('community_items', 'icono_cuarentena', 'quarantine_icon'), ('community_items', 'vista_cuarentena', 'quarantine_preview'),
    ('community_items', 'icono_publico', 'public_icon'), ('community_items', 'vista_publica', 'public_preview'),
    ('community_items', 'corazones', 'hearts'),
    ('profiles', 'nombre_publico', 'public_name'), ('profiles', 'mostrar_18', 'show_18'),
    ('profiles', 'acepto_terminos_en', 'terms_accepted_at'), ('profiles', 'creado_en', 'created_at'),
    ('profiles', 'declara_mayor_18_en', 'declared_adult_at'),
    ('public_profiles', 'nombre_publico', 'public_name'), ('public_profiles', 'enlaces', 'links'),
    ('public_profiles', 'avatar_publico', 'public_avatar'), ('public_profiles', 'banner_publico', 'public_banner'),
    ('public_profiles', 'avatar_pendiente', 'avatar_pending'), ('public_profiles', 'banner_pendiente', 'banner_pending'),
    ('public_profiles', 'actualizado_en', 'updated_at'), ('public_profiles', 'seguidores', 'followers'),
    ('public_profiles', 'corazones_recibidos', 'hearts_received'),
    ('reports', 'reportante', 'reporter'), ('reports', 'motivo', 'reason'), ('reports', 'detalle', 'detail'), ('reports', 'creado_en', 'created_at'),
    ('hearts', 'persona', 'person'), ('hearts', 'creado_en', 'created_at'),
    ('follows', 'seguidor', 'follower'), ('follows', 'seguido', 'followed'), ('follows', 'creado_en', 'created_at'),
    ('moderation_log', 'paso', 'step'), ('moderation_log', 'resultado', 'result'), ('moderation_log', 'detalle', 'detail'),
    ('moderation_log', 'creado_en', 'created_at'),
    ('moderation_tokens', 'nombre', 'name'), ('moderation_tokens', 'huella', 'fingerprint'), ('moderation_tokens', 'alcances', 'scopes'),
    ('moderation_tokens', 'creado_por', 'created_by'), ('moderation_tokens', 'creado_en', 'created_at'),
    ('moderation_tokens', 'expira_en', 'expires_at'), ('moderation_tokens', 'ultimo_uso', 'last_used_at'),
    ('moderation_tokens', 'revocado', 'revoked')
  ) as v(t, o, n) loop
    if to_regclass('public.' || r.t) is not null
       and exists (select 1 from pg_attribute a where a.attrelid = to_regclass('public.' || r.t) and a.attname = r.o and not a.attisdropped)
       and not exists (select 1 from pg_attribute a where a.attrelid = to_regclass('public.' || r.t) and a.attname = r.n and not a.attisdropped) then
      execute format('alter table public.%I rename column %I to %I', r.t, r.o, r.n);
    end if;
  end loop;

  -- RLS policies (names only; the expressions follow the renamed columns by themselves).
  for r in select * from (values
    ('profiles', 'perfiles_leer_propio', 'profiles_read_own'), ('profiles', 'perfiles_crear_propio', 'profiles_create_own'),
    ('profiles', 'perfiles_editar_propio', 'profiles_update_own'),
    ('community_items', 'items_leer_aprobados_anon', 'items_read_approved_anon'),
    ('community_items', 'items_leer_aprobados_sesion', 'items_read_approved_session'),
    ('community_items', 'items_leer_propios', 'items_read_own'), ('community_items', 'items_subir', 'items_upload'),
    ('community_items', 'items_retirar_propio', 'items_update_own'), ('community_items', 'items_borrar_propio', 'items_delete_own'),
    ('reports', 'reportes_crear', 'reports_create'),
    ('public_profiles', 'pp_leer_creadores', 'pp_read_creators_anon'), ('public_profiles', 'pp_leer_creadores_sesion', 'pp_read_creators_session'),
    ('hearts', 'corazones_leer_propios', 'hearts_read_own'), ('hearts', 'corazones_dar', 'hearts_give'), ('hearts', 'corazones_quitar', 'hearts_remove'),
    ('follows', 'seguidores_leer_propios', 'follows_read_own'), ('follows', 'seguidores_seguir', 'follows_follow'), ('follows', 'seguidores_dejar', 'follows_unfollow')
  ) as v(t, o, n) loop
    if exists (select 1 from pg_policies p where p.schemaname = 'public' and p.tablename = r.t and p.policyname = r.o)
       and not exists (select 1 from pg_policies p where p.schemaname = 'public' and p.tablename = r.t and p.policyname = r.n) then
      execute format('alter policy %I on public.%I rename to %I', r.o, r.t, r.n);
    end if;
  end loop;

  -- Triggers.
  for r in select * from (values
    ('profiles', 'perfiles_crear_publico', 'profiles_create_public'), ('profiles', 'perfiles_propagar_nombre', 'profiles_propagate_name'),
    ('community_items', 'items_limitar', 'items_limit'), ('community_items', 'items_autor_nombre', 'items_set_author_name'),
    ('reports', 'reportes_ocultar', 'reports_hide'),
    ('hearts', 'corazones_contar', 'hearts_count'), ('hearts', 'corazones_limitar', 'hearts_limit'),
    ('follows', 'seguidores_contar', 'follows_count'), ('follows', 'seguidores_limitar', 'follows_limit')
  ) as v(t, o, n) loop
    if exists (select 1 from pg_trigger g where g.tgrelid = to_regclass('public.' || r.t) and g.tgname = r.o and not g.tgisinternal)
       and not exists (select 1 from pg_trigger g where g.tgrelid = to_regclass('public.' || r.t) and g.tgname = r.n) then
      execute format('alter trigger %I on public.%I rename to %I', r.o, r.t, r.n);
    end if;
  end loop;

  -- Functions (the triggers keep pointing at them: they are bound by OID).
  for r in select * from (values
    ('ocultar_por_reportes', 'hide_on_reports'), ('limitar_subidas', 'limit_uploads'), ('poner_autor_nombre', 'set_author_name'),
    ('propagar_autor_nombre', 'propagate_author_name'), ('crear_perfil_publico', 'create_public_profile'),
    ('contar_corazon', 'count_heart'), ('contar_seguidor', 'count_follower'), ('limitar_corazones', 'limit_hearts'),
    ('limitar_seguidores', 'limit_follows')
  ) as v(o, n) loop
    if to_regprocedure('public.' || r.o || '()') is not null and to_regprocedure('public.' || r.n || '()') is null then
      execute format('alter function public.%I() rename to %I', r.o, r.n);
    end if;
  end loop;
  if to_regprocedure('public.registrar_descarga(uuid)') is not null and to_regprocedure('public.record_download(uuid)') is null then
    alter function public.registrar_descarga(uuid) rename to record_download;
  end if;

  -- Indexes (cosmetic).
  for r in select * from (values
    ('items_publico_idx', 'community_items_public_idx'), ('items_autor_idx', 'community_items_author_idx'),
    ('corazones_item_idx', 'hearts_item_idx'), ('seguidores_seguido_idx', 'follows_followed_idx'),
    ('tokens_moderacion_creador_idx', 'moderation_tokens_creator_idx')
  ) as v(o, n) loop
    if to_regclass('public.' || r.o) is not null and to_regclass('public.' || r.n) is null then
      execute format('alter index public.%I rename to %I', r.o, r.n);
    end if;
  end loop;
end
$do$;

-- ═════════════════════════ 2. Data labels and constraints (English) ═════════════════════════
-- Report reasons.
alter table public.reports drop constraint if exists reportes_motivo_check;
alter table public.reports drop constraint if exists reports_reason_check;
update public.reports set reason = case reason
  when 'contenido_18' then 'adult_content' when 'odio_o_acoso' then 'hate_or_harassment'
  when 'derechos_de_autor' then 'copyright' when 'otro' then 'other' else reason end
  where reason in ('contenido_18', 'odio_o_acoso', 'derechos_de_autor', 'otro');
alter table public.reports add constraint reports_reason_check
  check (reason in ('malware', 'adult_content', 'hate_or_harassment', 'copyright', 'other'));

-- Moderation tokens: scopes.
alter table public.moderation_tokens drop constraint if exists tokens_moderacion_alcances_check;
alter table public.moderation_tokens drop constraint if exists moderation_tokens_scopes_check;
update public.moderation_tokens set scopes = array(
  select case s when 'leer' then 'read' when 'aprobar' then 'approve' when 'rechazar' then 'reject' else s end from unnest(scopes) s)
  where scopes && array['leer', 'aprobar', 'rechazar'];
alter table public.moderation_tokens add constraint moderation_tokens_scopes_check
  check (scopes <> '{}' and scopes <@ array['read', 'approve', 'reject']);

-- Moderation log labels (free text columns, no constraint).
update public.moderation_log set step = case step
  when 'validacion' then 'validation' when 'automatico' then 'automatic' when 'humano' then 'human'
  when 'ia' then 'ai' when 'texto' then 'text' when 'imagen' then 'image' else step end
  where step in ('validacion', 'automatico', 'humano', 'ia', 'texto', 'imagen');
update public.moderation_log set result = case result
  when 'rechazado' then 'rejected' when 'revisar' then 'review' when 'aprobado' then 'approved' when 'pendiente' then 'pending'
  when 'rechazar' then 'reject' when 'publicar' then 'publish' else result end
  where result in ('rechazado', 'revisar', 'aprobado', 'pendiente', 'rechazar', 'publicar');

-- ═════════════════════════ 3. Real functions (English bodies, same security properties) ═════════════════════════
-- Function bodies are plain text: renaming a column does not rewrite them, so each one is redefined here.
create or replace function public.hide_on_reports() returns trigger language plpgsql security definer set search_path = public as $$
begin
  if (select count(*) from public.reports where item = new.item) >= 3 then
    update public.community_items set status = 'pending', rejection_reason = 'Hidden by reports: pending human review'
      where id = new.item and status = 'approved';
  end if;
  return new;
end $$;

create or replace function public.limit_uploads() returns trigger language plpgsql security definer set search_path = public as $$
begin
  if (select count(*) from public.community_items where author = new.author and status = 'pending') >= 10 then
    raise exception 'Too many uploads pending review (maximum 10).';
  end if;
  if (select count(*) from public.community_items where author = new.author and created_at > now() - interval '1 day') >= 20 then
    raise exception 'Daily upload limit reached (20).';
  end if;
  return new;
end $$;

create or replace function public.set_author_name() returns trigger language plpgsql security definer set search_path = public as $$
begin
  select coalesce(public_name, '') into new.author_name from public.profiles where id = new.author;
  return new;
end $$;

create or replace function public.propagate_author_name() returns trigger language plpgsql security definer set search_path = public as $$
begin
  update public.community_items set author_name = new.public_name where author = new.id;
  update public.public_profiles set public_name = new.public_name, updated_at = now() where id = new.id;
  return new;
end $$;

create or replace function public.create_public_profile() returns trigger language plpgsql security definer set search_path = public as $$
begin
  insert into public.public_profiles (id, public_name) values (new.id, new.public_name) on conflict (id) do nothing;
  return new;
end $$;

create or replace function public.count_heart() returns trigger language plpgsql security definer set search_path = public as $$
declare delta int := case tg_op when 'INSERT' then 1 else -1 end; it uuid := coalesce(new.item, old.item); au uuid;
begin
  update public.community_items set hearts = greatest(0, hearts + delta) where id = it returning author into au;
  if au is not null then
    update public.public_profiles set hearts_received = greatest(0, hearts_received + delta) where id = au;
  end if;
  return null;
end $$;

create or replace function public.count_follower() returns trigger language plpgsql security definer set search_path = public as $$
begin
  update public.public_profiles set followers = greatest(0, followers + case tg_op when 'INSERT' then 1 else -1 end) where id = coalesce(new.followed, old.followed);
  return null;
end $$;

create or replace function public.limit_hearts() returns trigger language plpgsql security definer set search_path = public as $$
begin
  if (select count(*) from public.hearts where person = new.person and created_at > now() - interval '1 day') >= 200 then
    raise exception 'Too many hearts today (maximum 200).';
  end if;
  return new;
end $$;

create or replace function public.limit_follows() returns trigger language plpgsql security definer set search_path = public as $$
begin
  if (select count(*) from public.follows where follower = new.follower) >= 1000 then
    raise exception 'You have reached the maximum number of followed creators (1000).';
  end if;
  return new;
end $$;

-- Trigger functions are internal: nobody calls them through /rest/v1/rpc.
revoke all on function public.hide_on_reports() from public, anon, authenticated;
revoke all on function public.limit_uploads() from public, anon, authenticated;
revoke all on function public.set_author_name() from public, anon, authenticated;
revoke all on function public.propagate_author_name() from public, anon, authenticated;
revoke all on function public.create_public_profile() from public, anon, authenticated;
revoke all on function public.count_heart() from public, anon, authenticated;
revoke all on function public.count_follower() from public, anon, authenticated;
revoke all on function public.limit_hearts() from public, anon, authenticated;
revoke all on function public.limit_follows() from public, anon, authenticated;

-- Downloads: counter without granting write permission on items (signed-in users only, as before).
create or replace function public.record_download(p_item uuid) returns void language sql security definer set search_path = public as $$
  update public.community_items set downloads = downloads + 1 where id = p_item and status = 'approved';
$$;
revoke all on function public.record_download(uuid) from public, anon;
grant execute on function public.record_download(uuid) to authenticated;

-- Legacy RPC: same name, same argument name (PostgREST matches by name), same audience. It is SECURITY INVOKER, so the caller
-- must hold EXECUTE on record_download (only `authenticated` does): anon still cannot move the counter.
create or replace function public.registrar_descarga(p_item uuid) returns void language sql security invoker set search_path = public as $$
  select public.record_download(p_item);
$$;
revoke all on function public.registrar_descarga(uuid) from public, anon;
grant execute on function public.registrar_descarga(uuid) to authenticated;

-- Upload policy (re-created to also pin the heart counter to 0; before, a client could insert an item with any value).
drop policy if exists items_upload on public.community_items;
create policy items_upload on public.community_items for insert to authenticated
  with check (author = auth.uid() and status = 'pending' and public_path is null and public_icon is null and public_preview is null
              and reviewed_at is null and downloads = 0 and hearts = 0);

-- ═════════════════════════ 4. Legacy helpers (schema `legacy`, not exposed) ═════════════════════════
create or replace function legacy.tr(p_map jsonb, p_value text) returns text language sql immutable as $$
  select coalesce(p_map ->> p_value, p_value)
$$;
create or replace function legacy.kind_es(p public.item_kind) returns text language sql immutable as $$
  select legacy.tr('{"image":"imagen","font":"fuente","template":"plantilla","element":"elemento","theme":"tema"}'::jsonb, p::text)
$$;
create or replace function legacy.kind_en(p text) returns public.item_kind language sql stable as $$
  select legacy.tr('{"imagen":"image","fuente":"font","plantilla":"template","elemento":"element","tema":"theme"}'::jsonb, p)::public.item_kind
$$;
create or replace function legacy.status_es(p public.item_status) returns text language sql immutable as $$
  select legacy.tr('{"pending":"pendiente","approved":"aprobado","rejected":"rechazado","withdrawn":"retirado"}'::jsonb, p::text)
$$;
create or replace function legacy.status_en(p text) returns public.item_status language sql stable as $$
  select legacy.tr('{"pendiente":"pending","aprobado":"approved","rechazado":"rejected","retirado":"withdrawn"}'::jsonb, p)::public.item_status
$$;
create or replace function legacy.reason_es(p text) returns text language sql immutable as $$
  select legacy.tr('{"adult_content":"contenido_18","hate_or_harassment":"odio_o_acoso","copyright":"derechos_de_autor","other":"otro"}'::jsonb, p)
$$;
create or replace function legacy.reason_en(p text) returns text language sql immutable as $$
  select legacy.tr('{"contenido_18":"adult_content","odio_o_acoso":"hate_or_harassment","derechos_de_autor":"copyright","otro":"other"}'::jsonb, p)
$$;
create or replace function legacy.step_es(p text) returns text language sql immutable as $$
  select legacy.tr('{"validation":"validacion","automatic":"automatico","human":"humano","ai":"ia","text":"texto","image":"imagen"}'::jsonb, p)
$$;
create or replace function legacy.step_en(p text) returns text language sql immutable as $$
  select legacy.tr('{"validacion":"validation","automatico":"automatic","humano":"human","ia":"ai","texto":"text","imagen":"image"}'::jsonb, p)
$$;
create or replace function legacy.result_es(p text) returns text language sql immutable as $$
  select legacy.tr('{"rejected":"rechazado","review":"revisar","approved":"aprobado","pending":"pendiente","reject":"rechazar","publish":"publicar"}'::jsonb, p)
$$;
create or replace function legacy.result_en(p text) returns text language sql immutable as $$
  select legacy.tr('{"rechazado":"rejected","revisar":"review","aprobado":"approved","pendiente":"pending","rechazar":"reject","publicar":"publish"}'::jsonb, p)
$$;
create or replace function legacy.scope_es(p text) returns text language sql immutable as $$
  select legacy.tr('{"read":"leer","approve":"aprobar","reject":"rechazar"}'::jsonb, p)
$$;
create or replace function legacy.scope_en(p text) returns text language sql immutable as $$
  select legacy.tr('{"leer":"read","aprobar":"approve","rechazar":"reject"}'::jsonb, p)
$$;

-- Columns of public_profiles that anon cannot read on the base table. A security_invoker view cannot reference them directly
-- (it would make every anon query fail), so they come from these definer helpers, which reveal a value ONLY to the owner or to service_role.
create or replace function legacy.pp_pending(p_id uuid, p_class text) returns boolean language sql stable security definer set search_path = public as $$
  select coalesce(case when auth.uid() = p_id or auth.role() = 'service_role' then
    (select case p_class when 'avatar' then avatar_pending else banner_pending end from public.public_profiles where id = p_id) end, false)
$$;
create or replace function legacy.pp_updated(p_id uuid) returns timestamptz language sql stable security definer set search_path = public as $$
  select case when auth.uid() = p_id or auth.role() = 'service_role' then
    (select updated_at from public.public_profiles where id = p_id) end
$$;

-- Applies {base_column: value} to one row, as the CALLER (security invoker): column grants and RLS of the base table decide.
-- Only the columns present in p_changes are written, so unchanged privileged columns never trigger a permission error.
create or replace function legacy.apply_changes(p_table regclass, p_key_col text, p_key anyelement, p_changes jsonb) returns void language plpgsql as $$
declare sets text;
begin
  select string_agg(format('%I = r.%I', k, k), ', ') into sets from jsonb_object_keys(p_changes) k;
  if sets is null then return; end if;
  execute format('update %s t set %s from jsonb_populate_record(null::%s, $1) r where t.%I = $2', p_table, sets, p_table, p_key_col)
    using p_changes, p_key;
end $$;

-- {legacy_col: base_col} + OLD/NEW as jsonb  ->  {base_col: new_value} for the columns that changed.
create or replace function legacy.changed(p_old jsonb, p_new jsonb, p_map jsonb) returns jsonb language sql immutable as $$
  select coalesce(jsonb_object_agg(m.value #>> '{}', p_new -> m.key), '{}'::jsonb)
  from jsonb_each(p_map) m where p_old -> m.key is distinct from p_new -> m.key
$$;

revoke all on all functions in schema legacy from public;
grant execute on all functions in schema legacy to anon, authenticated, service_role;

-- ═════════════════════════ 5. Legacy views and their write paths ═════════════════════════
-- 5.1 perfiles (auto-updatable: only renames columns)
create or replace view public.perfiles with (security_invoker = true) as
  select id, public_name as nombre_publico, show_18 as mostrar_18, terms_accepted_at as acepto_terminos_en,
         created_at as creado_en, declared_adult_at as declara_mayor_18_en
  from public.profiles;
revoke all on public.perfiles from public, anon, authenticated;
grant select, insert on public.perfiles to authenticated;
grant update (nombre_publico, mostrar_18, acepto_terminos_en, declara_mayor_18_en) on public.perfiles to authenticated;
grant all on public.perfiles to service_role;

-- 5.2 corazones / seguidores (auto-updatable)
create or replace view public.corazones with (security_invoker = true) as
  select person as persona, item, created_at as creado_en from public.hearts;
revoke all on public.corazones from public, anon, authenticated;
grant select, insert, delete on public.corazones to authenticated;
grant all on public.corazones to service_role;

create or replace view public.seguidores with (security_invoker = true) as
  select follower as seguidor, followed as seguido, created_at as creado_en from public.follows;
revoke all on public.seguidores from public, anon, authenticated;
grant select, insert, delete on public.seguidores to authenticated;
grant all on public.seguidores to service_role;

-- 5.3 perfiles_publicos (read-only for people; service_role can still update through the trigger)
create or replace view public.perfiles_publicos with (security_invoker = true) as
  select id, public_name as nombre_publico, bio, github, links as enlaces, public_avatar as avatar_publico, public_banner as banner_publico,
         legacy.pp_pending(id, 'avatar') as avatar_pendiente, legacy.pp_pending(id, 'banner') as banner_pendiente,
         legacy.pp_updated(id) as actualizado_en, followers as seguidores, hearts_received as corazones_recibidos
  from public.public_profiles;
revoke all on public.perfiles_publicos from public, anon, authenticated;
grant select (id, nombre_publico, bio, github, enlaces, avatar_publico, banner_publico, seguidores, corazones_recibidos) on public.perfiles_publicos to anon;
grant select on public.perfiles_publicos to authenticated;
grant all on public.perfiles_publicos to service_role;

create or replace function legacy.perfiles_publicos_update() returns trigger language plpgsql as $$
begin
  perform legacy.apply_changes('public.public_profiles', 'id', old.id, legacy.changed(to_jsonb(old), to_jsonb(new), jsonb_build_object(
    'nombre_publico', 'public_name', 'bio', 'bio', 'github', 'github', 'enlaces', 'links', 'avatar_publico', 'public_avatar',
    'banner_publico', 'public_banner', 'avatar_pendiente', 'avatar_pending', 'banner_pendiente', 'banner_pending',
    'actualizado_en', 'updated_at', 'seguidores', 'followers', 'corazones_recibidos', 'hearts_received')));
  return new;
end $$;
drop trigger if exists perfiles_publicos_instead_update on public.perfiles_publicos;
create trigger perfiles_publicos_instead_update instead of update on public.perfiles_publicos
  for each row execute function legacy.perfiles_publicos_update();

-- 5.4 items (translates kind/status)
create or replace view public.items with (security_invoker = true) as
  select id, author as autor, legacy.kind_es(kind) as tipo, title as titulo, description as descripcion, tags as etiquetas,
         is_18 as es_18, legacy.status_es(status) as estado, rejection_reason as motivo_rechazo, quarantine_path as ruta_cuarentena,
         public_path as ruta_publica, sha256, bytes, version, downloads as descargas, created_at as creado_en, reviewed_at as revisado_en,
         author_name as autor_nombre, quarantine_icon as icono_cuarentena, quarantine_preview as vista_cuarentena,
         public_icon as icono_publico, public_preview as vista_publica, hearts as corazones
  from public.community_items;
revoke all on public.items from public, anon, authenticated;
grant select on public.items to anon, authenticated;
grant insert, delete on public.items to authenticated;
grant update (titulo, descripcion, etiquetas) on public.items to authenticated;
grant all on public.items to service_role;

create or replace function legacy.items_insert() returns trigger language plpgsql as $$
begin
  new.id := coalesce(new.id, gen_random_uuid());
  new.creado_en := coalesce(new.creado_en, now());
  insert into public.community_items (id, author, kind, title, description, tags, is_18, status, rejection_reason, quarantine_path,
      public_path, sha256, bytes, version, downloads, created_at, reviewed_at, quarantine_icon, quarantine_preview, public_icon,
      public_preview, hearts)
  values (new.id, new.autor, legacy.kind_en(new.tipo), new.titulo, coalesce(new.descripcion, ''), coalesce(new.etiquetas, '{}'),
      coalesce(new.es_18, false), coalesce(legacy.status_en(new.estado), 'pending'), new.motivo_rechazo, new.ruta_cuarentena,
      new.ruta_publica, new.sha256, new.bytes, coalesce(new.version, '1.0.0'), coalesce(new.descargas, 0), new.creado_en,
      new.revisado_en, new.icono_cuarentena, new.vista_cuarentena, new.icono_publico, new.vista_publica, coalesce(new.corazones, 0));
  return new;
end $$;

create or replace function legacy.items_update() returns trigger language plpgsql as $$
declare ch jsonb;
begin
  ch := legacy.changed(to_jsonb(old), to_jsonb(new), jsonb_build_object(
    'autor', 'author', 'tipo', 'kind', 'titulo', 'title', 'descripcion', 'description', 'etiquetas', 'tags', 'es_18', 'is_18',
    'estado', 'status', 'motivo_rechazo', 'rejection_reason', 'ruta_cuarentena', 'quarantine_path', 'ruta_publica', 'public_path',
    'sha256', 'sha256', 'bytes', 'bytes', 'version', 'version', 'descargas', 'downloads', 'creado_en', 'created_at',
    'revisado_en', 'reviewed_at', 'autor_nombre', 'author_name', 'icono_cuarentena', 'quarantine_icon',
    'vista_cuarentena', 'quarantine_preview', 'icono_publico', 'public_icon', 'vista_publica', 'public_preview', 'corazones', 'hearts'));
  if ch ? 'kind' then ch := jsonb_set(ch, '{kind}', to_jsonb(legacy.kind_en(ch ->> 'kind')::text)); end if;
  if ch ? 'status' then ch := jsonb_set(ch, '{status}', to_jsonb(legacy.status_en(ch ->> 'status')::text)); end if;
  perform legacy.apply_changes('public.community_items', 'id', old.id, ch);
  return new;
end $$;

create or replace function legacy.items_delete() returns trigger language plpgsql as $$
begin
  delete from public.community_items where id = old.id;
  return old;
end $$;

drop trigger if exists items_instead_insert on public.items;
create trigger items_instead_insert instead of insert on public.items for each row execute function legacy.items_insert();
drop trigger if exists items_instead_update on public.items;
create trigger items_instead_update instead of update on public.items for each row execute function legacy.items_update();
drop trigger if exists items_instead_delete on public.items;
create trigger items_instead_delete instead of delete on public.items for each row execute function legacy.items_delete();

-- 5.5 reportes (insert only; translates the reason)
create or replace view public.reportes with (security_invoker = true) as
  select id, item, reporter as reportante, legacy.reason_es(reason) as motivo, detail as detalle, created_at as creado_en
  from public.reports;
revoke all on public.reportes from public, anon, authenticated;
grant insert on public.reportes to authenticated;
grant all on public.reportes to service_role;

create or replace function legacy.reportes_insert() returns trigger language plpgsql as $$
begin
  new.id := coalesce(new.id, gen_random_uuid());
  new.creado_en := coalesce(new.creado_en, now());
  insert into public.reports (id, item, reporter, reason, detail, created_at)
  values (new.id, new.item, new.reportante, legacy.reason_en(new.motivo), coalesce(new.detalle, ''), new.creado_en);
  return new;
end $$;
drop trigger if exists reportes_instead_insert on public.reportes;
create trigger reportes_instead_insert instead of insert on public.reportes for each row execute function legacy.reportes_insert();

-- 5.6 moderacion_log (server only; insert translates the labels)
create or replace view public.moderacion_log with (security_invoker = true) as
  select id, item, legacy.step_es(step) as paso, legacy.result_es(result) as resultado, detail as detalle, created_at as creado_en
  from public.moderation_log;
revoke all on public.moderacion_log from public, anon, authenticated;
grant all on public.moderacion_log to service_role;

create or replace function legacy.moderacion_log_insert() returns trigger language plpgsql as $$
begin
  new.creado_en := coalesce(new.creado_en, now());
  insert into public.moderation_log (item, step, result, detail, created_at)
  values (new.item, legacy.step_en(new.paso), legacy.result_en(new.resultado), coalesce(new.detalle, ''), new.creado_en)
  returning id into new.id;
  return new;
end $$;
drop trigger if exists moderacion_log_instead_insert on public.moderacion_log;
create trigger moderacion_log_instead_insert instead of insert on public.moderacion_log
  for each row execute function legacy.moderacion_log_insert();

-- 5.7 tokens_moderacion (server only; translates the scopes)
create or replace view public.tokens_moderacion with (security_invoker = true) as
  select id, name as nombre, fingerprint as huella, array(select legacy.scope_es(s) from unnest(scopes) s) as alcances,
         created_by as creado_por, created_at as creado_en, expires_at as expira_en, last_used_at as ultimo_uso, revoked as revocado
  from public.moderation_tokens;
revoke all on public.tokens_moderacion from public, anon, authenticated;
grant all on public.tokens_moderacion to service_role;

create or replace function legacy.tokens_moderacion_insert() returns trigger language plpgsql as $$
begin
  new.id := coalesce(new.id, gen_random_uuid());
  new.creado_en := coalesce(new.creado_en, now());
  insert into public.moderation_tokens (id, name, fingerprint, scopes, created_by, created_at, expires_at, last_used_at, revoked)
  values (new.id, new.nombre, new.huella, array(select legacy.scope_en(s) from unnest(new.alcances) s), new.creado_por, new.creado_en,
          new.expira_en, new.ultimo_uso, coalesce(new.revocado, false));
  return new;
end $$;
create or replace function legacy.tokens_moderacion_update() returns trigger language plpgsql as $$
begin
  perform legacy.apply_changes('public.moderation_tokens', 'id', old.id, legacy.changed(to_jsonb(old), to_jsonb(new), jsonb_build_object(
    'nombre', 'name', 'expira_en', 'expires_at', 'ultimo_uso', 'last_used_at', 'revocado', 'revoked')));
  return new;
end $$;
drop trigger if exists tokens_moderacion_instead_insert on public.tokens_moderacion;
create trigger tokens_moderacion_instead_insert instead of insert on public.tokens_moderacion
  for each row execute function legacy.tokens_moderacion_insert();
drop trigger if exists tokens_moderacion_instead_update on public.tokens_moderacion;
create trigger tokens_moderacion_instead_update instead of update on public.tokens_moderacion
  for each row execute function legacy.tokens_moderacion_update();

-- ═════════════════════════ 6. Grants for the English tables (unchanged from the Spanish ones, restated for clarity) ═════════════════════════
revoke all on public.moderation_log from anon, authenticated;
revoke all on public.moderation_tokens from anon, authenticated;
grant all on public.community_items, public.profiles, public.reports, public.moderation_log, public.public_profiles,
  public.hearts, public.follows, public.moderation_tokens to service_role;

-- Make PostgREST pick up the new relations right away.
notify pgrst, 'reload schema';
