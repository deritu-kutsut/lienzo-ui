# Plugins (add-ons)

Lienzo UI is extended with plugins, without recompiling the app. A plugin is a **folder with a `plugin.json`** that can add:

- **Commands**: they modify the open design (undone with Ctrl+Z).
- **Exporters**: they generate files in new formats.
- **Templates**: ready-to-use designs.

They appear in the **Plugins** tab of the left bar.

## Where they are installed

Lienzo UI looks for plugin folders in:

1. `plugins/` next to `lienzo-ui.exe` (those of the downloaded package).
2. `plugins/` in the working folder (useful when developing from the repository).
3. The user folder: `%APPDATA%\EditorGUI\plugins` on Windows, `~/Library/Application Support/EditorGUI/plugins` on macOS, `~/.local/share/EditorGUI/plugins` on Linux. The **Open folder** button in the Plugins tab opens it.

Copy the plugin folder and press **Reload**.

## plugin.json

```json
{
  "name": "My plugin",
  "version": "1.0.0",
  "author": "Your name",
  "description": "What it does, in one sentence.",
  "commands": [
    { "name": "Snap to 8 px", "description": "Help text", "script": "snap.rhai" }
  ],
  "exporters": [
    { "name": "Texts to translate", "extension": "csv", "script": "csv.rhai" }
  ],
  "templates": ["shop.editorgui.json"]
}
```

All paths are relative to the plugin folder and cannot leave it (`..` or absolute paths are rejected).

## Scripts

Scripts use [Rhai](https://rhai.rs), a language similar to Rust/JavaScript.

### Command

Define `fn run(design)`, which receives the design and **returns the modified design**:

```rust
fn run(design) {
    for i in 0..design.elements.len() {
        let e = design.elements[i];
        e.x = (e.x / 8.0).round() * 8.0;
        design.elements[i] = e;      // elements are copied: they must be stored back
    }
    log("done");                     // the message appears in the status bar
    design
}
```

### Exporter

Define `fn export_text(design)` and **return a text**; Lienzo UI asks where to save it, using the extension from the manifest.

```rust
fn export_text(design) {
    let output = "element,text\n";
    for e in design.elements {
        if e.text != "" { output += e.name + "," + e.text + "\n"; }
    }
    output
}
```

### The design

| Field | Contents |
|---|---|
| `title`, `width`, `height` | Title and reference resolution |
| `background` | Color `[r, g, b, a]` (0–255) |
| `canvas_match`, `pixel_art` | Canvas Scaler settings and pixel art mode |
| `elements` | List of elements, in drawing order |
| `next_id` | Next free id: add 1 to it if you create elements |

Each element has, among others: `id`, `name`, `kind` (`"Panel"`, `"Image"`, `"Text"`, `"Button"`, `"Toggle"`, `"Slider"`, `"InputField"`, `"Dropdown"`, `"ScrollView"`, `"Ellipse"`, `"Empty"`), `x`, `y`, `w`, `h`, `rot`, `fill`, `stroke`, `stroke_w`, `radius`, `opacity`, `text`, `font_size`, `text_color`, `bold`, `image`, `parent`, `visible`, `anchor_min`, `anchor_max`, `pivot`, `layout`. The full list is in `src/model.rs` (`struct Element`).

### Security and limits

- No access to files, network or processes; `import` is disabled.
- Limits (an infinite loop or a runaway script is cut off with an error): 5 million operations, 5 seconds, texts of at most 64 KB, lists and maps of at most 10,000 items (nested data counts), and at most 200 messages / 64 KB of output in total (`log`, `print` and `debug`). Past the output limit the script stops with an error. An exporter's text is limited by the same 64 KB.
- Memory: besides the size limits above, a command, exporter or on-click script that holds more than 256 MB of net new memory is stopped with a “memory limit exceeded” error. This is a counter in the program's allocator that is read between script operations (not an operating-system limit): one single operation can still overshoot by its own size, which the text and list limits cap. Memory of other threads is never counted.
- A command's result is untrusted: if the returned design is not valid, or it introduces control characters, external links, or image paths that are absolute, UNC (`\\server\share`), contain `..` or a drive/stream `:`, nothing is applied and the error is shown. Values that were already in your design are left alone.
- An exporter's `extension` must be lowercase letters and digits (at most 5) from this list: `txt`, `csv`, `json`, `xml`, `md`, `cs`, `uxml`, `uss`, `tsv`. A plugin with another extension is reported in the Plugins tab and that exporter is not offered.
- Plugins installed from the community go to a folder named `community-<name>-<hash>`: they are written to a temporary folder first and swapped in, so an interrupted install never leaves a half plugin.

## Included examples

| Folder | Provides |
|---|---|
| `plugins/snap-to-grid` | "Snap to 8 px" command |
| `plugins/text-to-csv` | Exporter of all texts to CSV (for translating) |
| `plugins/extra-templates` | "Blacksmith shop" template |

## Testing a plugin

```bash
cargo test plugins     # loads the examples and runs their command and exporter
```

To publish your plugin, upload it to your own repository or propose adding it to `plugins/` with a pull request (see [CONTRIBUTING.md](../CONTRIBUTING.md)).

## Legacy names

Plugins written with the old Spanish names **keep working unchanged**:

| Now | Before |
|---|---|
| `plugin.json` keys `name`, `author`, `description`, `commands`, `exporters`, `templates` | `nombre`, `autor`, `descripcion`, `comandos`, `exportadores`, `plantillas` |
| Command function `fn run(design)` | `fn ejecutar(diseno)` |
| Exporter function `fn export_text(design)` | `fn exportar(diseno)` |
| "On click" script functions `show`, `hide`, `toggle`, `set_text`, `read_text`, `set_color`, `set_text_color`, `move_by`, `set_position`, `rotate`, `set_value`, `read_value`, `set_opacity`, `message` and the constant `me` | `mostrar`, `ocultar`, `alternar`, `texto`, `leer_texto`, `color`, `color_texto`, `mover`, `posicion`, `girar`, `valor`, `leer_valor`, `transparencia`, `mensaje` and `yo` |
