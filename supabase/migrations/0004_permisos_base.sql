-- The project was created with «expose tables automatically» turned off: no table had base permissions and the API
-- returned 42501 even though the RLS policies were fine. Only the minimum permissions are granted here; the RLS policies
-- still decide which rows are seen or touched.

-- Catalog: anyone can read it (RLS: only approved and own items).
grant select on public.items to anon, authenticated;
grant insert on public.items to authenticated;      -- RLS: only as «pendiente» and under one's own name
grant delete on public.items to authenticated;      -- RLS: own items only
-- (update of titulo, descripcion and etiquetas was already granted in 0001)

-- Profiles: signed-in only and, by RLS, only one's own.
grant select, insert on public.perfiles to authenticated;
-- (update of nombre_publico, mostrar_18 and acepto_terminos_en was already granted in 0001)

-- Reports: can be created; nobody reads them through the API.
grant insert on public.reportes to authenticated;

-- moderacion_log: no permissions for anon or authenticated (server only).
revoke all on public.moderacion_log from anon, authenticated;
