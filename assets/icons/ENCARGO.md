Necesito un set de iconos SVG para la interfaz de un editor de UI para Unity (tema oscuro).

Estilo: minimalista, lineal, coherente entre sí (tipo Lucide / Feather).
Reglas técnicas, todas obligatorias:
- viewBox="0 0 24 24", width="24" height="24", sin <title>, sin estilos CSS, sin transform.
- Solo trazos: fill="none" stroke="#FFFFFF" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" en el <svg>.
- Si un detalle necesita relleno (un punto), usa fill="#FFFFFF" en ese elemento solamente.
- Solo <path>, <rect>, <circle>, <line>, <polyline>, <polygon>. Nada de texto ni fuentes.
- Margen de 2 px dentro del viewBox.

Entrégalos en UN solo bloque de código JSON con la forma {"nombre": "<svg ...>...</svg>", ...} usando exactamente estos nombres:

Tipos de elemento:
  panel (rectángulo redondeado), ellipse (círculo), image (marco con montaña y sol), text (letra T trazada con líneas),
  button (píldora con un punto/cursor), toggle (casilla con check), slider (línea con perilla), input (campo con cursor de texto),
  dropdown (caja con chevron abajo), scrollview (caja con barra de scroll), empty (rectángulo con línea discontinua)
Acciones:
  eye, eye_off, lock, unlock, duplicate, trash, parent_up (flecha arriba-izquierda saliendo de una caja), bolt (rayo),
  layout (rejilla 2x2), mask (cuadrado recortando un círculo), play, stop, undo, redo, export (flecha hacia abajo a una bandeja),
  save, search, plus, close, code (</>)
Alineación:
  align_left, align_center, align_right, align_top, align_middle, align_bottom
Pestañas laterales:
  tab_hierarchy (árbol de nodos), tab_elements (formas: círculo, cuadrado, triángulo), tab_templates (tres paneles de layout),
  tab_text (T con subrayado), tab_images (dos imágenes apiladas)
