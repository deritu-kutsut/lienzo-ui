-- Fixes the critical «Security Definer View» warning (public.autores_publicos) from the Supabase advisor.
-- Instead of a view that bypasses the access rules, the author's public name is copied to each item
-- (items.autor_nombre) with a trigger. This way the profiles table stays private and there is nothing to expose.

drop view if exists public.autores_publicos;

alter table public.items add column if not exists autor_nombre text not null default '';

-- On upload, the server sets the name (the person cannot invent another author).
create or replace function public.poner_autor_nombre() returns trigger language plpgsql security definer set search_path = public as $$
begin
  select coalesce(nombre_publico, '') into new.autor_nombre from public.perfiles where id = new.autor;
  return new;
end $$;
revoke all on function public.poner_autor_nombre() from public, anon, authenticated;
create trigger items_autor_nombre before insert on public.items for each row execute function public.poner_autor_nombre();

-- If the person changes their public name, their items are updated.
create or replace function public.propagar_autor_nombre() returns trigger language plpgsql security definer set search_path = public as $$
begin
  update public.items set autor_nombre = new.nombre_publico where autor = new.id;
  return new;
end $$;
revoke all on function public.propagar_autor_nombre() from public, anon, authenticated;
create trigger perfiles_propagar_nombre after update of nombre_publico on public.perfiles for each row execute function public.propagar_autor_nombre();

update public.items i set autor_nombre = p.nombre_publico from public.perfiles p where p.id = i.autor;
