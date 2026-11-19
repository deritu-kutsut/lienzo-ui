# Changelog

Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versions follow [SemVer](https://semver.org/).

## [Unreleased]

## [1.6.1] - 2026-11-19

Stability and security fixes found in a full audit of 1.6.0.

### Fixed
- **Saving and recovery:** Save only writes to the file the design is bound to; Save As is offered otherwise. Closing, opening and creating with unsaved changes now ask first. Autosave retries and warns when it cannot write; recovery files record the owning process so a second window never restores another window's work; 64 MB recovery files are handled.
- **Scripts:** deeply nested data returned by a script can no longer crash the app (scripts run on a large stack; results deeper than 64 levels are rejected). `on_click` of downloaded content is validated.
- **Review:** «Fix all» works in batches (up to 3000 fixes per click) and keeps the original name on the first duplicate.
- **Community:** an invalid session token is reported as an expired session; a request with `?error=` no longer cancels the sign-in; the busy state is always released.
- **Images:** decoding is limited to 8192 px, three images per pass, a 256 MB cache and 192 px thumbnails.
- **MCP:** designs without `next_id` are no longer corrupted and added elements never copy another element's `unity_name`.
- **Command line:** wrong usage exits with code 2 and prints to the console when started from a terminal.
- Files are written atomically with retries (antivirus / OneDrive locks); the built-in pack never overwrites a different file of the same name.

### Security
- Plugins are no longer loaded from the working directory; only the ones next to the executable and in the data folder.
- Manual JSON export no longer writes absolute paths; the reserved name `COM¹` is rejected.
- Server (migration 0016): profile dates and terms acceptance are stamped by the server; reports need an aged account and are limited to 10 per day; three young accounts can no longer hide an item; suspending an account hides its items; deleting an account no longer clears strikes or bans (hashed identity only, see [docs/PRIVACY.md](docs/PRIVACY.md)); copyright claims require accepted terms.

## [1.6.0] - 2026-11-19

More to build with, safer work, and clear rules for the community.

### Added
- **Built-in content pack** (Images tab): 108 ready-to-use images that ship inside the app, no download — 10 backgrounds, 8 frames and 8 buttons (nine-slice, ready to stretch), 27 icons and 55 pixel-art pieces (hearts, coins, tiles, buttons, panels). One click adds the image to the design; pixel art keeps crisp scaling and the right pixels-per-unit. See [docs/CONTENT.md](docs/CONTENT.md).
- **Two new templates** (15 in total), including a pixel-art RPG HUD.
- **Autosave and crash recovery:** every 60 seconds (when something changed) the design is saved to a recovery file; after a crash or power loss the app offers to restore it. Switch it off in the Accessibility window. Closing with unsaved changes keeps the recovery file.
- **Undo has a budget** (200 steps, 64 MB) so huge designs cannot exhaust memory; the oldest steps are dropped and the status bar says so once.
- **Review quick fixes:** a «Fix» button per problem and «Fix all safe issues» (text contrast, size, off-page elements, duplicate names…), each one an undo step; portrait problems are fixed as portrait overrides.
- **Community:** 14 new plugins (palette export, localization sheet, accessibility report, design spec…) and 18 free fonts; offline mode shows the last known list; thumbnails are cached; clear messages for offline, timeouts and expired sessions; read requests retry with backoff.
- **Terms and copyright:** creating a community profile now asks you to accept the Terms of Use and Community Guidelines; uploads declare a license, an optional source link and that you have the rights to share the file; items can be reported for copyright and creators can export their data. Texts: [docs/TERMS.md](docs/TERMS.md), [docs/COMMUNITY_GUIDELINES.md](docs/COMMUNITY_GUIDELINES.md), [docs/COPYRIGHT.md](docs/COPYRIGHT.md), [docs/PRIVACY.md](docs/PRIVACY.md).
- **Tutorials** (beginner and pro, all 10 languages) cover the content pack, vertical layouts, nine-slice, importing from Unity, recovery, review fixes and the community.

### Security
- The updater requires exactly one executable with a valid PE header and version resource before replacing anything.
- Recovery files cannot point at other files; community catalog cache is revalidated and bound to the server; undo memory is measured fully; plugin exporters neutralize spreadsheet formulas, comment and Markdown injection; the review no longer offers to remove images it cannot verify; links shown in community cards are strict http(s) and show the full host.
- Server: a user can no longer learn whether someone else is suspended or banned; hidden items edited while under a copyright claim are re-reviewed before coming back; copyright claims require an aged account with accepted terms and are rate-limited per creator; claimant contact details are masked for read-only moderation tokens.

## [1.5.1] - 2026-11-19

### Fixed
- The Portrait view now shows elements without a portrait override the way Unity will: they keep their landscape anchors and offsets and follow them when the page becomes portrait (a full-page background fills the portrait page, right-anchored elements stay at the right edge). Editing in portrait still creates an override only for the element you change.

## [1.5.0] - 2026-11-19

Bringing existing Unity interfaces in, and exporting them the way a real game needs. Also includes the 1.4.2 security fixes below.

### Security
- **Scripts and plugins:** `print`, `message` and plugin `log` output is capped (200 messages / 64 KB) and stops the script when exceeded; plugin scripts have lower limits (texts 64 KB, lists and maps 10,000 items, nested data counts) so a script can no longer exhaust memory with printing, nested copies or giant strings; the status bar text of a plugin command is bounded.
- **Plugins:** an exporter's `extension` must be one of `txt`, `csv`, `json`, `xml`, `md`, `cs`, `uxml`, `uss`, `tsv` (also for plugins on disk, and the Save dialog sanitizes it). Community plugin packages: every `.json` template or design-like file is validated even when it is also listed as a command script, file names that differ only in capitals are rejected, installs are atomic (temporary folder, then rename) and folder names can no longer collide (`community-<name>-<hash>`).
- **Plugin commands and community downloads are untrusted:** the design a plugin command returns is checked (new control characters, external links, absolute/UNC/drive image paths and `..` are rejected), and community templates and plugin templates are loaded as untrusted sources.
- **Sign-in:** the OAuth flow sends a random `state`, ignores callbacks that are not the answer to that login (a stray request no longer cancels the login), keeps working while connections are idle, and keeps PKCE.
- **Terminal output:** text from the server printed by the command line (`--community-review list`, tokens, errors...) has control characters stripped. Names, titles and plugin names with invisible or direction-changing characters (zero-width, left-to-right/right-to-left marks, BOM...) are rejected.
- **MCP server:** `null`, non-object messages and batches containing them return JSON-RPC errors instead of crashing (plus `unhandledRejection` handling); `add_element` and `duplicate_element` validate types and ranges like `modify_element`; `create_design` validates the title before creating anything; `review_design`, `screenshot` and `create_design` return an error when the app times out; `export_unity` resolves the project path first and refuses UNC paths; UNC image paths are rejected; `tools/call` without an id is refused; `protocolVersion` is negotiated and `serverInfo.version` is the app's version.
- **Windows build:** control-flow guard, CET shadow-stack compatibility and an application manifest (per-monitor-v2 DPI, long paths, Windows 10/11, no elevation).

### Changed
- `publicar.sh` checks the version, the changelog and the repository state, builds the package from `git ls-files`, writes forward-slash zip entries, refuses an executable that contains build-machine paths, adds `THIRD_PARTY_LICENSES.txt`, can be re-run after a failure and has a `--dry-run` mode. `Cargo.toml` now has `license`, `repository` and `description`.
- `docs/COMMUNITY.md` uses the current English names for functions, tables and flags.

### Added
- **Import from Unity** (File › Import from Unity…, or `--import-unity <file.prefab>` and `--import-unity-project <dir> --out <dir>`): turns uGUI prefabs and Lienzo UI exported designs into editable designs, keeping the GameObject names, anchors, layout groups, sprites (including sub-sprites), 9-slice borders and pixels per unit. See [docs/UNITY_IMPORT.md](docs/UNITY_IMPORT.md).
- **Documented file format** ([docs/FORMAT.md](docs/FORMAT.md) and a JSON schema) and `--validate <file>`. Files that lack optional fields such as `next_id` or `canvas_match` now load.
- **Portrait layout:** a second layout per design (Landscape / Portrait toggle). Edits made in portrait are stored as overrides; the export adds a `LienzoOrientationLayout` component that switches at runtime. The review also checks the portrait layout.
- **Sprites:** 9-slice borders, pixels per unit, sub-sprite, image mode and preserve aspect per image, drawn as in Unity on the canvas. Existing Unity sprites are referenced (`unity:Assets/...`), never copied or reset.
- **Unity name** per element: the exact GameObject name, independent of the editor label. Generated children are now `__border`, `__text`…, so they cannot collide with yours.
- **Canvas options:** render mode, sort order, safe area (`LienzoSafeArea`) and "embed" (no own Canvas) for overlays; a background with alpha 0 exports no background image.
- **Unity export options:** scripts folder, assembly (scripts are placed next to the asmdef you name), choose which outputs are generated (prefab, UI Toolkit, IMGUI, View scripts; IMGUI is off for new designs).
- **Fonts:** use the TMP font assets of your project by name, with fallback fonts; the review warns when a font lacks characters such as « » —.
- Generated View scripts bind elements through serialized references, so renaming elements no longer breaks your code.
- First export: prefabs are built automatically once the new scripts compile (for CI/batchmode: `-executeMethod EditorGui.EditorGuiImporter.ProcessPendingNow`).
- Re-exporting an unchanged design no longer touches any file, so Unity does not reimport or recompile anything. Two images that share a texture but ask for different pixels per unit now produce a warning.

## [1.4.2] - 2026-11-19

Security and stability release.

### Security
- Community templates can no longer smuggle code into the Unity scripts that Lienzo UI generates: Unicode line separators and control characters are escaped or rejected everywhere (names, texts, click scripts, titles).
- Generated `*View.cs` examples no longer break with unusual element names, and a re-export no longer creates a duplicate `*View` class when you moved the file.
- Click scripts and plugins run in a stricter sandbox: no imports, limits on memory and a time limit, so a script can no longer freeze the app or exhaust memory.
- Installing a community plugin always writes the validated manifest, validates the templates it ships, and rejects Windows alternate data streams, reserved device names and network paths.
- The published executable no longer contains build-machine paths.

### Fixed
- Undo after opening another project no longer brings back the previous design (and Save no longer overwrites the new one). Undo and Redo do nothing while Test mode is active.
- Files saved with a UTF-8 byte order mark (for example by PowerShell) open normally.
- High contrast with a light theme is readable.

## [1.4.1] - 2026-11-19

Small polish after 1.4.0.

### Fixed
- Spanish-style «guillemets» in English texts (tutorial and the comments of the generated Unity scripts) are now plain curly quotes.
- Old devlog and page images that pointed to the renamed `docs/images/` folder were repaired.

## [1.4.0] - 2026-11-19

English-first release: names, formats, server and documentation are in English; everything you already have keeps working.

### Changed
- **Everything is English-first.** Code, command-line flags, MCP tools and parameters, plugin manifests and Rhai function names, the `designs/` folder, sample designs, the Unity package (class, menu, sample, sprite and file names) and the documentation now use English names. Spanish stays available as a language of the interface, the tutorials and the written guides.
  - Command line: `--open`, `--save`, `--export`, `--template`, `--templates`, `--review`, `--language`, `--scale`, `--theme`, `--tab`, `--high-contrast`, `--no-tutorial`, `--test`, `--pro`…
  - MCP tools: `lienzo_info`, `list_templates`, `create_design`, `adjust_design`, `review_design`, `list_designs`, `read_design`, `modify_element`, `add_element`, `duplicate_element`, `delete_element`, `screenshot`, `export_unity`; parameters `file`, `template`, `title`, `background`, `element`, `changes`, `type`, `name`, `parent`, `text`, `new_name`, `unity_project`. Review lines use `[ERROR]` / `[WARNING]`. Screenshots go to `designs/screenshots/`.
  - Designs folder: `designs/` (was `disenos/`), environment variable `LIENZO_DESIGNS`; sample designs `settings`, `dialog`, `inventory`, `main_menu`, `shop`, `hud`, `hud_pixel` and images in `designs/pixel/` (`shield`, `sword`, `coin`, `potion`).
  - Plugins: `plugin.json` keys `name`, `author`, `description`, `commands`, `exporters`, `templates`; functions `run(design)` and `export_text(design)`; example plugins `snap-to-grid`, `text-to-csv`, `extra-templates`. Test-mode scripts: `show`, `hide`, `toggle`, `set_text`, `read_text`, `set_color`, `set_text_color`, `move_by`, `set_position`, `rotate`, `set_value`, `read_value`, `set_opacity`, `message` and `me`.
  - Unity package: `unity-package/build.py`, `store_images.py` and `source/`; window `Tools › Lienzo UI › Open Lienzo UI (app)`; samples and exported designs with English names, in sync with `designs/`.
  - Translation template: copy `assets/i18n/es.tsv` (the keys are the English texts).
  - Docs renamed (English names): `docs/MODERATION.md`, `docs/COMMUNITY.md`, `docs/EXTENDING.md`, `docs/TUTORIAL_BEGINNERS.es.md`, `docs/TUTORIAL_PROS.es.md`, `docs/RELEASE_NOTES_*.md` and `docs/images/` (screenshots renamed too). The old Spanish-named pages stay as one-line stubs that link to the new ones.

### Added
- The interface texts of all 10 languages are translated again against the new English source (266 new texts per language).
- Community server: English tables, functions and wire format (v2). App versions 1.0 - 1.3.3 keep working through a compatibility layer on the server.

### Compatibility
Nothing you already have stops working. The old Spanish command-line flags (`--abrir`, `--exportar`, `--guardar`, `--plantilla`, `--revisar`, `--comunidad-*`…), the old MCP tool names (`crear_diseno`, `captura`…, as hidden aliases) and parameter names (`archivo`, `elemento`, `cambios`…), the `LIENZO_DISENOS` variable and an existing `disenos/` folder, plugin manifests with Spanish keys and scripts with `ejecutar`/`exportar` or the Spanish test-mode functions (`mostrar`, `yo`…), and tutorial JSON files with Spanish keys are all still accepted. In the Unity package, renamed files keep the GUIDs of their old paths and the old `LienzoUIVentana` type remains as an `[Obsolete]` shim. See "Legacy names" in [docs/MCP.md](docs/MCP.md) and [docs/PLUGINS.md](docs/PLUGINS.md).

## [1.3.3] - 2026-11-18

Bug fixes: canvas, test mode, plugins and damaged files.

### Fixed
- Test mode: `mover`, `posicion` and `girar` also move the element's children; `valor` on a dropdown picks the option (before, it was limited to 0–1); `leer_texto` and `leer_valor` warn if the element does not exist; infinite numbers no longer leave broken coordinates.
- Double-clicking an element inside a Container no longer returns the selection to the Container on release.
- Alt+click without dragging no longer leaves a copy on top.
- Pasting (Ctrl+V) a Container or panel puts it next to itself instead of inside itself.
- Group and ungroup respect locked elements; ungrouping nested containers no longer leaves the selection holding deleted elements.
- Moving several elements forward or backward in a row with Ctrl+] / Ctrl+[ is no longer cancelled out, and "To back" keeps their order.
- Adding a component keeps its rotation and name (the "Combo" arrived straight and "Pill button" turned into "Button 12").
- Scaling a selection that includes locked elements no longer distorts the rest.
- Dropdown options can be typed with Enter in the inspector.
- Opening a project while editing a text no longer leaves the edit open over the new design.
- Plugins: a path like `\carpeta\script.rhai` can no longer escape the plugin folder on Windows.
- Uploading a font with the same file name as another no longer makes the old one get used after restarting.
- Damaged or third-party files: absurd font sizes, negative borders and radii, out-of-range opacity, infinite numbers, zero-size pages and control characters (which broke the exported UXML) are fixed.
- A design titled "CON", "NUL", "COM1"… is exported with a file name that is valid on Windows.

## [1.3.2] - 2026-11-18

Chinese, Japanese and Korean: the app now speaks ten languages.

### Added
- **Simplified Chinese, Japanese and Korean** in the interface, the built-in tutorial and the tutorials in `docs/` (10 languages in total). The app uses the fonts the system already provides (Microsoft YaHei, Yu Gothic, Malgun Gothic; on macOS and Linux, PingFang, Hiragino, Apple SD Gothic Neo or Noto Sans CJK) and only loads them when one of these languages is chosen; if it finds none, Accessibility warns about it.
- Written tutorials in `docs/` (beginners and pros) in Portuguese, French, German, Italian and Russian, with a line for switching language in each one.

## [1.3.1] - 2026-11-18

The interface and the tutorial, in seven languages.

### Added
- **5 new interface languages:** Portuguese, French, German, Italian and Russian (Accessibility › Language). The built-in tutorial is also translated into all five.
- English tutorials in `docs/` (`TUTORIAL_BEGINNERS.md` and `TUTORIAL_PROS.md`), and tutorials up to date with 1.3.
- `cargo test` validates every language: keys exist, no duplicates, `{}` placeholders and `\n` line breaks preserved, and at least 95% translated.

### Fixed
- Page sizes, test-mode screens, the shortcut list, the titles "UI Controls", "Containers and Layout Groups" and "Shapes", the element type, the default font and the inspector's component titles are now translated.
- Long thumbnail labels are shortened with "…" instead of overlapping the next one; the full name appears on hover.

## [1.3.0] - 2026-11-18

Seven new templates, ready-made components and styles, the **Review design** tool and fixes.

### Added
- **7 new templates:** Pause menu, Game over, Level select, Loading screen, Character dialogue, Leaderboard and Character select (13 in total). "Blank" is still number 5 and is shown last.
- **Review design** (**Review** button in the header, `--revisar` on the command line and `revisar_diseno` in the MCP): warns about text that is too small, low contrast (WCAG), controls too small to tap, elements outside the page, duplicate names that confuse `GameObject.Find`, and missing images. Clicking a problem selects the element.
- **Elements › Ready-made components:** pill button, outline button, close button, health and mana bars, card, translucent panel, badge and checkbox.
- **Text › For buttons and HUD:** timer, coins, damage, combo, level and dialogue subtitle.
- **Images:** 8 more gradients (Ocean, Desert, Neon, Ice, Volcano, Candy, Wood, Vignette).

### Fixed
- Templates applied to pages smaller than 1920×1080 (720p, 800×600, portrait) overflowed the page: they are now scaled to fit.
- Saving, saving as a template or exporting in test mode kept the changes made by the test clicks.
- Opening a project or a template in test mode and then leaving it brought back the previous design.
- **Distribute** moved locked elements.
- If an image was missing, the export left the Unity project half-written: this is now checked before anything is written.
- With the interface in English, element names, shapes, text styles, gradients and control types were still in Spanish. Search also matches by the translated name.
- More contrast on the orange and green buttons of the templates.

### Performance
- Template thumbnails are generated once (before, every frame) and library lists are no longer copied every frame.
- The visibility of elements when drawing the canvas is computed in a single pass (before, quadratic).

## [1.2.4] - 2026-11-17

English translation of buttons, tooltips and templates.

### Fixed
- With the interface in English, the tooltips of icon buttons (Undo, Redo, Duplicate, Delete, alignment…), the section titles (Interface templates, My templates, Text styles…), the Backward, Forward, To back, To front, Distribute, Group and Ungroup buttons, and the template names were still in Spanish. They are now translated.

## [1.2.3] - 2026-11-17

Fully English interface and limits in the MCP server.

### Fixed
- With the interface in English, the status bar and several inspector texts (hierarchy, Layout Group, "Parent") were still in Spanish. They are now translated.
- MCP server: an element named "3" no longer loses to the element with id 3 (the exact name takes priority).
- MCP server: size limits (name 120 characters, text 5000, lists of 200 elements). A design with 5 MB of text took 32 s to open.

## [1.2.2] - 2026-11-17

Robustness against malformed designs.

### Fixed
- A design with parents in a cycle (or an element that is its own parent) no longer freezes the app when selecting, deleting or duplicating.
- Exporting a design with negative or non-numeric width or height no longer closes the app.
- Duplicate ids and a `next_id` that is too low are fixed when the file is opened; non-numeric coordinates become 0.
- Very long titles are shortened to 64 characters in exported file and class names.

## [1.2.1] - 2026-11-17

Updater fix.

### Fixed
- After updating, **Restart now** relaunched the previous version: it now opens the new one.
- Release notes are shown as plain text in the update window.

## [1.2.0] - 2026-11-17

In-app updates.

### Added
- **In-app updates**: on startup, Lienzo UI checks the latest GitHub release (only `github.com`, sending no data) and, if there is a new version, the **Update to X.Y.Z** button appears in the header. It works the same whether you downloaded the app from GitHub or from itch.io. It downloads the zip, checks its SHA-256 fingerprint, replaces the `.exe` (the previous one stays as `.viejo` until the next launch) and offers to restart. If something cannot be verified, it opens the download page. You can turn off the startup check, skip a version or check manually from **File → Check for updates…**.

### Fixed
- MCP server: validates ranges (`w`, `h`, `font_size`, `opacity`…), empty or duplicate names when renaming, corrupt designs and reserved Windows file names.

## [1.1.1] - 2026-11-17

Automatic moderation and maximized startup.

### Added
- **Automatic moderation**: everything that is uploaded (profile pictures and banners too) is checked automatically; clean items are published and doubtful ones stay pending for a person. `reanalizar` re-runs the pending ones.
- The window starts **maximized**.
- Free **Unity package** (`LienzoUI-for-Unity`): importer, launcher window and 7 sample screens, submitted to the Asset Store.

## [1.1.0] - 2026-11-17

Community, creator profiles and themes.

### Added
- **Community**: a tab to browse, install, upload and report **plugins, images, fonts, templates, elements and themes**. Installs with one click after verifying the SHA-256 fingerprint and re-validating the file. Browsing does not need an account.
- **Accounts with GitHub or Google** (no passwords) and **creator profile**: photo, banner, bio, GitHub and social links (X, YouTube, itch.io, Twitch, Bluesky, Instagram). X-style profile editor; photos are cropped and reduced automatically and show up instantly while they are being reviewed. Profile button with your photo in the header.
- **Follow creators** and **hearts** (like GitHub's star), with a "Following" filter. Who follows whom is not public; only the totals.
- **Upload content** with an optional preview (enlarges with one click) and one SVG icon per type. Every upload goes through quarantine, format validation (data formats only), antivirus and text moderation; doubtful items wait for human review.
- **18+ filter** enforced on the server: only those who confirm they are over 18 can see it.
- **Themes**: **Themes** button to change colors, corners and font of the whole app. Five built-in themes (Dark canvas, Light canvas, Midnight, Sunset, Forest), custom themes that can be shared in the community.
- **Moderation web** (`web/moderacion`): panel to review pending items by category, with access for people (account) and for scripts (revocable tokens with scopes and expiry). The `--comunidad-revisar` command also approves profile photos and manages tokens.
- Own SVG icons instead of emojis; `Lienzo UI.exe` executable.
- Server in `supabase/` (schema with RLS, moderation functions) and the guides `docs/COMMUNITY.md` and `docs/MODERATION.md`.

### Changed
- Interface colors are read from the theme in use (before, they were constants).

### Security
- No passwords or emails in the app; session stored in the Windows Credential Manager; the app does not connect until you open the Community tab.

## [1.0.0] - 2026-11-16

First public release of **Lienzo UI** (formerly "Editor GUI").

### Added
- **Pixel art**: images without smoothing and whole pixels in the editor; Canvas Pixel Perfect and uncompressed Point sprites in Unity.
- **Plugins**: commands, exporters and templates with isolated Rhai scripts; Plugins tab and three sample plugins.
- **Accessibility**: interface in Spanish or English, size up to 200% (Ctrl+Alt+= / -), high contrast and reduced motion. Complete tutorial in English.
- Name, SVG icon and cover.
- Built-in tutorial for beginners (5 steps) and pros (9 steps): it opens the first time; when you finish it the editor opens; **Tutorial** button and F1 to reopen it.
- Command line: `--plantillas`, `--plantilla`, `--abrir`, `--guardar`, `--exportar` (no window), `--tutorial`, `--sin-tutorial`.
- MCP server (`mcp/lienzo-ui-mcp.mjs`) with 12 tools, queued calls and `mcp/probar-mcp.mjs` (32 checks).
- Documentation: tutorials, extension guide, MCP and contributing. MIT license.

### Tested
- The 5 templates and two designs created via MCP ("Blacksmith's shop" and "Pixel art HUD") exported to Unity 6000.3 (URP): uGUI, UI Toolkit and IMGUI prefabs without errors; pixel art mode leaves the Canvas as Pixel Perfect and the sprites as Point.

### Fixed (found while testing in Unity)
- Without the TMP Essential Resources, texts were left without a font and failed in RectMask2D: the importer now detects this, opens TMP Importer and regenerates the prefabs.
- Titles that wrapped in uGUI (wider TMP font): texts are shrunk until they fit.
- UI Toolkit grid with one column too few because of spacing.
