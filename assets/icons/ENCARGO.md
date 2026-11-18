I need a set of SVG icons for the interface of a UI editor for Unity (dark theme).

Style: minimalist, line-based, consistent with each other (Lucide / Feather style).
Technical rules, all mandatory:
- viewBox="0 0 24 24", width="24" height="24", no <title>, no CSS styles, no transform.
- Strokes only: fill="none" stroke="#FFFFFF" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" on the <svg>.
- If a detail needs a fill (a dot), use fill="#FFFFFF" on that element only.
- Only <path>, <rect>, <circle>, <line>, <polyline>, <polygon>. No text or fonts.
- 2 px margin inside the viewBox.

Deliver them in ONE single JSON code block with the shape {"name": "<svg ...>...</svg>", ...} using exactly these names:

Element types:
  panel (rounded rectangle), ellipse (circle), image (frame with mountain and sun), text (letter T drawn with lines),
  button (pill with a dot/cursor), toggle (box with check), slider (line with knob), input (field with text cursor),
  dropdown (box with chevron down), scrollview (box with scroll bar), empty (rectangle with dashed line)
Actions:
  eye, eye_off, lock, unlock, duplicate, trash, parent_up (up-left arrow leaving a box), bolt (lightning bolt),
  layout (2x2 grid), mask (square clipping a circle), play, stop, undo, redo, export (arrow down into a tray),
  save, search, plus, close, code (</>)
Alignment:
  align_left, align_center, align_right, align_top, align_middle, align_bottom
Side tabs:
  tab_hierarchy (tree of nodes), tab_elements (shapes: circle, square, triangle), tab_templates (three layout panels),
  tab_text (T with underline), tab_images (two stacked images)
