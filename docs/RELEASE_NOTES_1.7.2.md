# Lienzo UI 1.7.2

Export preview: see what an export would write before it writes anything.

## Added
- **Export preview (`dry_run`):** the MCP tool `export_unity` takes `dry_run: true`, and the command line takes `--export <project> --dry-run`. Nothing is written, created or removed: each file is listed as `new`, `update`, `same` or `remove`, with the same rules as a real export (your own C# files are still skipped, a missing image still fails). Meant for reviewing what an AI client would write into the Unity project before it does.
