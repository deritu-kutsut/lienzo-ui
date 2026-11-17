# Contribuir

¡Gracias por querer mejorar Lienzo UI! Cualquier persona puede proponer cambios.

1. Haz un **fork** y crea una rama: `git checkout -b feat/mi-mejora`.
2. Sigue [docs/EXTENDER.md](docs/EXTENDER.md).
3. Comprueba: `cargo test`, `node mcp/probar-mcp.mjs` y, si tocaste la exportación, una exportación real a Unity.
4. Añade una línea a `CHANGELOG.md` en **Sin publicar**.
5. Abre un **pull request** explicando qué cambia y por qué, con una captura si es visual.

¿Un error o una idea? Abre un **issue**.

## Convenciones

- Rust 2024, `cargo fmt` antes de enviar. Textos de la interfaz en español.
- Los cambios en el formato del diseño deben seguir leyendo los proyectos antiguos (`#[serde(default)]`).
- Lo que se exporta a Unity no debe pisar el código del usuario (`<Diseño>View.cs`).

## Publicar una versión (mantenedores)

```bash
bash publicar.sh 1.1.0
```

Compila en release, empaqueta `LienzoUI-windows-<versión>.zip` (ejecutable, servidor MCP y documentación), crea el tag y la release en GitHub y, si `butler` está instalado, la sube a itch.io.

Al contribuir aceptas publicar tu aporte bajo la [licencia MIT](LICENSE).
