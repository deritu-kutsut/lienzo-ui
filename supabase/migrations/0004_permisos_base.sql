-- El proyecto se creó con «exponer tablas automáticamente» desactivado: ninguna tabla tenía permisos base y la API
-- respondía 42501 aunque las políticas RLS estuvieran bien. Aquí se dan SOLO los permisos mínimos; las políticas RLS
-- siguen decidiendo qué filas se ven o se tocan.

-- Catálogo: cualquiera lo lee (RLS: solo lo aprobado y lo propio).
grant select on public.items to anon, authenticated;
grant insert on public.items to authenticated;      -- RLS: solo como «pendiente» y a nombre propio
grant delete on public.items to authenticated;      -- RLS: solo lo propio
-- (update de titulo, descripcion y etiquetas ya se concedió en 0001)

-- Perfiles: solo con sesión y, por RLS, solo el propio.
grant select, insert on public.perfiles to authenticated;
-- (update de nombre_publico, mostrar_18 y acepto_terminos_en ya se concedió en 0001)

-- Reportes: se puede crear; nadie los lee desde la API.
grant insert on public.reportes to authenticated;

-- moderacion_log: sin permisos para anon ni authenticated (solo el servidor).
revoke all on public.moderacion_log from anon, authenticated;
