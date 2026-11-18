# Pro tutorial

🌐 [English](TUTORIAL_PROS.md) · [Español](TUTORIAL_PROFESIONALES.md) · [Português](TUTORIAL_PROS.pt.md) · [Français](TUTORIAL_PROS.fr.md) · [Deutsch](TUTORIAL_PROS.de.md) · [Italiano](TUTORIAL_PROS.it.md) · [Русский](TUTORIAL_PROS.ru.md)

## What the export generates

Everything is written to `Assets/EditorGUI` in the Unity project:

| File | Purpose |
|---|---|
| `Designs/<Design>.json` | The design. The importer rebuilds the prefab whenever it changes. |
| `Editor/EditorGuiImporter.cs` | Builds the prefab with uGUI + TextMeshPro. **Tools › Lienzo UI** menu. |
| `Prefabs/<Design>.prefab` | Ready to use. |
| `UI Toolkit/<Design>.uxml` + `.uss` | UI Toolkit version (UIDocument, PanelSettings 1920×1080 Scale With Screen Size). |
| `Scripts/<Design>OnGUI.cs` | IMGUI version: add it to a GameObject. |
| `Scripts/<Design>ViewBase.cs` | Typed references and events (regenerated). |
| `Scripts/<Design>View.cs` | **Your** logic: created once and never overwritten. |
| `Sprites/`, `Fonts/` | Images and fonts used by the design. |

`<Design>` is the title in PascalCase without accents (`Menú principal` → `MenuPrincipal`).

If an image is missing, nothing is written: the export stops before touching the project and tells you which file it could not read.

## Anchors and scaling

- With nothing selected, the Inspector shows the **Canvas Scaler**: reference resolution, *match* and background.
- Each element has anchors, pivot and offsets like a `RectTransform`. When the reference resolution changes, elements follow their anchors.
- Built-in templates are designed at 1920×1080; on smaller or portrait pages they are scaled down evenly so they fit.
- Check with **Test** (F5) on several screens before exporting.

## Layouts and lists

A container can arrange its children horizontally, vertically or in a grid (like Unity's *Layout Groups*), with spacing and padding. ScrollViews compute their content. The Hierarchy order is the drawing order.

## Logic

- **In the editor**: in Test mode, each element can run a Rhai script on click (`mostrar("Name")`, `ocultar`, `alternar`, `texto`, `color`, `mover`…). Useful to prototype flows. Changes made while testing are discarded when you leave test mode and are never saved or exported.
- **In Unity**: `<Design>View` inherits from `<Design>ViewBase`, with one field per button, slider, toggle or input field, and `On<Element>Click` / `On<Element>Changed` methods to override.

```csharp
public class AjustesView : AjustesViewBase
{
    protected override void OnBotonVolverClick() => gameObject.SetActive(false);
}
```

Give interactive elements unique names: scripts and `GameObject.Find` only find the first match. **Review** warns about repeated names.

## Review design

The **Review** button (and `--revisar` on the command line) checks, at the reference resolution:

| Check | Rule |
|---|---|
| Text size | At least 18 px at 1080 p |
| Contrast | WCAG: 4.5:1, or 3:1 for large text (24 px, or 18.7 px bold); disabled controls are exempt |
| Touch size | Buttons, toggles, input fields and dropdowns at least 64 px on their short side at 1080 p |
| Page | Elements entirely or partly outside the page |
| Names | Repeated names on interactive elements |
| Images | Image files that no longer exist |

Contrast below 2:1, elements fully off the page and missing images are errors; the rest are warnings.

## Command line

```bash
lienzo-ui --plantillas                                   # lists the templates (0-12)
lienzo-ui --plantilla 4 --guardar inventario.editorgui.json
lienzo-ui --abrir inventario.editorgui.json --revisar    # exit code 1 if there are errors
lienzo-ui --abrir inventario.editorgui.json --exportar "C:/MyGame"   # no window
lienzo-ui --abrir inventario.editorgui.json --screenshot capture.png
lienzo-ui --probar 1080x2400                             # opens in Test mode with that screen
lienzo-ui --tutorial [--profesional]                     # forces the tutorial
lienzo-ui --sin-tutorial                                 # does not open it
```

`--guardar`, `--exportar` and `--revisar` do not open a window: use them for scripts, CI and the [MCP server](MCP.md) (`revisar_diseno` tool).

## Source code

Rust + [egui](https://github.com/emilk/egui):

| File | Contents |
|---|---|
| `src/model.rs` | Data: `Doc`, `Element`, `Kind`, anchors, layouts, templates |
| `src/canvas.rs` | Canvas: selection, handles, guides, zoom |
| `src/panels.rs` | Top bar, sidebar, Inspector, windows |
| `src/app.rs` | State, undo, commands, shortcuts |
| `src/render.rs` | Drawing elements and fonts |
| `src/revision.rs` | Review design checks |
| `src/export/` | Exporters: `ugui.rs`, `uitk.rs`, `imgui.rs`, `view.rs` and the Unity importer |
| `src/tutorial.rs` | Built-in tutorial (Spanish and English pages) |
| `src/script.rs` | Rhai scripts for Test mode |

To extend it: [EXTENDER.md](EXTENDER.md).
