"""Chapter 0: four boards driving Mario, the title, the lives counter running past nine into garbage tiles, and the rain."""

import numpy as np

from .. import draw
from ..model import Caption, Chapter, Shot

REGIONS = ["mario", "enemies", "floor", "wall"]


def bank_open(ctx, t, i):
    img = draw.blank()
    take = ctx.takes["bank_drive"]
    draw.nes(img, take.frames("smb")[i], (64, 60, 512, 480))
    steps = take.meta["best"]["steps"]
    boards = steps[min(i // 4, len(steps) - 1)]["boards"]
    for k, (cells, region) in enumerate(zip(boards, REGIONS)):
        x, y = 700 + (k % 2) * 260, 70 + (k // 2) * 240
        draw.board(img, cells, (x, y, 110, 220), colour=draw.REGION[region])
    return img


def title(ctx, t, i):
    return draw.title_card(None, "Can Tetris play Mario?", "A lab notebook in ten chapters")


def lives_value(t):
    """7 to 12 one a second, then accelerating to 254 by 14 s."""
    if t < 6:
        return 7 + int(t)
    # Stop at 254: at 255 the counter wraps round and draws a plain 0.
    return min(254, 13 + int((t - 6) ** 2.6 * 1.2))


# The "MARIO x N" line of the intermission screen, (x, y, w, h) in NES pixels.
LIVES_LINE = (72, 104, 112, 32)


def lives(ctx, t, i):
    img = draw.blank()
    frame = ctx.takes["lives_glyphs"].frames("screen")[lives_value(t)]
    x, y, w, h = LIVES_LINE
    draw.nes(img, frame[y:y + h, x:x + w], (192, 120, 896, 256))
    draw.text(img, (640, 420), f"lives: {lives_value(t) + 1}", 32, draw.DIM, anchor="ma")
    return img


def lives_stings(ctx):
    out, last = [], None
    for k in range(14 * 60):
        v = lives_value(k / 60)
        if v != last and len(out) < 20:
            out.append((k / 60, "blip"))
        last = v
    return out


_tiles = {}


def tiles(ctx):
    if "t" not in _tiles:
        take = ctx.takes["lives_glyphs"]
        _tiles["t"] = draw.glyph_tiles(take.frames("screen"), take.meta["digit_box"])
    return _tiles["t"]


def rain_begins(ctx, t, i):
    under = lives(ctx, 14.0, 0)
    img = draw.blank()
    draw.rain(img, tiles(ctx), t, seed=1, speed=0.3 + t / 4, density=min(1.0, t / 8))
    return draw.fade(under, img, min(1.0, t / 3))


CHAPTER = Chapter(0, "Cold open", "", [
    Shot("bank_open", 12, bank_open, [Caption(6, "Four games of Tetris are deciding when Mario jumps.")], takes=("bank_drive",)),
    Shot("title", 5, title),
    Shot("lives", 14, lives, [
        Caption(0, "In Super Mario Bros., the lives counter is a single number."),
        Caption(5, "Past nine, the game runs out of digits and draws whatever tiles come next."),
        Caption(10, "We didn't earn these lives. We typed them in."),
    ], takes=("lives_glyphs",), stings=lives_stings),
    Shot("rain_begins", 8, rain_begins, [Caption(2, "Everything Mario is, is numbers like these.")], takes=("lives_glyphs",)),
])
