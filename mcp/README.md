# lienzo-ui-mcp

MCP server for [Lienzo UI](https://github.com/deritu-kutsut/lienzo-ui) (interface editor that exports to Unity). No dependencies, Node 18+.

It needs the Lienzo UI app: download it from https://deritu.itch.io/lienzo-ui and point `LIENZO_UI` to the executable.

```json
{ "mcpServers": { "lienzo-ui": { "command": "npx", "args": ["-y", "lienzo-ui-mcp"], "env": { "LIENZO_UI": "C:/path/to/lienzo-ui.exe" } } } }
```

**Safety:** `export_unity` writes straight into `Assets/EditorGUI` of the Unity project, with no prompt in the editor. It only touches that folder, refuses network paths and never overwrites your own C# scripts. Call it with `dry_run: true` first to list what it would write. Full docs: https://github.com/deritu-kutsut/lienzo-ui/blob/main/docs/MCP.md
