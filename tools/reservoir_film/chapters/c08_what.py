"""Chapter 8: two theories of what Tetris adds, both wrong, and the answer:
the brain reads where this step's pieces landed."""

from .. import draw
from ..model import Caption, Chapter, Shot

REGIONS = ["mario", "enemies", "floor", "wall"]


def by_key(ctx, key):
    return next(r for r in ctx.numbers["rows"] if r["key"] == key)


def theory1(ctx, t, i):
    img = draw.blank()
    steps = ctx.takes["heights"].meta["steps"]
    k = min(len(steps) - 1, 40 + i // 8)
    step = steps[k]
    for n, region in enumerate(REGIONS):
        x = 60 + n * 150
        draw.board(img, step["boards"][n], (x, 40, 110, 220), colour=draw.REGION[region])
        draw.heights_bars(img, step["heights"][n], (x, 290, 110, 100), colour=draw.REGION[region])
    draw.text(img, (60, 410), "the stacks' heights", 22, draw.DIM)
    if t >= 14:
        rows = [dict(by_key(ctx, "bank-tidy"), label="the whole boards"),
                dict(by_key(ctx, "bank-tidy-heights"), label="only the heights")]
        draw.text(img, (700, 60), "jumps started on 4-1", 22, draw.DIM)
        draw.bar_chart(img, (660, 100, 600, 160), rows, "test", "label", shown=2)
    return img


def theory2(ctx, t, i):
    img = draw.blank()
    steps = ctx.takes["hold_bit"].meta["steps"]
    lo = min(len(steps) - 120, i // 6)
    d = draw.ImageDraw.Draw(img)
    for row, (key, label) in enumerate([("a", "jumping now"), ("held", "jumping a step ago")]):
        y = 80 + row * 80
        draw.text(img, (60, y), label, 22, draw.INK)
        for j, s in enumerate(steps[lo:lo + 120]):
            if s[key]:
                x = 330 + j * 7.5
                d.rectangle((x, y, x + 7, y + 30), fill=draw.GOLD if key == "a" else draw.DIM)
    if t >= 14:
        hold = ctx.numbers["hold"]
        rows = [{"label": "no Tetris", "v": hold["bank-searched-onehot"]["mean_x"]},
                {"label": "no Tetris, plus the bit", "v": hold["bank-searched-hold"]["mean_x"]},
                {"label": "through Tetris", "v": hold["bank-searched-tetris"]["mean_x"], "colour": draw.REGION["wall"]}]
        draw.text(img, (60, 270), "how far it drives, on average", 22, draw.DIM)
        draw.bar_chart(img, (60, 310, 1100, 200), rows, "v", "label", shown=3, fmt="x {:.0f}", max_value=1200)
    return img


def answer(ctx, t, i):
    img = draw.blank()
    steps = ctx.takes["echo_twins"].meta["steps"]
    k = min(len(steps) - 1, 1 + i // 12)
    now, before = steps[k]["a"], steps[k - 1]["a"]
    recent = draw.filled(now) - draw.filled(before)
    draw.board(img, now, (520, 30, 240, 480), recent=recent, dim_old=True)
    draw.text(img, (800, 80), "gold: where this step's pieces landed", 24, draw.GOLD)
    draw.text(img, (800, 120), "dim: everything older", 24, draw.DIM)
    return img


CHAPTER = Chapter(8, "So what is Tetris doing?", "Two theories", [
    Shot("theory1", 20, theory1, [
        Caption(0, "Theory one: Tetris remembers how much happened lately, in how tall the stacks are."),
        Caption(8, "Give a brain only the heights, and test it."),
        Caption(14, "It almost never jumps when the teacher does. The heights aren't it."),
    ], takes=("heights",)),
    Shot("theory2", 20, theory2, [
        Caption(0, "Theory two: Tetris just remembers 'I was jumping a moment ago'."),
        Caption(8, "Give a brain without Tetris that one bit of memory instead."),
        Caption(14, "The bit makes it worse, at timing and at distance. Tetris isn't one bit."),
    ], takes=("hold_bit",)),
    Shot("answer", 24, answer, [
        Caption(0, "What the brain really reads is where this step's pieces just landed."),
        Caption(8, "Tetris does keep a trace of the past, but too scrambled for a weighted sum to use."),
        Caption(16, "So it's a scrambler, not a memory, and only as good as the numbers we feed it."),
    ], takes=("echo_twins",)),
])
