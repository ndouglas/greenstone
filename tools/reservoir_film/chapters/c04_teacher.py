"""Chapter 4: the search-made examples disagree with each other, so we wrote
a teacher that jumps for reasons, and a score that asks the right question."""

from .. import draw
from ..model import Caption, Chapter, Shot

# The first frames of a stream are the level fading in from black.
FADE_IN = 40
LANE_COLOURS = [draw.GOLD, draw.RED, draw.GREEN, draw.REGION["wall"], draw.REGION["enemies"], draw.INK]


def onsets(steps):
    return [s["x"] for k, s in enumerate(steps) if s["a"] and (k == 0 or not steps[k - 1]["a"])]


def six(ctx, t, i):
    img = draw.blank()
    take = ctx.takes["coin_flip_teachers"]
    i += FADE_IN
    draw.nes(img, take.frames("demo0")[i], (448, 10, 384, 360))
    demos = take.meta["demos"]
    steps0 = demos[0]["steps"]
    x = steps0[min(len(steps0) - 1, i // 4)]["x"]
    draw.text(img, (40, 380), "six examples; a tick wherever each one jumped", 22, draw.DIM)
    draw.strip(img, (40, 420, 1200, 180), [onsets(d["steps"]) for d in demos], 3300, marker=x, colours=LANE_COLOURS)
    return img


def copy(ctx, t, i):
    img = draw.blank()
    demos = ctx.takes["coin_flip_teachers"].meta["demos"]
    # Zoom on the stretch of 1-1 from x 400 to 1200, where they disagree most visibly.
    lanes = [[x - 400 for x in onsets(d["steps"]) if 400 <= x <= 1200] for d in demos]
    draw.strip(img, (40, 120, 1200, 360), lanes, 800, colours=LANE_COLOURS)
    draw.text(img, (40, 80), "x 400 to 1200, close up", 22, draw.DIM)
    return img


LAMPS = [("enemy", "enemy close", draw.REGION["enemies"]), ("pit", "pit ahead", draw.REGION["floor"]),
         ("wall", "wall ahead", draw.REGION["wall"])]


def rule(ctx, t, i):
    img = draw.blank()
    take = ctx.takes["rule_tripwires"]
    draw.nes(img, take.frames("smb")[i], (40, 40, 512, 480))
    steps = take.meta["steps"]
    step = steps[min(len(steps) - 1, i // 4)]
    shown = 1 + (t >= 10) + (t >= 14)
    for n, (key, label, colour) in enumerate(LAMPS[:shown]):
        draw.lamp(img, (700 + n * 170, 180), step["tripwires"][key], colour, label)
    draw.lamp(img, (870, 400), step["buttons"] & 1 == 1, draw.GOLD, "JUMP")
    return img


def score(ctx, t, i):
    """An illustration: the teacher's jump starts, and an example brain that misses one in three."""
    img = draw.blank()
    steps = ctx.takes["rule_tripwires"].meta["steps"]
    a = [s["buttons"] & 1 == 1 for s in steps]
    starts = [k for k in range(len(a)) if a[k] and (k == 0 or not a[k - 1])]
    d = draw.ImageDraw.Draw(img)
    x0, x1 = 60, 1220
    span = max(1, len(a))
    draw.text(img, (60, 150), "teacher starts a jump", 22, draw.GOLD)
    draw.text(img, (60, 330), "an example brain", 22, draw.DIM)
    shown = int(len(starts) * min(1.0, t / 6))
    for n, k in enumerate(starts[:shown]):
        x = x0 + (x1 - x0) * k / span
        d.line((x, 190, x, 250), fill=draw.GOLD, width=3)
        hit = n % 3 != 2
        if hit:
            d.rectangle((x - 2, 360, x + 12, 400), fill=draw.INK)
        ring = draw.GREEN if hit else draw.RED
        d.ellipse((x - 12, 268, x + 12, 292), outline=ring, width=3)
    return img


CHAPTER = Chapter(4, "The teacher was flipping coins", "Where the examples came from", [
    Shot("six", 20, six, [
        Caption(0, "To learn, a brain needs examples. Ours came from a search that found ways through the level."),
        Caption(9, "Here are six. Each tick is a jump."),
        Caption(14, "At the same spot, two of them disagree about jumping {coin_flip_teachers.disagree:.0%} of the time."),
    ], takes=("coin_flip_teachers",)),
    Shot("copy", 8, copy, [
        Caption(0, "Copy all six? No brain can, because they don't agree with each other."),
    ], takes=("coin_flip_teachers",)),
    Shot("rule", 26, rule, [
        Caption(0, "So we wrote a teacher that jumps for reasons."),
        Caption(6, "Jump when an enemy is close."),
        Caption(10, "Or a pit is ahead."),
        Caption(14, "Or a wall."),
        Caption(18, "It clears the level, the same way every time."),
    ], takes=("rule_tripwires",)),
    Shot("score", 12, score, [
        Caption(0, "The score that matters: when the teacher started a jump, did the brain start one too?"),
    ], takes=("rule_tripwires",)),
])
