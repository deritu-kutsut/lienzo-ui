# Lienzo UI 1.5.1

Small fix on top of 1.5.0.

- The Portrait view now shows elements without a portrait override the way Unity will: they keep their landscape anchors and offsets and follow them when the page becomes portrait (a full-page background fills the portrait page, right-anchored elements stay at the right edge). Editing in portrait still creates an override only for the element you change. Exports to Unity are unchanged.
