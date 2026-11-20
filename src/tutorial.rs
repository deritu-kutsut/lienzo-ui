//! Built-in tutorial with two tracks (beginners and professionals).
//! It opens by itself the first time Lienzo UI starts; when finished or skipped, the editor remains.
//! The "Tutorial" button in the top bar or F1 reopens it.
//! To add a page, add an element to `BEGINNER` and `PRO` (English) and to `BEGINNER_ES` and `PRO_ES` (Spanish), plus a page in each `assets/tutorial/<code>.json`.
//! The template page names the number of templates: keep it in sync with `model::TEMPLATES` (a test checks it).

use crate::app::EditorApp;
use crate::theme::{ACCENT, LINE, MUTED, PANEL, TEXT};
use eframe::egui::{self, Align, Color32, Layout, RichText, Vec2};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Track {
    Beginner,
    Pro,
}

pub struct Page {
    pub title: &'static str,
    pub text: &'static str,
}

/// State of the open tutorial.
#[derive(Clone, Copy, Debug)]
pub struct Tutorial {
    pub track: Track,
    pub page: usize,
}

pub const BEGINNER_ES: &[Page] = &[
    Page {
        title: "Bienvenido a Lienzo UI",
        text: "Lienzo UI es un editor visual para diseñar las interfaces de tu juego (menús, HUD, ventanas, inventarios) \
como en una herramienta de diseño, y llevarlas a Unity con un clic.\n\n\
Al exportar obtienes la misma interfaz en uGUI con TextMeshPro, en UI Toolkit (UXML + USS) y en IMGUI.\n\n\
Al terminar este tutorial se abre el editor. Puedes volver aquí cuando quieras con el botón «Tutorial» de la barra superior o con F1.",
    },
    Page {
        title: "La pantalla del editor",
        text: "• Arriba: los menús Archivo, Editar, Ver y Lienzo, el botón Probar, el nombre del diseño, Accesibilidad, Revisar, Temas, Tutorial, Guardar, Código… y Exportar a Unity.\n\
• A la izquierda, la barra con Jerarquía (las capas), Elementos, Plantillas, Texto, Imágenes, Plugins y Comunidad.\n\
• En el centro, el lienzo: la pantalla de tu juego a la resolución de referencia (1920 × 1080 por defecto).\n\
• A la derecha, el Inspector: las propiedades del elemento seleccionado (posición, anclas, colores, texto…).",
    },
    Page {
        title: "Empieza con una plantilla",
        text: "Abre Plantillas en la barra izquierda y elige una de las 15: menú principal, HUD, pausa, fin de partida, selección de nivel, inventario, diálogo de personaje, clasificación… Dos de ellas tienen estilo pixel art: HUD RPG pixel art y Menú principal pixel art.\n\n\
Es la forma más rápida de ver cómo se construye una interfaz completa. Todo lo que trae se puede cambiar.\n\n\
Si prefieres empezar de cero, elige «En blanco» o Archivo › Nuevo diseño en blanco.",
    },
    Page {
        title: "Añadir y editar elementos",
        text: "• Arrastra elementos desde Elementos al lienzo: paneles, textos, botones, casillas, sliders, campos de texto, desplegables, listas con scroll e imágenes. En «Componentes listos» hay botones, barras de vida y maná, tarjetas e insignias; en Texto, piezas de HUD como temporizador, monedas o daño.\n\
• Clic selecciona, arrastrar mueve, las esquinas cambian el tamaño y el tirador superior gira.\n\
• Doble clic en un texto para escribir en él.\n\
• Ctrl+Z / Ctrl+Y deshacen y rehacen; Ctrl+D duplica; Supr borra; Ctrl+G agrupa; las flechas mueven 1 px (Shift: 10).\n\
• En el Inspector cambias colores, bordes, sombras, fuentes y las anclas (cómo se adapta el elemento a otras pantallas).",
    },
    Page {
        title: "Imágenes incluidas",
        text: "La pestaña Imágenes de la barra izquierda trae arte listo para usar, sin necesidad de archivos:\n\n• Las etiquetas de arriba filtran por Tus imágenes, Fondos, Marcos, Botones, Iconos, Pixel art y Degradados. El buscador encuentra por nombre o etiqueta.\n• Haz clic en una imagen para añadirla, o arrástrala al lienzo. Los marcos y botones traen su borde 9-slice, así que se estiran sin deformar las esquinas. Los fondos se colocan detrás de todo.\n• La imagen se copia a tu biblioteca la primera vez que la usas y se exporta a Unity con el diseño.\n• Con los sprites de pixel art, marca «Pixel art» en el Inspector del lienzo (sin nada seleccionado) para que se vean nítidos.",
    },
    Page {
        title: "Pantallas verticales",
        text: "Los móviles suelen usarse en vertical. Lienzo UI guarda un segundo diseño para eso, sin tocar el primero:\n\n• Cambia de vista con los botones Paisaje / Retrato de la barra superior.\n• Sin nada seleccionado, abre «Diseño vertical» en el Inspector y pulsa «Crear diseño vertical». Los elementos empiezan donde están en horizontal.\n• En la vista vertical, lo que muevas o redimensiones solo cambia el diseño vertical. Clic derecho en un elemento › «Volver a horizontal» deshace un cambio.\n• La exportación lleva los dos diseños: en Unity el prefab pasa solo al vertical cuando la pantalla es más alta que ancha.\n\nCada plantilla ya trae su propio diseño vertical: aplica una plantilla y pulsa Retrato para verla pensada para móviles (1080 × 1920).",
    },
    Page {
        title: "Probar y exportar a Unity",
        text: "• Probar (F5): el diseño se vuelve interactivo y puedes simular otras pantallas (móvil, 4:3, ultrapanorámica) para ver cómo se adapta.\n\
• Guardar (Ctrl+S): guarda el proyecto .editorgui.json para seguir otro día.\n\
• Revisar: avisa de letra pequeña, poco contraste, botones pequeños para tocar, elementos fuera de la página, nombres repetidos e imágenes que faltan. Junto a cada problema con solución segura hay un botón «Corregir», y «Corregir todos los problemas seguros» los arregla todos en un solo paso de deshacer.\n\
• Exportar a Unity (Ctrl+E): la primera vez elige la carpeta de tu proyecto (la que contiene Assets).\n\n\
En Unity, al volver a la ventana, el diseño aparece como prefab en Assets/EditorGUI/Prefabs. También tienes el menú Tools › Lienzo UI › Import design into scene….",
    },
    Page {
        title: "Importar desde Unity",
        text: "¿Ya tienes una interfaz en Unity? Tráela a Lienzo UI:\n\n• Archivo › Importar desde Unity…\n• Elige un prefab de uGUI (.prefab) o un diseño que Lienzo UI exportó antes (Assets/EditorGUI/Designs/….json).\n• Se abre como un diseño nuevo sin guardar: guárdalo como proyecto para conservarlo.\n• Si algo no se pudo convertir con exactitud, una ventana muestra los avisos, así que nada se pierde en silencio.\n• Si encuentra la carpeta de tu proyecto de Unity, las imágenes del diseño se localizan solas.",
    },
    Page {
        title: "No pierdas tu trabajo",
        text: "• Autoguardado: más o menos cada minuto Lienzo UI guarda una copia de recuperación. Puedes desactivarlo en Accesibilidad › Autoguardado.\n• Si la aplicación se cierra de forma inesperada, la próxima vez pregunta «¿Recuperar el trabajo sin guardar…?». Pulsa Restaurar para recuperarlo o Descartar para olvidarlo. Lo recuperado aún no es un archivo guardado: pulsa Ctrl+S para conservarlo.\n• Ctrl+S sigue guardando tu proyecto como siempre.\n• Deshacer (Ctrl+Z) recuerda hasta 200 pasos (y tiene un límite de memoria). Si se descartan los más antiguos, la barra de estado muestra «Historial recortado».",
    },
    Page {
        title: "Comunidad",
        text: "La pestaña Comunidad de la barra izquierda es un catálogo compartido de plugins, imágenes, fuentes, plantillas, elementos y temas:\n\n• Explorar e instalar no necesita cuenta: busca y pulsa Instalar.\n• Para subir, reportar o dar corazones inicias sesión con GitHub o Google y eliges un nombre público. Sin contraseñas: la app solo guarda ese nombre.\n• Al crear tu perfil aceptas las reglas: nada de malware, odio, plagio ni contenido +18 sin marcar. El texto completo está en docs/TERMS.md y docs/COMMUNITY_GUIDELINES.md.\n• El contenido +18 está oculto salvo que inicies sesión y actives «Mostrar contenido +18».\n• Todo lo que se comparte se revisa antes de publicarse, y puedes reportar lo que incumpla las reglas.\n\nEn fuentes y plugins, pulsa Vista previa para ver un texto de ejemplo, o para leer qué hace un plugin y su código, antes de instalar; no se instala nada. Pulsa Traducir para leer cualquier descripción en tu idioma. Usa un servicio gratuito en línea, solo cuando lo pulsas, y «Ver original» vuelve atrás.",
    },
];

pub const PRO_ES: &[Page] = &[
    Page {
        title: "Qué genera la exportación",
        text: "Todo va a Assets/EditorGUI de tu proyecto:\n\n\
• Designs/<Diseño>.json: el diseño; el importador rehace el prefab cada vez que cambia.\n\
• Editor/EditorGuiImporter.cs: construye el prefab con uGUI + TextMeshPro.\n\
• Prefabs/<Diseño>.prefab: listo para arrastrar a la escena.\n\
• UI Toolkit/<Diseño>.uxml y .uss: la versión para UIDocument.\n\
• Scripts/<Diseño>OnGUI.cs: la versión IMGUI.\n\
• Scripts/<Diseño>ViewBase.cs (se regenera) y <Diseño>View.cs (tuyo: se crea una vez y nunca se sobrescribe).",
    },
    Page {
        title: "Anclas y escalado",
        text: "El lienzo funciona como un Canvas con Canvas Scaler en modo Scale With Screen Size: referencia, match ancho/alto y fondo se ajustan en el Inspector con nada seleccionado.\n\n\
Cada elemento tiene anclas, pivote y desplazamientos como un RectTransform. Al cambiar la resolución de referencia los elementos siguen sus anclas, igual que en Unity.\n\n\
Usa Probar (F5) con varias pantallas para comprobarlo antes de exportar.",
    },
    Page {
        title: "Diseño vertical al detalle",
        text: "El diseño vertical es una segunda vista del mismo diseño (docs/FORMAT.md › PortraitLayout):\n\n• Tiene su propia resolución de referencia, «Tamaño vertical» (1080 × 1920 por defecto).\n• Los elementos sin excepción conservan su RectTransform horizontal y se reevalúan contra la página vertical: un fondo a pantalla completa se estira, los anclados a la derecha siguen el borde derecho.\n• Solo se guarda lo que cambias en la vista vertical, como excepciones (posición, tamaño, rotación, anclajes, visibilidad, tamaño de fuente, layout). Clic derecho › «Volver a horizontal» quita una.\n• Exportación: el JSON lleva las excepciones y el script generado LienzoOrientationLayout las aplica cuando Screen.width < Screen.height, y de nuevo si cambia el tamaño.\n• «Quitar diseño vertical» borra todas las excepciones.\n\nLas plantillas traen un diseño vertical ya incorporado, guardado como excepciones del horizontal, así que Retrato muestra desde el principio un diseño pensado para móviles.",
    },
    Page {
        title: "Layouts, grupos y listas",
        text: "Un contenedor puede ordenar a sus hijos en horizontal, vertical o cuadrícula (como Horizontal/Vertical/Grid Layout Group), con espaciado y relleno.\n\n\
Las listas con scroll (ScrollView) calculan su contenido solas.\n\n\
Agrupa (Ctrl+G) para mover partes enteras y usa la Jerarquía para reordenar capas: el orden de la lista es el orden de dibujo en Unity.",
    },
    Page {
        title: "9-slice y el paquete incluido",
        text: "Para marcos, botones y paneles que se estiran:\n\n• Selecciona una Imagen y define «9-slice» en «Ajustes de imagen (Unity)»: el borde en píxeles de la imagen original (izquierda, abajo, derecha, arriba), como en el Sprite Editor de Unity. La exportación genera un Sprite 9-slice.\n• «Píxeles por unidad»: 100 mantiene los bordes en su tamaño en píxeles, 200 los reduce a la mitad.\n• El paquete incluido (assets/pack/manifest.json, docs/CONTENT.md) ya define el corte y los píxeles por unidad: los marcos son de 512×512 con corte 96, los botones de 512×192.\n• Usar una imagen la copia a tu biblioteca como pack_<id>.<ext> (oculta en «Tus imágenes»); el exportador la copia a los Sprites de Unity.\n• Las plantillas HUD RPG pixel art y Menú principal pixel art usan el paquete pixel. Activa «Pixel art» para el filtro Point.",
    },
    Page {
        title: "Lógica: código al hacer clic y vistas",
        text: "En Probar, cada elemento puede ejecutar un pequeño script (Rhai) al hacer clic: show(\"Name\"), hide, toggle, set_text, set_color, move_by… Sirve para prototipar flujos sin Unity.\n\n\
En Unity, <Diseño>View.cs hereda de <Diseño>ViewBase con referencias tipadas a cada botón, slider o campo, y eventos de clic y de valor. Pon ahí la lógica del juego: las reexportaciones no la tocan.",
    },
    Page {
        title: "Revisión y correcciones rápidas",
        text: "La ventana Revisar lista errores y avisos; haz clic en uno para seleccionar su elemento. Seis problemas tienen un botón «Corregir» (un paso de deshacer):\n\n• Texto por debajo del tamaño mínimo: se sube al mínimo.\n• Control demasiado pequeño para tocar: se agranda alrededor de su centro.\n• Elemento parcialmente fuera de la página: se mueve dentro, si cabe.\n• Nombre repetido: se añade _2, _3…\n• Imagen que falta: se quita la referencia.\n• Poco contraste: se oscurece o aclara el color del texto.\n\n«Corregir todos los problemas seguros» los aplica en varias pasadas. Problemas como una fuente sin ciertos caracteres no tienen corrección automática. Las correcciones también funcionan en el diseño vertical. Por línea de comandos: lienzo-ui --open archivo --review (código 1 si hay errores).",
    },
    Page {
        title: "Importar desde Unity: prefabs y CLI",
        text: "Archivo › Importar desde Unity… lee un .prefab de uGUI (el YAML de texto de Unity) o un Designs/*.json exportado por Lienzo UI y crea un diseño normal.\n\n• Lo que no se pudo convertir aparece como avisos.\n• Si detecta la carpeta del proyecto de Unity, las imágenes se resuelven contra ella durante esa sesión.\n• Sin ventana: lienzo-ui --import-unity Menu.prefab --save Menu.editorgui.json (también con --review y --export).\n• Proyecto completo: lienzo-ui --import-unity-project MiProyecto --out Importados convierte todos los prefabs de interfaz. Nunca sobrescribe (añade _2) y omite los prefabs sin interfaz.\n• Detalles en docs/UNITY_IMPORT.md.",
    },
    Page {
        title: "Línea de comandos y automatización",
        text: "lienzo-ui --templates                          lista las plantillas\n\
lienzo-ui --template 0 --save menu.editorgui.json\n\
lienzo-ui --open menu.editorgui.json --review         revisa el diseño (código 1 si hay errores)\n\
lienzo-ui --open menu.editorgui.json --export <proyecto Unity>\n\
lienzo-ui --open menu.editorgui.json --screenshot capture.png\n\
lienzo-ui --test   1080x2400                    abre en Probar con esa pantalla\n\n\
Con --export, --save y --review no se abre ventana: sirve para scripts, CI y el servidor MCP.\n\nAñade --dry-run a --export para listar cada archivo como new, update, same o remove sin escribir nada.",
    },
    Page {
        title: "Servidor MCP y contribuir",
        text: "La carpeta mcp/ trae un servidor MCP sin dependencias (Node 18+): cualquier IA compatible (Claude, Cursor, VS Code…) puede listar plantillas, crear y editar diseños, exportarlos a Unity y sacar capturas.\n\nexport_unity escribe directamente en Assets/EditorGUI del proyecto que indique la IA, sin ningún aviso en el editor: la aprobación es la de herramientas de tu cliente MCP. Pide a la IA que lo llame antes con dry_run: true, lee la lista y después exporta de verdad.\n\n\
Lienzo UI es de código abierto (MIT). El código está en Rust con egui: src/model.rs (datos), src/canvas.rs (lienzo), src/panels.rs (paneles), src/export/ (Unity).\n\n\
Lee docs/EXTENDING.md para añadir elementos, plantillas o exportadores, y CONTRIBUTING.md para enviar cambios.",
    },
    Page {
        title: "Pixel art",
        text: "Con nada seleccionado, marca «Pixel art» en el Inspector del lienzo:\n\n\
• Las imágenes se dibujan sin suavizado (vecino más cercano).\n\
• Posiciones y tamaños se ajustan a píxeles enteros al soltar.\n\
• En Unity: Canvas «Pixel Perfect» y sprites con filtro Point, sin compresión ni mipmaps.\n\n\
Consejo: usa tamaños múltiplos exactos de tus sprites (16 px → 32, 48, 64…) para que cada píxel se vea igual.",
    },
    Page {
        title: "Plugins",
        text: "La pestaña Plugins de la barra izquierda lista los complementos instalados. Un plugin es una carpeta con plugin.json que puede añadir:\n\n\
• Comandos: scripts Rhai que reciben el diseño y lo devuelven modificado (se deshacen con Ctrl+Z).\n\
• Exportadores: scripts que generan archivos nuevos (por ejemplo, un CSV con todos los textos).\n\
• Plantillas: diseños listos para usar.\n\n\
Los scripts corren aislados, sin acceso a archivos ni red. «Abrir carpeta» lleva a tu carpeta de plugins; la guía está en docs/PLUGINS.md.",
    },
    Page {
        title: "Comunidad: plugins, fuentes y normas",
        text: "En la pestaña Comunidad, Instalar añade contenido compartido con un clic: plugins, fuentes, imágenes, plantillas, elementos y temas.\n\n• Un plugin de la comunidad va a una carpeta llamada community-<nombre>-<hash>. Se escribe primero en una carpeta temporal, así que una instalación interrumpida nunca deja un plugin a medias. Aparece en la pestaña Plugins.\n• Sus scripts son los mismos Rhai: aislados, sin acceso a archivos ni red y con límites de operaciones, tiempo y memoria (docs/PLUGINS.md).\n• Una fuente instalada se añade a la pestaña Texto y se exporta a Unity con el diseño.\n• Las descargas se comprueban con la huella SHA-256 publicada; si no coincide, no se instala nada.\n• Para publicar inicias sesión (GitHub o Google), eliges un nombre público y aceptas las reglas. Todo se revisa antes y el contenido +18 debe marcarse. Ver docs/TERMS.md, docs/COMMUNITY_GUIDELINES.md y docs/PRIVACY.md.\n\nVista previa muestra una fuente con texto de ejemplo, o los comandos, exportadores, archivos y el código fuente de un plugin (solo lectura), antes de instalar; no se instala nada ni se cuenta una descarga. Traducir envía solo esa descripción al servicio gratuito MyMemory cuando lo pulsas (docs/PRIVACY.md).",
    },
    Page {
        title: "Autoguardado, recuperación y deshacer",
        text: "• Más o menos cada minuto, mientras editas, el diseño completo (con el diseño vertical) se escribe en la carpeta de recuperación de la app. Nunca en modo Probar. Se conservan como máximo 10 recuperaciones.\n• Al iniciar, si hay una recuperación más reciente que el archivo del proyecto, se ofrece: Restaurar o Descartar. Tras un Guardar normal ya no hace falta.\n• Se puede desactivar en Accesibilidad › Autoguardado.\n• El historial de deshacer está limitado a 200 pasos y unos 64 MB. Cuando se descartan los pasos más antiguos, la barra de estado muestra «Historial recortado».\n• Cada comando de plugin y cada Corregir de la ventana Revisar es un solo paso de deshacer.",
    },
    Page {
        title: "Accesibilidad e idiomas",
        text: "El botón «Accesibilidad» de la barra superior permite:\n\n\
• Cambiar el idioma de la interfaz (español, inglés, portugués, francés, alemán, italiano, ruso, chino, japonés o coreano).\n\
• Agrandar la interfaz hasta el 250 % (también con Ctrl+Alt+= y Ctrl+Alt+-).\n\
• Activar alto contraste: texto blanco y bordes marcados.\n\
• Reducir el movimiento: sin animaciones.\n\n\
Los idiomas son archivos assets/i18n/<código>.tsv: para añadir uno basta con copiar es.tsv y traducir la copia.",
    },
];

pub const BEGINNER: &[Page] = &[
    Page {
        title: "Welcome to Lienzo UI",
        text: "Lienzo UI is a visual editor to design your game's interfaces (menus, HUDs, windows, inventories) \
like in a design tool, and bring them to Unity in one click.\n\n\
Exporting gives you the same interface in uGUI with TextMeshPro, in UI Toolkit (UXML + USS) and in IMGUI.\n\n\
When you finish this tutorial the editor opens. Come back any time with the “Tutorial” button in the top bar or F1.",
    },
    Page {
        title: "The editor screen",
        text: "• Top: the File, Edit, View and Canvas menus, the Test button, the design name, Accessibility, Review, Themes, Tutorial, Save, Code… and Export to Unity.\n\
• Left: the bar with Hierarchy (layers), Elements, Templates, Text, Images, Plugins and Community.\n\
• Center: the canvas, your game screen at the reference resolution (1920 × 1080 by default).\n\
• Right: the Inspector, with the properties of the selected element (position, anchors, colors, text…).",
    },
    Page {
        title: "Start from a template",
        text: "Open Templates in the left bar and pick one of the 15: main menu, HUD, pause, game over, level select, inventory, character dialogue, leaderboard… Two of them are in pixel art style: Pixel RPG HUD and Pixel main menu.\n\n\
It is the fastest way to see how a complete interface is built. Everything in it can be changed.\n\n\
To start from scratch, choose “Blank” or File › New blank design.",
    },
    Page {
        title: "Add and edit elements",
        text: "• Drag elements from Elements onto the canvas: panels, texts, buttons, toggles, sliders, input fields, dropdowns, scroll lists and images. “Ready-made components” has buttons, health and mana bars, cards and badges; Text has HUD pieces such as timer, coins or damage.\n\
• Click selects, drag moves, corners resize and the top handle rotates.\n\
• Double-click a text to type in it.\n\
• Ctrl+Z / Ctrl+Y undo and redo; Ctrl+D duplicates; Del deletes; Ctrl+G groups; arrows move 1 px (Shift: 10).\n\
• In the Inspector you change colors, borders, shadows, fonts and the anchors (how the element adapts to other screens).",
    },
    Page {
        title: "Built-in images",
        text: "The Images tab in the left bar includes ready-made art, no files needed:\n\n• The chips at the top filter by Your images, Backgrounds, Frames, Buttons, Icons, Pixel art and Gradients. The search box finds items by name or tag.\n• Click an item to add it, or drag it onto the canvas. Frames and buttons come with their 9-slice border, so they stretch without distorting the corners. Backgrounds go behind everything else.\n• The picture is copied to your library the first time you use it, and it is exported to Unity with the design.\n• With the pixel art sprites, tick “Pixel art” in the canvas Inspector (nothing selected) so they stay crisp.",
    },
    Page {
        title: "Vertical (portrait) screens",
        text: "Phones are often held upright. Lienzo UI keeps a second layout for that, without touching the first one:\n\n• Switch the view with the Landscape / Portrait buttons in the top bar.\n• With nothing selected, open “Portrait layout” in the Inspector and press “Create portrait layout”. Elements start where they are in landscape.\n• In Portrait view, whatever you move or resize changes only the portrait layout. Right-click an element › “Reset to landscape” to undo one change.\n• The export carries both layouts: in Unity the prefab switches to the vertical one by itself when the screen is taller than it is wide.\n\nEvery template already comes with its own portrait layout: apply a template, then press Portrait to see it laid out for phones (1080 × 1920).",
    },
    Page {
        title: "Test and export to Unity",
        text: "• Test (F5): the design becomes interactive and you can simulate other screens (phone, 4:3, ultrawide) to see how it adapts.\n\
• Save (Ctrl+S): saves the .editorgui.json project to keep working later.\n\
• Review: warns about small text, low contrast, buttons too small to tap, elements outside the page, repeated names and missing images. Next to each problem with a safe solution there is a “Fix” button, and “Fix all safe issues” applies them all in one undo step.\n\
• Export to Unity (Ctrl+E): the first time, choose your project folder (the one that contains Assets).\n\n\
Back in Unity, when you return to the window, the design appears as a prefab in Assets/EditorGUI/Prefabs. There is also the menu Tools › Lienzo UI › Import design into scene….",
    },
    Page {
        title: "Import from Unity",
        text: "Already have an interface in Unity? Bring it into Lienzo UI:\n\n• File › Import from Unity…\n• Choose a uGUI prefab (.prefab) or a design that Lienzo UI exported before (Assets/EditorGUI/Designs/….json).\n• It opens as a new, unsaved design: save it as a project to keep it.\n• If something could not be converted exactly, a window lists the warnings, so nothing is lost silently.\n• If it finds your Unity project folder, the images of the design are found automatically.",
    },
    Page {
        title: "Never lose your work",
        text: "• Autosave: about every minute Lienzo UI saves a recovery copy. You can turn it off in Accessibility › Autosave.\n• If the app closes unexpectedly, the next time it asks “Recover unsaved work…?”. Press Restore to get it back or Discard to drop it. Recovered work is not a saved file yet: press Ctrl+S to keep it.\n• Ctrl+S still saves your project as usual.\n• Undo (Ctrl+Z) remembers up to 200 steps (and a memory limit). If the oldest ones are dropped, the status bar says “History trimmed”.",
    },
    Page {
        title: "Community",
        text: "The Community tab in the left bar is a shared catalog of plugins, images, fonts, templates, elements and themes:\n\n• Browsing and installing needs no account: search and press Install.\n• To upload, report or give hearts you sign in with GitHub or Google and choose a public name. No passwords: the app only keeps that name.\n• When you create your profile you accept the rules: no malware, hate, plagiarism or unmarked 18+ content. The full text is in docs/TERMS.md and docs/COMMUNITY_GUIDELINES.md.\n• 18+ content is hidden unless you sign in and turn on “Show 18+ content”.\n• Everything shared is reviewed before it is published, and you can report anything that breaks the rules.\n\nOn fonts and plugins, press Preview to see sample text, or to read what a plugin does and its code, before installing; nothing is installed. Press Translate to read any description in your language. It uses a free online service, only when you press it, and “Show original” goes back.",
    },
];

pub const PRO: &[Page] = &[
    Page {
        title: "What the export generates",
        text: "Everything goes to Assets/EditorGUI in your project:\n\n\
• Designs/<Design>.json: the design; the importer rebuilds the prefab whenever it changes.\n\
• Editor/EditorGuiImporter.cs: builds the prefab with uGUI + TextMeshPro.\n\
• Prefabs/<Design>.prefab: ready to drop into a scene.\n\
• UI Toolkit/<Design>.uxml and .uss: the UIDocument version.\n\
• Scripts/<Design>OnGUI.cs: the IMGUI version.\n\
• Scripts/<Design>ViewBase.cs (regenerated) and <Design>View.cs (yours: created once, never overwritten).",
    },
    Page {
        title: "Anchors and scaling",
        text: "The canvas works like a Canvas with a Canvas Scaler in Scale With Screen Size mode: reference resolution, width/height match and background are set in the Inspector with nothing selected.\n\n\
Each element has anchors, pivot and offsets like a RectTransform. When the reference resolution changes, elements follow their anchors, just like in Unity.\n\n\
Use Test (F5) with several screens before exporting.",
    },
    Page {
        title: "Portrait layout in detail",
        text: "The portrait layout is a second view of the same design (docs/FORMAT.md › PortraitLayout):\n\n• It has its own reference resolution, “Portrait size” (1080 × 1920 by default).\n• Elements without an override keep their landscape RectTransform and are re-evaluated against the portrait page: a full-page background stretches, right-anchored elements follow the right edge.\n• Only what you change in Portrait view is stored, as overrides (position, size, rotation, anchors, visibility, font size, layout). Right-click › “Reset to landscape” removes one.\n• Export: the JSON carries the overrides and the generated LienzoOrientationLayout script applies them when Screen.width < Screen.height, and again when the size changes.\n• “Remove portrait layout” deletes every override.\n\nTemplates ship with a portrait layout already built in, stored as overrides of the landscape one, so Portrait shows a layout designed for phones from the start.",
    },
    Page {
        title: "Layouts, groups and lists",
        text: "A container can arrange its children horizontally, vertically or in a grid (like Horizontal/Vertical/Grid Layout Group), with spacing and padding.\n\n\
Scroll lists (ScrollView) compute their content automatically.\n\n\
Group (Ctrl+G) to move whole parts and use the Hierarchy to reorder layers: the list order is the drawing order in Unity.",
    },
    Page {
        title: "9-slice and the built-in pack",
        text: "For frames, buttons and panels that stretch:\n\n• Select an Image and set “9-slice” in “Image settings (Unity)”: the border in source-image pixels (left, bottom, right, top), like Unity's Sprite Editor. The export generates a 9-slice Sprite.\n• “Pixels per unit”: 100 keeps the borders at their pixel size, 200 makes them half as big.\n• The built-in pack (assets/pack/manifest.json, docs/CONTENT.md) already sets slice and pixels per unit: frames are 512×512 with slice 96, buttons 512×192.\n• Using an item copies it to your library as pack_<id>.<ext> (hidden from “Your images”); the exporter copies it to the Unity Sprites.\n• The templates Pixel RPG HUD and Pixel main menu use the pixel pack. Turn on “Pixel art” for Point filtering.",
    },
    Page {
        title: "Logic: click scripts and views",
        text: "In Test mode each element can run a small Rhai script on click: show(\"Name\"), hide, toggle, set_text, set_color, move_by… Useful to prototype flows without Unity.\n\n\
In Unity, <Design>View.cs inherits from <Design>ViewBase with typed references to every button, slider or field, and click and value events. Put your game logic there: re-exports never touch it.",
    },
    Page {
        title: "Review and quick fixes",
        text: "The Review window lists errors and warnings; click one to select its element. Six problems have a “Fix” button (one undo step):\n\n• Text below the minimum size: raised to the minimum.\n• Control too small to tap: enlarged around its center.\n• Element partly outside the page: moved inside, if it fits.\n• Repeated name: _2, _3… is appended.\n• Missing image: the reference is removed.\n• Low contrast: the text color is darkened or lightened.\n\n“Fix all safe issues” applies them in a few passes. Problems such as a font that lacks characters have no automatic fix. Fixes also work in the portrait layout. From the command line: lienzo-ui --open file --review (exit code 1 on errors).",
    },
    Page {
        title: "Import from Unity: prefabs and CLI",
        text: "File › Import from Unity… reads a uGUI .prefab (Unity's text YAML) or a Designs/*.json exported by Lienzo UI and creates an ordinary design.\n\n• What could not be mapped is listed as warnings.\n• When it detects the Unity project folder, images resolve against it for that session.\n• Headless: lienzo-ui --import-unity Menu.prefab --save Menu.editorgui.json (also with --review and --export).\n• Whole project: lienzo-ui --import-unity-project MyProject --out Imported converts every UI prefab. It never overwrites (it adds _2) and skips prefabs without UI.\n• Details in docs/UNITY_IMPORT.md.",
    },
    Page {
        title: "Command line and automation",
        text: "lienzo-ui --templates                          lists the templates\n\
lienzo-ui --template 0 --save menu.editorgui.json\n\
lienzo-ui --open menu.editorgui.json --review         reviews the design (exit code 1 on errors)\n\
lienzo-ui --open menu.editorgui.json --export <Unity project>\n\
lienzo-ui --open menu.editorgui.json --screenshot capture.png\n\
lienzo-ui --test   1080x2400                    opens in Test mode with that screen\n\n\
With --export, --save and --review no window opens: use them for scripts, CI and the MCP server.\n\nAdd --dry-run to --export to list each file as new, update, same or remove without writing anything.",
    },
    Page {
        title: "MCP server and contributing",
        text: "The mcp/ folder ships a dependency-free MCP server (Node 18+): any compatible AI (Claude, Cursor, VS Code…) can list templates, create and edit designs, export them to Unity and take screenshots.\n\nexport_unity writes straight into Assets/EditorGUI of the project the AI names, with no prompt in the editor: the approval is your MCP client’s tool approval. Ask the AI to call it with dry_run: true first, read the list, and then export for real.\n\n\
Lienzo UI is open source (MIT). The code is Rust with egui: src/model.rs (data), src/canvas.rs (canvas), src/panels.rs (panels), src/export/ (Unity).\n\n\
Read docs/EXTENDING.md to add elements, templates or exporters, and CONTRIBUTING.md to send changes.",
    },
    Page {
        title: "Pixel art",
        text: "With nothing selected, tick “Pixel art” in the canvas Inspector:\n\n\
• Images are drawn without smoothing (nearest neighbor).\n\
• Positions and sizes snap to whole pixels when you release.\n\
• In Unity: Canvas “Pixel Perfect” and sprites with Point filter, no compression or mipmaps.\n\n\
Tip: use exact multiples of your sprite size (16 px → 32, 48, 64…) so every pixel looks the same.",
    },
    Page {
        title: "Plugins",
        text: "The Plugins tab in the left bar lists the installed add-ons. A plugin is a folder with a plugin.json that can add:\n\n\
• Commands: Rhai scripts that receive the design and return it modified (undo with Ctrl+Z).\n\
• Exporters: scripts that generate new files (for example, a CSV with every text).\n\
• Templates: ready-made designs.\n\n\
Scripts run sandboxed, without file or network access. “Open folder” takes you to your plugins folder; the guide is docs/PLUGINS.md.",
    },
    Page {
        title: "Community: plugins, fonts and rules",
        text: "In the Community tab, Install adds shared content with one click: plugins, fonts, images, templates, elements and themes.\n\n• A community plugin goes to a folder named community-<name>-<hash>. It is written to a temporary folder first, so an interrupted install never leaves half a plugin. It shows up in the Plugins tab.\n• Its scripts are the same Rhai scripts: sandboxed, with no file or network access and limits on operations, time and memory (docs/PLUGINS.md).\n• An installed font is added to the Text tab and exported to Unity with the design.\n• Downloads are checked against the published SHA-256 fingerprint; if it does not match, nothing is installed.\n• To publish you sign in (GitHub or Google), choose a public name and accept the rules. Everything is reviewed first and 18+ content must be marked. See docs/TERMS.md, docs/COMMUNITY_GUIDELINES.md and docs/PRIVACY.md.\n\nPreview shows a font with sample text, or a plugin’s commands, exporters, files and source code (read-only), before you install; nothing is installed and no download is counted. Translate sends only that description to the free MyMemory service when you press it (docs/PRIVACY.md).",
    },
    Page {
        title: "Autosave, recovery and undo",
        text: "• About every minute, while you edit, the full design (portrait layout included) is written to the app's recovery folder. Never in Test mode. At most 10 recoveries are kept.\n• At startup, a recovery newer than the project file is offered: Restore or Discard. After a normal Save it is no longer needed.\n• It can be turned off in Accessibility › Autosave.\n• The undo history is bounded to 200 steps and about 64 MB. When the oldest steps are dropped the status bar says “History trimmed”.\n• Each plugin command and each Fix from the Review window is one undo step.",
    },
    Page {
        title: "Accessibility and languages",
        text: "The “Accessibility” button in the top bar lets you:\n\n\
• Change the interface language (English, Spanish, Portuguese, French, German, Italian, Russian, Chinese, Japanese or Korean).\n\
• Enlarge the interface up to 250 % (also Ctrl+Alt+= and Ctrl+Alt+-).\n\
• Turn on high contrast: white text and strong borders.\n\
• Reduce motion: no animations.\n\n\
Languages are assets/i18n/<code>.tsv files: to add one, copy es.tsv and translate the copy.",
    },
];

/// Tutorials for the other languages: `assets/tutorial/<code>.json` with the same pages as `BEGINNER` and `PRO`.
const TUTORIALS: &[(&str, &str)] = &[
    ("pt", include_str!("../assets/tutorial/pt.json")),
    ("fr", include_str!("../assets/tutorial/fr.json")),
    ("de", include_str!("../assets/tutorial/de.json")),
    ("it", include_str!("../assets/tutorial/it.json")),
    ("ru", include_str!("../assets/tutorial/ru.json")),
    ("zh", include_str!("../assets/tutorial/zh.json")),
    ("ja", include_str!("../assets/tutorial/ja.json")),
    ("ko", include_str!("../assets/tutorial/ko.json")),
];

/// Tutorial files. Files with the old Spanish keys (`novato`, `profesional`, `titulo`, `texto`) still load through the aliases.
#[derive(serde::Deserialize)]
struct PageJson {
    #[serde(alias = "titulo")]
    title: String,
    #[serde(alias = "texto")]
    text: String,
}

#[derive(serde::Deserialize)]
struct TutorialJson {
    #[serde(alias = "novato")]
    beginner: Vec<PageJson>,
    #[serde(alias = "profesional")]
    pro: Vec<PageJson>,
}

type Tracks = (&'static [Page], &'static [Page]);

/// Read only once; a malformed or incomplete file is ignored (English is used).
static OTHERS: std::sync::LazyLock<std::collections::HashMap<&'static str, Tracks>> = std::sync::LazyLock::new(|| {
    let leak = |v: Vec<PageJson>| -> &'static [Page] {
        Box::leak(v.into_iter().map(|p| Page { title: Box::leak(p.title.into_boxed_str()), text: Box::leak(p.text.into_boxed_str()) }).collect::<Vec<_>>().into_boxed_slice())
    };
    TUTORIALS
        .iter()
        .filter_map(|(code, json)| {
            let t: TutorialJson = serde_json::from_str(json).ok()?;
            (t.beginner.len() == BEGINNER_ES.len() && t.pro.len() == PRO_ES.len()).then(|| (*code, (leak(t.beginner), leak(t.pro))))
        })
        .collect()
});

pub fn pages(p: Track) -> &'static [Page] {
    let (beginner, pro) = match crate::i18n::code() {
        "es" => (BEGINNER_ES, PRO_ES),
        code => OTHERS.get(code).copied().unwrap_or((BEGINNER, PRO)),
    };
    match p {
        Track::Beginner => beginner,
        Track::Pro => pro,
    }
}

impl EditorApp {
    pub fn open_tutorial(&mut self, track: Track) {
        self.tutorial = Some(Tutorial { track, page: 0 });
    }

    /// Closes the tutorial (the editor remains) and remembers it so it does not open by itself next time.
    pub fn close_tutorial(&mut self) {
        self.tutorial = None;
        if !self.settings.tutorial_seen {
            self.settings.tutorial_seen = true;
            self.settings.save();
        }
    }

    /// Next page; on the last one, closes the tutorial and leaves the editor.
    pub fn advance_tutorial(&mut self) {
        if let Some(t) = &mut self.tutorial {
            if t.page + 1 < pages(t.track).len() {
                t.page += 1;
            } else {
                self.close_tutorial();
            }
        }
    }

    pub fn tutorial_window(&mut self, ctx: &egui::Context) {
        let Some(t) = self.tutorial else { return };
        let list = pages(t.track);
        let page = t.page.min(list.len() - 1);
        let last = page + 1 == list.len();

        // Keyboard: arrows turn pages, Enter advances, Esc closes.
        let (left, der, intro, esc) = ctx.input_mut(|i| {
            (
                i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft),
                i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight),
                i.consume_key(egui::Modifiers::NONE, egui::Key::Enter),
                i.consume_key(egui::Modifiers::NONE, egui::Key::Escape),
            )
        });

        let mut action: Option<u8> = None; // 0 previous, 1 next, 2 close, 3 beginner, 4 pro
        #[allow(deprecated)]
        let id = egui::Id::new("tutorial");
        let frame = egui::Frame::new().fill(PANEL()).corner_radius(12.0).inner_margin(22.0).stroke(egui::Stroke::new(1.0, LINE()));
        let modal = egui::Modal::new(id).frame(frame).show(ctx, |ui| {
            ui.set_width(760.0);
            ui.horizontal(|ui| {
                for (p, name, a) in [(Track::Beginner, "Beginner tutorial", 3u8), (Track::Pro, "Pro tutorial", 4u8)] {
                    let sel = t.track == p;
                    let txt = RichText::new(crate::i18n::t(name)).size(14.5).color(if sel { ACCENT() } else { MUTED() });
                    if ui.add(egui::Button::selectable(sel, txt)).clicked() {
                        action = Some(a);
                    }
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button(crate::i18n::t("Skip and go to editor")).clicked() {
                        action = Some(2);
                    }
                });
            });
            ui.add(egui::Separator::default().spacing(14.0));
            let track = crate::i18n::t(if t.track == Track::Beginner { "BEGINNERS" } else { "PROFESSIONALS" });
            ui.label(RichText::new(format!("{} · {} {} / {}", track, crate::i18n::t("STEP"), page + 1, list.len())).size(11.5).strong().color(ACCENT()));
            ui.add_space(4.0);
            ui.label(RichText::new(list[page].title).size(24.0).strong().color(TEXT()));
            ui.add_space(10.0);
            egui::ScrollArea::vertical().max_height(300.0).min_scrolled_height(300.0).show(ui, |ui| {
                ui.set_min_height(300.0);
                ui.label(RichText::new(list[page].text).size(15.0).color(TEXT()));
            });
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.add_enabled(page > 0, egui::Button::new(crate::i18n::t("Previous")).min_size(Vec2::new(96.0, 30.0))).clicked() {
                    action = Some(0);
                }
                let width = ui.available_width();
                let points = list.len() as f32 * 14.0;
                ui.add_space(((width - points - 170.0) / 2.0).max(0.0));
                for i in 0..list.len() {
                    let (r, _) = ui.allocate_exact_size(Vec2::new(14.0, 14.0), egui::Sense::hover());
                    ui.painter().circle_filled(r.center(), 4.0, if i == page { ACCENT() } else { LINE() });
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let text = crate::i18n::t(if last { "Open the editor" } else { "Next" });
                    let b = egui::Button::new(RichText::new(text).strong().color(Color32::WHITE)).fill(ACCENT()).min_size(Vec2::new(130.0, 30.0));
                    if ui.add(b).clicked() {
                        action = Some(1);
                    }
                });
            });
        });
        if modal.should_close() || esc {
            action = Some(2);
        }
        if der || intro {
            action = Some(1);
        }
        if left && page > 0 {
            action = Some(0);
        }
        match action {
            Some(0) => {
                if let Some(t) = &mut self.tutorial {
                    t.page = t.page.saturating_sub(1);
                }
            }
            Some(1) => self.advance_tutorial(),
            Some(2) => self.close_tutorial(),
            Some(3) => self.open_tutorial(Track::Beginner),
            Some(4) => self.open_tutorial(Track::Pro),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_tracks_have_content() {
        assert_eq!(BEGINNER.len(), BEGINNER_ES.len(), "the Spanish beginner track must have the same pages");
        assert_eq!(PRO.len(), PRO_ES.len(), "the Spanish pro track must have the same pages");
        assert!(BEGINNER_ES.len() >= 3 && PRO_ES.len() >= 3);
        for p in BEGINNER_ES.iter().chain(PRO_ES) {
            assert!(!p.title.trim().is_empty() && !p.text.trim().is_empty());
        }
    }

    #[test]
    fn every_language_has_a_complete_tutorial() {
        for (code, json) in TUTORIALS {
            let t: TutorialJson = serde_json::from_str(json).unwrap_or_else(|e| panic!("{}.json is not valid: {}", code, e));
            assert_eq!(t.beginner.len(), BEGINNER_ES.len(), "{}: beginner pages", code);
            assert_eq!(t.pro.len(), PRO_ES.len(), "{}: pro pages", code);
            for p in t.beginner.iter().chain(&t.pro) {
                assert!(!p.title.trim().is_empty() && !p.text.trim().is_empty(), "{}: empty page", code);
            }
            assert!(OTHERS.contains_key(code), "{} was not loaded", code);
        }
        // Every interface language has its own tutorial (Spanish and English are in the code).
        for (code, _, _) in crate::i18n::LANGUAGES {
            assert!(matches!(*code, "es" | "en") || TUTORIALS.iter().any(|(c, _)| c == code), "{} has no tutorial", code);
        }
    }

    /// The template page (beginner page 3) must state the real number of templates.
    #[test]
    fn template_count_is_current() {
        let n = crate::model::TEMPLATES.len().to_string();
        let has_number = |t: &str| t.split(|c: char| !c.is_ascii_digit()).any(|w| w == n);
        assert!(has_number(BEGINNER[2].text), "English template page does not mention {}", n);
        assert!(has_number(BEGINNER_ES[2].text), "Spanish template page does not mention {}", n);
        for (code, json) in TUTORIALS {
            let t: TutorialJson = serde_json::from_str(json).unwrap();
            assert!(has_number(&t.beginner[2].text), "{}: template page does not mention {}", code, n);
        }
    }

    #[test]
    fn old_spanish_tutorial_keys_still_load() {
        let old = r#"{ "novato": [ { "titulo": "A", "texto": "B" } ], "profesional": [ { "titulo": "C", "texto": "D" } ] }"#;
        let t: TutorialJson = serde_json::from_str(old).expect("a tutorial file with Spanish keys must load");
        assert_eq!((t.beginner[0].title.as_str(), t.beginner[0].text.as_str(), t.pro[0].title.as_str()), ("A", "B", "C"));
        let new = r#"{ "beginner": [ { "title": "A", "text": "B" } ], "pro": [ { "title": "C", "text": "D" } ] }"#;
        assert!(serde_json::from_str::<TutorialJson>(new).is_ok());
    }
}
