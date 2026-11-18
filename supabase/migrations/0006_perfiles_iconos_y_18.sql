-- 1) +18 filter enforced ON THE SERVER; 2) icon and preview for each item; 3) public creator profiles.

-- ───────── 1. +18: only visible to those who declared they are over 18 ─────────
-- Before, the filter was app-only: the API also returned +18 content to anyone. Now the server decides.
alter table public.perfiles add column if not exists declara_mayor_18_en timestamptz;
alter table public.perfiles drop constraint if exists perfiles_mostrar_18_declarado;
alter table public.perfiles add constraint perfiles_mostrar_18_declarado check (mostrar_18 = false or declara_mayor_18_en is not null);
grant update (declara_mayor_18_en) on public.perfiles to authenticated;

drop policy if exists items_leer_aprobados on public.items;
create policy items_leer_aprobados_anon on public.items for select to anon
  using (estado = 'aprobado' and not es_18);
create policy items_leer_aprobados_sesion on public.items for select to authenticated
  using (estado = 'aprobado' and (not es_18 or exists (select 1 from public.perfiles p where p.id = (select auth.uid()) and p.mostrar_18)));

-- ───────── 2. Icon and preview ─────────
alter table public.items
  add column if not exists icono_cuarentena text,
  add column if not exists vista_cuarentena text,
  add column if not exists icono_publico text,
  add column if not exists vista_publica text;

drop policy if exists items_subir on public.items;
create policy items_subir on public.items for insert to authenticated
  with check (autor = auth.uid() and estado = 'pendiente' and ruta_publica is null and icono_publico is null and vista_publica is null and revisado_en is null and descargas = 0);

-- ───────── 3. Public creator profile ─────────
-- Separate from «perfiles» (private): this holds only what the person chooses to show. No email or preferences.
-- Only the server writes it («perfil» function), which validates and moderates; the person only reads it.
create table if not exists public.perfiles_publicos (
  id uuid primary key references public.perfiles (id) on delete cascade,
  nombre_publico text not null,
  bio text not null default '' check (char_length(bio) <= 300),
  github text check (github is null or github ~ '^[A-Za-z0-9](?:[A-Za-z0-9-]{0,38})$'),
  enlaces jsonb not null default '{}'::jsonb,
  avatar_publico text,
  banner_publico text,
  avatar_pendiente boolean not null default false,
  banner_pendiente boolean not null default false,
  actualizado_en timestamptz not null default now()
);
alter table public.perfiles_publicos enable row level security;
-- A profile is public only if the person has at least one approved item; they always see their own.
create policy pp_leer_creadores on public.perfiles_publicos for select to anon
  using (exists (select 1 from public.items i where i.autor = perfiles_publicos.id and i.estado = 'aprobado' and not i.es_18));
create policy pp_leer_creadores_sesion on public.perfiles_publicos for select to authenticated
  using (id = (select auth.uid()) or exists (select 1 from public.items i where i.autor = perfiles_publicos.id and i.estado = 'aprobado'));
grant select (id, nombre_publico, bio, github, enlaces, avatar_publico, banner_publico) on public.perfiles_publicos to anon;
grant select on public.perfiles_publicos to authenticated;
grant all on public.perfiles_publicos to service_role;

-- Created automatically with the private profile and keeps the name in sync.
create or replace function public.crear_perfil_publico() returns trigger language plpgsql security definer set search_path = public as $$
begin
  insert into public.perfiles_publicos (id, nombre_publico) values (new.id, new.nombre_publico) on conflict (id) do nothing;
  return new;
end $$;
revoke all on function public.crear_perfil_publico() from public, anon, authenticated;
drop trigger if exists perfiles_crear_publico on public.perfiles;
create trigger perfiles_crear_publico after insert on public.perfiles for each row execute function public.crear_perfil_publico();

create or replace function public.propagar_autor_nombre() returns trigger language plpgsql security definer set search_path = public as $$
begin
  update public.items set autor_nombre = new.nombre_publico where autor = new.id;
  update public.perfiles_publicos set nombre_publico = new.nombre_publico, actualizado_en = now() where id = new.id;
  return new;
end $$;

insert into public.perfiles_publicos (id, nombre_publico) select id, nombre_publico from public.perfiles on conflict (id) do nothing;
