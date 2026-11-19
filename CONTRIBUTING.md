# Contributing

Thank you for wanting to improve Lienzo UI! Anyone can propose changes.

1. Make a **fork** and create a branch: `git checkout -b feat/my-improvement`.
2. Follow [docs/EXTENDER.md](docs/EXTENDER.md).
3. Check: `cargo test`, `node mcp/probar-mcp.mjs` and, if you touched the export, a real export to Unity.
4. Add a line to `CHANGELOG.md` under **Unreleased**.
5. Open a **pull request** explaining what changes and why, with a screenshot if it is visual.

A bug or an idea? Open an **issue**.

## Conventions

- Rust 2024, `cargo fmt` before submitting. Everything is English-first: code, identifiers, file and folder names, tool and flag names, and the texts passed to `crate::i18n::t("…")` are English; other languages are `assets/i18n/<code>.tsv` files (see [docs/EXTENDER.md](docs/EXTENDER.md)).
- Changes to the design format must keep reading old projects (`#[serde(default)]`).
- What is exported to Unity must not overwrite the user's code (`<Design>View.cs`).

## Publishing a release (maintainers)

```bash
bash publicar.sh 1.1.0
```

It builds in release, packages `LienzoUI-windows-<version>.zip` (executable, MCP server and documentation), creates the tag and the release on GitHub and, if `butler` is installed, uploads it to itch.io.

By contributing you agree to publish your contribution under the [MIT license](LICENSE).
