# Contributing

Thank you for wanting to improve Lienzo UI! Anyone can propose changes.

1. Make a **fork** and create a branch: `git checkout -b feat/my-improvement`.
2. Follow [docs/EXTENDING.md](docs/EXTENDING.md).
3. Check: `cargo test`, `node mcp/probar-mcp.mjs` and, if you touched the export, a real export to Unity.
4. Add a line to `CHANGELOG.md` under **Unreleased**.
5. Open a **pull request** explaining what changes and why, with a screenshot if it is visual.

A bug or an idea? Open an **issue**.

## Conventions

- Rust 2024, `cargo fmt` before submitting. Everything is English-first: code, identifiers, file and folder names, tool and flag names, and the texts passed to `crate::i18n::t("…")` are English; other languages are `assets/i18n/<code>.tsv` files (see [docs/EXTENDING.md](docs/EXTENDING.md)).
- Changes to the design format must keep reading old projects (`#[serde(default)]`).
- What is exported to Unity must not overwrite the user's code (`<Design>View.cs`).

## Publishing a release (maintainers)

Bump `version` in `Cargo.toml` and write the `## [x.y.z]` section of `CHANGELOG.md` first (the script refuses to run if either is missing or the section is empty), commit, and then:

```bash
bash publicar.sh 1.1.0 --dry-run   # builds, packages and runs every check; never tags, pushes, releases or uploads
bash publicar.sh 1.1.0             # the real thing
```

It checks the repository state (clean tree, no untracked files in `designs`, `docs`, `plugins` or `mcp`, branch pushed), runs the tests, builds in release with path remapping and control-flow guard, refuses to continue if the executable contains the build user name, `C:\Users`, the repository path or `.cargo`, packages `LienzoUI-windows-<version>.zip` **from `git ls-files`** (executable, MCP server, documentation, examples and `THIRD_PARTY_LICENSES.txt`), pushes the branch, creates the tag and the release on GitHub (a re-run reuses what already exists) and, if `butler` is installed, uploads it to itch.io. It needs `bash`, `python`, `node`, `git`, `cargo` and `gh`.

### Windows hardening and code signing

The executable is built with control-flow guard (`-C control-flow-guard`, set by `publicar.sh` and in the local `.cargo/config.toml`), `/CETCOMPAT` (from `build.rs`) and an embedded application manifest (per-monitor-v2 DPI awareness, long paths, Windows 10/11, no elevation). `publicar.sh` prints what the final executable really has.

**The executable is not code-signed.** Signing needs a certificate that only the maintainer can provide (an OV/EV code-signing certificate, or Azure Trusted Signing). Once there is one, add a `signtool sign /fd SHA256 /tr <timestamp-url> /td SHA256 ...` step to `publicar.sh` right after the build (see the comment in the script header) and verify with `signtool verify /pa`. Never commit the certificate or its password. Until then, Windows SmartScreen may warn on first run.

By contributing you agree to publish your contribution under the [MIT license](LICENSE).
