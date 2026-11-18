# Lienzo UI 1.3.3

A bug-fix release (20 fixes):
- **Canvas:** double-clicking into a Container no longer snaps back on release; Alt+click without dragging no longer leaves a copy on top; scaling a selection that includes locked elements no longer distorts the rest.
- **Editing:** pasting a panel puts the copy next to it (not inside itself); group and ungroup respect locked elements; moving several elements forward or backward works; components keep their rotation and name; dropdown options can be typed with Enter.
- **Test mode:** `mover`, `posicion` and `girar` also move children; `valor` picks a dropdown option; `leer_texto` and `leer_valor` warn when the element does not exist.
- **Security:** a plugin can no longer read scripts outside its folder with paths like `\folder\x.rhai`.
- **Damaged or third-party files:** absurd font sizes, negative or infinite values, zero-size pages and control characters (which broke the exported UXML) are fixed on load; titles such as "CON" or "NUL" export with a valid Windows file name.
- A font uploaded with the same file name as another no longer switches back to the old one after restarting.
