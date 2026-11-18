# Beginner tutorial

🌐 [English](TUTORIAL_BEGINNERS.md) · [Español](TUTORIAL_NOVATOS.md) · [Português](TUTORIAL_BEGINNERS.pt.md) · [Français](TUTORIAL_BEGINNERS.fr.md) · [Deutsch](TUTORIAL_BEGINNERS.de.md) · [Italiano](TUTORIAL_BEGINNERS.it.md) · [Русский](TUTORIAL_BEGINNERS.ru.md)

Your first interface with Lienzo UI and your first export to Unity. No programming needed.

## 1. Install and open

1. Download `LienzoUI-windows.zip` from the GitHub releases or from itch.io.
2. Unzip it and open `lienzo-ui.exe`.
3. The first time, the **tutorial** opens (this same content, inside the app). When you finish it, or with «Skip and go to editor», the editor opens. To see it again: the **Tutorial** button in the top bar or **F1**.

To use the app in English: **Accessibility** button › **Language** › English.

![Tutorial inside the app](imagenes/tutorial.png)

## 2. The screen

| Area | What is there |
|---|---|
| Top bar | File, Edit, View, Canvas, **Test**, design name, **Accessibility**, **Review**, **Themes**, **Tutorial**, **Save**, **Code…**, **Export to Unity** |
| Left bar | **Hierarchy** (layers), **Elements**, **Templates**, **Text**, **Images**, **Plugins**, **Community** |
| Center | The canvas: your game screen (1920 × 1080 by default) |
| Right | The **Inspector**: properties of the selection |

![Editor](imagenes/editor.png)

## 3. Start from a template

In **Templates** pick one of the 12 layouts: Main menu, Game HUD, Dialog window, Settings, Inventory, Pause menu, Game over, Level select, Loading screen, Character dialogue, Leaderboard or Character select. Everything in it can be changed, and Ctrl+Z undoes the change. To start from scratch: «Blank».

## 4. Edit

- Drag elements from **Elements** onto the canvas. **Ready-made components** has buttons, health and mana bars, cards, badges and inventory slots.
- **Text** has heading styles and HUD pieces: score, timer, coins, damage numbers, combo, level tag and subtitles.
- Click selects; drag moves; corners resize; the top handle rotates.
- Double-click a text to type.
- In the **Inspector** change colors, borders, shadows, fonts and anchors.

| Shortcut | Action |
|---|---|
| Ctrl+Z / Ctrl+Y | Undo / redo |
| Ctrl+D / Del | Duplicate / delete |
| Ctrl+G / Ctrl+Shift+G | Group / ungroup |
| Arrows (Shift) | Move 1 px (10 px) |
| Ctrl+S | Save |
| F5 | Test |
| Ctrl+E | Export to Unity |
| F1 | Tutorial |

## 5. Test

Press **Test** (F5): the interface becomes interactive and you can pick other screens (phone, 4:3, ultrawide) to see how it adapts. Esc leaves test mode, and the design goes back to how it was before the test.

## 6. Review

Press **Review** in the top bar. It warns about text too small to read, low contrast, buttons too small to tap, elements outside the page, repeated names and missing images. Click a warning to select that element and fix it.

## 7. Export to Unity

1. Press **Export to Unity** (Ctrl+E). The first time, choose your project folder (the one that contains `Assets`).
2. Go back to Unity: the design appears as a prefab in `Assets/EditorGUI/Prefabs`.
3. Drag it into the scene, or use the menu **Tools › Lienzo UI › Importar diseño en la escena** (the Unity menu is in Spanish: «Import design into the scene»).

Every time you export again, Unity updates the prefab by itself.

## Next steps

- [Pro tutorial](TUTORIAL_PROS.md): what the export generates and where to put your logic.
- [MCP server](MCP.md): ask an AI for changes in plain language.
