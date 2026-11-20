# Lienzo UI 1.7.3

The MCP server tells every AI that the export writes directly and to dry-run first, and both tutorials cover what is new.

## Changed
- **MCP safety notice:** the server instructions, `lienzo_info`, the `export_unity` description and its answers now say that the export writes straight into the Unity project with no prompt in the editor, and ask the AI to call it with `dry_run: true` first, show you the list and wait for your confirmation. A new «Safe exports» section in `docs/MCP.md` explains it.
- **Tutorials (beginner and pro, in the app and in `docs/`, 10 languages):** they now cover the Portrait layout that every template ships with, previewing fonts and plugins before installing, the Translate button, `--dry-run` and the safe-export rule of the MCP server.
