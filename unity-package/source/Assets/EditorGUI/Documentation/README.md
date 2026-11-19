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

## Support

Issues and questions: https://github.com/deritu-kutsut/lienzo-ui/issues · MIT license.
