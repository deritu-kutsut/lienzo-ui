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
  hierarchy and writes `<name>.editorgui.json` into the output folder (nothing is written anywhere else). Existing files are never overwritten:
  on a name collision a numeric suffix is added (`Menu_2.editorgui.json`) and a warning says so. Prefabs without UI are skipped;
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

### How image references are resolved

* `unity:Assets/...` and `unity:Packages/...` are resolved against the Unity project root. The path is validated component by component:
  relative, only normal components (no `..`, no root, no empty component), the first one exactly `Assets` or `Packages` (case-sensitive, so
  `AssetsEvil/...` fails), no `:` (drive letters, NTFS streams), no reserved device names. The file found must also stay inside the project
  after resolving junctions / symlinks. Anything else is refused and reported as a missing image.
* In the editor's «Import from Unity» dialog the root is the project that holds the imported file (used for this session; the saved
  `unity_project` setting is only filled when it was empty, and the status line says "Unity project detected: ...").
* Relative image names (no `unity:`) may not contain `..` or be rooted, and must stay inside the library images folder or the folder of
  the open design. Absolute paths are only honoured for designs the user opened from disk (`library::set_trusted_source`); designs produced by
  plugins or downloaded from the community must be loaded with the flag set to false.
* In an exported Lienzo `Designs/*.json`, `image.source` is only accepted as `Assets/...` / `Packages/...` (turned into `unity:`) or a
  plain file name; anything else (absolute, UNC, `..`, drive, subfolders) is dropped with a warning and never read.

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
becomes `stroke` / `stroke_w`, and the `Texto` child of a panel is its `text`. These name-based matches (also `Arrow` of a Dropdown and
`Background` of a Slider) are reported with a warning, since a child that happens to have such a name is absorbed. A ScrollView keeps the
sibling order of `m_Children` (its content is emitted where the viewport sits).

## Warnings

Each kind of problem is reported once, naming the first GameObject and counting the others ("... on 'Panel' and 3 more").
Typical ones:

* `Unsupported component <Name>`: scripts, Animator, AudioSource, LayoutElement, AspectRatioFitter... They are not imported.
* `ContentSizeFitter ignored`, `LayoutElement ignored`, `Scrollbar imported as a plain container`, `RawImage imported as a plain panel`.
* `UnityEvent listener(s) are not imported`: OnClick / OnValueChanged calls cannot be stored in a design.
* `Layout group controls the size of its children`: children driven by a layout group keep the sizes saved in the prefab, which may be 0.
* `sprite reference(s) could not be resolved`: no `.meta` with that GUID was found (prefab outside a project, or the asset lives in a package).
* `nested prefab instance(s) are not expanded`: a nested prefab instance (and prefab variants) is not followed; import the source prefab.
* `3D rotation ignored`, `scale folded into the size`, `Negative scale (mirrored) imported as a positive scale`, `Filled image not supported`, `Vertical slider imported as horizontal`,
  `Border ring imported with an approximate stroke width`, `Mask imported as RectMask2D`, `CanvasGroup alpha folded`.

## Limits

* Input files above 20 MB are refused, as are files with more than 100 000 YAML documents. Nesting deeper than 48 levels of YAML or 128 levels
  of GameObjects, designs above 5 000 elements, scalars above 64 KB or spanning more than 256 lines, more than 256 components per GameObject,
  text above 4 KB per element and above 8 MB per import are cut (each with a warning). Multi-line scalars are scanned incrementally, so an
  unclosed quote or bracket followed by hundreds of thousands of lines costs linear time. A hostile file cannot make the importer panic or hang.
* Warnings are stripped of control characters (ESC, BEL, bidi overrides...) because names, script names, GUIDs and file names come from the file.
* Only the `.prefab` text format is read (not binary-serialised assets, scenes or ScriptableObjects). Prefab variants only hold overrides: open the source prefab.
* The Viewport of a ScrollView is taken as the full ScrollView box, like Lienzo's own export (the width a visible scrollbar takes away at runtime is not modelled).
* Layout groups are not evaluated: elements keep the geometry saved in the prefab.
* A Button / Toggle / Slider / InputField / Dropdown is imported as one Lienzo element; its standard sub-objects are not kept, so the export
  recreates them with Unity's default names.
* Images that point to a Unity asset are stored as `unity:Assets/...` only (the importer does not copy or decode the picture files).
* Colours are read as the values saved in the file (no colour-space conversion).
