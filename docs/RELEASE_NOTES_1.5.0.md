# Lienzo UI 1.5.0

Bringing existing Unity interfaces in, and exporting them the way a real game needs. Includes the 1.4.2 security fixes.

- **Import from Unity:** uGUI prefabs and exported designs become editable designs (names, anchors, layouts, sprites, 9-slice, PPU).
- **Portrait layout** per design, switched at runtime in Unity.
- **Sprites:** 9-slice, pixels per unit, sub-sprites; existing Unity sprites are referenced, not copied.
- **Export options:** transparent overlays, canvas settings, safe area, scripts folder and assembly, choose the outputs, project fonts with fallback.
- **Stable names:** exact GameObject names and rename-proof View scripts.
- Documented file format, JSON schema and `--validate`.
- **Security hardening (two audits):** stricter validation of community content and plugins, sandboxed scripts with memory and time limits, the exporter never overwrites your own scripts, safer paths and images, a safer release build (no build-machine paths) and server-side fixes.
- **Stability:** deep or huge designs no longer crash the exporter, much faster duplicate/move on big designs, atomic saves, undo after opening another project.
