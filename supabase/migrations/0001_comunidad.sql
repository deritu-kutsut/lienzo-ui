-- Lienzo UI · Comunidad. Esquema, permisos y almacenamiento.
-- Principios: mínimo de datos personales, todo cerrado por defecto (RLS), nada se publica sin pasar por la moderación.

create extension if not exists pgcrypto;

-- ───────── Tipos ─────────
create type tipo_item as enum ('plugin', 'imagen', 'fuente', 'plantilla', 'elemento');
create type estado_item as enum ('pendiente', 'aprobado', 'rechazado', 'retirado');

-- ───────── Perfiles ─────────
-- Solo se guarda un nombre público elegido por la persona. El correo vive únicamente en auth.users (no se copia aquí)
-- y ninguna política lo expone. Nunca se guardan contraseñas: el acceso es por proveedores externos (OAuth).
create table public.perfiles (
  id uuid primary key references auth.users (id) on delete cascade,
  nombre_publico text not null check (char_length(nombre_publico) between 3 and 32 and nombre_publico ~ '^[A-Za-z0-9_.-]+$'),
  mostrar_18 boolean not null default false,          -- preferencia del filtro +18 (apagado por defecto)
  acepto_terminos_en timestamptz,
  creado_en timestamptz not null default now(),
  unique (nombre_publico)
);

-- ───────── Contenido compartido ─────────
create table public.items (
  id uuid primary key default gen_random_uuid(),
  autor uuid not null references public.perfiles (id) on delete cascade,
  tipo tipo_item not null,
  titulo text not null check (char_length(titulo) between 3 and 80),
  descripcion text not null default '' check (char_length(descripcion) <= 1000),
  etiquetas text[] not null default '{}' check (cardinality(etiquetas) <= 8),
  es_18 boolean not null default false,               -- marcado por el autor o por la moderación (solo puede subir, nunca bajar)
  estado estado_item not null default 'pendiente',
  motivo_rechazo text,
  ruta_cuarentena text,                               -- bucket «cuarentena»: <uid>/<id>/<archivo>
  ruta_publica text,                                  -- bucket «publico»: solo la escribe el servidor tras aprobar
  sha256 text,
  bytes integer,
  version text not null default '1.0.0',
  descargas integer not null default 0,
  creado_en timestamptz not null default now(),
  revisado_en timestamptz
);
create index items_publico_idx on public.items (tipo, creado_en desc) where estado = 'aprobado';
create index items_autor_idx on public.items (autor);

-- ───────── Reportes de la comunidad ─────────
create table public.reportes (
  id uuid primary key default gen_random_uuid(),
  item uuid not null references public.items (id) on delete cascade,
  reportante uuid not null references public.perfiles (id) on delete cascade,
  motivo text not null check (motivo in ('malware', 'contenido_18', 'odio_o_acoso', 'derechos_de_autor', 'otro')),
  detalle text not null default '' check (char_length(detalle) <= 500),
  creado_en timestamptz not null default now(),
  unique (item, reportante)
);

-- ───────── Registro de moderación (solo servidor) ─────────
-- Guarda QUÉ se decidió y por qué, nunca datos personales ni claves.
create table public.moderacion_log (
  id bigserial primary key,
  item uuid references public.items (id) on delete set null,
  paso text not null,                                  -- validacion | texto | malware | imagen | humano
  resultado text not null,                             -- ok | rechazado | revisar
  detalle text not null default '',
  creado_en timestamptz not null default now()
);

-- ───────── Seguridad por filas (RLS) ─────────
alter table public.perfiles enable row level security;
alter table public.items enable row level security;
alter table public.reportes enable row level security;
alter table public.moderacion_log enable row level security;

-- Perfiles: cada persona ve y edita solo el suyo. Los nombres públicos de autores se exponen por la vista autores_publicos.
create policy perfiles_leer_propio on public.perfiles for select using (id = auth.uid());
create policy perfiles_crear_propio on public.perfiles for insert with check (id = auth.uid());
create policy perfiles_editar_propio on public.perfiles for update using (id = auth.uid()) with check (id = auth.uid());
revoke update on public.perfiles from authenticated;
grant update (nombre_publico, mostrar_18, acepto_terminos_en) on public.perfiles to authenticated;

create view public.autores_publicos with (security_invoker = false) as
  select id, nombre_publico from public.perfiles
  where exists (select 1 from public.items i where i.autor = perfiles.id and i.estado = 'aprobado');
grant select on public.autores_publicos to anon, authenticated;

-- Items: lo aprobado lo ve cualquiera (el filtro +18 lo aplica la consulta según la preferencia); lo propio, su autor.
create policy items_leer_aprobados on public.items for select using (estado = 'aprobado');
create policy items_leer_propios on public.items for select using (autor = auth.uid());
-- Subir: solo como «pendiente», a nombre propio y sin tocar campos de decisión.
create policy items_subir on public.items for insert to authenticated
  with check (autor = auth.uid() and estado = 'pendiente' and ruta_publica is null and revisado_en is null and descargas = 0);
create policy items_retirar_propio on public.items for update to authenticated
  using (autor = auth.uid()) with check (autor = auth.uid());
-- El autor solo puede editar texto y retirar su contenido; jamás aprobarlo ni cambiar rutas, hashes o el marcado +18.
revoke update on public.items from authenticated;
grant update (titulo, descripcion, etiquetas) on public.items to authenticated;
create policy items_borrar_propio on public.items for delete to authenticated using (autor = auth.uid());

-- Reportes: se puede reportar (una vez por contenido); solo el servidor los lee.
create policy reportes_crear on public.reportes for insert to authenticated with check (reportante = auth.uid());

-- El registro de moderación no tiene políticas: solo la service_role (que ignora RLS) lo toca.

-- Con 3 reportes de personas distintas, el contenido se oculta hasta que lo revise una persona.
create function public.ocultar_por_reportes() returns trigger language plpgsql security definer set search_path = public as $$
begin
  if (select count(*) from public.reportes where item = new.item) >= 3 then
    update public.items set estado = 'pendiente', motivo_rechazo = 'Ocultado por reportes: pendiente de revisión humana'
      where id = new.item and estado = 'aprobado';
  end if;
  return new;
end $$;
create trigger reportes_ocultar after insert on public.reportes for each row execute function public.ocultar_por_reportes();

-- Descargas: contador sin dar permiso de escritura sobre items.
create function public.registrar_descarga(p_item uuid) returns void language sql security definer set search_path = public as $$
  update public.items set descargas = descargas + 1 where id = p_item and estado = 'aprobado';
$$;
revoke all on function public.registrar_descarga(uuid) from public;
grant execute on function public.registrar_descarga(uuid) to authenticated, anon;

-- ───────── Almacenamiento ─────────
-- cuarentena: privado; cada persona solo escribe en su carpeta <uid>/ y nunca lee lo de otras.
-- publico: lectura abierta; escribe solo el servidor (service_role) cuando el contenido ya fue aprobado.
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
-- Sin políticas de insert/update/delete en «publico» para usuarios: solo service_role.

-- ───────── Límites contra abuso ─────────
-- Máximo 10 subidas pendientes por persona y 20 al día.
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
