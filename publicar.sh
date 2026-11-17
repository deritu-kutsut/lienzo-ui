#!/usr/bin/env bash
# Publica una versión de Lienzo UI.
#   bash publicar.sh 1.1.0
# 1. Ejecuta las pruebas y compila en release
# 2. Empaqueta dist/LienzoUI-windows-<versión>.zip (ejecutable, servidor MCP, documentación, ejemplos)
# 3. Crea el tag v<versión> y la release en GitHub con las notas de CHANGELOG.md (requiere gh)
# 4. Si butler está instalado y con sesión, sube el paquete a itch.io (canal windows)
# Variables: REMOTO (remoto de git, por defecto origin), ITCH_DESTINO.
set -euo pipefail
cd "$(dirname "$0")"

VERSION="${1:?Uso: bash publicar.sh <versión>, p. ej. 1.1.0}"
REMOTO="${REMOTO:-origin}"
REPO_URL="$(git remote get-url "$REMOTO")"
ITCH_DESTINO="${ITCH_DESTINO:-deritu/lienzo-ui:windows}"
PAQUETE="dist/LienzoUI-${VERSION}"
ZIP="dist/LienzoUI-windows-${VERSION}.zip"

git diff --quiet && git diff --cached --quiet || { echo "Hay cambios sin commitear. Haz commit antes de publicar."; exit 1; }

cargo test --quiet
cargo build --release --quiet
node mcp/probar-mcp.mjs > /dev/null

rm -rf "$PAQUETE" "$ZIP"
mkdir -p "$PAQUETE/mcp" "$PAQUETE/docs" "$PAQUETE/disenos"
cp target/release/lienzo-ui.exe README.md LICENSE CHANGELOG.md "$PAQUETE/"
cp mcp/lienzo-ui-mcp.mjs "$PAQUETE/mcp/"
cp -r docs/. "$PAQUETE/docs/"
cp disenos/*.editorgui.json "$PAQUETE/disenos/" 2>/dev/null || true
cp -r disenos/pixel "$PAQUETE/disenos/pixel"
cp -r plugins "$PAQUETE/plugins"
( cd dist && powershell -NoProfile -Command "Compress-Archive -Path 'LienzoUI-${VERSION}' -DestinationPath 'LienzoUI-windows-${VERSION}.zip' -Force" )
echo "Paquete: $ZIP ($(du -h "$ZIP" | cut -f1))"

NOTAS="$(awk -v v="$VERSION" '$0 ~ "^## \\[" v "\\]" {f=1; next} /^## \[/ {f=0} f' CHANGELOG.md)"
[ -n "$NOTAS" ] || NOTAS="Versión $VERSION"
git tag -a "v$VERSION" -m "Lienzo UI $VERSION"
git push "$REMOTO" "v$VERSION"
gh release create "v$VERSION" "$ZIP" --repo "$REPO_URL" --title "Lienzo UI $VERSION" --notes "$NOTAS"

if command -v butler >/dev/null 2>&1; then
  butler push "$PAQUETE" "$ITCH_DESTINO" --userversion "$VERSION"
else
  echo "butler no está instalado: actualiza la descarga de itch.io a mano (enlace a la release de GitHub)."
fi
echo "Versión $VERSION publicada."
