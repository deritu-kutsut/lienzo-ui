# Lienzo UI 1.4.2

Security and stability release.

- Community templates cannot inject code into generated Unity scripts any more; unusual element names no longer break them, and re-exporting does not duplicate your View class.
- Click scripts and plugins run in a stricter sandbox with memory and time limits.
- Community plugins are validated more strictly on install.
- Undo after opening another project, files with a UTF-8 BOM and high contrast with light themes are fixed.
- The executable no longer contains build-machine paths.
