-- Los temas de la interfaz (colores, esquinas y fuente) también se comparten en la comunidad.
alter type public.tipo_item add value if not exists 'tema';
