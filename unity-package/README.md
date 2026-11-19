# Unity package: Lienzo UI for Unity

The Unity side of Lienzo UI (importer, launcher window and 7 sample screens), ready for the Asset Store (free).

```bash
python unity-package/build.py            # dist/LienzoUI-for-Unity-<version>.unitypackage (no Unity needed)
python unity-package/store_images.py   # icon, card, cover and screenshots for the listing, in unity-package/asset-store/
```

- `source/Assets/EditorGUI/`: what the package contains. The importer is copied from `src/export/EditorGuiImporter.cs` at build time, so the package and the app never drift apart.
- The `.meta` files are generated with **stable GUIDs** (derived from the path): when the package is updated, nobody loses references. Files renamed from Spanish to English keep the GUID of their old path (`LEGACY_PATHS` in `build.py`), so projects that already imported the package keep working.
- `EditorGui.Editor.asmdef` makes the importer **compile only if Unity UI (uGUI/TextMeshPro) is installed**: in a project without it, there are no errors.
- `asset-store/ficha.md`: the texts and data for the Publisher Portal and the steps to upload it.

Validated on Unity 6000.3.25f1: with `com.unity.ugui` it compiles without errors, and without it the project doesn't break either.
