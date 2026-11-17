# Comunidad de Lienzo UI: seguridad y puesta en marcha

La comunidad (plugins, imágenes, fuentes, plantillas y elementos) usa **Supabase** como servidor. Este documento explica cómo se protege y cómo se monta. El código del servidor está en `supabase/`.

> Estado: **servidor listo y probado en su lógica; falta crear el proyecto de Supabase y conectar la pestaña Comunidad de la app.** Todo lo que se puede añadir de forma externa (carpeta `plugins/`, abrir imágenes y diseños a mano) sigue funcionando sin cuenta ni internet.

## Qué datos se guardan de cada persona

| Dato | Dónde | Para qué |
|---|---|---|
| Identificador de la cuenta y correo | `auth.users` (gestionado por Supabase) | Iniciar sesión. **El correo no se copia a ninguna tabla nuestra ni se muestra a nadie.** |
| Nombre público (3-32 caracteres) | `perfiles` | Firmar lo que subes |
| Preferencia +18 | `perfiles` | Mostrar u ocultar contenido para mayores de 18 |
| Contenido que subes y su estado | `items` | Catálogo |

**No hay contraseñas**: el acceso es solo con proveedores externos (GitHub, Google…). No se guardan IP, dispositivo ni analítica. Tu cuenta y todo lo tuyo se borra con un botón (función `borrar-cuenta`).

## Defensas, capa por capa

1. **Cerrado por defecto (RLS).** Ninguna tabla es legible o editable sin una política explícita. Un usuario solo ve lo aprobado y lo suyo; no puede aprobar, cambiar rutas ni quitar la marca +18 (permisos de columna + políticas, `migrations/0001_comunidad.sql`).
2. **Cuarentena.** Todo archivo entra a un bucket privado (`cuarentena`) en la carpeta del propio usuario. No es público hasta ser aprobado; solo el servidor lo mueve al bucket `publico`.
3. **Solo formatos de datos.** PNG, JPEG, GIF, WebP, TTF, OTF y JSON. Se identifica el formato por los **bytes reales**, no por la extensión: un `.exe` renombrado a `.png` se rechaza. Se prohíben ejecutables, instaladores, scripts de sistema, comprimidos y SVG.
4. **Límites.** 2 MB imágenes, 5 MB fuentes, 512 KB plantillas, 256 KB plugins y elementos; 4096×4096 px; 10 subidas pendientes y 20 al día por persona.
5. **Plugins sin binarios.** Un plugin se sube como un único JSON con scripts Rhai en texto. Rutas con `..` o absolutas, archivos que no sean `.rhai`/`.json`, `eval`, `import`, enlaces o llamadas a procesos se rechazan. Además Rhai corre en una caja aislada en la app (sin archivos, red ni procesos, con límite de operaciones).
6. **Antivirus.** El archivo se comprueba automáticamente. Un plugin sin historial va a revisión humana.
7. **Moderación del texto.** El texto de la ficha (título, descripción, etiquetas y textos del diseño) se revisa automáticamente. **No se usa tu nombre, correo ni identificador.**
8. **Filtro +18.** Marca el autor; La moderación automática puede subir la marca, nunca bajarla. Por defecto está **apagado**: el contenido +18 no se muestra a nadie que no lo active en su perfil.
9. **Reportes.** Cualquier persona puede reportar. Con 3 reportes distintos el contenido se oculta hasta que lo revise una persona.
10. **Descargas verificadas.** La app comprueba el SHA-256 del archivo descargado antes de instalarlo, y vuelve a validarlo con las mismas reglas.
11. **Registro de moderación.** Guarda qué se decidió y por qué (sin datos personales). Solo el servidor lo lee.

## Límites honestos

- **Imágenes:** pasan por revisión automática y, si hay duda, por una persona.
- **Ningún antivirus es infalible.** Por eso el contenido es solo datos y los plugins corren aislados; el antivirus es una capa más, no la única.
- Alguien tiene que revisar la cola de «pendiente». Sin moderadores, lo dudoso simplemente no se publica.

## Puesta en marcha (paso a paso)

1. Crea un proyecto en [supabase.com](https://supabase.com) (región cercana a tus usuarios).
2. Instala el CLI (`npm i -g supabase`) y enlaza: `supabase link --project-ref <ref>`.
3. Aplica el esquema: `supabase db push`.
4. En **Authentication → Providers**, activa **GitHub** y **Google** con las credenciales de cada uno (cada proveedor te da un *client id* y un *secret*; la URL de retorno es la que muestra Supabase). Para **itch.io** y **Unity**, que no vienen incluidos, se configuran como proveedor OIDC/personalizado si el servicio lo permite; se añaden después.
5. Guarda los secretos **en Supabase, nunca en el repositorio**:
   ```bash
   # los secretos del servidor se guardan en Supabase, nunca en el repositorio
   ```
6. Despliega las funciones: `supabase functions deploy moderar` y `supabase functions deploy borrar-cuenta`.
7. En **Authentication → URL Configuration → Redirect URLs** añade `http://127.0.0.1:53682/callback` (la app recibe el inicio de sesión en ese puerto local temporal).
8. La app solo necesita dos valores **públicos** (URL del proyecto y clave publicable, en `src/comunidad.rs`); antes:  la app solo necesita dos valores **públicos**: la URL y la clave `anon`. Con RLS la clave `anon` no da acceso a nada que no esté permitido. **La clave `service_role` jamás va en la app ni en el repositorio.**

## Pruebas

```bash
node --test supabase/functions/_compartido/validar.test.ts supabase/functions/_compartido/decidir.test.ts   # 23 pruebas
```

## Notas de la app

- **No hay conexión hasta que abres la pestaña Comunidad.**
- El token de sesión se guarda en el Administrador de credenciales de Windows (no en un archivo). En otros sistemas la sesión no se recuerda entre ejecuciones.
- Cada descarga se verifica con el SHA-256 aprobado y se valida de nuevo localmente antes de instalarse (`src/comunidad_validar.rs`, mismas reglas que el servidor).
- Los permisos base de las tablas están en `migrations/0004_permisos_base.sql`: el proyecto se creó con «exponer tablas automáticamente» desactivado, así que **cada tabla nueva necesita su `grant` explícito además de su política RLS**.

## Línea de comandos

Usan la sesión guardada por la app (misma que el botón de la pestaña). **Cierra la app antes**: el token de sesión rota y la app abierta con el token anterior perdería la sesión.

```bash
lienzo-ui --comunidad-subir <plugin|imagen|fuente|plantilla|elemento> <archivo|carpeta> --titulo "…" [--descripcion "…"] [--etiquetas a,b] [--18]
lienzo-ui --comunidad-reintentar     # vuelve a moderar tus fichas que quedaron pendientes sin decisión
```

Una ficha puede quedar **pendiente** con motivo (por ejemplo, una imagen mientras no haya moderador de imágenes, o un plugin que el antivirus no conoce): espera a una persona que la revise. Las rechazadas muestran el motivo y su archivo se borra.

> Las Edge Functions usan la clave de servicio, que también necesita sus `grant` (`migrations/0005_permisos_servidor.sql`). Tras cambiar `validar.ts` o `decidir.ts` hay que volver a desplegar `moderar`.

## Revisión humana (moderadores)

Lo que queda **pendiente** lo revisa una persona. Solo pueden hacerlo las cuentas cuyo identificador esté en el secreto `MODERADORES` del servidor (separadas por comas). Tu identificador sale con `lienzo-ui --comunidad-yo`; se configura con `supabase secrets set MODERADORES=<id>[,<id>…]`.

```bash
lienzo-ui --comunidad-revisar listar                      # pendientes, con motivo y un enlace temporal (10 min) al archivo
lienzo-ui --comunidad-revisar aprobar <id> [--18]         # lo publica (se vuelve a validar y se comprueba la huella)
lienzo-ui --comunidad-revisar rechazar <id> "motivo"      # lo rechaza y borra el archivo
```

La función `revisar` (`supabase/functions/revisar/`) comprueba que quien llama es moderador. **Conviene que quien aprueba no sea el autor del contenido.**

## Iconos, vistas previas y perfiles de creador

- **Icono y vista previa** (opcionales, para cualquier tipo de contenido): PNG de hasta 512×512 / 256 KB y 1920×1080 / 1 MB. Pasan la misma moderación: mientras no haya moderador de imágenes, el contenido que los lleva espera revisión humana. Por línea de comandos: `--icono icono.png --vista vista.png`.
- **Perfil de creador** (`perfiles_publicos`): foto de perfil (PNG ≤ 512×512), banner (PNG ≤ 1920×640), descripción (≤ 300, sin enlaces, revisada automáticamente), usuario de GitHub y enlaces a X, YouTube, itch.io, Twitch, Bluesky e Instagram (solo `https://` de esos dominios). Solo se edita a través de la función `perfil`; la tabla no se puede escribir desde la app.
- Un perfil es **público solo si la persona tiene al menos un contenido aprobado**; el propio siempre se ve. Nunca incluye el correo ni las preferencias.
- Moderadores: `--comunidad-revisar listar` muestra también las imágenes de perfil pendientes; se aprueban con `aprobar_perfil` desde la función `revisar`.

## Filtro +18

Lo decide **el servidor**: un visitante anónimo nunca recibe contenido +18, y con sesión solo se recibe si la cuenta declaró ser mayor de 18 años (`perfiles.declara_mayor_18_en`, obligatorio para activar `mostrar_18`).

## Seguir creadores y corazones

- **Corazón** (la «estrella» de la comunidad): se da a un contenido aprobado; cada creador acumula los que recibe. **Seguir**: en el perfil de un creador. El filtro **♥ Siguiendo** muestra solo el contenido de a quienes sigues.
- **Privacidad:** quién sigue a quién y quién dio cada corazón **no es público**: solo cada persona ve lo suyo (RLS) y el resto ve únicamente los totales, que solo mueven triggers del servidor.
- **Límites:** 200 corazones al día y 1000 creadores seguidos por persona; no se puede seguir a uno mismo.
- **Cabecera de la app:** un botón con tu foto de perfil (una imagen por defecto si no tienes) abre tu perfil de creador; sin sesión, abre la pestaña Comunidad. En una vista previa, un clic la amplía.
- Línea de comandos: `--comunidad-corazon <id-del-contenido>` y `--comunidad-seguir <id-del-creador>` (alternan).

## Temas

- Botón **Temas** en la cabecera: cinco temas incluidos (Lienzo oscuro, Lienzo claro, Medianoche, Atardecer, Bosque), colores editables uno a uno, esquinas, base clara u oscura y **fuente de la interfaz** (cualquier fuente del sistema o de tu biblioteca; las de la comunidad se instalan desde Comunidad › Fuentes).
- **Guardar como tema** lo deja en tu biblioteca (`temas/`); **Compartir en la comunidad** abre el formulario de subida con el tema listo. En Comunidad › Temas, **Instalar** lo guarda y lo pone en uso.
- Un tema es un JSON pequeño (≤ 32 KB) que solo admite colores `#RRGGBB`, esquinas 0-16, base clara y el **nombre** de una fuente: nunca archivos, enlaces ni código. Lo comprueban el servidor y la app.
- Para capturas o pruebas: `lienzo-ui --tema "Lienzo claro"` (solo esa sesión).

## Mi perfil de creador

Ventana al estilo de X: banner con la foto superpuesta, botones de cámara para cambiarlos, campos con etiqueta (nombre, biografía, GitHub y redes) y **Guardar** arriba. Cualquier imagen (PNG, JPEG, WebP, GIF o BMP) se recorta y se reduce sola (foto 400×400, banner 1500×500). Mientras una persona la revisa, tú ya la ves en tu perfil y en el botón de la cabecera; los demás, cuando se aprueba.

## Moderación automática

Cada subida de contenido y cada foto o banner de perfil pasa por una comprobación automática en las funciones `moderar` y `perfil`:

- **Formato:** solo formatos de datos, comprobados por su contenido real (no por la extensión).
- **Antivirus:** todos los archivos. Muchas detecciones = rechazo; pocas = revisión por una persona.
- **Texto:** moderación automática de la ficha.

**Reglas de decisión** (`decidir.ts`, con pruebas): lo limpio se publica solo; **ante la duda o un servicio caído no se publica** y queda pendiente para una persona. Todo queda en `moderacion_log` (paso, resultado y motivo, sin datos personales). `lienzo-ui --comunidad-revisar reanalizar [id]` vuelve a pasar el pendiente más antiguo.

Los secretos del servidor se guardan **en Supabase, nunca en el repositorio** (`supabase secrets set ...`).

> **Privacidad:** sube solo contenido pensado para ser público.
