# Importing from Unity

Lienzo UI can read a Unity uGUI prefab (`.prefab`, Unity's text YAML) or the `Designs/*.json` file that Lienzo itself exports,
and turn it into an ordinary design (`.editorgui.json`, see [FORMAT.md](FORMAT.md)). Everything that could not be mapped is listed as a
warning, so nothing is lost silently.

## How to import

Headless (no window, usable from scripts and CI):

```
lienzo-ui --import-unity Assets/UI/MainMenu.prefab --save MainMenu.editorgui.json
lienzo-ui --import-unity Assets/EditorGUI/Designs/Settings.json --save Settings.editorgui.json
lienzo-ui --import-unity MainMenu.prefab --review          # review what was imported
lienzo-ui --import-unity MainMenu.prefab --export MyUnityProject   # and export it back

lienzo-ui --import-unity-project MyUnityProject --out Imported    # every UI prefab of the project
```

* `--import-unity <file>` works with `--save`, `--export` and `--review` like `--open` does. Warnings go to stderr, a summary to stdout.
* `--import-unity-project <project folder> --out <folder>` walks `<project>/Assets`, converts every `.prefab` that has a RectTransform
  hierarchy and writes `<name>.editorgui.json` into the output folder (nothing is written anywhere else). Prefabs without UI are skipped;
  the summary says how many were converted, skipped or failed.
* Exit codes: `0` success, `1` the import failed (unreadable / invalid file, a prefab failed in project mode), `2` wrong usage.

From Rust: `unity_import::import_file(path) -> Result<Imported, String>` (`Imported { doc, warnings }`),
`unity_import::import_unity_project(dir, out_dir) -> Result<Vec<(String, usize)>, String>` (file written, element count) and
`import_unity_project_report` (the same with the skipped / failed prefabs and the warnings of each one).

## Where the sprites come from

When the prefab sits inside a Unity project (an ancestor folder is called `Assets`), the importer reads the `.meta` files of the project to
turn sprite GUIDs into asset paths. A prefab inside a package folder (`package.json`) reads that package only.

| Unity | Lienzo element |
|---|---|
| `Image.sprite` (path from the GUID) | `image: "unity:Assets/UI/frame.png"` |
| sub-sprite of a multi-sprite texture (fileID in the `.meta` sprite sheet) | `sprite: "<sprite name>"`, with that sprite's own border |
| texture importer `spriteBorder` | `slice: [left, bottom, right, top]` |
| texture importer `spritePixelsToUnits` (times `Image.pixelsPerUnitMultiplier` when it is not 1) | `ppu` |
| `Image.type` Sliced / Tiled / Filled | `image_mode` |
| `Image.preserveAspect` | `preserve_aspect` |
| tint alpha of a picture | `opacity` |

Sprites that Lienzo generated itself (`Sprites/Generated/rr_<radius>_<stroke>.png`, `ellipse_<n>.png`, `grad_<RRGGBBAA>_<RRGGBBAA>.png`)
are turned back into what they were: a rounded panel (`radius`), a circle, a gradient. Unity's built-in sprites (UISprite, Background, Knob...)
cannot be read from outside Unity: those images become plain colour panels (with `image_mode` kept) and one warning says how many.

## What maps to what

Components are identified by `m_EditorClassIdentifier`, then by the well-known script GUIDs of uGUI / TextMeshPro, then by their serialised
fields. Custom scripts are reported (with their class name when the project's `.cs.meta` can be found).

| Unity | Lienzo |
|---|---|
| Canvas root + CanvasScaler (Scale With Screen Size) | the page: `width`, `height` = reference resolution, `canvas_match` = Match; `canvas.render_mode`, `sort_order`, `plane_distance`; Pixel Perfect -> `pixel_art` |
| Canvas with CanvasScaler in another mode / without one | page size from the canvas RectTransform, else 1920x1080 (warning) |
| root without Canvas | `canvas.embed = true`; a plain container root is the page (its size), anything else becomes an element centered on a 1920x1080 page |
| GameObject + RectTransform | one element: anchors, pivot, position, size and Z rotation become the element's box (`apply_rect_transform`, the exact inverse of the exporter), children keep their sibling order |
| GameObject name | `name` and `unity_name` (always set, so names stay stable for game code) |
| `m_IsActive: 0` | `visible: false` |
| Image | Panel (no picture) or Image (picture); colour -> `fill`; raycast target -> `raycast` |
| Text (legacy) / TextMeshProUGUI | Text element: text, size (auto size uses the maximum), colour, bold / italic / underline, alignment, font |
| TMP font asset | `font` (and `unity.default_font` = the most used one; elements that use it keep `font` empty) |
| Button | Button; its stretched text child becomes the button's `text` and is absorbed |
| Toggle | Toggle with `value` (isOn); Background / Checkmark / Label are absorbed; the checkmark colour is `fill` |
| Slider | Slider with `value` normalised to 0..1; fill colour -> `fill`, Background colour -> `fill2`; Fill Area / Handle Slide Area are absorbed |
| InputField / TMP_InputField | InputField; its text (or the placeholder when empty) is `text`; Text Area is absorbed |
| Dropdown / TMP_Dropdown | Dropdown with `options` and `value`; Label / Arrow / Template are absorbed |
| ScrollRect | ScrollView; `mask` from the viewport's Mask / RectMask2D; the children of Content become children of the ScrollView; a layout group on Content moves to the ScrollView |
| Mask, RectMask2D | `mask: true` (a Mask is a rectangular clip) |
| Horizontal / Vertical / Grid Layout Group | `layout` (padding, spacing, child alignment, cell size; "control size" and "force expand" both on = `expand_w` / `expand_h`) |
| Shadow | `shadow` |
| Outline | `stroke` / `stroke_w` (not on text) |
| CanvasGroup alpha | `opacity` of the element and of its descendants (the product of the groups above) |
| RectTransform scale | folded into size and position |
| no component other than the RectTransform | Empty (container) |

Lienzo's own prefab structure is recognised: the `Fondo del lienzo` child of the canvas is the page `background`, the `Borde` ring child
becomes `stroke` / `stroke_w`, and the `Texto` child of a panel is its `text`.

## Warnings

Each kind of problem is reported once, naming the first GameObject and counting the others ("... on 'Panel' and 3 more").
Typical ones:

* `Unsupported component <Name>`: scripts, Animator, AudioSource, LayoutElement, AspectRatioFitter... They are not imported.
* `ContentSizeFitter ignored`, `LayoutElement ignored`, `Scrollbar imported as a plain container`, `RawImage imported as a plain panel`.
* `UnityEvent listener(s) are not imported`: OnClick / OnValueChanged calls cannot be stored in a design.
* `Layout group controls the size of its children`: children driven by a layout group keep the sizes saved in the prefab, which may be 0.
* `sprite reference(s) could not be resolved`: no `.meta` with that GUID was found (prefab outside a project, or the asset lives in a package).
* `nested prefab instance(s) are not expanded`: a nested prefab instance (and prefab variants) is not followed; import the source prefab.
* `3D rotation ignored`, `scale folded into the size`, `Filled image not supported`, `Vertical slider imported as horizontal`,
  `Border ring imported with an approximate stroke width`, `Mask imported as RectMask2D`, `CanvasGroup alpha folded`.

## Limits

* Input files above 20 MB are refused, as are files with more than 100 000 YAML documents. Nesting deeper than 48 levels of YAML or 128 levels
  of GameObjects, strings above 64 KB and designs above 5 000 elements are cut (with a warning). A hostile file cannot make the importer
  panic or hang.
* Only the `.prefab` text format is read (not binary-serialised assets, scenes or ScriptableObjects). Prefab variants only hold overrides: open the source prefab.
* The Viewport of a ScrollView is taken as the full ScrollView box, like Lienzo's own export (the width a visible scrollbar takes away at runtime is not modelled).
* Layout groups are not evaluated: elements keep the geometry saved in the prefab.
* A Button / Toggle / Slider / InputField / Dropdown is imported as one Lienzo element; its standard sub-objects are not kept, so the export
  recreates them with Unity's default names.
* Images that point to a Unity asset are stored as `unity:Assets/...` only (the importer does not copy or decode the picture files).
* Colours are read as the values saved in the file (no colour-space conversion).
