-- Edge Functions use the service key (service_role). With «expose tables automatically» turned off,
-- that key also had no permissions on our tables (it ignores RLS, but needs the base permissions).
-- The «moderar» function returned «No encontrado» because it could not read the item.
grant all on public.items, public.perfiles, public.reportes, public.moderacion_log to service_role;
grant usage, select on all sequences in schema public to service_role;
