# Paquete de Unity: Lienzo UI for Unity

El lado de Unity de Lienzo UI (importador, ventana de acceso y 7 pantallas de ejemplo), listo para la Asset Store (gratis).

```bash
python unity-package/construir.py        # dist/LienzoUI-for-Unity-<versión>.unitypackage (sin necesitar Unity)
python unity-package/imagenes_store.py   # icono, tarjeta, portada y capturas para la ficha, en unity-package/asset-store/
```

- `fuente/Assets/EditorGUI/`: lo que contiene el paquete. El importador se copia de `src/export/EditorGuiImporter.cs` al construir, así que el paquete y la app no se separan.
- Los `.meta` se generan con **GUID estables** (derivados de la ruta): al actualizar el paquete nadie pierde referencias.
- `EditorGui.Editor.asmdef` hace que el importador **solo se compile si Unity UI (uGUI/TextMeshPro) está instalado**: en un proyecto sin él no da errores.
- `asset-store/ficha.md`: los textos y datos para el Publisher Portal y los pasos para subirlo.

Validado en Unity 6000.3.25f1: con `com.unity.ugui` compila sin errores y sin él tampoco rompe el proyecto.
