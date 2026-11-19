#!/usr/bin/env python3
"""Builds dist/LienzoUI-for-Unity-<version>.unitypackage without needing Unity.

  python unity-package/build.py

1. Copies the current importer (src/export/EditorGuiImporter.cs) and the sample designs to a work folder.
2. Generates the .meta files with STABLE GUIDs (derived from the path): when the package is updated in the Asset
   Store, users do not lose references. Files that were renamed to English keep the GUID of their old Spanish path
   (see LEGACY_PATHS), so projects that already imported the package keep their references.
3. Writes the .unitypackage (tar.gz with one folder per GUID: asset, asset.meta, pathname).
"""
import gzip, io, re, shutil, sys, tarfile, uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "unity-package" / "source"
DIST = ROOT / "dist"
NAMESPACE = uuid.UUID("6f6c6c0a-4c69-656e-7a6f-554921c0ffee")  # fixed: GUIDs always come out the same

_E = "Assets/EditorGUI/"
# New (English) path -> path it had before the English rename. The GUID is derived from the OLD path.
LEGACY_PATHS = {
    _E + "Editor/LienzoUIWindow.cs": _E + "Editor/LienzoUIVentana.cs",
    **{_E + f"Designs/{new}.json": _E + f"Designs/{old}.json" for new, old in [
        ("Settings", "Ajustes"), ("GameHUD", "HUDDeJuego"), ("PixelArtHUD", "HUDPixelArt"), ("Inventory", "Inventario"),
        ("MainMenu", "MenuPrincipal"), ("BlacksmithShop", "TiendaDelHerrero"), ("DialogWindow", "VentanaDeDialogo")]},
    **{_E + f"Samples/{new}.editorgui.json": _E + f"Samples/{old}.editorgui.json" for new, old in [
        ("settings", "ajustes"), ("dialog", "dialogo"), ("inventory", "inventario"), ("main_menu", "menu_principal"), ("shop", "tienda")]},
    **{_E + f"Sprites/{new}.png": _E + f"Sprites/{old}.png" for new, old in [
        ("shield", "escudo"), ("sword", "espada"), ("coin", "moneda"), ("potion", "pocion")]},
}


def version() -> str:
    m = re.search(r'^version\s*=\s*"([^"]+)"', (ROOT / "Cargo.toml").read_text(encoding="utf-8"), re.M)
    return m.group(1) if m else "0.0.0"


def guid(path: str) -> str:
    return uuid.uuid5(NAMESPACE, LEGACY_PATHS.get(path, path)).hex  # 32 hex, like Unity GUIDs


META = {
    ".cs": "MonoImporter:\n  externalObjects: {}\n  serializedVersion: 2\n  defaultReferences: []\n  executionOrder: 0\n  icon: {instanceID: 0}\n  userData: \n  assetBundleName: \n  assetBundleVariant: \n",
    ".png": "TextureImporter:\n  textureType: 8\n  spriteMode: 1\n  spritePixelsToUnits: 100\n  filterMode: 0\n  textureCompression: 0\n  alphaIsTransparency: 1\n  mipmaps:\n    enableMipMap: 0\n  userData: \n  assetBundleName: \n  assetBundleVariant: \n",
    ".json": "TextScriptImporter:\n  externalObjects: {}\n  userData: \n  assetBundleName: \n  assetBundleVariant: \n",
    ".asmdef": "AssemblyDefinitionImporter:\n  externalObjects: {}\n  userData: \n  assetBundleName: \n  assetBundleVariant: \n",
    ".md": "TextScriptImporter:\n  externalObjects: {}\n  userData: \n  assetBundleName: \n  assetBundleVariant: \n",
}
FOLDER = "DefaultImporter:\n  externalObjects: {}\n  userData: \n  assetBundleName: \n  assetBundleVariant: \n"


def meta(path: str, is_folder: bool) -> str:
    if is_folder:
        return f"fileFormatVersion: 2\nguid: {guid(path)}\nfolderAsset: yes\n{FOLDER}"
    body = META.get(Path(path).suffix.lower(), "DefaultImporter:\n  externalObjects: {}\n  userData: \n  assetBundleName: \n  assetBundleVariant: \n")
    return f"fileFormatVersion: 2\nguid: {guid(path)}\n{body}"


def main() -> int:
    v = version()
    work = ROOT / "unity-package" / "work"
    if work.exists():
        shutil.rmtree(work)
    shutil.copytree(SOURCE, work)
    # the importer is always the app's: this keeps the package and the app in sync
    shutil.copy2(ROOT / "src" / "export" / "EditorGuiImporter.cs", work / "Assets" / "EditorGUI" / "Editor" / "EditorGuiImporter.cs")

    files = []  # (path inside Unity, is_folder, file on disk or None)
    folders = set()
    for f in sorted(work.rglob("*")):
        rel = f.relative_to(work).as_posix()
        if f.is_dir():
            folders.add(rel)
        else:
            if f.suffix == ".meta":
                continue
            files.append((rel, False, f))
            parent = Path(rel).parent
            while str(parent) not in (".", "Assets"):
                folders.add(parent.as_posix())
                parent = parent.parent
    folders.add("Assets/EditorGUI")
    everything = [(c, True, None) for c in sorted(folders)] + files

    DIST.mkdir(exist_ok=True)
    output = DIST / f"LienzoUI-for-Unity-{v}.unitypackage"
    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w") as tar:
        def put(name: str, data: bytes):
            ti = tarfile.TarInfo(name)
            ti.size = len(data)
            ti.mtime = 1763337600  # fixed date: the file comes out identical every time
            tar.addfile(ti, io.BytesIO(data))

        for path, is_folder, file in everything:
            g = guid(path)
            if not is_folder:
                put(f"{g}/asset", file.read_bytes())
            put(f"{g}/asset.meta", meta(path, is_folder).encode())
            put(f"{g}/pathname", path.encode())
    with open(output, "wb") as f:
        with gzip.GzipFile(fileobj=f, mode="wb", mtime=0) as gz:
            gz.write(buf.getvalue())
    shutil.rmtree(work)
    print(f"{output}  ({output.stat().st_size // 1024} KB, {len(files)} files, {len(folders)} folders)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
