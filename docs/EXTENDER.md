# Cómo añadir cosas a Lienzo UI

Guía práctica para ampliar el código. Después de cada cambio: `cargo test` y prueba en la app.

## Añadir una página al tutorial

Edita `src/tutorial.rs` y añade un elemento a `NOVATO` o `PROFESIONAL`:

```rust
Pagina {
    titulo: "Mi nueva página",
    texto: "Texto con saltos de línea.\n\n• Viñetas con «•».",
},
```

Los puntos y la numeración se generan solos. Actualiza también `docs/TUTORIAL_*.md`.

## Añadir una plantilla

1. En `src/model.rs`, añade el nombre al final de `TEMPLATES` (antes de «En blanco» si quieres que salga antes).
2. En `template(index, width, height)`, añade un brazo al `match` que construya el diseño con `put(&mut d, padre, Kind::..., "Nombre", [x, y, w, h], |e| { ... })` y las anclas con `anchor(e, min, max)`. Copia una plantilla existente como base.
3. Comprueba: `lienzo-ui --plantilla N --screenshot prueba.png` y exporta a un proyecto de Unity.

## Añadir un tipo de elemento

1. `src/model.rs`: nueva variante en `enum Kind` y sus valores por defecto donde se crean elementos.
2. `src/render.rs`: cómo se dibuja en el lienzo.
3. `src/panels.rs`: dónde aparece en **Elementos** y qué muestra el Inspector.
4. `src/export/ugui.rs` (JSON del importador) y `src/export/EditorGuiImporter.cs` (cómo se construye en Unity).
5. `src/export/uitk.rs` e `imgui.rs`: su equivalente en UI Toolkit e IMGUI.
6. `src/export/view.rs`: si tiene eventos (clic, valor), su campo y método en la vista.
7. Añade un test en `src/export/mod.rs`.

## Añadir un exportador

Crea `src/export/<formato>.rs` con una función `pub fn <formato>(doc: &Doc) -> String`, regístrala en `src/export/mod.rs` (`write_unity_package` escribe los archivos) y añádela a la ventana **Código…** en `src/panels.rs` (`export_window`).

## Añadir una opción de línea de comandos

En `src/main.rs`: las opciones sin ventana van en `headless()`; las que configuran el editor al arrancar, dentro del cierre de `eframe::run_native`. Documenta la opción en `docs/TUTORIAL_PROFESIONALES.md`.

## Añadir una herramienta al servidor MCP

En `mcp/lienzo-ui-mcp.mjs`, añade un objeto a `HERRAMIENTAS`:

```js
{
  name: "mi_herramienta",
  description: "Qué hace, en una frase.",
  inputSchema: { type: "object", properties: { archivo: { type: "string" } }, required: ["archivo"] },
  run: ({ archivo }) => { const { doc } = leer(archivo); return `${doc.elements.length} elementos`; },
},
```

Usa siempre `rutaDiseno()`/`leer()` para tocar archivos, añade casos a `mcp/probar-mcp.mjs` y documenta la herramienta en [MCP.md](MCP.md).

## Antes de abrir un pull request

- `cargo test` y `node mcp/probar-mcp.mjs` en verde.
- Si cambiaste la exportación, exporta las plantillas a un proyecto de Unity y comprueba que los prefabs se generan sin errores.
- Añade una línea a `CHANGELOG.md` en **Sin publicar**.

## Añadir un idioma

1. Copia `assets/i18n/en.tsv` como `assets/i18n/<código>.tsv` (por ejemplo `pt.tsv`) y traduce la segunda columna. Cada línea es «texto en español<TAB>traducción»; los textos sin traducción se ven en español.
2. En `src/i18n.rs`, añade `("pt", "Português", include_str!("../assets/i18n/pt.tsv"))` a `IDIOMAS`.
3. Para el tutorial, añade sus páginas traducidas en `src/tutorial.rs` siguiendo `NOVATO_EN` y `PROFESIONAL_EN`.
4. `cargo test` comprueba que el archivo está bien formado.

Al escribir textos nuevos en la interfaz, envuélvelos con `crate::i18n::t("…")` y añade su traducción a cada `.tsv`.

## Escribir un plugin

No hace falta tocar el código: lee [PLUGINS.md](PLUGINS.md).
