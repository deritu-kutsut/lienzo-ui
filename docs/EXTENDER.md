# How to add things to Lienzo UI

A practical guide to extending the code. After every change: `cargo test` and try it in the app.

## Add a tutorial page

Edit `src/tutorial.rs` and add an item to `BEGINNER` and `PRO` (English) and to `BEGINNER_ES` and `PRO_ES` (Spanish; the other languages must have the same number of pages):

```rust
Page {
    title: "My new page",
    text: "Text with line breaks.\n\n• Bullets with «•».",
},
```

Bullets and numbering are generated automatically. Also add the page to each `assets/tutorial/<code>.json` (keys `title` and `text`) and update `docs/TUTORIAL_*.md`.

## Add a template

1. In `src/model.rs`, add the name at the end of `TEMPLATES` (the order they are shown in comes from `template_order()`; "Blank" always goes last).
2. In `template(index, width, height)`, add an arm to the `match` that builds the design with `put(&mut doc, parent, Kind::..., "Name", [x, y, w, h], |e| { ... })` and the anchors with `anchor(e, min, max)`. Copy an existing template as a starting point.
3. Check: `lienzo-ui --template N --screenshot test.png` and export to a Unity project.

## Add an element type

1. `src/model.rs`: a new variant in `enum Kind` and its default values where elements are created.
2. `src/render.rs`: how it is drawn on the canvas.
3. `src/panels.rs`: where it appears in **Elements** and what the Inspector shows.
4. `src/export/ugui.rs` (importer JSON) and `src/export/EditorGuiImporter.cs` (how it is built in Unity).
5. `src/export/uitk.rs` and `imgui.rs`: its equivalent in UI Toolkit and IMGUI.
6. `src/export/view.rs`: if it has events (click, value), its field and method in the view.
7. Add a test in `src/export/mod.rs`.

## Add an exporter

Create `src/export/<format>.rs` with a function `pub fn <format>(doc: &Doc) -> String`, register it in `src/export/mod.rs` (`write_unity_package` writes the files) and add it to the **Code…** window in `src/panels.rs` (`export_window`).

## Add a command-line option

In `src/main.rs`: options without a window go in `headless()`; those that configure the editor at startup go inside the `eframe::run_native` closure. Document the option in `docs/TUTORIAL_PROFESIONALES.md`.

## Add a tool to the MCP server

In `mcp/lienzo-ui-mcp.mjs`, add an object to `TOOLS`:

```js
{
  name: "my_tool",
  description: "What it does, in one sentence.",
  inputSchema: { type: "object", properties: { file: { type: "string" } }, required: ["file"] },
  run: ({ file }) => { const { doc } = load(file); return `${doc.elements.length} elements`; },
},
```

Always use `designPath()`/`load()` to touch files, add cases to `mcp/probar-mcp.mjs` and document the tool in [MCP.md](MCP.md).

## Before opening a pull request

- `cargo test` and `node mcp/probar-mcp.mjs` passing.
- If you changed the export, export the templates to a Unity project and check that the prefabs are generated without errors.
- Add a line to `CHANGELOG.md` under **Unreleased**.

## Add a language

1. Copy `assets/i18n/es.tsv` as `assets/i18n/<code>.tsv` (for example `pt.tsv`) and translate the second column. Each line is "English text<TAB>translation"; untranslated texts appear in English.
2. In `src/i18n.rs`, add `("pt", "Português", include_str!("../assets/i18n/pt.tsv"))` to `LANGUAGES`.
3. For the tutorial, copy `assets/tutorial/pt.json` as `assets/tutorial/<code>.json`, translate `title` and `text` of each page (same number of pages) and add it to `TUTORIALS` in `src/tutorial.rs`. Without it, the tutorial is shown in English.
4. `cargo test` checks that the files are well formed (for the tutorial, that it has all its pages) and, for the `.tsv`: every key exists in `es.tsv` and is not repeated, the `{}` placeholders and the `\n` line breaks are preserved, and at least 95% of the texts are translated.

When writing new interface texts, wrap them with `crate::i18n::t("…")` and add their translation to each `.tsv`.

## Write a plugin

No need to touch the code: read [PLUGINS.md](PLUGINS.md).
