"""Chapter 6: how much memory a brain needs. Seven brains against Von
Kaiser in Defence, side by side, and the sweep's numbers."""

from reservoir_film import draw
from reservoir_film.model import Caption, Chapter, Shot

from .common import hits_so_far, plural

SWEEP = ["stack1", "stack2", "stack4", "stack8", "echo0.2", "echo0.5", "echo0.9"]
# All seven at four times speed, so the short-memory brains go down in view.
SPEED = 4


def take_name(kind):
    return f"p1-{kind}-von-kaiser"


def grid(ctx, t, i):
    img = draw.blank()
    for k, kind in enumerate(SWEEP):
        take = ctx.takes[take_name(kind)]
        frames = take.frames("punchout")
        f = min(i * SPEED, frames.n - 1)
        col, row = k % 4, k // 4
        x, y = 40 + col * 305, 20 + row * 300
        draw.nes(img, frames[f], (x, y, 285, 267))
        over = i * SPEED >= frames.n
        label = f"{kind}  {plural(hits_so_far(take, f), 'hit')}"
        if over:
            secs = take.meta["score"]["steps"] * 4 / 60
            label = f"{kind}  down at {int(secs // 60)}:{int(secs % 60):02d}"
        draw.text(img, (x, y + 271), label, 18, draw.RED if over else draw.INK, bold=True)
    return img


def chart(ctx, t, i):
    img = draw.blank()
    rows = [{"label": kind, "v": ctx.numbers["sweep"][kind.replace(".", "_")]["von-kaiser"]["hpm"]} for kind in SWEEP]
    rows.append({"label": "standing still", "v": ctx.numbers["idle"]["von-kaiser"]["hpm"], "colour": draw.DIM})
    draw.text(img, (60, 50), "hits a minute against Von Kaiser, Defence (lower is better)", 26, draw.DIM)
    draw.bar_chart(img, (60, 100, 1160, 470), rows, "v", "label", len(rows), fmt="{:.1f}", max_value=40)
    return img


CHAPTER = Chapter(6, "How much memory?", "Seven brains, one opponent", [
    Shot("grid", 30, grid, [
        Caption(0, "How much memory does a brain need? Each punch is telegraphed a fraction of a second before it lands."),
        Caption(7, "Seven brains against Von Kaiser: stacks of 1 to 8 glimpses, and echoes with three leaks."),
        Caption(14, "Von Kaiser beats them all in the end. The difference is how long Mac lasts."),
        Caption(22, "Four times speed. By now only two of the echoes are still standing."),
    ], takes=tuple(take_name(k) for k in SWEEP)),
    Shot("chart", 30, chart, [
        Caption(0, "Crisp history helps. stack8 takes {numbers.sweep.stack8.von-kaiser.hpm:.1f} hits a minute, stack1 {numbers.sweep.stack1.von-kaiser.hpm:.1f}."),
        Caption(7, "Long, blurry memory doesn't. echo0.2 remembers longest and is the worst echo here, at {numbers.sweep.echo0_2.von-kaiser.hpm:.1f}."),
        Caption(14, "Against Bald Bull, an opponent none of them trained on, the order turns over."),
        Caption(20, "And a blind spot: a brain sees one frame in four, so a tell shorter than that can fall in the gap."),
    ]),
])
