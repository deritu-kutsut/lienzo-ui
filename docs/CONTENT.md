# Built-in content (asset pack and presets)

## Asset pack

`assets/pack/manifest.json` is an array of items; every file lives in `assets/pack/<category>/`.
`build.rs` embeds each listed file (`include_bytes!`, table `PACK_FILES`); `src/pack.rs` reads the manifest.

Item fields: `id`, `category`, `name` (English display), `file`, `tags`, `pixel` (bool), optional `slice` `[l,b,r,t]`
(source-image pixels), `ppu`, `native` `[w,h]` (pixel art: size before scaling).

Using an item copies its file to the library images folder as `pack_<id>.<ext>` (skipped when identical) and creates an
Image element (`slice`/`ppu` from the manifest). Designs may also refer to `pack_<id>.<ext>` directly: the file is
extracted the first time the image is resolved (rendering, review, export). `pack_*` files are hidden from "Your images".
The exporter copies them to the Unity Sprites like any library image.

To replace placeholder art keep the same file name, size and format (the tests check them):
backgrounds `*.jpg` 1280x720; frames `*.png` 512x512 (slice 96, ppu 100); buttons `*.png` 512x192 (slice 64/48/64/48);
icons `*.png` 256x256. Ids: `bg_<name>`, `frame_<name>`, `btn_<name>`, `icon_<name>`, `px_<name>`.

## Pixel art

Sprites are authored at 16x16 (or 32x32) and stored scaled with nearest neighbour: x8 for sprites (128/256 px), x4 for
9-slice UI (panels 48x48 -> 192, buttons 48x16, bars 64x8, slot 24, dialog 64x32). `ppu` is chosen so that **one native pixel
is 8 canvas units** with a reference of 100 pixels per unit: sprites `ppu 100` (8 source px per native px), sliced UI
`ppu 50` (4 source px per native px, borders are drawn twice as large). When inserted, pixel items get
`source size * 100 / ppu` (heart 128, stone panel 384). The design's `pixel_art` switch is never changed automatically; turn
it on (Page settings) for Point filtering. Palette: Sweetie-16 plus three browns and a cream.

## Presets

`src/presets.rs`: Buttons (12), HUD (18) and More components (12) for the Elements tab, plus 9 extra text styles in the
Text tab. Each is one element or a tree built from the existing kinds; they sanitize, pass the review without errors and export.
Templates 13 "Pixel RPG HUD" and 14 "Pixel main menu" use the pixel pack.
