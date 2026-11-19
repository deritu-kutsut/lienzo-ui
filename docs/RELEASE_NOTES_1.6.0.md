# Lienzo UI 1.6.0

More to build with, safer work, and clear rules for the community.

## Added
- **Built-in content pack** (Images tab): 108 ready-to-use images that ship inside the app, no download — 10 backgrounds, 8 frames and 8 buttons (nine-slice, ready to stretch), 27 icons and 55 pixel-art pieces (hearts, coins, tiles, buttons, panels). One click adds the image to the design; pixel art keeps crisp scaling and the right pixels-per-unit. See [docs/CONTENT.md](docs/CONTENT.md).
- **Two new templates** (15 in total), including a pixel-art RPG HUD.
- **Autosave and crash recovery:** every 60 seconds (when something changed) the design is saved to a recovery file; after a crash or power loss the app offers to restore it. Switch it off in the Accessibility window. Closing with unsaved changes keeps the recovery file.
- **Undo has a budget** (200 steps, 64 MB) so huge designs cannot exhaust memory; the oldest steps are dropped and the status bar says so once.
- **Review quick fixes:** a «Fix» button per problem and «Fix all safe issues» (text contrast, size, off-page elements, duplicate names…), each one an undo step; portrait problems are fixed as portrait overrides.
- **Community:** 14 new plugins (palette export, localization sheet, accessibility report, design spec…) and 18 free fonts; offline mode shows the last known list; thumbnails are cached; clear messages for offline, timeouts and expired sessions; read requests retry with backoff.
- **Terms and copyright:** creating a community profile now asks you to accept the Terms of Use and Community Guidelines; uploads declare a license, an optional source link and that you have the rights to share the file; items can be reported for copyright and creators can export their data. Texts: [docs/TERMS.md](docs/TERMS.md), [docs/COMMUNITY_GUIDELINES.md](docs/COMMUNITY_GUIDELINES.md), [docs/COPYRIGHT.md](docs/COPYRIGHT.md), [docs/PRIVACY.md](docs/PRIVACY.md).
- **Tutorials** (beginner and pro, all 10 languages) cover the content pack, vertical layouts, nine-slice, importing from Unity, recovery, review fixes and the community.

## Security
- The updater requires exactly one executable with a valid PE header and version resource before replacing anything.
- Recovery files cannot point at other files; community catalog cache is revalidated and bound to the server; undo memory is measured fully; plugin exporters neutralize spreadsheet formulas, comment and Markdown injection; the review no longer offers to remove images it cannot verify; links shown in community cards are strict http(s) and show the full host.
- Server: a user can no longer learn whether someone else is suspended or banned; hidden items edited while under a copyright claim are re-reviewed before coming back; copyright claims require an aged account with accepted terms and are rate-limited per creator; claimant contact details are masked for read-only moderation tokens.

