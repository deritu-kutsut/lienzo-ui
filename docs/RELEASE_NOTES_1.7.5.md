# Lienzo UI 1.7.5

IMGUI export fix: the text typed in input fields uses the design's colour.

## Fixed
- **IMGUI export, input fields:** the typed text now uses the design's text colour. It was always a fixed near-black (0.1, 0.1, 0.13), so on a dark field it could not be read. uGUI and UI Toolkit already used the design's colour. New test `input_field_text_uses_design_colour`.
