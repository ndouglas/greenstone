"""Widgets for the reservoir film. Every function draws onto a 1280x720 RGB
image or returns one; the chapters compose them."""

import math
from functools import lru_cache
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFont

from .model import H, NES_H, NES_W, W

BG = (12, 14, 20)
PANEL = (26, 29, 40)
INK = (235, 235, 235)
DIM = (140, 145, 160)
GREEN = (90, 220, 120)
RED = (235, 90, 80)
GOLD = (245, 200, 80)
MATRIX = (60, 255, 120)
REGION = {"mario": (240, 90, 80), "enemies": (250, 170, 60), "floor": (90, 200, 120), "wall": (90, 150, 250)}

# An empty playfield cell (drive.rs PLAYFIELD_EMPTY); any other tile is a block.
PLAYFIELD_EMPTY = 0xEF
CAPTION_TOP = H - 96

SANS = ["/System/Library/Fonts/Supplemental/Arial.ttf", "/System/Library/Fonts/Helvetica.ttc",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", "DejaVuSans.ttf"]
SANS_BOLD = ["/System/Library/Fonts/Supplemental/Arial Bold.ttf", "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
             "DejaVuSans-Bold.ttf"] + SANS
MONO = ["/System/Library/Fonts/Menlo.ttc", "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf", "DejaVuSansMono.ttf"]


def _first(paths, size):
    for p in paths:
        try:
            return ImageFont.truetype(p, size)
        except OSError:
            continue
    return ImageFont.load_default()


@lru_cache(maxsize=None)
def font(size, bold=False):
    return _first(SANS_BOLD if bold else SANS, size)


@lru_cache(maxsize=None)
def mono(size):
    return _first(MONO, size)


def blank():
    return Image.new("RGB", (W, H), BG)


def as_bytes(cells):
    return bytes.fromhex(cells) if isinstance(cells, str) else bytes(cells)


def text(img, xy, s, size=24, colour=INK, anchor="la", bold=False, face=None):
    ImageDraw.Draw(img).text(xy, s, font=face or font(size, bold), fill=colour, anchor=anchor)


def panel(img, box, title=None):
    x, y, w, h = box
    d = ImageDraw.Draw(img)
    d.rounded_rectangle((x, y, x + w, y + h), radius=8, fill=PANEL)
    if title:
        d.text((x + 10, y + 6), title, font=font(16), fill=DIM)


def nes(img, frame, box):
    """An NES frame (240x256x3) scaled with nearest neighbour into box (x, y, w, h)."""
    x, y, w, h = box
    im = Image.fromarray(np.asarray(frame, dtype=np.uint8)).resize((w, h), Image.NEAREST)
    img.paste(im, (x, y))


def wrap(s, f, width):
    lines, cur = [], ""
    for word in s.split():
        trial = (cur + " " + word).strip()
        if f.getlength(trial) > width and cur:
            lines.append(cur)
            cur = word
        else:
            cur = trial
    if cur:
        lines.append(cur)
    return lines


def caption(img, s):
    """The bottom bar: at most two lines at 30 px; longer raises ValueError."""
    f = font(30)
    lines = wrap(s, f, W - 80)
    if len(lines) > 2:
        raise ValueError(f"caption needs {len(lines)} lines: {s}")
    d = ImageDraw.Draw(img, "RGBA")
    d.rectangle((0, CAPTION_TOP, W, H), fill=(0, 0, 0, 210))
    y = CAPTION_TOP + (48 - 18 * len(lines)) + 4
    for line in lines:
        d.text((W // 2, y), line, font=f, fill=INK, anchor="ma")
        y += 38


def title_card(number, title, subtitle):
    img = blank()
    if number is not None:
        text(img, (W // 2, 230), f"Chapter {number}", 30, GOLD, anchor="ma")
    text(img, (W // 2, 290), title, 64, INK, anchor="ma", bold=True)
    if subtitle:
        text(img, (W // 2, 390), subtitle, 30, DIM, anchor="ma")
    return img


def text_card(lines, kicker=None):
    """One to four lines centred above the caption area, with an optional gold kicker."""
    img = blank()
    f = font(40)
    wrapped = [w for line in lines for w in (wrap(line, f, W - 240) or [""]) + ["\n"]][:-1]
    rows = [r for r in wrapped if r != "\n"]
    gaps = wrapped.count("\n")
    height = len(rows) * 54 + gaps * 24 + (60 if kicker else 0)
    y = max(40, (CAPTION_TOP - height) // 2)
    if kicker:
        text(img, (W // 2, y), kicker, 28, GOLD, anchor="ma", bold=True)
        y += 60
    for r in wrapped:
        if r == "\n":
            y += 24
            continue
        text(img, (W // 2, y), r, 40, INK, anchor="ma")
        y += 54
    return img


def _cell(box, cols, rows, i):
    x, y, w, h = box
    cw, ch = w / cols, h / rows
    c, r = i % cols, i // cols
    return x + c * cw, y + r * ch, cw, ch


def byte_grid(img, data, box, cols, highlight=None, labels=None, changed=None, colours=None):
    """Cells lit by value (black to MATRIX), or by `colours[i]` when given;
    `highlight` index -> colour outlines; `changed` indices in INK; `labels`
    (index, text) with leader lines to the right of the grid."""
    data = as_bytes(data)
    rows = math.ceil(len(data) / cols)
    d = ImageDraw.Draw(img)
    for i, v in enumerate(data):
        cx, cy, cw, ch = _cell(box, cols, rows, i)
        if colours and i in colours:
            fill = colours[i]
        elif changed and i in changed:
            fill = INK
        else:
            a = v / 255
            fill = tuple(int(BG[k] + (MATRIX[k] - BG[k]) * a) for k in range(3))
        d.rectangle((cx + 0.5, cy + 0.5, cx + cw - 1, cy + ch - 1), fill=fill)
    for i, colour in (highlight or {}).items():
        cx, cy, cw, ch = _cell(box, cols, rows, i)
        d.rectangle((cx - 1, cy - 1, cx + cw, cy + ch), outline=colour, width=2)
    x, y, w, h = box
    for k, (i, label) in enumerate(labels or []):
        cx, cy, cw, ch = _cell(box, cols, rows, i)
        ly = y + 10 + k * 34
        d.line((cx + cw, cy + ch / 2, x + w + 20, ly + 10), fill=GOLD, width=2)
        d.text((x + w + 26, ly), label, font=font(22), fill=GOLD)


def hex_grid(img, data, box, cols, alpha=1.0, highlight=None):
    """Bytes as two-digit hex in MATRIX green, faded by alpha."""
    data = as_bytes(data)
    rows = math.ceil(len(data) / cols)
    d = ImageDraw.Draw(img)
    f = mono(max(8, int(box[3] / rows * 0.7)))
    for i, v in enumerate(data):
        cx, cy, cw, ch = _cell(box, cols, rows, i)
        colour = (highlight or {}).get(i, MATRIX)
        shade = 0.35 + 0.65 * (v / 255)
        fill = tuple(int(BG[k] + (colour[k] - BG[k]) * alpha * shade) for k in range(3))
        d.text((cx + cw / 2, cy + ch / 2), f"{v:02X}", font=f, fill=fill, anchor="mm")


def board(img, cells, box, diff=None, colour=INK, recent=None, dim_old=False):
    """A 10x20 Tetris playfield from 200 bytes; `diff` cells outlined RED,
    `recent` cells filled GOLD, older cells dimmed when `dim_old`."""
    cells = as_bytes(cells)
    x, y, w, h = box
    d = ImageDraw.Draw(img)
    d.rectangle((x - 3, y - 3, x + w + 2, y + h + 2), outline=DIM, width=2)
    cw, ch = w / 10, h / 20
    old = tuple(int(c * 0.45) for c in colour)
    for i, v in enumerate(cells[:200]):
        c, r = i % 10, i // 10
        cx, cy = x + c * cw, y + r * ch
        if v != PLAYFIELD_EMPTY:
            fill = GOLD if recent and i in recent else (old if dim_old else colour)
            d.rectangle((cx + 1, cy + 1, cx + cw - 1, cy + ch - 1), fill=fill)
        if diff and i in diff:
            d.rectangle((cx, cy, cx + cw, cy + ch), outline=RED, width=2)


def filled(cells):
    return {i for i, v in enumerate(as_bytes(cells)[:200]) if v != PLAYFIELD_EMPTY}


def heights_bars(img, heights, box, colour=GOLD):
    x, y, w, h = box
    d = ImageDraw.Draw(img)
    bw = w / len(heights)
    for i, v in enumerate(heights):
        bh = h * v / 20
        d.rectangle((x + i * bw + 1, y + h - bh, x + (i + 1) * bw - 2, y + h), fill=colour)
    d.line((x, y + h, x + w, y + h), fill=DIM, width=1)


def gauge(img, history, truth, box, threshold=0.5):
    """A scrolling strip: the readout's A output as a line over true presses as blocks."""
    x, y, w, h = box
    d = ImageDraw.Draw(img)
    d.rectangle((x, y, x + w, y + h), fill=PANEL)
    n = max(2, len(history))
    step = w / max(60, n)
    for i, pressed in enumerate(truth):
        if pressed:
            d.rectangle((x + i * step, y + h * 0.15, x + (i + 1) * step, y + h), fill=(60, 110, 70))
    ty = y + h - h * threshold
    for k in range(0, int(w), 14):
        d.line((x + k, ty, x + k + 7, ty), fill=DIM, width=1)
    pts = [(x + i * step, y + h - h * max(0.0, min(1.0, v))) for i, v in enumerate(history)]
    if len(pts) > 1:
        d.line(pts, fill=GOLD, width=3)


def strip(img, box, lanes, x_max, marker=None, colours=None):
    """The level left to right, one lane per list of jump xs, a marker at x."""
    x, y, w, h = box
    d = ImageDraw.Draw(img)
    d.rectangle((x, y, x + w, y + h), fill=PANEL)
    lh = h / max(1, len(lanes))
    for k, jumps in enumerate(lanes):
        colour = (colours or [GOLD] * len(lanes))[k]
        ly = y + k * lh
        d.line((x, ly + lh / 2, x + w, ly + lh / 2), fill=(50, 55, 70), width=1)
        for jx in jumps:
            px = x + w * min(1.0, jx / x_max)
            d.line((px, ly + 4, px, ly + lh - 4), fill=colour, width=2)
    if marker is not None:
        px = x + w * min(1.0, marker / x_max)
        d.line((px, y - 6, px, y + h + 6), fill=INK, width=2)


def bar_chart(img, box, rows, value_key, label_key, shown, se=None, show_se=False, fmt="{:.0%}", max_value=1.0):
    """Horizontal bars for the first `shown` rows, with ±se whiskers when `show_se`."""
    x, y, w, h = box
    d = ImageDraw.Draw(img)
    label_w = 360
    bh = h / max(1, len(rows))
    for i, row in enumerate(rows[:shown]):
        v = row[value_key]
        by = y + i * bh
        d.text((x + label_w - 12, by + bh / 2), row[label_key], font=font(20), fill=INK, anchor="rm")
        bw = (w - label_w - 80) * max(0.0, v) / max_value
        d.rectangle((x + label_w, by + bh * 0.2, x + label_w + bw, by + bh * 0.8), fill=row.get("colour", GOLD))
        label_x = x + label_w + bw + 8
        if show_se and se:
            lo = x + label_w + (w - label_w - 80) * max(0.0, v - se) / max_value
            hi = x + label_w + (w - label_w - 80) * min(max_value, v + se) / max_value
            d.line((lo, by + bh / 2, hi, by + bh / 2), fill=INK, width=2)
            for e in (lo, hi):
                d.line((e, by + bh * 0.3, e, by + bh * 0.7), fill=INK, width=2)
            label_x = max(label_x, hi + 8)
        d.text((label_x, by + bh / 2), fmt.format(v), font=font(18), fill=DIM, anchor="lm")


def lamp(img, centre, on, colour, label):
    cx, cy = centre
    d = ImageDraw.Draw(img)
    r = 34
    fill = colour if on else tuple(int(c * 0.2) for c in colour)
    d.ellipse((cx - r, cy - r, cx + r, cy + r), fill=fill, outline=DIM, width=2)
    d.text((cx, cy + r + 12), label, font=font(20), fill=INK if on else DIM, anchor="ma")


def arrow(img, a, b, colour=DIM, width=3):
    d = ImageDraw.Draw(img)
    d.line((a, b), fill=colour, width=width)
    ang = math.atan2(b[1] - a[1], b[0] - a[0])
    for s in (-0.5, 0.5):
        d.line((b, (b[0] - 14 * math.cos(ang + s), b[1] - 14 * math.sin(ang + s))), fill=colour, width=width)


# Three of 1-1's own colour sets, light to dark (NES palette entries):
# cloud, brick, question block and Mario.
SMB_PALETTES = [
    ((252, 252, 252), (60, 188, 252), (0, 88, 248)),
    ((252, 188, 176), (200, 76, 12), (136, 20, 0)),
    ((252, 160, 68), (228, 92, 16), (136, 20, 0)),
    ((252, 160, 68), (216, 40, 0), (136, 112, 0)),
]


def glyph_tiles(screens, digit_box, size=16):
    """The ones-digit tile of each lives screen past nine, once per colour set
    in SMB_PALETTES: a list of tile lists. The game draws them in three
    shades over black; each shade maps, by brightness, to a set's light, mid
    and dark colour, and black to the background."""
    x, y, w, h = digit_box
    luts = []
    for light, mid, dark in SMB_PALETTES:
        lut = np.zeros((4, 3), dtype=np.uint8)
        lut[0], lut[1], lut[2], lut[3] = BG, dark, mid, light
        luts.append(lut)
    shades = []
    for i in range(10, len(screens)):
        tile = np.asarray(screens[i])[y:y + 8, x:x + 8].astype(np.float32)
        lum = tile.mean(axis=2) / 255
        if lum.max() < 0.05:
            continue
        # Black, then the game's dark, mid and light shades (about 0.3, 0.6, 0.9).
        shades.append(np.digitize(lum, [0.05, 0.45, 0.75]))
    return [[Image.fromarray(lut[s]).resize((size, size), Image.NEAREST) for s in shades] for lut in luts]


def rain(img, sets, t, seed, speed=1.0, density=1.0):
    """Columns of falling tiles; each column has its own speed, phase and
    colour set (one of `sets`, lists of tiles) from `seed`, and a tile's glyph
    changes as it falls."""
    rng = np.random.default_rng(seed)
    size = sets[0][0].size[0]
    cols = W // size
    speeds = rng.uniform(80, 260, cols) * speed
    phases = rng.uniform(0, H, cols)
    lengths = rng.integers(6, 22, cols)
    picks = rng.integers(0, len(sets[0]), (cols, 64))
    keep = rng.random(cols)
    which = rng.integers(0, len(sets), cols)
    for c in range(cols):
        if keep[c] > density:
            continue
        tiles = sets[which[c]]
        head = (phases[c] + speeds[c] * t) % (H + lengths[c] * size)
        for k in range(lengths[c]):
            y = int(head - k * size)
            if 0 <= y < H:
                glyph = tiles[picks[c, (k + int(t * 8)) % 64] % len(tiles)]
                shade = 1.0 - k / lengths[c]
                if k > 0:
                    glyph = Image.eval(glyph, lambda v, s=shade: int(v * s))
                img.paste(glyph, (c * size, y))


def ripple(img, box, t, drops):
    """A tank seen from above: rings spread from each (time, fx, fy) drop
    (fractions of the box), and a ruler along the bottom reading their sum."""
    x0, y0, w, h = box
    tank = Image.new("RGB", (w, h), (20, 40, 70))
    d = ImageDraw.Draw(tank)
    x, y = 0, 0
    for (t0, fx, fy) in drops:
        age = t - t0
        cx, cy = x + fx * w, y + fy * h
        if -0.6 < age < 0:
            sy = cy + age * 300
            d.ellipse((cx - 10, sy - 10, cx + 10, sy + 10), fill=DIM)
        if age < 0:
            continue
        for k in range(4):
            r = age * 120 - k * 30
            if r <= 0:
                continue
            fade = max(0.0, 1 - age / 6) * (1 - k * 0.2)
            col = tuple(int(c * fade + 20 * (1 - fade)) for c in (140, 200, 255))
            d.ellipse((cx - r, cy - r, cx + r, cy + r), outline=col, width=3)
    ry = y + h - 30
    pts = []
    for k in range(0, w, 4):
        v = 0.0
        for (t0, fx, fy) in drops:
            age = t - t0
            if age <= 0:
                continue
            dist = math.hypot(x + k - (x + fx * w), ry - (y + fy * h))
            v += math.sin(dist / 18 - age * 6) * max(0.0, 1 - age / 6) * math.exp(-abs(dist - age * 120) / 80)
        pts.append((x + k, ry - 20 * v))
    d.line(pts, fill=GOLD, width=3)
    img.paste(tank, (x0, y0))


def fade(img, other, a):
    """A new image, `img` blended toward `other` by `a` in 0..1."""
    return Image.blend(img, other, max(0.0, min(1.0, a)))
