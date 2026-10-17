"""Every widget drawn once, from real takes where they exist, for eyeballing.

    cd tools && python3 -m reservoir_film.gallery
"""

import json

from PIL import Image

from . import draw
from .model import TAKES_DIR, Takes, load_numbers


def frames():
    takes = Takes()
    out = []

    img = draw.title_card(3, "Pour Mario into Tetris", "Reservoir computing")
    draw.caption(img, "A caption of ordinary length sits here, centred in the bar.")
    out.append(("title_card + caption", img))

    out.append(("text_card", draw.text_card(["The brain could see its own hands."], kicker="Mistake 1")))

    img = draw.blank()
    walk = takes["matrix_walk"]
    step = walk.meta["steps"][120]
    draw.nes(img, walk.frames("smb")[480], (64, 60, 256, 240))
    window = bytes.fromhex(step["window"])
    draw.byte_grid(img, window[:256], (360, 60, 320, 320), 16, highlight={0x86: draw.GOLD, 0x1D: draw.RED},
                   labels=[(0x86, "Mario's x"), (0x1D, "on the ground")], changed={0x87})
    draw.hex_grid(img, window[:256], (64, 330, 256, 256), 16)
    out.append(("nes + byte_grid + hex_grid", img))

    img = draw.blank()
    echo = takes["echo_twins"].meta["steps"][40]
    a, b = echo["a"], echo["b"]
    diff = draw.filled(a) ^ draw.filled(b)
    draw.board(img, a, (100, 60, 200, 400), diff=diff)
    draw.board(img, b, (360, 60, 200, 400), diff=diff, colour=draw.REGION["wall"])
    draw.heights_bars(img, [3, 5, 0, 9, 12, 4, 4, 2, 0, 7], (640, 260, 300, 200))
    draw.lamp(img, (1050, 120), True, draw.RED, "enemy close")
    draw.lamp(img, (1150, 120), False, draw.GREEN, "pit ahead")
    out.append(("board + diff + heights + lamps", img))

    img = draw.blank()
    brain = takes["simple_brain_drives"].meta["demo_steps"][:120]
    draw.gauge(img, [s["a_out"] for s in brain], [s["buttons"] & 1 == 1 for s in brain], (64, 60, 1150, 160))
    coin = takes["coin_flip_teachers"].meta["demos"]
    lanes = [[s["x"] for i, s in enumerate(d["steps"]) if s["a"] and (i == 0 or not d["steps"][i - 1]["a"])] for d in coin]
    draw.strip(img, (64, 260, 1150, 200), lanes, 3300, marker=1200)
    out.append(("gauge + strip", img))

    img = draw.blank()
    n = load_numbers()
    draw.bar_chart(img, (64, 40, 1150, 520), n["rows"], "test", "label", shown=len(n["rows"]), se=n["test_se"], show_se=True)
    out.append(("bar_chart", img))

    img = draw.blank()
    lives = takes["lives_glyphs"]
    tiles = draw.glyph_tiles(lives.frames("screen"), lives.meta["digit_box"])
    draw.rain(img, tiles, 3.0, seed=1)
    out.append((f"rain ({len(tiles[0])} tiles)", img))

    img = draw.blank()
    draw.ripple(img, (64, 40, 1150, 520), 6.0, [(1, 0.3, 0.4), (4, 0.6, 0.5), (5, 0.45, 0.3)])
    out.append(("ripple", img))
    return out


def main():
    shots = frames()
    sheet = Image.new("RGB", (2 * 640, ((len(shots) + 1) // 2) * 380), draw.BG)
    for k, (label, img) in enumerate(shots):
        x, y = (k % 2) * 640, (k // 2) * 380
        sheet.paste(img.resize((640, 360)), (x, y + 20))
        draw.text(sheet, (x + 6, y + 2), label, 16, draw.GOLD)
    out = TAKES_DIR / "gallery.png"
    sheet.save(out)
    print(out)


if __name__ == "__main__":
    main()
