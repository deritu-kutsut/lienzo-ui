# Beginner tutorial

🌐 [English](TUTORIAL_BEGINNERS.md) · [Español](TUTORIAL_BEGINNERS.es.md) · [Português](TUTORIAL_BEGINNERS.pt.md) · [Français](TUTORIAL_BEGINNERS.fr.md) · [Deutsch](TUTORIAL_BEGINNERS.de.md) · [Italiano](TUTORIAL_BEGINNERS.it.md) · [Русский](TUTORIAL_BEGINNERS.ru.md) · [简体中文](TUTORIAL_BEGINNERS.zh.md) · [日本語](TUTORIAL_BEGINNERS.ja.md) · [한국어](TUTORIAL_BEGINNERS.ko.md)

Your first interface with Lienzo UI and your first export to Unity. No programming needed.

## 1. Install and open

1. Download `LienzoUI-windows.zip` from the GitHub releases or from itch.io.
2. Unzip it and open `lienzo-ui.exe`.
3. The first time, the **tutorial** opens (this same content, inside the app). When you finish it, or with «Skip and go to editor», the editor opens. To see it again: the **Tutorial** button in the top bar or **F1**.

To use the app in English: **Accessibility** button › **Language** › English.

![Tutorial inside the app](images/tutorial.png)

## 2. The screen

| Area | What is there |
|---|---|
| Top bar | File, Edit, View, Canvas, **Test**, design name, **Accessibility**, **Review**, **Themes**, **Tutorial**, **Save**, **Code…**, **Export to Unity** |
| Left bar | **Hierarchy** (layers), **Elements**, **Templates**, **Text**, **Images**, **Plugins**, **Community** |
| Center | The canvas: your game screen (1920 × 1080 by default) |
| Right | The **Inspector**: properties of the selection |

![Editor](images/editor.png)

## 3. Start from a template

In **Templates** pick one of the 15 layouts: Main menu, Game HUD, Dialog window, Settings, Inventory, Pause menu, Game over, Level select, Loading screen, Character dialogue, Leaderboard or Character select. Everything in it can be changed, and Ctrl+Z undoes the change. To start from scratch: «Blank». Two of them are in pixel art style: Pixel RPG HUD and Pixel main menu.

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
3. Drag it into the scene, or use the menu **Tools › Lienzo UI › Import design into scene…**.

Every time you export again, Unity updates the prefab by itself.

## Built-in images

The Images tab in the left bar includes ready-made art, no files needed:

- The chips at the top filter by Your images, Backgrounds, Frames, Buttons, Icons, Pixel art and Gradients. The search box finds items by name or tag.
- Click an item to add it, or drag it onto the canvas. Frames and buttons come with their 9-slice border, so they stretch without distorting the corners. Backgrounds go behind everything else.
- The picture is copied to your library the first time you use it, and it is exported to Unity with the design.
- With the pixel art sprites, tick “Pixel art” in the canvas Inspector (nothing selected) so they stay crisp.

## Vertical (portrait) screens

Phones are often held upright. Lienzo UI keeps a second layout for that, without touching the first one:

- Switch the view with the Landscape / Portrait buttons in the top bar.
- With nothing selected, open “Portrait layout” in the Inspector and press “Create portrait layout”. Elements start where they are in landscape.
- In Portrait view, whatever you move or resize changes only the portrait layout. Right-click an element › “Reset to landscape” to undo one change.
- The export carries both layouts: in Unity the prefab switches to the vertical one by itself when the screen is taller than it is wide.

## Import from Unity

Already have an interface in Unity? Bring it into Lienzo UI:

- File › Import from Unity…
- Choose a uGUI prefab (.prefab) or a design that Lienzo UI exported before (Assets/EditorGUI/Designs/….json).
- It opens as a new, unsaved design: save it as a project to keep it.
- If something could not be converted exactly, a window lists the warnings, so nothing is lost silently.
- If it finds your Unity project folder, the images of the design are found automatically.

## Never lose your work

- Autosave: about every minute Lienzo UI saves a recovery copy. You can turn it off in Accessibility › Autosave.
- If the app closes unexpectedly, the next time it asks “Recover unsaved work…?”. Press Restore to get it back or Discard to drop it. Recovered work is not a saved file yet: press Ctrl+S to keep it.
- Ctrl+S still saves your project as usual.
- Undo (Ctrl+Z) remembers up to 200 steps (and a memory limit). If the oldest ones are dropped, the status bar says “History trimmed”.

## Community

The Community tab in the left bar is a shared catalog of plugins, images, fonts, templates, elements and themes:

- Browsing and installing needs no account: search and press Install.
- To upload, report or give hearts you sign in with GitHub or Google and choose a public name. No passwords: the app only keeps that name.
- When you create your profile you accept the rules: no malware, hate, plagiarism or unmarked 18+ content. The full text is in docs/TERMS.md and docs/COMMUNITY_GUIDELINES.md.
- 18+ content is hidden unless you sign in and turn on “Show 18+ content”.
- Everything shared is reviewed before it is published, and you can report anything that breaks the rules.

## Next steps

- [Pro tutorial](TUTORIAL_PROS.md): what the export generates and where to put your logic.
- [MCP server](MCP.md): ask an AI for changes in plain language.
