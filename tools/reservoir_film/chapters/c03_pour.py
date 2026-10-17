"""Chapter 3: reservoir computing, then two ways of pouring Mario into
Tetris, and the echo between twin boards."""

from .. import draw
from ..model import Caption, Chapter, Shot

DROPS = [(1, 0.3, 0.4), (4, 0.6, 0.5), (5, 0.45, 0.3), (9, 0.7, 0.35), (12, 0.2, 0.6)]


def ripple(ctx, t, i):
    img = draw.blank()
    draw.ripple(img, (140, 30, 1000, 540), t, DROPS)
    return img


def why_tetris(ctx, t, i):
    img = draw.blank()
    draw.nes(img, ctx.takes["placements"].frames("tetris")[60 + int(t * 6)], (400, 20, 480, 450))
    draw.text(img, (640, 500), "also 2,048 bytes", 28, draw.GOLD, anchor="ma")
    return img


def step_of(ctx, take, i, per):
    steps = ctx.takes[take].meta["steps"]
    return min(len(steps) - 1, i // per), steps


def paste(ctx, t, i):
    img = draw.blank()
    k, steps = step_of(ctx, "paste_board", i, 8)
    draw.nes(img, ctx.takes["paste_board"].frames("smb")[k], (30, 60, 384, 360))
    w = bytes.fromhex(steps[k]["window"])[:160]
    lit = {j: draw.GOLD for j, v in enumerate(w) if v >= 0xC0}
    draw.text(img, (460, 30), "bytes ≥ 192", 22, draw.DIM)
    draw.byte_grid(img, w, (460, 60, 200, 320), 10, colours={**{j: draw.PANEL for j in range(160)}, **lit})
    draw.arrow(img, (680, 220), (760, 220))
    draw.text(img, (790, 30), "the Tetris board", 22, draw.DIM)
    draw.board(img, steps[k]["board"], (790, 60, 200, 400))
    return img


def screen_lies(ctx, t, i):
    img = draw.blank()
    k, steps = step_of(ctx, "paste_board", i, 6)
    draw.nes(img, ctx.takes["paste_board"].frames("tetris")[k], (80, 60, 480, 450))
    draw.text(img, (320, 520), "the screen", 28, draw.DIM, anchor="ma")
    draw.board(img, steps[k]["board"], (800, 80, 220, 440))
    draw.text(img, (910, 540), "the memory", 28, draw.GOLD, anchor="ma")
    return img


FEATURES = ["nearest enemy", "next enemy", "Mario's height", "Mario's speed"]


def placements(ctx, t, i):
    img = draw.blank()
    k, steps = step_of(ctx, "placements", i, 12)
    step = steps[k]
    draw.nes(img, ctx.takes["placements"].frames("smb")[k], (30, 40, 384, 360))
    for n, (name, (col, rot)) in enumerate(zip(FEATURES, step["placements"])):
        y = 70 + n * 90
        draw.text(img, (450, y), name, 22, draw.INK)
        for c in range(10):
            x = 450 + c * 26
            fill = draw.GOLD if c == col else draw.PANEL
            draw.ImageDraw.Draw(img).rectangle((x, y + 34, x + 22, y + 56), fill=fill)
        draw.text(img, (720, y + 34), f"turn {rot}", 20, draw.DIM)
    draw.board(img, step["board"], (900, 60, 220, 440))
    return img


def echo(ctx, t, i):
    img = draw.blank()
    meta = ctx.takes["echo_twins"].meta
    k, steps = step_of(ctx, "echo_twins", i, 11)
    a, b = steps[k]["a"], steps[k]["b"]
    diff = draw.filled(a) ^ draw.filled(b)
    draw.board(img, a, (240, 40, 220, 440), diff=diff)
    draw.board(img, b, (560, 40, 220, 440), diff=diff, colour=draw.REGION["wall"])
    draw.text(img, (350, 495), "board A", 22, draw.DIM, anchor="ma")
    draw.text(img, (670, 495), "board B", 22, draw.DIM, anchor="ma")
    label = "one piece moved" if k == meta["perturbed_step"] else f"step {k}"
    draw.text(img, (850, 60), label, 28, draw.GOLD if k == meta["perturbed_step"] else draw.INK)
    draw.text(img, (850, 110), f"{steps[k]['diff']} cells differ", 28, draw.RED)
    diffs = [s["diff"] for s in steps[:k + 1]]
    peak = max(1, max(s["diff"] for s in steps))
    pts = [(850 + j * 3, 470 - 250 * d / peak) for j, d in enumerate(diffs)]
    if len(pts) > 1:
        draw.ImageDraw.Draw(img).line(pts, fill=draw.RED, width=2)
    return img


def echo_caption(ctx):
    if ctx.takes["echo_twins"].meta["agree_again"] is None:
        return "The difference never washes out. Tetris keeps it, scrambled."
    return "The boards agree again after {echo_twins.agree_again} steps."


CHAPTER = Chapter(3, "Pour Mario into Tetris", "Reservoir computing", [
    Shot("ripple", 20, ripple, [
        Caption(0, "Scientists have computed with buckets of water."),
        Caption(5, "Drop numbers in as stones. The ripples mix them up in complicated ways."),
        Caption(12, "A simple brain reads the ripples instead of the stones. That's reservoir computing."),
    ]),
    Shot("why_tetris", 10, why_tetris, [
        Caption(0, "Tetris is a messy system that lives in 2,048 bytes too."),
        Caption(5, "What if Tetris were the bucket?"),
    ], takes=("placements",)),
    Shot("paste", 18, paste, [
        Caption(0, "First try: paste Mario's numbers straight onto the Tetris board."),
        Caption(7, "Numbers of 192 or more become blocks. There are only a handful."),
        Caption(13, "So Tetris has almost nothing to work with."),
    ], takes=("paste_board",)),
    Shot("screen_lies", 12, screen_lies, [
        Caption(0, "And the Tetris screen lies: it only redraws when a piece lands."),
        Caption(6, "The board in memory is the real one, so that's the one we draw."),
    ], takes=("paste_board",)),
    Shot("placements", 24, placements, [
        Caption(0, "Second try: pick four numbers about Mario."),
        Caption(5, "How far the nearest enemy is. The next one. Mario's height. His speed."),
        Caption(12, "Each number picks where a piece lands and which way it turns."),
        Caption(18, ""),
    ], takes=("placements",)),
    Shot("echo", 22, echo, [
        Caption(0, "How long does Tetris remember? Feed two boards the same pieces but one."),
        Caption(8, "Red shows where they differ."),
        Caption(13, echo_caption),
    ], takes=("echo_twins",)),
])
