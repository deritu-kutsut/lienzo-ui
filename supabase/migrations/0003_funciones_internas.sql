-- Las funciones de los triggers son internas: nadie debe poder llamarlas por la API (/rest/v1/rpc/...).
-- Los triggers se ejecutan igual; solo se quita el permiso de llamarlas a mano.
revoke all on function public.limitar_subidas() from public, anon, authenticated;
revoke all on function public.ocultar_por_reportes() from public, anon, authenticated;
revoke all on function public.rls_auto_enable() from public, anon, authenticated;

-- El contador de descargas solo lo puede mover quien tiene sesión (evita inflarlo de forma anónima).
revoke all on function public.registrar_descarga(uuid) from public, anon;
grant execute on function public.registrar_descarga(uuid) to authenticated;
