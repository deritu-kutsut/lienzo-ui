# Lienzo UI for Unity

Free, open source visual editor for game interfaces. Design a screen once and export it to Unity as **uGUI + TextMeshPro**, **UI Toolkit** (UXML/USS) and **IMGUI**.

This package contains the Unity side of Lienzo UI:

- **Importer** (`Editor/EditorGuiImporter.cs`): turns the designs the app exports into ready prefabs (uGUI + TextMeshPro). Menu: **Tools > Lienzo UI**.
- **Window** (`Tools > Lienzo UI > Open Lienzo UI (app)`): opens the desktop app, shows this project's path and lists the sample projects.
- **Sample designs**: seven complete screens (main menu, game HUD, pixel-art HUD, dialog, settings, inventory, shop), as exported designs (`Designs/`), their sprites, and as Lienzo UI projects (`Samples/`, to open in the app).

## Requirements

- Unity 6 (6000.0 or newer). Tested on 6000.3 with URP. It is plain uGUI, so it works with the Built-in, URP and HDRP render pipelines.
- **Unity UI** (uGUI) and **TextMeshPro** with its Essential Resources (*Window > TextMeshPro > Import TMP Essential Resources*; the importer opens that window by itself if they are missing).
- The **Lienzo UI desktop app** (Windows, free): https://github.com/deritu-kutsut/lienzo-ui/releases/latest · https://deritu.itch.io/lienzo-ui . On macOS / Linux build it from source (Rust).

## Quick start

1. Download and run Lienzo UI. Follow its tutorial and pick a template.
2. Press **Export to Unity** (Ctrl+E) and choose this project folder (the one that contains `Assets`).
3. Back in Unity the prefab appears in `Assets/EditorGUI/Prefabs`. Drag it into a Canvas, or use **Tools > Lienzo UI > Import design into scene…**.
4. Edit the design in the app and export again: the prefab is rebuilt automatically.

## Folder layout

```
Assets/EditorGUI/
  Editor/        EditorGuiImporter.cs, LienzoUIWindow.cs
  Designs/       exported designs (JSON) used by the importer
  Sprites/       sprites of the sample designs
  Samples/       Lienzo UI projects to open in the app (*.editorgui.json)
  Documentation/ this guide, license and third-party notices
```

## How the generated code behaves

- **Prefab first generation.** The prefab is built *after* the new `*View` scripts have compiled: the designs waiting for it are kept in `Library/LienzoUI/pending.txt`, which survives the domain reload. So the first export ends with the View component attached and its references assigned. `Tools > Lienzo UI > Regenerate all prefabs` rebuilds everything and logs a summary; on opening Unity a prefab that is missing, outdated or without its View is rebuilt.
- **References, not paths.** `<Name>ViewBase` has public fields that the importer fills as serialized references, so renaming an element in Lienzo or in Unity does not break your code. `Bind()` only fills empty fields (old prefabs) by path.
- **Names.** An element's `unity_name` is the exact GameObject name; the importer's own helper objects are called `__border`, `__text`, `__tick`, `__canvas_background` and `__safe_area`.
- **Scripts folder / assembly.** Generated scripts go to `unity.scripts_dir` (default `Assets/EditorGUI/Scripts`). With `unity.assembly` set to an existing assembly definition the scripts are written under the folder of that `.asmdef` (or `.asmref`) so they compile into it (the asmdef needs to reference `Unity.TextMeshPro` and `UnityEngine.UI`); if no such assembly exists they stay in `scripts_dir` (Assembly-CSharp). The runtime helpers `LienzoSafeArea` and `LienzoOrientationLayout` go into a `Runtime` folder next to them, once per project.
- **Outputs.** Only what the design selects is written (prefab, UI Toolkit, IMGUI, view scripts). `*OnGUI.cs` exists only when IMGUI is on.
- **Canvas.** Render mode (Overlay / Camera / World Space), sorting order, plane distance, *embed* (no Canvas / CanvasScaler: a plain stretched RectTransform to live under your own canvas) and *safe area* (a `__safe_area` container with `LienzoSafeArea`). A transparent design page creates no background image.
- **Portrait.** If the design has a portrait layout, the prefab gets `LienzoOrientationLayout`, which stores both states (anchors, offsets, visibility, font size, layout) and applies the portrait one when `Screen.width < Screen.height` (or call `Apply(bool portrait)`). Changing a layout group's *type* between orientations is not applied at runtime.
- **Sprites.** `slice`, `ppu`, `image_mode`, `preserve_aspect` and `sprite` (a sub-sprite of a sheet, by name) are applied. A picture written as `unity:Assets/Path/file.png` (or `...#sub-sprite`) already lives in your project: it is referenced, never copied or overwritten, and its import settings are only changed where the design says so (9-slice border, pixels per unit, Point filter in pixel-art mode). Textures that Lienzo created are marked in their `userData` and may be re-configured. Smooth and pixel-art generated sprites use different files (`*_px.png`).
- **Fonts.** `default_font` / per-element fonts are resolved as existing TextMeshPro font assets by name (also `Fonts & Materials`); an SDF asset is created from `Assets/EditorGUI/Fonts/<name>.ttf` only when none exists. `fallback_fonts` are added to the font asset's fallback table when missing, and the Console lists characters (for example « » —) that the chosen font lacks.

## Support

Issues and questions: https://github.com/deritu-kutsut/lienzo-ui/issues · MIT license.
