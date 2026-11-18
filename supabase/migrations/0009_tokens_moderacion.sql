-- Moderation tokens: credentials so a script or automation can review content through the API / the moderation web.
-- Design:
--   · The full token is shown ONCE when created; only its SHA-256 fingerprint is stored here (it cannot be recovered).
--   · Each token has scopes (read, approve, reject), an expiry (max. 90 days) and can be revoked instantly.
--   · Only a moderator (through their own account) can create, view or revoke them; a token cannot create others.
--   · No RLS policies: nobody reads them through the public API; only Edge Functions with the service key.
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
