# Lienzo UI design file format (`.editorgui.json`)

A design is one JSON object (UTF-8, a BOM is tolerated). Machine-readable schema: [`editorgui.schema.json`](editorgui.schema.json).
Check a file headless with:

```
lienzo-ui --validate my.editorgui.json     # prints "$.elements[3].fill[1]: ..." errors, exit code 1 if invalid
```

**Every field is optional.** A missing field takes the default listed below; unknown fields are ignored. On load the editor
repairs the file (`Doc::sanitize`): missing / `0` / duplicate element ids get fresh ids, `next_id` becomes at least
`max(id) + 1` (minimum 1), parents that do not exist or form cycles are cleared, numbers are clamped, NaN / infinity are replaced,
control characters and line separators are removed from text. Nothing in a hostile file can make the loader panic.
New fields are skipped on save when they hold their default, so files that do not use them round-trip unchanged.

## Minimal example

```json
{
  "title": "Pause menu",
  "width": 1920,
  "height": 1080,
  "background": [20, 20, 30, 255],
  "elements": [
    { "id": 1, "name": "Window", "kind": "Panel",  "x": 700, "y": 300, "w": 520, "h": 360 },
    { "id": 2, "name": "Resume", "kind": "Button", "x": 800, "y": 500, "w": 320, "h": 84, "parent": 1, "text": "Resume" }
  ]
}
```

No `next_id` (becomes 3), no `canvas_match` (0.5); `fill`, `font_size`... take the defaults below.

## Doc (top level)

| Field | Type | Default | Meaning |
|---|---|---|---|
| `title` | string | `"Untitled design"` | Design name; becomes the prefab / class name (`class_name(title)`). |
| `width`, `height` | number | 1920, 1080 | Reference resolution in pixels (CanvasScaler reference resolution). Clamped to 1..16384. |
| `background` | `[r,g,b,a]` 0-255 | `[255,255,255,255]` | Page colour. **Alpha 0 = transparent**: exporters must not emit a canvas background image / "Fondo del lienzo" (`Doc::has_background()` is false). |
| `elements` | Element[] | `[]` | Depth-first list: a parent comes before its children; siblings draw in list order (last = on top), like a Unity hierarchy. Re-ordered on load. |
| `next_id` | integer | `max(id)+1` (min 1) | Next free element id. |
| `canvas_match` | number 0..1 | 0.5 | CanvasScaler "Match" (0 = width, 1 = height). |
| `pixel_art` | bool | false | Pixel-art mode: unsmoothed images, integer positions; Unity: Canvas Pixel Perfect, Point filter, no compression. |
| `canvas` | CanvasSettings | defaults | See below. Omitted when default. |
| `unity` | UnitySettings | absent | See below. Omitted when default. |
| `portrait` | PortraitLayout or null | null | Second layout for portrait screens. |

### CanvasSettings (`canvas`)

| Field | Type | Default | Meaning |
|---|---|---|---|
| `render_mode` | `"ScreenSpaceOverlay"` \| `"ScreenSpaceCamera"` \| `"WorldSpace"` | `ScreenSpaceOverlay` | `Canvas.renderMode`. |
| `sort_order` | integer | 0 | `Canvas.sortingOrder`, clamped to -32768..32767. |
| `safe_area` | bool | false | Apply the device safe area to the root. |
| `embed` | bool | false | Export without its own Canvas / CanvasScaler: the root is a plain RectTransform meant to live under the game's canvas. |
| `plane_distance` | number | 100 | `Canvas.planeDistance` (ScreenSpaceCamera), clamped 0.01..100000. |

### UnitySettings (`unity`)

| Field | Type | Default | Meaning |
|---|---|---|---|
| `scripts_dir` | string or null | null | Folder for the generated scripts, relative to the Unity project and starting with `Assets` (`Assets/Game/UI`). Forward slashes; no `..`, no absolute path, no drive letter, no empty segment. Invalid values are dropped on load. |
| `assembly` | string or null | null | Name of an existing assembly definition the generated scripts live in / refer to. `[A-Za-z_][A-Za-z0-9_.-]*`, max 128. |
| `generate` | GenerateOutputs | see below | Which outputs to generate. |
| `default_font` | string or null | null | Name of a TextMeshPro font asset in the Unity project. |
| `fallback_fonts` | string[] | `[]` | Fallback TMP font asset names (max 16). |

`generate`: `prefab` (true), `uitk` (true), `view_scripts` (true), `imgui` (see below).

**IMGUI default.** A file with **no `unity` block** behaves as before: the IMGUI script is generated. A **new design** is created with
`unity.generate.imgui = false`. Inside an explicit `unity` block a missing `imgui` is `false`. In code use
`doc.unity.generate.imgui_enabled()` (an `Option<bool>`: `None` = legacy "on").

### PortraitLayout (`portrait`)

The same design with a second layout used when the screen is portrait. Elements without an override inherit the landscape values.

| Field | Type | Default | Meaning |
|---|---|---|---|
| `width`, `height` | number | 1080, 1920 | Reference resolution of the portrait canvas (1..16384). |
| `overrides` | ElementOverride[] | `[]` | At most one per element id. Overrides of deleted elements are dropped on load / delete. |

`ElementOverride`: `id` (required, element id) and any of `x, y, w, h, rot` (same meaning and units as Element, in the portrait page),
`anchor_min`, `anchor_max`, `pivot` (`[x,y]`, same convention as Element), `visible` (bool), `font_size` (number), `layout` (a LayoutGroup).
Absent / null = inherit. Children do **not** follow a parent's override automatically: override each element that moves.
API: `Doc::resolved_for(Orientation::Portrait)`, `Doc::set_override(ElementOverride)`, `Doc::clear_override(id)`, `Doc::override_of(id)`.

## Element

Geometry: `x`, `y`, `w`, `h` describe the **unrotated box in page pixels** (origin top-left, y grows downward, absolute / "world", not relative
to the parent). `rot` is a clockwise rotation in degrees around the box centre. Unity values (anchors, pivot, anchoredPosition, sizeDelta)
are computed relative to the parent's box on export.

| Field | Type | Default | Meaning |
|---|---|---|---|
| `id` | integer | assigned | Unique id. 0 / missing / duplicate gets a new id. |
| `name` | string | `"Panel 0"` | Display name (editor label). Exported as the GameObject name unless `unity_name` is set. |
| `kind` | Kind | `Panel` | `Panel`, `Ellipse`, `Image`, `Text`, `Button`, `Toggle`, `Slider`, `InputField`, `Dropdown`, `ScrollView`, `Empty` (container, plain RectTransform). |
| `x`, `y`, `w`, `h` | number | 0, 0, 300, 200 | Box in page pixels. `w`, `h` are clamped to >= 0. |
| `rot` | number | 0 | Degrees clockwise. |
| `fill` | RGBA | `[124,92,255,255]` | Background colour (Image without picture: gradient start). |
| `fill2` | RGBA | `[0,196,204,255]` | Gradient end (Image without picture); slider track. |
| `stroke`, `stroke_w` | RGBA, number | `[30,30,40,255]`, 0 | Outline colour and width (px, 0..1000). |
| `radius` | number | 0 | Corner radius px (0..10000; never more than half the short side when exported). |
| `opacity` | number 0..1 | 1 | Alpha multiplier. |
| `text` | string | `""` | Caption / content. |
| `font_size` | number | 32 | px, 1..1000. |
| `text_color` | RGBA | `[20,20,30,255]` | |
| `bold`, `italic`, `underline` | bool | false | |
| `align` | 0/1/2 | 1 | Horizontal: 0 left, 1 centre, 2 right. |
| `valign` | 0/1/2 | 1 | Vertical: 0 top, 1 middle, 2 bottom (Text elements always export top). |
| `font` | string | `""` | Font family; empty = default. |
| `on_click` | string | `""` | Script run when the element is clicked in test mode (rhai, see docs/EXTENDING.md). Also exported as a comment / hook. |
| `image` | string | `""` | Picture of an Image: library file name or path. |
| `value` | number | 0.5 | Slider value 0..1, toggle on (1) / off (0), dropdown selected index. |
| `visible` | bool | true | GameObject active. |
| `locked` | bool | false | Editor only: cannot be selected / moved. |
| `group` | integer or null | null | Legacy flat group (pre-hierarchy); converted to an `Empty` parent on load, never written. |
| `parent` | integer or null | null | Parent element id; null = child of the Canvas. |
| `anchor_min`, `anchor_max`, `pivot` | `[x,y]` | `[0.5,0.5]` | Unity RectTransform anchors / pivot, normalised, **y grows upward** (Unity convention), relative to the parent rectangle. |
| `options` | string[] | `[]` | Dropdown options. |
| `interactable` | bool | true | Selectable.interactable (Button, Toggle, Slider, InputField, Dropdown). |
| `raycast` | bool | true | Graphic.raycastTarget. |
| `mask` | bool | false | RectMask2D: children are clipped to this box. |
| `shadow` | Shadow | off | uGUI `Shadow` effect. |
| `layout` | LayoutGroup | none | Layout group applied to the direct children. |
| `collapsed` | bool | false | Hierarchy row collapsed (editor only). |
| `unity_name` | string or null | null | **Exact GameObject name to export** instead of `name`. Not blank, no `/`, no control characters, max 128 chars. Control / line-separator characters are stripped on load; an invalid value is ignored by the exporter and flagged by the review. |
| `slice` | `[l,b,r,t]` or null | null | 9-slice border in **source-image pixels**, each >= 0. Implies `image_mode = Sliced` when `image_mode` is absent. |
| `ppu` | number or null | null | Sprite pixels per unit, clamped 0.01..10000. |
| `sprite` | string or null | null | Sub-sprite name inside a multi-sprite texture / atlas. |
| `image_mode` | `"Simple"` \| `"Sliced"` \| `"Tiled"` \| `"Filled"` or null | null | `Image.type`. |
| `preserve_aspect` | bool | false | `Image.preserveAspect`. |

### Shadow (`shadow`)
`on` (false), `color` (RGBA, default `[0,0,0,110]`), `offset` (`[x,y]`, default `[6,-6]`, Unity convention: x right, y up).

### LayoutGroup (`layout`)
Applied by the editor to the visible direct children (like Unity does) and exported as a Horizontal / Vertical / Grid Layout Group.

| Field | Type | Default | Meaning |
|---|---|---|---|
| `mode` | `"None"` \| `"Horizontal"` \| `"Vertical"` \| `"Grid"` | `None` | |
| `padding` | `[left,right,top,bottom]` | `[16,16,16,16]` | px, -10000..10000. |
| `spacing` | `[x,y]` | `[12,12]` | px. |
| `align` | 0..8 | 0 | Unity `TextAnchor`, row-major: 0 UpperLeft, 1 UpperCenter, 2 UpperRight, 3 MiddleLeft ... 8 LowerRight. |
| `cell` | `[w,h]` | `[100,100]` | Grid cell size, 1..10000. |
| `expand_w`, `expand_h` | bool | false | Control child size + force expand: children share the space. |

### `on_click` scripts
A short rhai snippet run in the editor's test mode when the element is clicked (move / show / hide elements, change values). It cannot do I/O. It is exported as data (`onClick`).

## Export JSON (`Designs/*.json`, read by the Unity importer)

Produced by `export::ugui::json`. Format id `editor-gui/ugui/2`; the importer ignores keys it does not know. The file never contains `null`
(Unity `JsonUtility` cannot read it): optional values are simply absent.

New keys, written **only when the design uses them**:

* Per element: `unityName` (string; the valid `unity_name`; `name` already holds the final, de-duplicated GameObject name),
  `slice` (`[l,b,r,t]`), `ppu` (number), `sprite` (string), `imageMode` (`"Simple"|"Sliced"|"Tiled"|"Filled"`, written when `image_mode` is set or a `slice` exists;
  `Sliced` is implied by `slice`). `image.preserveAspect` now carries `preserve_aspect`.
* Inside the existing `canvas` object (only when a canvas option is not default): `renderMode`, `sortOrder`, `safeArea`, `embed`, `planeDistance`.
* `unity` (only when the block is not default): `scriptsDir`, `assembly`, `defaultFont`, `fallbackFonts` (each only when set) and
  `generate: { prefab, uitk, imgui, viewScripts }` (`imgui` already resolved with the legacy rule).
* `portrait` (only when present): `{ width, height, overrides: [ { id, x?, y?, w?, h?, rot?, anchorMin?, anchorMax?, pivot?, visible?, fontSize?, layout? } ] }`.
  Values are the raw model values of the portrait page (the same units as the element's `x, y, w, h`, **not** converted to RectTransform values);
  `id` is the element `id` of the `elements` list; `layout` uses the same keys as an element's `layout`.

Name stability: a valid `unity_name` is used verbatim. Siblings that end up with the same name get a ` (2)`, ` (3)` suffix; explicit names are
assigned before automatic ones, and the design review warns about both clashing siblings and invalid `unity_name` values.

## Export notes (Unity)

* **Transparent page.** With `background` alpha 0 the uGUI JSON has no `canvas.background` key (the importer then creates no background image), the USS root has no `background-color` and the IMGUI script draws no background.
* **Existing project pictures.** `image` = `unity:<asset path>` or a path under `Assets/` (also `Packages/`) is referenced as it is: nothing is copied or overwritten, and `source` in the JSON is that asset path. `unity:<asset path>#<sub-sprite>` picks a sub-sprite of a multi-sprite sheet (same as `sprite`, which wins). On the Unity side the import settings of such a texture are only changed where the design says so: `slice` (border), `ppu`, Point filter when `pixel_art`. A texture Lienzo copied itself is marked (`userData = lienzo:sprite`) and may be re-configured. Generated sprites of a pixel-art design are separate files (`rr_*_px.png`, `ellipse_*_px.png`, `grad_*_px.png`).
* **UI Toolkit.** A `slice` (or `image_mode` Sliced) becomes `-unity-slice-left/bottom/right/top` (integer px) and `preserve_aspect` becomes `-unity-background-scale-mode: scale-to-fit` (otherwise `stretch-to-fill`).
* **Element `field`.** Each element of the JSON carries `field`, the name of the serialized field in `<Class>ViewBase` that the importer assigns (a valid C# identifier `unity_name` is used verbatim, otherwise a camelCase name derived from it). `Bind()` only fills fields that are still empty, by path (`__safe_area/` comes first when `canvas.safe_area` is on).
* **Outputs (`unity.generate`).** `prefab`: importer, `Designs/*.json` and the runtime helpers; `uitk`: UXML + USS; `imgui` (`imgui_enabled()`): `*OnGUI.cs`; `view_scripts`: `*ViewBase.cs` and the one-time `*View.cs`. Pictures and fonts are copied when `prefab` or `uitk` is on.
* **Scripts folder / assembly.** Scripts go to `unity.scripts_dir` (default `Assets/EditorGUI/Scripts`). If `unity.assembly` names an `.asmdef` (or an `.asmref` pointing to it) found under `Assets`, the scripts are written inside that folder (`scripts_dir` if it is inside, else `<folder>/LienzoUI`), so they compile into it; if it is not found they stay in `scripts_dir` and a note is printed. Runtime helpers (`LienzoSafeArea.cs`, `LienzoOrientationLayout.cs`) are written once into `<scripts folder>/Runtime` when needed, unless a class of that name already exists elsewhere in the project. A generated `*ViewBase.cs` left in another folder by an earlier export is removed so the class is not duplicated.
* **Canvas.** `renderMode`, `sortOrder`, `planeDistance` (ScreenSpaceCamera; the camera is assigned in the scene), WorldSpace (the root is sized to the reference resolution and scaled 0.01), `embed` (root = plain stretched RectTransform, no Canvas / CanvasScaler / GraphicRaycaster), `safeArea` (a stretched `__safe_area` child with `LienzoSafeArea` holds every top-level element).
* **Portrait.** Each `portrait.overrides[]` entry keeps the raw keys and adds the real Unity state: `rect` (`anchorMin, anchorMax, pivot, anchoredPosition, sizeDelta, rotationZ`, computed on `resolved_for(Portrait)` against the parent's portrait box; only when the override changes geometry) and the flags `hasRect`, `hasVisible`, `hasFontSize`, `hasLayout` (`visible`, `fontSize` and `layout` hold the values). The importer adds `LienzoOrientationLayout` with both states per overridden element; it applies portrait when `Screen.width < Screen.height` and has `Apply(bool portrait)`. Changing a layout group's type between orientations is not supported at runtime.
* **Fonts.** `unity.defaultFont` is used by elements without `font`; fonts are existing TextMeshPro font assets found by name (also `Fonts & Materials`), an SDF asset is only created from `Assets/EditorGUI/Fonts/<name>.ttf|otf` when none exists, `unity.fallbackFonts` are appended to the font asset's fallback table when missing, and the Console warns with the characters the font cannot draw.
* **Generated object names.** The importer's own helpers are `__border`, `__text`, `__tick`, `__canvas_background` and `__safe_area` (they can no longer collide with a `unity_name`); element GameObjects get exactly the exported `name`.
* **First generation.** The designs waiting for a prefab are kept in `Library/LienzoUI/pending.txt` and built when Unity is idle after the generated scripts compiled; `Tools > Lienzo UI > Regenerate all prefabs` rebuilds all of them. On start / reload a prefab that is missing, older than its design, or lacks the View (or runtime helper) component is rebuilt.

## Limits applied when a file is loaded

Files are sanitized on load (and the same rules apply to community content), so hostile or damaged files cannot crash the editor:

- At most **20 000 elements**; nesting deeper than **128 levels** is flattened (the deeper elements are moved to the root, in order).
- Element ids must be below 2^52; an id outside that range is renumbered (parents and portrait overrides follow).
- `anchor_min`, `anchor_max` and `pivot` (also in portrait overrides) are clamped to ±10 000; `canvas_match` to 0..1; `x`, `y`, `w`, `h` and `value` to ±1 000 000.
- `unity_name` must not equal a name Lienzo UI generates (`__border`, `__text`, `__tick`, `__safe_area`, `__canvas_background`).
- `unity.scripts_dir` must stay under `Assets` and cannot contain `< > " | ? *`, trailing dots or spaces, or Windows reserved device names. Font names (`font`, `unity.default_font`, `unity.fallback_fonts`) cannot contain `/`, `\`, `:` or `..`.
- `unity.generate.imgui` is always written when the `unity` block exists; a file without a `unity` block keeps the legacy behavior (IMGUI script generated).
