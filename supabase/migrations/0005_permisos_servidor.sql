-- Las Edge Functions usan la clave de servicio (service_role). Con «exponer tablas automáticamente» desactivado,
-- esa clave tampoco tenía permisos sobre nuestras tablas (ignora RLS, pero necesita los permisos base).
-- La función «moderar» respondía «No encontrado» porque no podía leer la ficha.
grant all on public.items, public.perfiles, public.reportes, public.moderacion_log to service_role;
grant usage, select on all sequences in schema public to service_role;
