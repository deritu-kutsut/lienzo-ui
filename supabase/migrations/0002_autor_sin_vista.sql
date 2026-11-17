-- Corrige el aviso crítico «Security Definer View» (public.autores_publicos) del asesor de Supabase.
-- En lugar de una vista que salta las reglas de acceso, el nombre público del autor se copia a cada ficha
-- (items.autor_nombre) con un trigger. Así la tabla de perfiles sigue siendo privada y no hay nada que exponer.

drop view if exists public.autores_publicos;

alter table public.items add column if not exists autor_nombre text not null default '';

-- Al subir, el nombre lo pone el servidor (la persona no puede inventar otro autor).
create or replace function public.poner_autor_nombre() returns trigger language plpgsql security definer set search_path = public as $$
begin
  select coalesce(nombre_publico, '') into new.autor_nombre from public.perfiles where id = new.autor;
  return new;
end $$;
revoke all on function public.poner_autor_nombre() from public, anon, authenticated;
create trigger items_autor_nombre before insert on public.items for each row execute function public.poner_autor_nombre();

-- Si la persona cambia su nombre público, se actualizan sus fichas.
create or replace function public.propagar_autor_nombre() returns trigger language plpgsql security definer set search_path = public as $$
begin
  update public.items set autor_nombre = new.nombre_publico where autor = new.id;
  return new;
end $$;
revoke all on function public.propagar_autor_nombre() from public, anon, authenticated;
create trigger perfiles_propagar_nombre after update of nombre_publico on public.perfiles for each row execute function public.propagar_autor_nombre();

update public.items i set autor_nombre = p.nombre_publico from public.perfiles p where p.id = i.autor;
