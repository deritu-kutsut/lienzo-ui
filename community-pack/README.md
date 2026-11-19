# Lienzo UI community pack

Content prepared for the Lienzo UI community (Community tab): the bundled plugins of `plugins/` and free fonts from Google Fonts.

```
community-pack/
  fonts.json            metadata of every font (family, authors, license, source, sha256, size, labels)
  fonts/<family>/       the .ttf and its license text (OFL.txt or LICENSE.txt), exactly as published by google/fonts
  tests/                scratch test for the plugins and fonts (see "Tests")
  publish.ps1           uploads everything with the app's command line
```

## Plugins (14 new)

| Folder | What it does |
|---|---|
| `rename-by-hierarchy` | Names elements Parent_Child, fixes duplicated names |
| `recolor-palette` | Recolors the design with Ocean, Sunset, Forest or Mono palettes, or inverts it |
| `placeholder-text` | Fills empty texts with lorem ipsum that fits the box, and removes it again |
| `grid-generator` | Creates 3x3, 6x4 inventory, 9-slot hotbar and 3x2 card grids with a Grid Layout Group |
| `mirror-layout` | Mirrors the layout horizontally (RTL languages) or vertically |
| `distribute-siblings` | Spreads sibling elements evenly or stacks them with a fixed gap |
| `size-normalizer` | Makes buttons or siblings the same size, or rounds sizes to even numbers |
| `safe-area-guides` | Adds and removes notch / title-safe guides |
| `button-set-generator` | Main menu, OK / Cancel, Yes / No / Cancel and tab bar buttons |
| `text-case-tools` | UPPERCASE, lowercase, Title Case, Sentence case |
| `accessibility-report` | Exporter (.md): contrast, tiny text, touch targets, overflow |
| `design-spec-markdown` | Exporter (.md): element tree with sizes, colors and texts |
| `localization-sheet` | Exporters (.tsv, .json): texts with stable keys for translators |
| `palette-export` | Exporters (.md, .uss): colors in use as a table or USS variables |

Plugins are MIT licensed like the rest of the repository. See [docs/PLUGINS.md](../docs/PLUGINS.md) for the API.

## Fonts (18)

All are static `.ttf` files of at most 600 KB, taken unmodified from [github.com/google/fonts](https://github.com/google/fonts).
Seventeen are under the SIL Open Font License 1.1 and one (Luckiest Guy) under the Apache License 2.0. Both allow free personal and commercial use,
including embedding in games and apps; the OFL requires shipping the license with the font, which is why each folder contains it.
Fonts are not sold on their own and keep their original names.

See `fonts.json` for the family, authors, copyright line, source URL, SHA-256 and size of each one.

## Publishing

1. Build the release app if needed: `cargo build --release`.
2. Open Lienzo UI once, go to the Community tab and sign in (the command line reuses that saved session). Close the app afterwards: the token rotates.
3. Preview the commands without sending anything:

```powershell
powershell -NoProfile -File community-pack\publish.ps1 -WhatIf
```

4. Publish:

```powershell
powershell -NoProfile -File community-pack\publish.ps1                      # plugins + fonts, 20 s between uploads
powershell -NoProfile -File community-pack\publish.ps1 -Only plugins
powershell -NoProfile -File community-pack\publish.ps1 -Only fonts
powershell -NoProfile -File community-pack\publish.ps1 -Only fonts -Filter "press*"   # a single item
powershell -NoProfile -File community-pack\publish.ps1 -DelaySeconds 40     # slower, if the server rate-limits
powershell -NoProfile -File community-pack\publish.ps1 -IncludeBundled      # also the 3 older example plugins
```

Each upload is validated locally, then moderated by the server. An item can end approved, pending (waiting for a person) or rejected;
the message printed for every upload says which, and everything is written to `publish-log.txt` (ignored by git).
`lienzo-ui.exe --community-retry` re-moderates items left pending without a decision.

Equivalent single command:

```powershell
target\release\lienzo-ui.exe --community-upload font community-pack\fonts\press-start-2p\PressStart2P-Regular.ttf --title "Press Start 2P" --description "..." --labels font,pixel
target\release\lienzo-ui.exe --community-upload plugin plugins\grid-generator --title "Grid generator" --description "..." --labels grid,inventory
```

## Tests

`tests/plugin_pack_tests.rs` runs every plugin command and exporter through the real plugin engine over all `designs/*.editorgui.json` and all built-in templates,
checks that the resulting design is still valid (unique ids, parents, schema), packs each plugin like the community upload does and validates it with
`community_validate`, and validates every font with the app's validator and lays text out with it in egui. To run it, copy it to `src/plugin_pack_tests.rs`,
add `#[cfg(test)] mod plugin_pack_tests;` to `src/main.rs`, run `cargo test plugin_pack_tests` and undo both changes.

Note: a plugin receives the whole design as a script value, and the engine limits arrays and maps to 10,000 items (nested data counts), so designs of roughly
more than 200 elements are refused by every plugin (the bundled `snap-to-grid` too); nothing is applied in that case.
