"""Chapter 5: three buggy builds, each replayed as it happened: the
controller leak, the clock that started when Mario did, and the search
that found Mario's position in disguise."""

from .. import draw
from ..model import Caption, Chapter, Shot

FADE_IN = 40


def card(lines, kicker):
    def shot(ctx, t, i):
        return draw.text_card(lines, kicker=kicker)
    return shot


def leak_pick(ctx, t, i):
    img = draw.blank()
    pick = ctx.takes["controller_leak"].meta["first_pick"]
    walk = ctx.takes["matrix_walk"].meta["steps"][60]
    w = bytes.fromhex(walk["window"])[:256]
    highlight = {pick["address"]: draw.RED} if t >= 6 else {}
    draw.byte_grid(img, w, (120, 60, 400, 400), 16, highlight=highlight)
    if t >= 6:
        draw.text(img, (620, 160), pick["describe"].split()[0], 96, draw.RED, face=draw.mono(96))
        draw.text(img, (620, 300), "the controller:", 30, draw.INK)
        draw.text(img, (620, 345), "last frame's buttons", 30, draw.INK)
    return img


def slow_stream(take, stream, slow, skip=FADE_IN):
    def shot(ctx, t, i):
        img = draw.blank()
        draw.nes(img, ctx.takes[take].frames(stream)[skip + i // slow], (256, 20, 768, 576))
        return img
    return shot


def fast_stream(take, stream, speed, label=None):
    def shot(ctx, t, i):
        img = draw.blank()
        draw.nes(img, ctx.takes[take].frames(stream)[FADE_IN + i * speed], (256, 20, 768, 576))
        if label:
            draw.text(img, (1050, 40), label, 30, draw.GOLD)
        return img
    return shot


def end_sting(take, stream, rate, skip=FADE_IN):
    """`wahwah` when a stream played at `rate` stream frames per film frame runs out."""
    def stings(ctx):
        n = ctx.takes[take].meta["streams"][stream]
        return [(max(0.0, (n - skip) / rate / 60), "wahwah")]
    return stings


def memo_features(ctx, t, i):
    img = draw.blank()
    feats = ctx.takes["memoriser"].meta["features"]
    draw.text(img, (200, 80), "the four numbers it picked", 26, draw.DIM)
    for n, f in enumerate(feats):
        y = 150 + n * 80
        colour = draw.RED if f["position"] and t >= 6 else draw.INK
        draw.text(img, (200, y), f["describe"], 44, colour, face=draw.mono(44))
        if f["position"] and t >= 6:
            draw.text(img, (560, y + 8), "← a number minus Mario's x", 30, draw.RED)
    return img


def memo_caption(ctx):
    feats = ctx.takes["memoriser"].meta["features"]
    if feats[0]["position"]:
        return "Its first pick was a number minus Mario's x: where Mario is, in disguise."
    if any(f["position"] for f in feats):
        return "One of its picks was a number minus Mario's x: where Mario is, in disguise."
    return "It picked numbers that change with where Mario is."


def memo_drive(ctx, t, i):
    img = draw.blank()
    frames = ctx.takes["memoriser"].frames("drive")
    k = FADE_IN + (i if t < 6 else 360 + (i - 360) * 8)
    draw.nes(img, frames[k], (256, 20, 768, 576))
    if t >= 6:
        draw.text(img, (1050, 40), "×8", 36, draw.GOLD)
    return img


def memo_stuck_caption(ctx):
    m = ctx.takes["memoriser"].meta
    if m["death_x"] is None:
        return "So it stood in front of this pipe for the rest of the run."
    return "It dies at x {memoriser.death_x}."


CLOCK_SPEED = 2

CHAPTER = Chapter(5, "Three ways we fooled ourselves", "Lab notes", [
    Shot("leak_card", 6, card(["The brain could see its own hands."], "Mistake 1")),
    Shot("leak_pick", 12, leak_pick, [
        Caption(0, "We asked a search for the single most useful number in Mario's memory."),
        Caption(6, "It picked {controller_leak.first_pick.describe}: the controller. The brain was reading the buttons it had just pressed."),
    ], takes=("controller_leak", "matrix_walk")),
    Shot("leak_drive", 12, slow_stream("controller_leak", "drive", 2), [
        Caption(0, "Every score we had was flattered by it, and the brain still died at the first Goomba."),
        Caption(8, ""),
    ], takes=("controller_leak",), stings=end_sting("controller_leak", "drive", 0.5)),
    Shot("leak_fix", 6, card(["The fix: hide the controller from the brain."], "Fixed")),
    Shot("clock_card", 6, card(["A brain that only knows the time."], "Mistake 2")),
    Shot("clock_buggy", 16, fast_stream("clock_late_start", "buggy", CLOCK_SPEED, "×2"), [
        Caption(0, "This brain knows nothing but how many steps ago Mario started moving."),
        Caption(7, "It still reaches x {clock_late_start.buggy.max_x}: every example started at step 0, so it learned the timing by heart."),
    ], takes=("clock_late_start",), stings=end_sting("clock_late_start", "buggy", CLOCK_SPEED)),
    Shot("clock_fixed", 10, slow_stream("clock_late_start", "fixed", 1), [
        Caption(0, "Count from the start of the level instead, and the timing stops lining up. It dies at x {clock_late_start.fixed.max_x}."),
    ], takes=("clock_late_start",)),
    Shot("clock_fix", 6, card(["The fix: count time from the level's start,", "and start the examples at different moments."], "Fixed")),
    Shot("memo_card", 6, card(["The search that found a shortcut."], "Mistake 3")),
    Shot("memo_score", 12, memo_features, [
        Caption(0, "We let a search pick four numbers. They scored {memoriser.held_out_onsets:.0%} on runs it hadn't seen."),
        Caption(6, memo_caption),
    ], takes=("memoriser",)),
    Shot("memo_drive", 14, memo_drive, [
        Caption(0, "It had learned where the jumps are in this level, not why."),
        Caption(7, memo_stuck_caption),
    ], takes=("memoriser",), stings=[(7.0, "wahwah")]),
    Shot("memo_fix", 8, card(["The fix: never let the brain know where Mario is.", "Make it read what's around him."], "Fixed")),
])
