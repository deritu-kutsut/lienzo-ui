# Lienzo UI 1.3.3

## Español
Versión de corrección de bugs (20 arreglos):
- **Lienzo:** el doble clic para entrar en un Contenedor ya no se deshace al soltar; Alt+clic sin arrastrar ya no deja una copia encima; escalar una selección con elementos bloqueados ya no deforma el resto.
- **Edición:** pegar un panel lo pone a su lado (no dentro de sí mismo); agrupar y desagrupar respetan los bloqueados; subir o bajar varios elementos seguidos funciona; los componentes conservan su rotación y su nombre; las opciones de un desplegable se escriben con Enter.
- **Modo prueba:** `mover`, `posicion` y `girar` mueven también los hijos; `valor` elige la opción de un desplegable; `leer_texto` y `leer_valor` avisan si el elemento no existe.
- **Seguridad:** un plugin ya no puede leer scripts fuera de su carpeta con rutas como `\carpeta\x.rhai`.
- **Archivos dañados o ajenos:** se corrigen tamaños de letra absurdos, valores negativos o infinitos, páginas sin tamaño y caracteres de control que rompían el UXML; los títulos «CON», «NUL»… se exportan con un nombre válido en Windows.
- Una fuente subida con el mismo nombre de archivo que otra ya no se cambia por la vieja al reiniciar.

## English
A bug-fix release (20 fixes):
- **Canvas:** double-clicking into a Container no longer snaps back on release; Alt+click without dragging no longer leaves a copy on top; scaling a selection that includes locked elements no longer distorts the rest.
- **Editing:** pasting a panel puts the copy next to it (not inside itself); group and ungroup respect locked elements; moving several elements forward or backward works; components keep their rotation and name; dropdown options can be typed with Enter.
- **Test mode:** `mover`, `posicion` and `girar` also move children; `valor` picks a dropdown option; `leer_texto` and `leer_valor` warn when the element does not exist.
- **Security:** a plugin can no longer read scripts outside its folder with paths like `\folder\x.rhai`.
- **Damaged or third-party files:** absurd font sizes, negative or infinite values, zero-size pages and control characters (which broke the exported UXML) are fixed on load; titles such as "CON" or "NUL" export with a valid Windows file name.
- A font uploaded with the same file name as another no longer switches back to the old one after restarting.
