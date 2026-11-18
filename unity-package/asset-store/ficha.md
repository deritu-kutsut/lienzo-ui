# Asset Store listing · Lienzo UI for Unity

Everything to paste into the Publisher Portal (https://publisher.unity.com/). Price: **free (Free)**.

## Basic data

| Field | Value |
|---|---|
| **Package name** | Lienzo UI for Unity |
| **Category** | Editor Extensions › Utilities (alternative: Tools › GUI) |
| **Price** | Free |
| **Version** | 1.1.0 |
| **Publisher** | Deritu Kutsut |
| **Support email / website** | the ones from your publisher profile |
| **Unity version (minimum)** | 6000.0 |
| **Tested on** | 6000.3.25f1 (URP) |
| **Render pipelines** | Built-in, URP, HDRP (standard uGUI) |
| **Package type** | Assets (folder `Assets/EditorGUI`) |
| **Keywords (max. 15)** | ui, gui, ugui, textmeshpro, ui toolkit, imgui, editor, tool, canvas, layout, hud, menu, pixel art, designer, export |

## Short description (max. 200 characters)

Free visual editor for game UI: design a screen once and export it to Unity as uGUI + TextMeshPro, UI Toolkit and IMGUI. Includes the importer, a launcher window and 7 sample screens.

## Description

**Lienzo UI is a free, open source visual editor for game interfaces.** Lay out menus, HUDs, dialogs, settings and inventories on a canvas like in a design tool, then export them to Unity with one click.

**This package is the Unity side of Lienzo UI**: the importer that turns exported designs into ready prefabs, a window that launches the app, and seven complete sample screens. The editor itself is a separate free desktop app (Windows) that you download from GitHub or itch.io; this package works together with it.

**What you get**
- Importer: rebuilds a uGUI + TextMeshPro prefab automatically every time you export from the app (Tools > Lienzo UI).
- Launcher window: opens the app, shows your project path, lists the sample projects and opens them in the app.
- 7 sample screens: main menu, game HUD, pixel-art HUD, dialog, settings, inventory and shop (exported designs, sprites and Lienzo UI projects).
- Pixel-art mode: Pixel Perfect canvas and point-filtered sprites.
- Documentation in English.

**What the app does**
- Design-tool editing: handles, smart guides, groups, layers, undo/redo.
- Real Unity concepts: RectTransform anchors and pivots, Canvas Scaler, layout groups, ScrollView.
- One design, three outputs: uGUI + TextMeshPro, UI Toolkit (UXML/USS) and IMGUI.
- Plugins and themes, an in-app tutorial, an MCP server so any AI can edit designs, and a community to share templates, fonts, images and plugins.

**Requirements**: Unity 6 (6000.0+), Unity UI (uGUI) and TextMeshPro with its Essential Resources, and the Lienzo UI desktop app (Windows; on macOS/Linux build it from source). MIT license. No executables are included in this package.

Links: https://github.com/deritu-kutsut/lienzo-ui · https://deritu.itch.io/lienzo-ui

## Key features (5-8 short lines)

- Free and open source (MIT)
- Visual canvas editor with real Unity concepts
- Exports to uGUI + TextMeshPro, UI Toolkit and IMGUI
- Automatic prefab rebuild when you re-export
- Pixel-art mode
- 7 complete sample screens
- Community: share and install templates, fonts, images and plugins
- Spanish and English

## Release notes (v1.1.0)

First Asset Store release: importer, launcher window, 7 sample screens, English docs.

## Images (in this folder)

| File | For |
|---|---|
| `icon_160x160.png` | Package icon (160×160) |
| `card_420x280.png` | Card image (420×280) |
| `cover_1950x1300.png` | Cover image (1950×1300) |
| `screenshot_*_1920x1080.png` | Screenshots (editor, Unity, community, profile, themes, tutorial) |

## How to upload it (you do this yourself, because it requires your account and accepting its terms)

1. In the **Publisher Portal** create a new package ("Create package"), fill it in with this listing and upload the images.
2. In Unity, install **Asset Store Tools** (Package Manager › My Assets › "Asset Store Publishing Tools"), drag `dist/LienzoUI-for-Unity-1.1.0.unitypackage` into an empty project (or open `unity-package/fuente`) and use **Asset Store Tools › Package Upload**: pick the `Assets/EditorGUI` folder, sign in and upload.
3. Go back to the portal, check the validation and click **Submit for approval**. Unity's review takes a few days.

Note: the Asset Store requires the package to work on its own and any external software it needs to be declared; this is already explained in the description and in the package README.
