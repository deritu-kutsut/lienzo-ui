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
  "nombre": "Mi plugin",
  "version": "1.0.0",
  "autor": "Tu nombre",
  "descripcion": "Qué hace, en una frase.",
  "comandos": [
    { "nombre": "Ajustar a 8 px", "descripcion": "Texto de ayuda", "script": "ajustar.rhai" }
  ],
  "exportadores": [
    { "nombre": "Textos para traducir", "extension": "csv", "script": "csv.rhai" }
  ],
  "plantillas": ["tienda.editorgui.json"]
}
```

All paths are relative to the plugin folder and cannot leave it (`..` or absolute paths are rejected).

## Scripts

Scripts use [Rhai](https://rhai.rs), a language similar to Rust/JavaScript.

### Command

Define `fn ejecutar(diseno)`, which receives the design and **returns the modified design**:

```rust
fn ejecutar(diseno) {
    for i in 0..diseno.elements.len() {
        let e = diseno.elements[i];
        e.x = (e.x / 8.0).round() * 8.0;
        diseno.elements[i] = e;      // elements are copied: they must be stored back
    }
    log("listo");                    // the message appears in the status bar
    diseno
}
```

### Exporter

Define `fn exportar(diseno)` and **return a text**; Lienzo UI asks where to save it, using the extension from the manifest.

```rust
fn exportar(diseno) {
    let salida = "elemento,texto\n";
    for e in diseno.elements {
        if e.text != "" { salida += e.name + "," + e.text + "\n"; }
    }
    salida
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
- Operation limit (an infinite loop is cut off with an error) and size limit for texts, lists and maps.
- If the returned design is not valid, nothing is applied and the error is shown.

## Included examples

| Folder | Provides |
|---|---|
| `plugins/ajustar-cuadricula` | "Ajustar a 8 px" command |
| `plugins/textos-csv` | Exporter of all texts to CSV (for translating) |
| `plugins/plantillas-extra` | "Tienda del herrero" template |

## Testing a plugin

```bash
cargo test plugins     # loads the examples and runs their command and exporter
```

To publish your plugin, upload it to your own repository or propose adding it to `plugins/` with a pull request (see [CONTRIBUTING.md](../CONTRIBUTING.md)).
