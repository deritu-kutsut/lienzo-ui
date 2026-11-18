from PIL import Image, ImageOps
import os
import pathlib
R = str(pathlib.Path(__file__).resolve().parent.parent) + "/"
O = R + "unity-package/asset-store/"
os.makedirs(O, exist_ok=True)
FONDO = (20, 22, 31)


def encajar(im, w, h, fondo=FONDO):
    """Escala sin deformar y centra sobre un fondo liso."""
    im = im.convert("RGB")
    r = min(w / im.width, h / im.height)
    n = im.resize((max(1, int(im.width * r)), max(1, int(im.height * r))), Image.LANCZOS)
    lienzo = Image.new("RGB", (w, h), fondo)
    lienzo.paste(n, ((w - n.width) // 2, (h - n.height) // 2))
    return lienzo


def cubrir(im, w, h):
    return ImageOps.fit(im.convert("RGB"), (w, h), Image.LANCZOS, centering=(0.5, 0.5))


portada = Image.open(R + "assets/brand/cover.png")
icono = Image.open(R + "assets/brand/icon.png").convert("RGBA")
cubrir(portada, 1950, 1300).save(O + "cover_1950x1300.png", optimize=True)
cubrir(portada, 420, 280).save(O + "card_420x280.png", optimize=True)
fondo = Image.new("RGBA", icono.size, (255, 255, 255, 255))
fondo.alpha_composite(icono)
fondo.convert("RGB").resize((160, 160), Image.LANCZOS).save(O + "icon_160x160.png", optimize=True)
for i, n in enumerate(["editor", "unity", "comunidad", "perfil", "temas", "tutorial"], 1):
    p = R + f"docs/imagenes/{n}.png"
    if os.path.exists(p):
        encajar(Image.open(p), 1920, 1080).save(O + f"screenshot_{i}_{n}_1920x1080.png", optimize=True)
print(sorted(os.listdir(O)))
