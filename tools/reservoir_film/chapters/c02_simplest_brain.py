"""Chapter 2: the simplest brain, one weight per number; it watches a run,
then drives, and dies at the first Goomba."""

import numpy as np

from .. import draw
from ..model import Caption, Chapter, Shot

GRID = (600, 40, 368, 529)
# The first frames of each stream are the level fading in from black.
SKIP_FADE_IN = 40


def weight_colours(weights):
    w = np.asarray(weights[1:], dtype=float)
    scale = np.abs(w).max() or 1.0
    out = {}
    for i, v in enumerate(w):
        a = min(1.0, abs(v) / scale) ** 0.5
        base = draw.GREEN if v > 0 else draw.RED
        out[i] = tuple(int(draw.BG[k] + (base[k] - draw.BG[k]) * a) for k in range(3))
    return out


def weights_meta(ctx):
    return ctx.takes["simple_brain_drives"].meta


def sum_bar(img, level, box=(1060, 60, 60, 480)):
    x, y, w, h = box
    d = draw.ImageDraw.Draw(img)
    d.rectangle((x, y, x + w, y + h), outline=draw.DIM, width=2)
    fill = draw.GREEN if level >= 0.5 else draw.GOLD
    d.rectangle((x + 3, y + h - h * min(1.0, level), x + w - 3, y + h - 3), fill=fill)
    ty = y + h / 2
    d.line((x - 14, ty, x + w + 14, ty), fill=draw.INK, width=2)
    draw.text(img, (x + w + 20, ty), "jump", 22, draw.INK, anchor="lm")


def weights(ctx, t, i):
    img = draw.blank()
    meta = weights_meta(ctx)
    draw.byte_grid(img, bytes(360), GRID, 16, colours=weight_colours(meta["weights_a"]))
    draw.text(img, (GRID[0] - 20, GRID[1] + 10), "one weight per number", 24, draw.DIM, anchor="ra")
    draw.text(img, (GRID[0] - 20, GRID[1] + 50), "green: towards jumping", 22, draw.GREEN, anchor="ra")
    draw.text(img, (GRID[0] - 20, GRID[1] + 84), "red: away from it", 22, draw.RED, anchor="ra")
    if t >= 10:
        steps = meta["demo_steps"]
        k = min(len(steps) - 1, 60 + int((t - 10) * 8))
        sum_bar(img, max(0.0, steps[k]["a_out"]))
    return img


def training(ctx, t, i):
    """The weights settling from noise to their fitted values."""
    img = draw.blank()
    final = np.asarray(weights_meta(ctx)["weights_a"], dtype=float)
    noise = np.random.default_rng(3).normal(0, np.abs(final).max(), final.shape)
    a = min(1.0, t / 7)
    draw.byte_grid(img, bytes(360), GRID, 16, colours=weight_colours(list(final * a + noise * (1 - a))))
    draw.text(img, (GRID[0] - 20, GRID[1] + 10), "training" if a < 1 else "trained", 30, draw.GOLD, anchor="ra")
    return img


def gauge(ctx, t, i):
    img = draw.blank()
    meta = weights_meta(ctx)
    i += SKIP_FADE_IN
    draw.nes(img, ctx.takes["simple_brain_drives"].frames("demo")[i], (40, 40, 512, 480))
    steps = meta["demo_steps"]
    k = min(len(steps) - 1, i // 4)
    lo = max(0, k - 59)
    draw.panel(img, (600, 40, 640, 300), "how much it wants to jump")
    draw.gauge(img, [s["a_out"] for s in steps[lo:k + 1]], [s["buttons"] & 1 == 1 for s in steps[lo:k + 1]],
               (620, 80, 600, 240))
    draw.text(img, (620, 360), "green: when the teacher jumped", 22, draw.GREEN)
    draw.text(img, (620, 394), "gold line: the brain", 22, draw.GOLD)
    draw.text(img, (620, 428), "dashes: the line it must cross", 22, draw.DIM)
    return img


DRIVE_SLOW = 3


def drives(ctx, t, i):
    img = draw.blank()
    draw.nes(img, ctx.takes["simple_brain_drives"].frames("drive")[SKIP_FADE_IN // DRIVE_SLOW + i // DRIVE_SLOW], (256, 20, 768, 576))
    return img


def death_sting(ctx):
    n = ctx.takes["simple_brain_drives"].meta["streams"]["drive"]
    return [((n - SKIP_FADE_IN // DRIVE_SLOW) * DRIVE_SLOW / 60, "death")] if weights_meta(ctx)["death_x"] is not None else []


def death_caption(ctx):
    if weights_meta(ctx)["death_x"] is not None:
        return "It dies at x {simple_brain_drives.death_x}."
    return "It never gets anywhere."


CHAPTER = Chapter(2, "The simplest brain", "One weight per number", [
    Shot("weights", 18, weights, [
        Caption(0, "The simplest brain gives every number a weight."),
        Caption(5, "Multiply each number by its weight and add them all up."),
        Caption(10, "If the total is over one half, press jump."),
    ], takes=("simple_brain_drives",)),
    Shot("training", 10, training, [
        Caption(0, "Training means picking the weights that best copy a teacher's examples."),
        Caption(5, "Green weights push toward jumping. Red ones push away."),
    ], takes=("simple_brain_drives",)),
    Shot("gauge", 20, gauge, [
        Caption(0, "Here it watches a run it wasn't trained on. The line is how much it wants to jump."),
        Caption(9, "It rises near the jumps, but rarely crosses the line."),
        Caption(15, "A weighted sum can't say 'jump when the Goomba is close'."),
    ], takes=("simple_brain_drives",)),
    Shot("drives", 14, drives, [
        Caption(0, "Now let it play."),
        Caption(8, death_caption),
    ], takes=("simple_brain_drives",), stings=death_sting),
])
