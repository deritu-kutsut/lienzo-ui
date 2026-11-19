from PIL import Image, ImageOps
import os
import pathlib
R = str(pathlib.Path(__file__).resolve().parent.parent) + "/"
O = R + "unity-package/asset-store/"
os.makedirs(O, exist_ok=True)
BACKGROUND = (20, 22, 31)


def fit(im, w, h, background=BACKGROUND):
    """Scales without distorting and centers on a plain background."""
    im = im.convert("RGB")
    r = min(w / im.width, h / im.height)
    n = im.resize((max(1, int(im.width * r)), max(1, int(im.height * r))), Image.LANCZOS)
    canvas = Image.new("RGB", (w, h), background)
    canvas.paste(n, ((w - n.width) // 2, (h - n.height) // 2))
    return canvas


def cover(im, w, h):
    return ImageOps.fit(im.convert("RGB"), (w, h), Image.LANCZOS, centering=(0.5, 0.5))


cover_image = Image.open(R + "assets/brand/cover.png")
icon = Image.open(R + "assets/brand/icon.png").convert("RGBA")
cover(cover_image, 1950, 1300).save(O + "cover_1950x1300.png", optimize=True)
cover(cover_image, 420, 280).save(O + "card_420x280.png", optimize=True)
white = Image.new("RGBA", icon.size, (255, 255, 255, 255))
white.alpha_composite(icon)
white.convert("RGB").resize((160, 160), Image.LANCZOS).save(O + "icon_160x160.png", optimize=True)
# (file in docs/imagenes, name used in the screenshot file name)
for i, (source, label) in enumerate([("editor", "editor"), ("unity", "unity"), ("comunidad", "community"), ("perfil", "profile"), ("temas", "themes"), ("tutorial", "tutorial")], 1):
    p = R + f"docs/imagenes/{source}.png"
    if os.path.exists(p):
        fit(Image.open(p), 1920, 1080).save(O + f"screenshot_{i}_{label}_1920x1080.png", optimize=True)
print(sorted(os.listdir(O)))
