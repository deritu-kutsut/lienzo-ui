# Lienzo UI 1.6.2

Small template fixes found in a review of every bundled template.

## Fixed
- **Game HUD template:** the score now sits on a dark panel, so the yellow text keeps a readable contrast over the bright sky (it was 2.4:1; WCAG asks for 3:1).
- **Settings template:** the «Fullscreen» toggle is 70 px tall (it was 50 px), above the 64 px touch-target minimum.
- Internal cleanups: one deprecated egui call and an unneeded `mut` removed.

## Notes
- Versions before 1.6.1 are no longer offered on GitHub (their releases were hidden). Older builds cannot accept the Terms of Use, so they cannot upload to the community; update to the latest version.
