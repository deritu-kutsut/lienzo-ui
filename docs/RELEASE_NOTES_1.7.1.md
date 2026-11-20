# Lienzo UI 1.7.1

Portrait layouts for every template, and Translate also covers the plugin preview.

## Fixed
- **Portrait button on templates:** every template now carries its own portrait layout (1080 x 1920), stored as overrides of the landscape one. Before, the landscape layout was squeezed into a tall page: Level select was cut off, Character select stayed in one row and the pixel HUD and menu overlapped. The landscape layout is unchanged.
- **Community, Translate:** it now also translates what a plugin preview shows (description and commands), including when the preview is opened after translating.

## Added
- `--portrait` flag (with `--template` or `--open`): opens the design in the Portrait view (screenshots).
