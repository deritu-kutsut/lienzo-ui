-- Trigger functions are internal: nobody should be able to call them through the API (/rest/v1/rpc/...).
-- Triggers run the same; only the permission to call them manually is removed.
revoke all on function public.limitar_subidas() from public, anon, authenticated;
revoke all on function public.ocultar_por_reportes() from public, anon, authenticated;
revoke all on function public.rls_auto_enable() from public, anon, authenticated;

-- Only signed-in users can move the download counter (prevents inflating it anonymously).
revoke all on function public.registrar_descarga(uuid) from public, anon;
grant execute on function public.registrar_descarga(uuid) to authenticated;
