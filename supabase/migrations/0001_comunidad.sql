-- Lienzo UI · Community. Schema, permissions and storage.
-- Principles: minimal personal data, everything closed by default (RLS), nothing is published without going through moderation.

create extension if not exists pgcrypto;

-- ───────── Types ─────────
create type tipo_item as enum ('plugin', 'imagen', 'fuente', 'plantilla', 'elemento');
create type estado_item as enum ('pendiente', 'aprobado', 'rechazado', 'retirado');

-- ───────── Profiles ─────────
-- Only a public name chosen by the person is stored. The email lives only in auth.users (it is not copied here)
-- and no policy exposes it. Passwords are never stored: access is through external providers (OAuth).
create table public.perfiles (
  id uuid primary key references auth.users (id) on delete cascade,
  nombre_publico text not null check (char_length(nombre_publico) between 3 and 32 and nombre_publico ~ '^[A-Za-z0-9_.-]+$'),
  mostrar_18 boolean not null default false,          -- +18 filter preference (off by default)
  acepto_terminos_en timestamptz,
  creado_en timestamptz not null default now(),
  unique (nombre_publico)
);

-- ───────── Shared content ─────────
create table public.items (
  id uuid primary key default gen_random_uuid(),
  autor uuid not null references public.perfiles (id) on delete cascade,
  tipo tipo_item not null,
  titulo text not null check (char_length(titulo) between 3 and 80),
  descripcion text not null default '' check (char_length(descripcion) <= 1000),
  etiquetas text[] not null default '{}' check (cardinality(etiquetas) <= 8),
  es_18 boolean not null default false,               -- flagged by the author or by moderation (can only go up, never down)
  estado estado_item not null default 'pendiente',
  motivo_rechazo text,
  ruta_cuarentena text,                               -- bucket «cuarentena»: <uid>/<id>/<file>
  ruta_publica text,                                  -- bucket «publico»: only the server writes it after approval
  sha256 text,
  bytes integer,
  version text not null default '1.0.0',
  descargas integer not null default 0,
  creado_en timestamptz not null default now(),
  revisado_en timestamptz
);
create index items_publico_idx on public.items (tipo, creado_en desc) where estado = 'aprobado';
create index items_autor_idx on public.items (autor);

-- ───────── Community reports ─────────
create table public.reportes (
  id uuid primary key default gen_random_uuid(),
  item uuid not null references public.items (id) on delete cascade,
  reportante uuid not null references public.perfiles (id) on delete cascade,
  motivo text not null check (motivo in ('malware', 'contenido_18', 'odio_o_acoso', 'derechos_de_autor', 'otro')),
  detalle text not null default '' check (char_length(detalle) <= 500),
  creado_en timestamptz not null default now(),
  unique (item, reportante)
);

-- ───────── Moderation log (server only) ─────────
-- Stores WHAT was decided and why, never personal data or keys.
create table public.moderacion_log (
  id bigserial primary key,
  item uuid references public.items (id) on delete set null,
  paso text not null,                                  -- validacion | texto | malware | imagen | humano
  resultado text not null,                             -- ok | rechazado | revisar
  detalle text not null default '',
  creado_en timestamptz not null default now()
);

-- ───────── Row-level security (RLS) ─────────
alter table public.perfiles enable row level security;
alter table public.items enable row level security;
alter table public.reportes enable row level security;
alter table public.moderacion_log enable row level security;

-- Profiles: each person sees and edits only their own. Public author names are exposed through the autores_publicos view.
create policy perfiles_leer_propio on public.perfiles for select using (id = auth.uid());
create policy perfiles_crear_propio on public.perfiles for insert with check (id = auth.uid());
create policy perfiles_editar_propio on public.perfiles for update using (id = auth.uid()) with check (id = auth.uid());
revoke update on public.perfiles from authenticated;
grant update (nombre_publico, mostrar_18, acepto_terminos_en) on public.perfiles to authenticated;

create view public.autores_publicos with (security_invoker = false) as
  select id, nombre_publico from public.perfiles
  where exists (select 1 from public.items i where i.autor = perfiles.id and i.estado = 'aprobado');
grant select on public.autores_publicos to anon, authenticated;

-- Items: anyone sees what is approved (the query applies the +18 filter according to the preference); the author sees their own.
create policy items_leer_aprobados on public.items for select using (estado = 'aprobado');
create policy items_leer_propios on public.items for select using (autor = auth.uid());
-- Uploading: only as «pendiente», under one's own name and without touching decision fields.
create policy items_subir on public.items for insert to authenticated
  with check (autor = auth.uid() and estado = 'pendiente' and ruta_publica is null and revisado_en is null and descargas = 0);
create policy items_retirar_propio on public.items for update to authenticated
  using (autor = auth.uid()) with check (autor = auth.uid());
-- The author can only edit text and withdraw their content; never approve it or change paths, hashes or the +18 flag.
revoke update on public.items from authenticated;
grant update (titulo, descripcion, etiquetas) on public.items to authenticated;
create policy items_borrar_propio on public.items for delete to authenticated using (autor = auth.uid());

-- Reports: anyone can report (once per item); only the server reads them.
create policy reportes_crear on public.reportes for insert to authenticated with check (reportante = auth.uid());

-- The moderation log has no policies: only service_role (which ignores RLS) touches it.

-- With 3 reports from different people, the content is hidden until a person reviews it.
create function public.ocultar_por_reportes() returns trigger language plpgsql security definer set search_path = public as $$
begin
  if (select count(*) from public.reportes where item = new.item) >= 3 then
    update public.items set estado = 'pendiente', motivo_rechazo = 'Ocultado por reportes: pendiente de revisión humana'
      where id = new.item and estado = 'aprobado';
  end if;
  return new;
end $$;
create trigger reportes_ocultar after insert on public.reportes for each row execute function public.ocultar_por_reportes();

-- Downloads: counter without granting write permission on items.
create function public.registrar_descarga(p_item uuid) returns void language sql security definer set search_path = public as $$
  update public.items set descargas = descargas + 1 where id = p_item and estado = 'aprobado';
$$;
revoke all on function public.registrar_descarga(uuid) from public;
grant execute on function public.registrar_descarga(uuid) to authenticated, anon;

-- ───────── Storage ─────────
-- cuarentena: private; each person writes only in their own <uid>/ folder and never reads others'.
-- publico: open read; only the server (service_role) writes, once the content has been approved.
insert into storage.buckets (id, name, public, file_size_limit, allowed_mime_types) values
  ('cuarentena', 'cuarentena', false, 5242880, array['image/png','image/jpeg','image/webp','image/gif','font/ttf','font/otf','application/json','application/octet-stream']),
  ('publico', 'publico', true, 5242880, null)
on conflict (id) do nothing;

create policy cuarentena_subir on storage.objects for insert to authenticated
  with check (bucket_id = 'cuarentena' and (storage.foldername(name))[1] = auth.uid()::text);
create policy cuarentena_leer_propio on storage.objects for select to authenticated
  using (bucket_id = 'cuarentena' and (storage.foldername(name))[1] = auth.uid()::text);
create policy cuarentena_borrar_propio on storage.objects for delete to authenticated
  using (bucket_id = 'cuarentena' and (storage.foldername(name))[1] = auth.uid()::text);
create policy publico_leer on storage.objects for select using (bucket_id = 'publico');
-- No insert/update/delete policies on «publico» for users: service_role only.

-- ───────── Abuse limits ─────────
-- Maximum 10 pending uploads per person and 20 per day.
create function public.limitar_subidas() returns trigger language plpgsql security definer set search_path = public as $$
begin
  if (select count(*) from public.items where autor = new.autor and estado = 'pendiente') >= 10 then
    raise exception 'Demasiadas subidas pendientes de revisión (máximo 10).';
  end if;
  if (select count(*) from public.items where autor = new.autor and creado_en > now() - interval '1 day') >= 20 then
    raise exception 'Límite diario de subidas alcanzado (20).';
  end if;
  return new;
end $$;
create trigger items_limitar before insert on public.items for each row execute function public.limitar_subidas();
