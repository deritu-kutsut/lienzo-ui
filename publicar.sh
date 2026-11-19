#!/usr/bin/env bash
# Publishes a Lienzo UI version.
#   bash publicar.sh 1.1.0
# 1. Runs the tests and builds in release
# 2. Packages dist/LienzoUI-windows-<version>.zip (executable, MCP server, documentation, examples)
# 3. Creates the v<version> tag and the GitHub release with the notes from CHANGELOG.md (requires gh)
# 4. If butler is installed and signed in, uploads the package to itch.io (windows channel)
# Variables: REMOTE (git remote, default origin; the old REMOTO is still read), ITCH_TARGET (the old ITCH_DESTINO is still read).
set -euo pipefail
cd "$(dirname "$0")"

VERSION="${1:?Usage: bash publicar.sh <version>, e.g. 1.1.0}"
REMOTE="${REMOTE:-${REMOTO:-origin}}"
REPO_URL="$(git remote get-url "$REMOTE")"
ITCH_TARGET="${ITCH_TARGET:-${ITCH_DESTINO:-deritu/lienzo-ui:windows}}"
PACKAGE="dist/LienzoUI-${VERSION}"
ZIP="dist/LienzoUI-windows-${VERSION}.zip"

git diff --quiet && git diff --cached --quiet || { echo "There are uncommitted changes. Commit before publishing."; exit 1; }

cargo test --quiet
cargo build --release --quiet
node mcp/probar-mcp.mjs > /dev/null

rm -rf "$PACKAGE" "$ZIP"
mkdir -p "$PACKAGE/mcp" "$PACKAGE/docs" "$PACKAGE/designs"
cp target/release/lienzo-ui.exe README.md LICENSE CHANGELOG.md "$PACKAGE/"
cp mcp/lienzo-ui-mcp.mjs "$PACKAGE/mcp/"
cp -r docs/. "$PACKAGE/docs/"
cp designs/*.editorgui.json "$PACKAGE/designs/" 2>/dev/null || true
cp -r designs/pixel "$PACKAGE/designs/pixel"
cp -r plugins "$PACKAGE/plugins"
( cd dist && powershell -NoProfile -Command "Compress-Archive -Path 'LienzoUI-${VERSION}' -DestinationPath 'LienzoUI-windows-${VERSION}.zip' -Force" )
echo "Package: $ZIP ($(du -h "$ZIP" | cut -f1))"

NOTES="$(awk -v v="$VERSION" '$0 ~ "^## \\[" v "\\]" {f=1; next} /^## \[/ {f=0} f' CHANGELOG.md)"
[ -n "$NOTES" ] || NOTES="Version $VERSION"
git tag -a "v$VERSION" -m "Lienzo UI $VERSION"
git push "$REMOTE" "v$VERSION"
gh release create "v$VERSION" "$ZIP" --repo "$REPO_URL" --title "Lienzo UI $VERSION" --notes "$NOTES"

if command -v butler >/dev/null 2>&1; then
  butler push "$PACKAGE" "$ITCH_TARGET" --userversion "$VERSION"
else
  echo "butler is not installed: update the itch.io download by hand (link to the GitHub release)."
fi
echo "Version $VERSION published."
