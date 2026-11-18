# Changelog

Formato basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/); versiones con [SemVer](https://semver.org/lang/es/).

## [Sin publicar]

### Corregido
- Modo prueba: `mover`, `posicion` y `girar` mueven también los hijos del elemento; `valor` en un desplegable elige la opción (antes se limitaba a 0–1); `leer_texto` y `leer_valor` avisan si el elemento no existe; números infinitos ya no dejan coordenadas rotas.
- Doble clic sobre un elemento dentro de un Contenedor ya no devuelve la selección al Contenedor al soltar.
- Alt+clic sin arrastrar ya no deja una copia encima.
- Pegar (Ctrl+V) un Contenedor o panel lo pone a su lado en vez de dentro de sí mismo.
- Agrupar y desagrupar respetan los elementos bloqueados; desagrupar contenedores anidados ya no deja la selección con elementos borrados.
- Subir o bajar varios elementos seguidos con Ctrl+] / Ctrl+[ ya no se anula, y «Al fondo» conserva su orden.
- Al agregar un componente se conservan su rotación y su nombre (el «Combo» llegaba recto y «Botón píldora» pasaba a «Botón 12»).
- Escalar una selección con elementos bloqueados ya no deforma el resto.
- Las opciones de un desplegable se pueden escribir con Enter en el inspector.
- Abrir un proyecto mientras se edita un texto ya no deja la edición abierta sobre el diseño nuevo.
- Plugins: una ruta como `\carpeta\script.rhai` ya no puede salir de la carpeta del plugin en Windows.
- Subir una fuente con el mismo nombre de archivo que otra ya no hace que se use la vieja al reiniciar.
- Archivos dañados o ajenos: se corrigen letra de tamaño absurdo, bordes y radios negativos, opacidad fuera de rango, números infinitos, página sin tamaño y caracteres de control (que rompían el UXML exportado).
- Un diseño titulado «CON», «NUL», «COM1»… se exporta con un nombre de archivo válido en Windows.

## [1.3.2] - 2026-11-18

Chino, japonés y coreano: la app ya habla diez idiomas.

### Añadido
- **Chino simplificado, japonés y coreano** en la interfaz, el tutorial integrado y los tutoriales de `docs/` (10 idiomas en total). La app usa las fuentes que ya trae el sistema (Microsoft YaHei, Yu Gothic, Malgun Gothic; en macOS y Linux, PingFang, Hiragino, Apple SD Gothic Neo o Noto Sans CJK) y solo las carga al elegir uno de estos idiomas; si no encuentra ninguna, Accesibilidad lo avisa.
- Tutoriales escritos de `docs/` (novatos y profesionales) en portugués, francés, alemán, italiano y ruso, con una línea para cambiar de idioma en cada uno.

## [1.3.1] - 2026-11-18

La interfaz y el tutorial, en siete idiomas.

### Añadido
- **5 idiomas nuevos en la interfaz:** portugués, francés, alemán, italiano y ruso (Accesibilidad › Idioma). El tutorial integrado también está traducido a los cinco.
- Tutoriales en inglés en `docs/` (`TUTORIAL_BEGINNERS.md` y `TUTORIAL_PROS.md`), y tutoriales al día con la 1.3.
- `cargo test` valida cada idioma: claves existentes, sin repetir, huecos `{}` y saltos `\n` conservados y al menos el 95 % traducido.

### Corregido
- Se traducen los tamaños de página, las pantallas del modo prueba, la lista de atajos, los títulos «Controles de UI», «Contenedores y Layout Groups» y «Formas», el tipo de elemento, la fuente predeterminada y los títulos de componente del inspector.
- Las etiquetas largas de las miniaturas se acortan con «…» en vez de pisar la de al lado; el nombre completo aparece al pasar el ratón.

## [1.3.0] - 2026-11-18

Siete plantillas nuevas, componentes y estilos listos, la herramienta **Revisar diseño** y correcciones.

### Añadido
- **7 plantillas nuevas:** Menú de pausa, Fin de partida, Selección de nivel, Pantalla de carga, Diálogo de personaje, Clasificación y Selección de personaje (13 en total). «En blanco» sigue siendo la 5 y se muestra al final.
- **Revisar diseño** (botón **Revisar** en la cabecera, `--revisar` en la línea de comandos y `revisar_diseno` en el MCP): avisa de letra demasiado pequeña, poco contraste (WCAG), controles demasiado pequeños para tocar, elementos fuera de la página, nombres repetidos que confunden a `GameObject.Find` e imágenes que faltan. Clic en un problema selecciona el elemento.
- **Elementos › Componentes listos:** botón píldora, botón contorno, botón cerrar, barras de vida y maná, tarjeta, panel translúcido, insignia y casilla.
- **Texto › Para botones y HUD:** temporizador, monedas, daño, combo, nivel y subtítulo de diálogo.
- **Imágenes:** 8 degradados más (Océano, Desierto, Neón, Hielo, Volcán, Caramelo, Madera, Viñeta).

### Corregido
- Las plantillas aplicadas a páginas más pequeñas que 1920×1080 (720p, 800×600, vertical) se salían de la página: ahora se escalan para caber.
- Guardar, guardar como plantilla o exportar en modo prueba guardaba los cambios hechos por los clics de prueba.
- Abrir un proyecto o una plantilla en modo prueba y después salir de él devolvía el diseño anterior.
- **Distribuir** movía elementos bloqueados.
- Si faltaba una imagen, la exportación dejaba el proyecto de Unity a medias: ahora se comprueba antes de escribir nada.
- Con la interfaz en inglés seguían en español los nombres de los elementos, formas, estilos de texto, degradados y tipos de control. La búsqueda encuentra también por el nombre traducido.
- Más contraste en los botones naranjas y verdes de las plantillas.

### Rendimiento
- Las miniaturas de las plantillas se generan una vez (antes, cada frame) y las listas de la biblioteca ya no se copian cada frame.
- La visibilidad de los elementos al dibujar el lienzo se calcula en una pasada (antes, cuadrática).

## [1.2.4] - 2026-11-17

Traducción al inglés de los botones, tooltips y plantillas.

### Corregido
- Con la interfaz en inglés seguían en español los tooltips de los botones con icono (Deshacer, Rehacer, Duplicar, Eliminar, alineación…), los títulos de sección (Plantillas de interfaz, Mis plantillas, Estilos de texto…), los botones Atrás, Adelante, Al fondo, Al frente, Distribuir, Agrupar y Desagrupar, y los nombres de las plantillas. Ahora se traducen.

## [1.2.3] - 2026-11-17

Interfaz en inglés completa y límites en el servidor MCP.

### Corregido
- Con la interfaz en inglés, la barra de estado y varios textos del inspector (jerarquía, Layout Group, «Padre») seguían en español. Ahora se traducen.
- Servidor MCP: un elemento llamado «3» ya no pierde ante el elemento de id 3 (el nombre exacto tiene prioridad).
- Servidor MCP: límites de tamaño (nombre 120 caracteres, texto 5000, listas de 200 elementos). Un diseño con 5 MB de texto tardaba 32 s en abrirse.

## [1.2.2] - 2026-11-17

Robustez ante diseños malformados.

### Corregido
- Un diseño con padres en ciclo (o un elemento padre de sí mismo) ya no cuelga la app al seleccionar, borrar o duplicar.
- Exportar un diseño con ancho o alto negativos o no numéricos ya no cierra la app.
- Los ids repetidos y un `next_id` demasiado bajo se corrigen al abrir el archivo; las coordenadas no numéricas pasan a 0.
- Los títulos muy largos se acortan a 64 caracteres en los nombres de archivo y de clase exportados.

## [1.2.1] - 2026-11-17

Corrección del actualizador.

### Corregido
- Tras actualizar, **Reiniciar ahora** relanzaba la versión anterior: ahora abre la nueva.
- Las notas de la versión se muestran como texto plano en la ventana de actualización.

## [1.2.0] - 2026-11-17

Actualizaciones desde la app.

### Añadido
- **Actualizaciones desde la app**: al abrir, Lienzo UI consulta la última release de GitHub (solo `github.com`, sin enviar datos) y, si hay una versión nueva, aparece el botón **Actualizar a X.Y.Z** en la cabecera. Funciona igual si descargaste la app de GitHub o de itch.io. Descarga el zip, comprueba su huella SHA-256, reemplaza el `.exe` (el anterior queda como `.viejo` hasta el siguiente arranque) y ofrece reiniciar. Si algo no se puede verificar, abre la página de descarga. Se puede desactivar la búsqueda al abrir, omitir una versión o buscar a mano desde **Archivo → Buscar actualizaciones…**.

### Corregido
- Servidor MCP: valida rangos (`w`, `h`, `font_size`, `opacity`…), nombres vacíos o duplicados al renombrar, diseños corruptos y nombres de archivo reservados de Windows.

## [1.1.1] - 2026-11-17

Moderación automática y arranque maximizado.

### Añadido
- **Moderación automática**: todo lo que se sube (también fotos de perfil y banners) se comprueba solo; lo limpio se publica y lo dudoso queda pendiente para una persona. `reanalizar` vuelve a pasar lo pendiente.
- La ventana arranca **maximizada**.
- **Paquete de Unity** gratuito (`LienzoUI-for-Unity`): importador, ventana de acceso y 7 pantallas de ejemplo, enviado a la Asset Store.

## [1.1.0] - 2026-11-17

Comunidad, perfiles de creador y temas.

### Añadido
- **Comunidad**: pestaña para explorar, instalar, subir y reportar **plugins, imágenes, fuentes, plantillas, elementos y temas**. Se instala con un clic tras verificar la huella SHA-256 y volver a validar el archivo. Explorar no necesita cuenta.
- **Cuentas con GitHub o Google** (sin contraseñas) y **perfil de creador**: foto, banner, biografía, GitHub y redes (X, YouTube, itch.io, Twitch, Bluesky, Instagram). Editor de perfil al estilo de X; las fotos se recortan y reducen solas y se ven al momento mientras se revisan. Botón de perfil con tu foto en la cabecera.
- **Seguir creadores** y **corazones** (como la estrella de GitHub), con filtro «Siguiendo». Quién sigue a quién no es público; solo los totales.
- **Subir contenido** con vista previa opcional (se amplía con un clic) y un icono SVG por tipo. Cada subida pasa por cuarentena, validación de formato (solo formatos de datos), antivirus y moderación de texto; lo dudoso espera revisión humana.
- **Filtro +18** aplicado en el servidor: solo lo ve quien confirma ser mayor de 18 años.
- **Temas**: botón **Temas** para cambiar colores, esquinas y fuente de toda la app. Cinco temas incluidos (Lienzo oscuro, Lienzo claro, Medianoche, Atardecer, Bosque), temas propios y compartibles en la comunidad.
- **Web de moderación** (`web/moderacion`): panel para revisar lo pendiente por categorías, con acceso para personas (cuenta) y para scripts (tokens revocables con alcances y caducidad). La orden `--comunidad-revisar` también aprueba fotos de perfil y gestiona tokens.
- Iconos SVG propios en lugar de emojis; ejecutable `Lienzo UI.exe`.
- Servidor en `supabase/` (esquema con RLS, funciones de moderación) y guías `docs/COMUNIDAD.md` y `docs/MODERACION.md`.

### Cambiado
- Los colores de la interfaz se leen del tema en uso (antes eran constantes).

### Seguridad
- Sin contraseñas ni correos en la app; sesión guardada en el Administrador de credenciales de Windows; la app no se conecta hasta que abres la pestaña Comunidad.

## [1.0.0] - 2026-11-16

Primera versión pública de **Lienzo UI** (antes «Editor GUI»).

### Añadido
- **Pixel art**: imágenes sin suavizado y píxeles enteros en el editor; Canvas Pixel Perfect y sprites Point sin compresión en Unity.
- **Plugins**: comandos, exportadores y plantillas con scripts Rhai aislados; pestaña Plugins y tres plugins de ejemplo.
- **Accesibilidad**: interfaz en español o inglés, tamaño hasta 200 % (Ctrl+Alt+= / -), alto contraste y movimiento reducido. Tutorial completo en inglés.
- Nombre, icono SVG y portada.
- Tutorial integrado para novatos (5 pasos) y profesionales (9 pasos): se abre la primera vez; al terminarlo se abre el editor; botón **Tutorial** y F1 para reabrirlo.
- Línea de comandos: `--plantillas`, `--plantilla`, `--abrir`, `--guardar`, `--exportar` (sin ventana), `--tutorial`, `--sin-tutorial`.
- Servidor MCP (`mcp/lienzo-ui-mcp.mjs`) con 12 herramientas, llamadas en cola y `mcp/probar-mcp.mjs` (32 comprobaciones).
- Documentación: tutoriales, guía para extender, MCP y contribución. Licencia MIT.

### Probado
- Las 5 plantillas y dos diseños creados por MCP («Tienda del herrero» y «HUD pixel art») exportados a Unity 6000.3 (URP): prefabs uGUI, UI Toolkit e IMGUI sin errores; el modo pixel art deja el Canvas en Pixel Perfect y los sprites en Point.

### Corregido (encontrado al probar en Unity)
- Sin los TMP Essential Resources los textos quedaban sin fuente y fallaban en RectMask2D: el importador ahora lo detecta, abre TMP Importer y regenera los prefabs.
- Títulos que se partían en uGUI (fuente de TMP más ancha): los textos se reducen hasta caber.
- Cuadrícula de UI Toolkit con una columna de menos por el espaciado.
