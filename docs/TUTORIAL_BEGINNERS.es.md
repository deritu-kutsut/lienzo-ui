# Tutorial para novatos

🌐 [English](TUTORIAL_BEGINNERS.md) · [Español](TUTORIAL_BEGINNERS.es.md) · [Português](TUTORIAL_BEGINNERS.pt.md) · [Français](TUTORIAL_BEGINNERS.fr.md) · [Deutsch](TUTORIAL_BEGINNERS.de.md) · [Italiano](TUTORIAL_BEGINNERS.it.md) · [Русский](TUTORIAL_BEGINNERS.ru.md) · [简体中文](TUTORIAL_BEGINNERS.zh.md) · [日本語](TUTORIAL_BEGINNERS.ja.md) · [한국어](TUTORIAL_BEGINNERS.ko.md)

Tu primera interfaz con Lienzo UI y tu primera exportación a Unity. No necesitas saber programar.

## 1. Instalar y abrir

1. Descarga `LienzoUI-windows.zip` desde las releases de GitHub o desde itch.io.
2. Descomprímelo y abre `lienzo-ui.exe`.
3. La primera vez se abre el **tutorial** (este mismo contenido, dentro de la app). Al terminarlo, o con «Saltar e ir al editor», se abre el editor. Para volver a verlo: botón **Tutorial** de la barra superior o **F1**.

![Tutorial dentro de la app](images/tutorial.png)

## 2. La pantalla

| Zona | Qué hay |
|---|---|
| Barra superior | Archivo, Editar, Ver, Lienzo, **Probar**, nombre del diseño, **Accesibilidad**, **Revisar**, **Temas**, **Tutorial**, **Guardar**, **Código…**, **Exportar a Unity** |
| Barra izquierda | **Jerarquía** (capas), **Elementos**, **Plantillas**, **Texto**, **Imágenes**, **Plugins**, **Comunidad** |
| Centro | El lienzo: la pantalla de tu juego (1920 × 1080 por defecto) |
| Derecha | El **Inspector**: propiedades de lo seleccionado |

![Editor](images/editor.png)

## 3. Empieza con una plantilla

En **Plantillas** elige una de las 15: Menú principal, HUD de juego, Ventana de diálogo, Ajustes, Inventario, Menú de pausa, Fin de partida, Selección de nivel, Pantalla de carga, Diálogo de personaje, Clasificación o Selección de personaje. Todo lo que trae se puede cambiar y Ctrl+Z deshace el cambio. Para empezar de cero: «En blanco». Dos de ellas tienen estilo pixel art: HUD RPG pixel art y Menú principal pixel art.

## 4. Edita

- Arrastra elementos desde **Elementos** al lienzo. En **Componentes listos** hay botones, barras de vida y maná, tarjetas, insignias y casillas de inventario.
- **Texto** trae estilos de título y piezas de HUD: marcador, temporizador, monedas, daño, combo, nivel y subtítulos.
- Clic selecciona; arrastrar mueve; las esquinas cambian el tamaño; el tirador superior gira.
- Doble clic en un texto para escribir.
- En el **Inspector** cambia colores, bordes, sombras, fuentes y anclas.

| Atajo | Acción |
|---|---|
| Ctrl+Z / Ctrl+Y | Deshacer / rehacer |
| Ctrl+D / Supr | Duplicar / borrar |
| Ctrl+G / Ctrl+Shift+G | Agrupar / desagrupar |
| Flechas (Shift) | Mover 1 px (10 px) |
| Ctrl+S | Guardar |
| F5 | Probar |
| Ctrl+E | Exportar a Unity |
| F1 | Tutorial |

## 5. Prueba

Pulsa **Probar** (F5): la interfaz se vuelve interactiva y puedes elegir otras pantallas (móvil, 4:3, ultrapanorámica) para ver cómo se adapta. Esc sale y el diseño vuelve a como estaba antes de la prueba.

## 6. Revisa

Pulsa **Revisar** en la barra superior. Avisa de letra demasiado pequeña, poco contraste, botones demasiado pequeños para tocar, elementos fuera de la página, nombres repetidos e imágenes que faltan. Clic en un aviso selecciona ese elemento para corregirlo.

## 7. Exporta a Unity

1. Pulsa **Exportar a Unity** (Ctrl+E). La primera vez elige la carpeta de tu proyecto (la que contiene `Assets`).
2. Vuelve a Unity: el diseño aparece como prefab en `Assets/EditorGUI/Prefabs`.
3. Arrástralo a la escena, o usa **Tools › Lienzo UI › Import design into scene…**.

Cada vez que vuelvas a exportar, Unity actualiza el prefab solo.

## Imágenes incluidas

La pestaña Imágenes de la barra izquierda trae arte listo para usar, sin necesidad de archivos:

- Las etiquetas de arriba filtran por Tus imágenes, Fondos, Marcos, Botones, Iconos, Pixel art y Degradados. El buscador encuentra por nombre o etiqueta.
- Haz clic en una imagen para añadirla, o arrástrala al lienzo. Los marcos y botones traen su borde 9-slice, así que se estiran sin deformar las esquinas. Los fondos se colocan detrás de todo.
- La imagen se copia a tu biblioteca la primera vez que la usas y se exporta a Unity con el diseño.
- Con los sprites de pixel art, marca «Pixel art» en el Inspector del lienzo (sin nada seleccionado) para que se vean nítidos.

## Pantallas verticales

Los móviles suelen usarse en vertical. Lienzo UI guarda un segundo diseño para eso, sin tocar el primero:

- Cambia de vista con los botones Paisaje / Retrato de la barra superior.
- Sin nada seleccionado, abre «Diseño vertical» en el Inspector y pulsa «Crear diseño vertical». Los elementos empiezan donde están en horizontal.
- En la vista vertical, lo que muevas o redimensiones solo cambia el diseño vertical. Clic derecho en un elemento › «Volver a horizontal» deshace un cambio.
- La exportación lleva los dos diseños: en Unity el prefab pasa solo al vertical cuando la pantalla es más alta que ancha.

Cada plantilla ya trae su propio diseño vertical: aplica una plantilla y pulsa Retrato para verla pensada para móviles (1080 × 1920).

## Importar desde Unity

¿Ya tienes una interfaz en Unity? Tráela a Lienzo UI:

- Archivo › Importar desde Unity…
- Elige un prefab de uGUI (.prefab) o un diseño que Lienzo UI exportó antes (Assets/EditorGUI/Designs/….json).
- Se abre como un diseño nuevo sin guardar: guárdalo como proyecto para conservarlo.
- Si algo no se pudo convertir con exactitud, una ventana muestra los avisos, así que nada se pierde en silencio.
- Si encuentra la carpeta de tu proyecto de Unity, las imágenes del diseño se localizan solas.

## No pierdas tu trabajo

- Autoguardado: más o menos cada minuto Lienzo UI guarda una copia de recuperación. Puedes desactivarlo en Accesibilidad › Autoguardado.
- Si la aplicación se cierra de forma inesperada, la próxima vez pregunta «¿Recuperar el trabajo sin guardar…?». Pulsa Restaurar para recuperarlo o Descartar para olvidarlo. Lo recuperado aún no es un archivo guardado: pulsa Ctrl+S para conservarlo.
- Ctrl+S sigue guardando tu proyecto como siempre.
- Deshacer (Ctrl+Z) recuerda hasta 200 pasos (y tiene un límite de memoria). Si se descartan los más antiguos, la barra de estado muestra «Historial recortado».

## Comunidad

La pestaña Comunidad de la barra izquierda es un catálogo compartido de plugins, imágenes, fuentes, plantillas, elementos y temas:

- Explorar e instalar no necesita cuenta: busca y pulsa Instalar.
- Para subir, reportar o dar corazones inicias sesión con GitHub o Google y eliges un nombre público. Sin contraseñas: la app solo guarda ese nombre.
- Al crear tu perfil aceptas las reglas: nada de malware, odio, plagio ni contenido +18 sin marcar. El texto completo está en docs/TERMS.md y docs/COMMUNITY_GUIDELINES.md.
- El contenido +18 está oculto salvo que inicies sesión y actives «Mostrar contenido +18».
- Todo lo que se comparte se revisa antes de publicarse, y puedes reportar lo que incumpla las reglas.

En fuentes y plugins, pulsa Vista previa para ver un texto de ejemplo, o para leer qué hace un plugin y su código, antes de instalar; no se instala nada. Pulsa Traducir para leer cualquier descripción en tu idioma. Usa un servicio gratuito en línea, solo cuando lo pulsas, y «Ver original» vuelve atrás.

## Siguientes pasos

- [Tutorial para profesionales](TUTORIAL_PROS.es.md): qué genera la exportación y cómo poner tu lógica.
- [Servidor MCP](MCP.md): pide cambios a una IA en lenguaje natural.
