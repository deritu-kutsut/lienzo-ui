# Tutorial para profesionales

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

## Anclas y escalado

- Con nada seleccionado, el Inspector muestra el **Canvas Scaler**: resolución de referencia, *match* y fondo.
- Cada elemento tiene anclas, pivote y desplazamientos como un `RectTransform`. Al cambiar la resolución de referencia, los elementos siguen sus anclas.
- Prueba en **Probar** (F5) con varias pantallas antes de exportar.

## Layouts y listas

Un contenedor puede ordenar sus hijos en horizontal, vertical o cuadrícula (como los *Layout Group* de Unity), con espaciado y relleno. Las ScrollView calculan su contenido. El orden de la Jerarquía es el orden de dibujo.

## Lógica

- **En el editor**: en Probar, cada elemento puede ejecutar un script Rhai al hacer clic (`mostrar("Nombre")`, `ocultar`, `alternar`, `texto`, `color`, `mover`…). Útil para prototipar flujos.
- **En Unity**: `<Diseño>View` hereda de `<Diseño>ViewBase`, con un campo por botón, slider, toggle o campo de texto, y métodos `On<Elemento>Click` / `On<Elemento>Changed` para sobrescribir.

```csharp
public class AjustesView : AjustesViewBase
{
    protected override void OnBotonVolverClick() => gameObject.SetActive(false);
}
```

## Línea de comandos

```bash
lienzo-ui --plantillas                                   # lista las plantillas
lienzo-ui --plantilla 4 --guardar inventario.editorgui.json
lienzo-ui --abrir inventario.editorgui.json --exportar "C:/MiJuego"   # sin ventana
lienzo-ui --abrir inventario.editorgui.json --screenshot captura.png
lienzo-ui --probar 1080x2400                             # abre en Probar con esa pantalla
lienzo-ui --tutorial [--profesional]                     # fuerza el tutorial
lienzo-ui --sin-tutorial                                 # no lo abre
```

`--guardar` y `--exportar` no abren ventana: sirven para scripts, CI y el [servidor MCP](MCP.md).

## Código fuente

Rust + [egui](https://github.com/emilk/egui):

| Archivo | Contenido |
|---|---|
| `src/model.rs` | Datos: `Doc`, `Element`, `Kind`, anclas, layouts, plantillas |
| `src/canvas.rs` | Lienzo: selección, tiradores, guías, zoom |
| `src/panels.rs` | Barra superior, barra lateral, Inspector, ventanas |
| `src/app.rs` | Estado, deshacer, comandos, atajos |
| `src/render.rs` | Dibujo de elementos y fuentes |
| `src/export/` | Exportadores: `ugui.rs`, `uitk.rs`, `imgui.rs`, `view.rs` y el importador de Unity |
| `src/tutorial.rs` | Tutorial integrado |
| `src/script.rs` | Scripts Rhai del modo Probar |

Para ampliarlo: [EXTENDER.md](EXTENDER.md).
