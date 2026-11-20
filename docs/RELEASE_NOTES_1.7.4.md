# Lienzo UI 1.7.4

Security fix for the MCP server published on npm.

## Security
- **npm package (`lienzo-ui-mcp`):** when run from node_modules it no longer looks for `lienzo-ui.exe` or `target/` next to node_modules (a planted file could have been run as the app) and saves designs in `~/LienzoUI/designs` instead of inside node_modules, where an update would delete them. The app path now comes only from `LIENZO_UI`. Four new checks cover this layout.
