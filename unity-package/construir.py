#!/usr/bin/env python3
"""Construye dist/LienzoUI-for-Unity-<versión>.unitypackage sin necesitar Unity.

  python unity-package/construir.py

1. Copia el importador actual (src/export/EditorGuiImporter.cs) y los diseños de ejemplo a una carpeta de trabajo.
2. Genera los .meta con GUID ESTABLES (derivados de la ruta): al actualizar el paquete en la Asset Store, los
   usuarios no pierden referencias.
3. Escribe el .unitypackage (tar.gz con una carpeta por GUID: asset, asset.meta, pathname).
"""
import gzip, io, re, shutil, sys, tarfile, uuid
from pathlib import Path

RAIZ = Path(__file__).resolve().parent.parent
FUENTE = RAIZ / "unity-package" / "fuente"
DIST = RAIZ / "dist"
ESPACIO = uuid.UUID("6f6c6c0a-4c69-656e-7a6f-554921c0ffee")  # fijo: los GUID salen siempre iguales


def version() -> str:
    m = re.search(r'^version\s*=\s*"([^"]+)"', (RAIZ / "Cargo.toml").read_text(encoding="utf-8"), re.M)
    return m.group(1) if m else "0.0.0"


def guid(ruta: str) -> str:
    return uuid.uuid5(ESPACIO, ruta).hex  # 32 hex, como los GUID de Unity


META = {
    ".cs": "MonoImporter:\n  externalObjects: {}\n  serializedVersion: 2\n  defaultReferences: []\n  executionOrder: 0\n  icon: {instanceID: 0}\n  userData: \n  assetBundleName: \n  assetBundleVariant: \n",
    ".png": "TextureImporter:\n  textureType: 8\n  spriteMode: 1\n  spritePixelsToUnits: 100\n  filterMode: 0\n  textureCompression: 0\n  alphaIsTransparency: 1\n  mipmaps:\n    enableMipMap: 0\n  userData: \n  assetBundleName: \n  assetBundleVariant: \n",
    ".json": "TextScriptImporter:\n  externalObjects: {}\n  userData: \n  assetBundleName: \n  assetBundleVariant: \n",
    ".asmdef": "AssemblyDefinitionImporter:\n  externalObjects: {}\n  userData: \n  assetBundleName: \n  assetBundleVariant: \n",
    ".md": "TextScriptImporter:\n  externalObjects: {}\n  userData: \n  assetBundleName: \n  assetBundleVariant: \n",
}
CARPETA = "DefaultImporter:\n  externalObjects: {}\n  userData: \n  assetBundleName: \n  assetBundleVariant: \n"


def meta(ruta: str, es_carpeta: bool) -> str:
    if es_carpeta:
        return f"fileFormatVersion: 2\nguid: {guid(ruta)}\nfolderAsset: yes\n{CARPETA}"
    cuerpo = META.get(Path(ruta).suffix.lower(), "DefaultImporter:\n  externalObjects: {}\n  userData: \n  assetBundleName: \n  assetBundleVariant: \n")
    return f"fileFormatVersion: 2\nguid: {guid(ruta)}\n{cuerpo}"


def main() -> int:
    v = version()
    trabajo = RAIZ / "unity-package" / "trabajo"
    if trabajo.exists():
        shutil.rmtree(trabajo)
    shutil.copytree(FUENTE, trabajo)
    # el importador siempre es el de la app: así el paquete y la app no se separan
    shutil.copy2(RAIZ / "src" / "export" / "EditorGuiImporter.cs", trabajo / "Assets" / "EditorGUI" / "Editor" / "EditorGuiImporter.cs")

    entradas = []  # (ruta dentro de Unity, es_carpeta, archivo en disco o None)
    carpetas = set()
    for f in sorted(trabajo.rglob("*")):
        rel = f.relative_to(trabajo).as_posix()
        if f.is_dir():
            carpetas.add(rel)
        else:
            if f.suffix == ".meta":
                continue
            entradas.append((rel, False, f))
            padre = Path(rel).parent
            while str(padre) not in (".", "Assets"):
                carpetas.add(padre.as_posix())
                padre = padre.parent
    carpetas.add("Assets/EditorGUI")
    todas = [(c, True, None) for c in sorted(carpetas)] + entradas

    DIST.mkdir(exist_ok=True)
    salida = DIST / f"LienzoUI-for-Unity-{v}.unitypackage"
    buf = io.BytesIO()
    with tarfile.open(fileobj=buf, mode="w") as tar:
        def poner(nombre: str, datos: bytes):
            ti = tarfile.TarInfo(nombre)
            ti.size = len(datos)
            ti.mtime = 1763337600  # fecha fija: el archivo sale idéntico cada vez
            tar.addfile(ti, io.BytesIO(datos))

        for ruta, carpeta, archivo in todas:
            g = guid(ruta)
            if not carpeta:
                poner(f"{g}/asset", archivo.read_bytes())
            poner(f"{g}/asset.meta", meta(ruta, carpeta).encode())
            poner(f"{g}/pathname", ruta.encode())
    with open(salida, "wb") as f:
        with gzip.GzipFile(fileobj=f, mode="wb", mtime=0) as gz:
            gz.write(buf.getvalue())
    shutil.rmtree(trabajo)
    print(f"{salida}  ({salida.stat().st_size // 1024} KB, {len(entradas)} archivos, {len(carpetas)} carpetas)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
