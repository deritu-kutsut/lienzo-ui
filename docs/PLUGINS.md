# Plugins (complementos)

Lienzo UI se amplía con plugins sin recompilar la app. Un plugin es una **carpeta con un `plugin.json`** que puede añadir:

- **Comandos**: modifican el diseño abierto (se deshacen con Ctrl+Z).
- **Exportadores**: generan archivos en formatos nuevos.
- **Plantillas**: diseños listos para usar.

Aparecen en la pestaña **Plugins** de la barra izquierda.

## Dónde se instalan

Lienzo UI busca carpetas de plugins en:

1. `plugins/` junto a `lienzo-ui.exe` (los del paquete descargado).
2. `plugins/` en la carpeta de trabajo (útil al desarrollar desde el repositorio).
3. La carpeta del usuario: `%APPDATA%\EditorGUI\plugins` en Windows, `~/Library/Application Support/EditorGUI/plugins` en macOS, `~/.local/share/EditorGUI/plugins` en Linux. El botón **Abrir carpeta** de la pestaña Plugins la abre.

Copia la carpeta del plugin y pulsa **Recargar**.

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

Todas las rutas son relativas a la carpeta del plugin y no pueden salir de ella (`..` o rutas absolutas se rechazan).

## Scripts

Los scripts usan [Rhai](https://rhai.rs), un lenguaje parecido a Rust/JavaScript.

### Comando

Define `fn ejecutar(diseno)`, que recibe el diseño y **devuelve el diseño modificado**:

```rust
fn ejecutar(diseno) {
    for i in 0..diseno.elements.len() {
        let e = diseno.elements[i];
        e.x = (e.x / 8.0).round() * 8.0;
        diseno.elements[i] = e;      // los elementos se copian: hay que volver a guardarlos
    }
    log("listo");                    // el mensaje aparece en la barra de estado
    diseno
}
```

### Exportador

Define `fn exportar(diseno)` y **devuelve un texto**; Lienzo UI pide dónde guardarlo con la extensión del manifiesto.

```rust
fn exportar(diseno) {
    let salida = "elemento,texto\n";
    for e in diseno.elements {
        if e.text != "" { salida += e.name + "," + e.text + "\n"; }
    }
    salida
}
```

### El diseño

| Campo | Contenido |
|---|---|
| `title`, `width`, `height` | Título y resolución de referencia |
| `background` | Color `[r, g, b, a]` (0–255) |
| `canvas_match`, `pixel_art` | Ajustes del Canvas Scaler y modo pixel art |
| `elements` | Lista de elementos, en orden de dibujo |
| `next_id` | Siguiente id libre: súmale 1 si creas elementos |

Cada elemento tiene, entre otros: `id`, `name`, `kind` (`"Panel"`, `"Image"`, `"Text"`, `"Button"`, `"Toggle"`, `"Slider"`, `"InputField"`, `"Dropdown"`, `"ScrollView"`, `"Ellipse"`, `"Empty"`), `x`, `y`, `w`, `h`, `rot`, `fill`, `stroke`, `stroke_w`, `radius`, `opacity`, `text`, `font_size`, `text_color`, `bold`, `image`, `parent`, `visible`, `anchor_min`, `anchor_max`, `pivot`, `layout`. La lista completa está en `src/model.rs` (`struct Element`).

### Seguridad y límites

- Sin acceso a archivos, red ni procesos; `import` está desactivado.
- Límite de operaciones (un bucle infinito se corta con un error) y de tamaño de textos, listas y mapas.
- Si el diseño devuelto no es válido, no se aplica nada y se muestra el error.

## Ejemplos incluidos

| Carpeta | Aporta |
|---|---|
| `plugins/ajustar-cuadricula` | Comando «Ajustar a 8 px» |
| `plugins/textos-csv` | Exportador de todos los textos a CSV (para traducir) |
| `plugins/plantillas-extra` | Plantilla «Tienda del herrero» |

## Probar un plugin

```bash
cargo test plugins     # carga los ejemplos y ejecuta su comando y su exportador
```

Para publicar tu plugin, súbelo a tu propio repositorio o propón añadirlo a `plugins/` con un pull request (ver [CONTRIBUTING.md](../CONTRIBUTING.md)).
