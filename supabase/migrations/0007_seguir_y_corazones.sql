-- Seguir a creadores y dar corazones (la «estrella» de la comunidad) a los contenidos.
-- Privacidad: quién sigue a quién y quién dio un corazón NO es público; solo se publican los totales.

-- ───────── Tablas ─────────
create table if not exists public.corazones (
  persona uuid not null references public.perfiles (id) on delete cascade,
  item uuid not null references public.items (id) on delete cascade,
  creado_en timestamptz not null default now(),
  primary key (persona, item)
);
create table if not exists public.seguidores (
  seguidor uuid not null references public.perfiles (id) on delete cascade,
  seguido uuid not null references public.perfiles_publicos (id) on delete cascade,
  creado_en timestamptz not null default now(),
  primary key (seguidor, seguido),
  check (seguidor <> seguido)
);
create index if not exists corazones_item_idx on public.corazones (item);
create index if not exists seguidores_seguido_idx on public.seguidores (seguido);

-- ───────── Totales públicos ─────────
alter table public.items add column if not exists corazones integer not null default 0 check (corazones >= 0);
alter table public.perfiles_publicos
  add column if not exists seguidores integer not null default 0 check (seguidores >= 0),
  add column if not exists corazones_recibidos integer not null default 0 check (corazones_recibidos >= 0);
-- Los totales solo los mueve el servidor (triggers): la persona nunca puede escribirlos.
grant select (id, nombre_publico, bio, github, enlaces, avatar_publico, banner_publico, seguidores, corazones_recibidos) on public.perfiles_publicos to anon;

create or replace function public.contar_corazon() returns trigger language plpgsql security definer set search_path = public as $$
declare delta int := case tg_op when 'INSERT' then 1 else -1 end; it uuid := coalesce(new.item, old.item); au uuid;
begin
  update public.items set corazones = greatest(0, corazones + delta) where id = it returning autor into au;
  if au is not null then
    update public.perfiles_publicos set corazones_recibidos = greatest(0, corazones_recibidos + delta) where id = au;
  end if;
  return null;
end $$;
create or replace function public.contar_seguidor() returns trigger language plpgsql security definer set search_path = public as $$
begin
  update public.perfiles_publicos set seguidores = greatest(0, seguidores + case tg_op when 'INSERT' then 1 else -1 end) where id = coalesce(new.seguido, old.seguido);
  return null;
end $$;
revoke all on function public.contar_corazon() from public, anon, authenticated;
revoke all on function public.contar_seguidor() from public, anon, authenticated;
drop trigger if exists corazones_contar on public.corazones;
create trigger corazones_contar after insert or delete on public.corazones for each row execute function public.contar_corazon();
drop trigger if exists seguidores_contar on public.seguidores;
create trigger seguidores_contar after insert or delete on public.seguidores for each row execute function public.contar_seguidor();

-- ───────── Límites contra abuso ─────────
create or replace function public.limitar_corazones() returns trigger language plpgsql security definer set search_path = public as $$
begin
  if (select count(*) from public.corazones where persona = new.persona and creado_en > now() - interval '1 day') >= 200 then
    raise exception 'Demasiados corazones hoy (máximo 200).';
  end if;
  return new;
end $$;
create or replace function public.limitar_seguidores() returns trigger language plpgsql security definer set search_path = public as $$
begin
  if (select count(*) from public.seguidores where seguidor = new.seguidor) >= 1000 then
    raise exception 'Has llegado al máximo de creadores seguidos (1000).';
  end if;
  return new;
end $$;
revoke all on function public.limitar_corazones() from public, anon, authenticated;
revoke all on function public.limitar_seguidores() from public, anon, authenticated;
drop trigger if exists corazones_limitar on public.corazones;
create trigger corazones_limitar before insert on public.corazones for each row execute function public.limitar_corazones();
drop trigger if exists seguidores_limitar on public.seguidores;
create trigger seguidores_limitar before insert on public.seguidores for each row execute function public.limitar_seguidores();

-- ───────── Seguridad por filas ─────────
alter table public.corazones enable row level security;
alter table public.seguidores enable row level security;

-- Cada persona ve y mueve solo lo suyo.
create policy corazones_leer_propios on public.corazones for select to authenticated using (persona = (select auth.uid()));
create policy corazones_dar on public.corazones for insert to authenticated
  with check (persona = (select auth.uid()) and exists (select 1 from public.items i where i.id = item and i.estado = 'aprobado'));
create policy corazones_quitar on public.corazones for delete to authenticated using (persona = (select auth.uid()));

create policy seguidores_leer_propios on public.seguidores for select to authenticated using (seguidor = (select auth.uid()));
create policy seguidores_seguir on public.seguidores for insert to authenticated
  with check (seguidor = (select auth.uid()) and seguido <> (select auth.uid()));
create policy seguidores_dejar on public.seguidores for delete to authenticated using (seguidor = (select auth.uid()));

grant select, insert, delete on public.corazones to authenticated;
grant select, insert, delete on public.seguidores to authenticated;
grant all on public.corazones, public.seguidores to service_role;
