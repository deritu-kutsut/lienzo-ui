-- Tokens de moderación: credenciales para que un script o una automatización revise contenido por la API / la web de moderación.
-- Diseño:
--   · El token completo se muestra UNA vez al crearlo; aquí solo se guarda su huella SHA-256 (no se puede recuperar).
--   · Cada token tiene alcances (leer, aprobar, rechazar), caducidad (máx. 90 días) y se puede revocar al instante.
--   · Solo una persona moderadora (por su cuenta) puede crearlos, verlos o revocarlos; un token no puede crear otros.
--   · Sin políticas RLS: nadie los lee desde la API pública; solo las Edge Functions con la clave de servicio.
create table if not exists public.tokens_moderacion (
  id uuid primary key default gen_random_uuid(),
  nombre text not null check (char_length(nombre) between 3 and 60),
  huella text not null unique check (huella ~ '^[0-9a-f]{64}$'),
  alcances text[] not null check (alcances <> '{}' and alcances <@ array['leer', 'aprobar', 'rechazar']),
  creado_por uuid not null references public.perfiles (id) on delete cascade,
  creado_en timestamptz not null default now(),
  expira_en timestamptz not null,
  ultimo_uso timestamptz,
  revocado boolean not null default false
);
alter table public.tokens_moderacion enable row level security;
revoke all on public.tokens_moderacion from anon, authenticated;
grant all on public.tokens_moderacion to service_role;
create index if not exists tokens_moderacion_creador_idx on public.tokens_moderacion (creado_por);
