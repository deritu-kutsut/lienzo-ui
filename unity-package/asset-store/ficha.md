# Ficha de la Asset Store · Lienzo UI for Unity

Todo lo que se pega en el Publisher Portal (https://publisher.unity.com/). Precio: **gratis (Free)**.

## Datos básicos

| Campo | Valor |
|---|---|
| **Package name** | Lienzo UI for Unity |
| **Category** | Editor Extensions › Utilities (alternativa: Tools › GUI) |
| **Price** | Free |
| **Version** | 1.1.0 |
| **Publisher** | Deritu Kutsut |
| **Support email / website** | los de tu perfil de publisher |
| **Unity version (minimum)** | 6000.0 |
| **Tested on** | 6000.3.25f1 (URP) |
| **Render pipelines** | Built-in, URP, HDRP (uGUI normal) |
| **Package type** | Assets (carpeta `Assets/EditorGUI`) |
| **Keywords (máx. 15)** | ui, gui, ugui, textmeshpro, ui toolkit, imgui, editor, tool, canvas, layout, hud, menu, pixel art, designer, export |

## Short description (máx. 200 caracteres)

Free visual editor for game UI: design a screen once and export it to Unity as uGUI + TextMeshPro, UI Toolkit and IMGUI. Includes the importer, a launcher window and 7 sample screens.

## Description

**Lienzo UI is a free, open source visual editor for game interfaces.** Lay out menus, HUDs, dialogs, settings and inventories on a canvas like in a design tool, then export them to Unity with one click.

**This package is the Unity side of Lienzo UI**: the importer that turns exported designs into ready prefabs, a window that launches the app, and seven complete sample screens. The editor itself is a separate free desktop app (Windows) that you download from GitHub or itch.io; this package works together with it.

**What you get**
- Importer: rebuilds a uGUI + TextMeshPro prefab automatically every time you export from the app (Tools > Lienzo UI).
- Launcher window: opens the app, shows your project path, lists the sample projects and opens them in the app.
- 7 sample screens: main menu, game HUD, pixel-art HUD, dialog, settings, inventory and shop (exported designs, sprites and Lienzo UI projects).
- Pixel-art mode: Pixel Perfect canvas and point-filtered sprites.
- Documentation in English and Spanish.

**What the app does**
- Design-tool editing: handles, smart guides, groups, layers, undo/redo.
- Real Unity concepts: RectTransform anchors and pivots, Canvas Scaler, layout groups, ScrollView.
- One design, three outputs: uGUI + TextMeshPro, UI Toolkit (UXML/USS) and IMGUI.
- Plugins and themes, an in-app tutorial, an MCP server so any AI can edit designs, and a community to share templates, fonts, images and plugins.

**Requirements**: Unity 6 (6000.0+), Unity UI (uGUI) and TextMeshPro with its Essential Resources, and the Lienzo UI desktop app (Windows; on macOS/Linux build it from source). MIT license. No executables are included in this package.

Links: https://github.com/deritu-kutsut/lienzo-ui · https://deritu.itch.io/lienzo-ui

## Key features (5-8 líneas cortas)

- Free and open source (MIT)
- Visual canvas editor with real Unity concepts
- Exports to uGUI + TextMeshPro, UI Toolkit and IMGUI
- Automatic prefab rebuild when you re-export
- Pixel-art mode
- 7 complete sample screens
- Community: share and install templates, fonts, images and plugins
- Spanish and English

## Release notes (v1.1.0)

First Asset Store release: importer, launcher window, 7 sample screens, English/Spanish docs.

## Imágenes (en esta carpeta)

| Archivo | Para |
|---|---|
| `icon_160x160.png` | Package icon (160×160) |
| `card_420x280.png` | Card image (420×280) |
| `cover_1950x1300.png` | Cover image (1950×1300) |
| `screenshot_*_1920x1080.png` | Screenshots (editor, Unity, comunidad, perfil, temas, tutorial) |

## Cómo subirlo (lo haces tú, porque exige tu cuenta y aceptar sus términos)

1. En el **Publisher Portal** crea un paquete nuevo («Create package»), rellénalo con esta ficha y sube las imágenes.
2. En Unity, instala **Asset Store Tools** (Package Manager › My Assets › «Asset Store Publishing Tools»), arrastra `dist/LienzoUI-for-Unity-1.1.0.unitypackage` a un proyecto vacío (o abre `unity-package/fuente`) y usa **Asset Store Tools › Package Upload**: elige la carpeta `Assets/EditorGUI`, inicia sesión y sube.
3. Vuelve al portal, comprueba la validación y pulsa **Submit for approval**. La revisión de Unity tarda unos días.

Nota: la Asset Store pide que el paquete funcione por sí solo y que se declare el software externo que necesita; ya está explicado en la descripción y en el README del paquete.
