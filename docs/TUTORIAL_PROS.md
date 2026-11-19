# Pro tutorial

🌐 [English](TUTORIAL_PROS.md) · [Español](TUTORIAL_PROS.es.md) · [Português](TUTORIAL_PROS.pt.md) · [Français](TUTORIAL_PROS.fr.md) · [Deutsch](TUTORIAL_PROS.de.md) · [Italiano](TUTORIAL_PROS.it.md) · [Русский](TUTORIAL_PROS.ru.md) · [简体中文](TUTORIAL_PROS.zh.md) · [日本語](TUTORIAL_PROS.ja.md) · [한국어](TUTORIAL_PROS.ko.md)

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

- **In the editor**: in Test mode, each element can run a Rhai script on click (`show("Name")`, `hide`, `toggle`, `set_text`, `set_color`, `move_by`…). Useful to prototype flows. Changes made while testing are discarded when you leave test mode and are never saved or exported.
- **In Unity**: `<Design>View` inherits from `<Design>ViewBase`, with one field per button, slider, toggle or input field, and `On<Element>Click` / `On<Element>Changed` methods to override.

```csharp
public class AjustesView : AjustesViewBase
{
    protected override void OnBotonVolverClick() => gameObject.SetActive(false);
}
```

Give interactive elements unique names: scripts and `GameObject.Find` only find the first match. **Review** warns about repeated names.

## Review design

The **Review** button (and `--review` on the command line) checks, at the reference resolution:

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
lienzo-ui --templates                                   # lists the templates (0-12)
lienzo-ui --template 4 --save inventory.editorgui.json
lienzo-ui --open inventory.editorgui.json --review    # exit code 1 if there are errors
lienzo-ui --open inventory.editorgui.json --export "C:/MyGame"   # no window
lienzo-ui --open inventory.editorgui.json --screenshot capture.png
lienzo-ui --test 1080x2400                             # opens in Test mode with that screen
lienzo-ui --tutorial [--pro]                     # forces the tutorial
lienzo-ui --no-tutorial                                 # does not open it
```

`--save`, `--export` and `--review` do not open a window: use them for scripts, CI and the [MCP server](MCP.md) (`review_design` tool).

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

To extend it: [EXTENDING.md](EXTENDING.md).

## Portrait layout in detail

The portrait layout is a second view of the same design (docs/FORMAT.md › PortraitLayout):

- It has its own reference resolution, “Portrait size” (1080 × 1920 by default).
- Elements without an override keep their landscape RectTransform and are re-evaluated against the portrait page: a full-page background stretches, right-anchored elements follow the right edge.
- Only what you change in Portrait view is stored, as overrides (position, size, rotation, anchors, visibility, font size, layout). Right-click › “Reset to landscape” removes one.
- Export: the JSON carries the overrides and the generated LienzoOrientationLayout script applies them when Screen.width < Screen.height, and again when the size changes.
- “Remove portrait layout” deletes every override.

## 9-slice and the built-in pack

For frames, buttons and panels that stretch:

- Select an Image and set “9-slice” in “Image settings (Unity)”: the border in source-image pixels (left, bottom, right, top), like Unity's Sprite Editor. The export generates a 9-slice Sprite.
- “Pixels per unit”: 100 keeps the borders at their pixel size, 200 makes them half as big.
- The built-in pack (assets/pack/manifest.json, docs/CONTENT.md) already sets slice and pixels per unit: frames are 512×512 with slice 96, buttons 512×192.
- Using an item copies it to your library as pack_<id>.<ext> (hidden from “Your images”); the exporter copies it to the Unity Sprites.
- The templates Pixel RPG HUD and Pixel main menu use the pixel pack. Turn on “Pixel art” for Point filtering.

## Review and quick fixes

The Review window lists errors and warnings; click one to select its element. Six problems have a “Fix” button (one undo step):

- Text below the minimum size: raised to the minimum.
- Control too small to tap: enlarged around its center.
- Element partly outside the page: moved inside, if it fits.
- Repeated name: _2, _3… is appended.
- Missing image: the reference is removed.
- Low contrast: the text color is darkened or lightened.

“Fix all safe issues” applies them in a few passes. Problems such as a font that lacks characters have no automatic fix. Fixes also work in the portrait layout. From the command line: lienzo-ui --open file --review (exit code 1 on errors).

## Import from Unity: prefabs and CLI

File › Import from Unity… reads a uGUI .prefab (Unity's text YAML) or a Designs/*.json exported by Lienzo UI and creates an ordinary design.

- What could not be mapped is listed as warnings.
- When it detects the Unity project folder, images resolve against it for that session.
- Headless: lienzo-ui --import-unity Menu.prefab --save Menu.editorgui.json (also with --review and --export).
- Whole project: lienzo-ui --import-unity-project MyProject --out Imported converts every UI prefab. It never overwrites (it adds _2) and skips prefabs without UI.
- Details in docs/UNITY_IMPORT.md.

## Community: plugins, fonts and rules

In the Community tab, Install adds shared content with one click: plugins, fonts, images, templates, elements and themes.

- A community plugin goes to a folder named community-<name>-<hash>. It is written to a temporary folder first, so an interrupted install never leaves half a plugin. It shows up in the Plugins tab.
- Its scripts are the same Rhai scripts: sandboxed, with no file or network access and limits on operations, time and memory (docs/PLUGINS.md).
- An installed font is added to the Text tab and exported to Unity with the design.
- Downloads are checked against the published SHA-256 fingerprint; if it does not match, nothing is installed.
- To publish you sign in (GitHub or Google), choose a public name and accept the rules. Everything is reviewed first and 18+ content must be marked. See docs/TERMS.md, docs/COMMUNITY_GUIDELINES.md and docs/PRIVACY.md.

## Autosave, recovery and undo

- About every minute, while you edit, the full design (portrait layout included) is written to the app's recovery folder. Never in Test mode. At most 10 recoveries are kept.
- At startup, a recovery newer than the project file is offered: Restore or Discard. After a normal Save it is no longer needed.
- It can be turned off in Accessibility › Autosave.
- The undo history is bounded to 200 steps and about 64 MB. When the oldest steps are dropped the status bar says “History trimmed”.
- Each plugin command and each Fix from the Review window is one undo step.
