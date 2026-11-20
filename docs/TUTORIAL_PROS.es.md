# Tutorial para profesionales

🌐 [English](TUTORIAL_PROS.md) · [Español](TUTORIAL_PROS.es.md) · [Português](TUTORIAL_PROS.pt.md) · [Français](TUTORIAL_PROS.fr.md) · [Deutsch](TUTORIAL_PROS.de.md) · [Italiano](TUTORIAL_PROS.it.md) · [Русский](TUTORIAL_PROS.ru.md) · [简体中文](TUTORIAL_PROS.zh.md) · [日本語](TUTORIAL_PROS.ja.md) · [한국어](TUTORIAL_PROS.ko.md)

## Qué genera la exportación

Todo se escribe en `Assets/EditorGUI` del proyecto de Unity:

| Archivo | Para qué |
|---|---|
| `Designs/<Diseño>.json` | El diseño. El importador rehace el prefab cada vez que cambia. |
| `Editor/EditorGuiImporter.cs` | Construye el prefab con uGUI + TextMeshPro. Menú **Tools › Lienzo UI**. |
| `Prefabs/<Diseño>.prefab` | Listo para usar. |
| `UI Toolkit/<Diseño>.uxml` + `.uss` | Versión UI Toolkit (UIDocument, PanelSettings 1920×1080 Scale With Screen Size). |
| `Scripts/<Diseño>OnGUI.cs` | Versión IMGUI: añádelo a un GameObject. |
| `Scripts/<Diseño>ViewBase.cs` | Referencias tipadas y eventos (se regenera). |
| `Scripts/<Diseño>View.cs` | **Tu** lógica: se crea una vez y nunca se sobrescribe. |
| `Sprites/`, `Fonts/` | Imágenes y fuentes usadas por el diseño. |

`<Diseño>` es el título en PascalCase sin tildes (`Menú principal` → `MenuPrincipal`).

Si falta una imagen no se escribe nada: la exportación se detiene antes de tocar el proyecto y dice qué archivo no pudo leer.

## Anclas y escalado

- Con nada seleccionado, el Inspector muestra el **Canvas Scaler**: resolución de referencia, *match* y fondo.
- Cada elemento tiene anclas, pivote y desplazamientos como un `RectTransform`. Al cambiar la resolución de referencia, los elementos siguen sus anclas.
- Las plantillas incluidas están pensadas a 1920×1080; en páginas más pequeñas o verticales se reducen de forma uniforme para caber.
- Prueba en **Probar** (F5) con varias pantallas antes de exportar.

## Layouts y listas

Un contenedor puede ordenar sus hijos en horizontal, vertical o cuadrícula (como los *Layout Group* de Unity), con espaciado y relleno. Las ScrollView calculan su contenido. El orden de la Jerarquía es el orden de dibujo.

## Lógica

- **En el editor**: en Probar, cada elemento puede ejecutar un script Rhai al hacer clic (`show("Name")`, `hide`, `toggle`, `set_text`, `set_color`, `move_by`…). Útil para prototipar flujos. Los cambios de la prueba se descartan al salir y nunca se guardan ni se exportan.
- **En Unity**: `<Diseño>View` hereda de `<Diseño>ViewBase`, con un campo por botón, slider, toggle o campo de texto, y métodos `On<Elemento>Click` / `On<Elemento>Changed` para sobrescribir.

```csharp
public class AjustesView : AjustesViewBase
{
    protected override void OnBotonVolverClick() => gameObject.SetActive(false);
}
```

Pon nombres únicos a los elementos interactivos: los scripts y `GameObject.Find` encuentran solo el primero. **Revisar** avisa de los nombres repetidos.

## Revisar diseño

El botón **Revisar** (y `--review` en la línea de comandos) comprueba, a la resolución de referencia:

| Comprobación | Regla |
|---|---|
| Tamaño de letra | Al menos 18 px a 1080 p |
| Contraste | WCAG: 4,5:1, o 3:1 para texto grande (24 px, o 18,7 px en negrita); los controles desactivados quedan exentos |
| Tamaño táctil | Botones, casillas, campos y desplegables de al menos 64 px en su lado corto a 1080 p |
| Página | Elementos fuera de la página, del todo o en parte |
| Nombres | Nombres repetidos en elementos interactivos |
| Imágenes | Archivos de imagen que ya no existen |

Contraste por debajo de 2:1, elementos del todo fuera de la página e imágenes que faltan son errores; lo demás, avisos.

## Línea de comandos

```bash
lienzo-ui --templates                                   # lista las plantillas (0-12)
lienzo-ui --template 4 --save inventory.editorgui.json
lienzo-ui --open inventory.editorgui.json --review    # código 1 si hay errores
lienzo-ui --open inventory.editorgui.json --export "C:/MiJuego"   # sin ventana
lienzo-ui --open inventory.editorgui.json --screenshot capture.png
lienzo-ui --test 1080x2400                             # abre en Probar con esa pantalla
lienzo-ui --tutorial [--pro]                     # fuerza el tutorial
lienzo-ui --no-tutorial                                 # no lo abre
```

`--save`, `--export` y `--review` no abren ventana: sirven para scripts, CI y el [servidor MCP](MCP.md) (herramienta `review_design`).

Añade --dry-run a --export para listar cada archivo como new, update, same o remove sin escribir nada.

export_unity escribe directamente en Assets/EditorGUI del proyecto que indique la IA, sin ningún aviso en el editor: la aprobación es la de herramientas de tu cliente MCP. Pide a la IA que lo llame antes con dry_run: true, lee la lista y después exporta de verdad.

## Código fuente

Rust + [egui](https://github.com/emilk/egui):

| Archivo | Contenido |
|---|---|
| `src/model.rs` | Datos: `Doc`, `Element`, `Kind`, anclas, layouts, plantillas |
| `src/canvas.rs` | Lienzo: selección, tiradores, guías, zoom |
| `src/panels.rs` | Barra superior, barra lateral, Inspector, ventanas |
| `src/app.rs` | Estado, deshacer, comandos, atajos |
| `src/render.rs` | Dibujo de elementos y fuentes |
| `src/revision.rs` | Comprobaciones de Revisar diseño |
| `src/export/` | Exportadores: `ugui.rs`, `uitk.rs`, `imgui.rs`, `view.rs` y el importador de Unity |
| `src/tutorial.rs` | Tutorial integrado (páginas en español e inglés) |
| `src/script.rs` | Scripts Rhai del modo Probar |

Para ampliarlo: [EXTENDING.md](EXTENDING.md).

## Diseño vertical al detalle

El diseño vertical es una segunda vista del mismo diseño (docs/FORMAT.md › PortraitLayout):

- Tiene su propia resolución de referencia, «Tamaño vertical» (1080 × 1920 por defecto).
- Los elementos sin excepción conservan su RectTransform horizontal y se reevalúan contra la página vertical: un fondo a pantalla completa se estira, los anclados a la derecha siguen el borde derecho.
- Solo se guarda lo que cambias en la vista vertical, como excepciones (posición, tamaño, rotación, anclajes, visibilidad, tamaño de fuente, layout). Clic derecho › «Volver a horizontal» quita una.
- Exportación: el JSON lleva las excepciones y el script generado LienzoOrientationLayout las aplica cuando Screen.width < Screen.height, y de nuevo si cambia el tamaño.
- «Quitar diseño vertical» borra todas las excepciones.

Las plantillas traen un diseño vertical ya incorporado, guardado como excepciones del horizontal, así que Retrato muestra desde el principio un diseño pensado para móviles.

## 9-slice y el paquete incluido

Para marcos, botones y paneles que se estiran:

- Selecciona una Imagen y define «9-slice» en «Ajustes de imagen (Unity)»: el borde en píxeles de la imagen original (izquierda, abajo, derecha, arriba), como en el Sprite Editor de Unity. La exportación genera un Sprite 9-slice.
- «Píxeles por unidad»: 100 mantiene los bordes en su tamaño en píxeles, 200 los reduce a la mitad.
- El paquete incluido (assets/pack/manifest.json, docs/CONTENT.md) ya define el corte y los píxeles por unidad: los marcos son de 512×512 con corte 96, los botones de 512×192.
- Usar una imagen la copia a tu biblioteca como pack_<id>.<ext> (oculta en «Tus imágenes»); el exportador la copia a los Sprites de Unity.
- Las plantillas HUD RPG pixel art y Menú principal pixel art usan el paquete pixel. Activa «Pixel art» para el filtro Point.

## Revisión y correcciones rápidas

La ventana Revisar lista errores y avisos; haz clic en uno para seleccionar su elemento. Seis problemas tienen un botón «Corregir» (un paso de deshacer):

- Texto por debajo del tamaño mínimo: se sube al mínimo.
- Control demasiado pequeño para tocar: se agranda alrededor de su centro.
- Elemento parcialmente fuera de la página: se mueve dentro, si cabe.
- Nombre repetido: se añade _2, _3…
- Imagen que falta: se quita la referencia.
- Poco contraste: se oscurece o aclara el color del texto.

«Corregir todos los problemas seguros» los aplica en varias pasadas. Problemas como una fuente sin ciertos caracteres no tienen corrección automática. Las correcciones también funcionan en el diseño vertical. Por línea de comandos: lienzo-ui --open archivo --review (código 1 si hay errores).

## Importar desde Unity: prefabs y CLI

Archivo › Importar desde Unity… lee un .prefab de uGUI (el YAML de texto de Unity) o un Designs/*.json exportado por Lienzo UI y crea un diseño normal.

- Lo que no se pudo convertir aparece como avisos.
- Si detecta la carpeta del proyecto de Unity, las imágenes se resuelven contra ella durante esa sesión.
- Sin ventana: lienzo-ui --import-unity Menu.prefab --save Menu.editorgui.json (también con --review y --export).
- Proyecto completo: lienzo-ui --import-unity-project MiProyecto --out Importados convierte todos los prefabs de interfaz. Nunca sobrescribe (añade _2) y omite los prefabs sin interfaz.
- Detalles en docs/UNITY_IMPORT.md.

## Comunidad: plugins, fuentes y normas

En la pestaña Comunidad, Instalar añade contenido compartido con un clic: plugins, fuentes, imágenes, plantillas, elementos y temas.

- Un plugin de la comunidad va a una carpeta llamada community-<nombre>-<hash>. Se escribe primero en una carpeta temporal, así que una instalación interrumpida nunca deja un plugin a medias. Aparece en la pestaña Plugins.
- Sus scripts son los mismos Rhai: aislados, sin acceso a archivos ni red y con límites de operaciones, tiempo y memoria (docs/PLUGINS.md).
- Una fuente instalada se añade a la pestaña Texto y se exporta a Unity con el diseño.
- Las descargas se comprueban con la huella SHA-256 publicada; si no coincide, no se instala nada.
- Para publicar inicias sesión (GitHub o Google), eliges un nombre público y aceptas las reglas. Todo se revisa antes y el contenido +18 debe marcarse. Ver docs/TERMS.md, docs/COMMUNITY_GUIDELINES.md y docs/PRIVACY.md.

Vista previa muestra una fuente con texto de ejemplo, o los comandos, exportadores, archivos y el código fuente de un plugin (solo lectura), antes de instalar; no se instala nada ni se cuenta una descarga. Traducir envía solo esa descripción al servicio gratuito MyMemory cuando lo pulsas (docs/PRIVACY.md).

## Autoguardado, recuperación y deshacer

- Más o menos cada minuto, mientras editas, el diseño completo (con el diseño vertical) se escribe en la carpeta de recuperación de la app. Nunca en modo Probar. Se conservan como máximo 10 recuperaciones.
- Al iniciar, si hay una recuperación más reciente que el archivo del proyecto, se ofrece: Restaurar o Descartar. Tras un Guardar normal ya no hace falta.
- Se puede desactivar en Accesibilidad › Autoguardado.
- El historial de deshacer está limitado a 200 pasos y unos 64 MB. Cuando se descartan los pasos más antiguos, la barra de estado muestra «Historial recortado».
- Cada comando de plugin y cada Corregir de la ventana Revisar es un solo paso de deshacer.
