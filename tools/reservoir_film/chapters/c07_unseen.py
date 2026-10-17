"""Chapter 7: trained on two levels, tested on a third that nothing ever saw."""

from .. import draw
from ..model import Caption, Chapter, Shot

FADE_IN = 40


def why(ctx, t, i):
    return draw.text_card(["Trained on 1-1 and 6-1.", "Tested on 4-1."])


def teacher(ctx, t, i):
    img = draw.blank()
    draw.nes(img, ctx.takes["unseen_4_1"].frames("teacher")[FADE_IN + i], (256, 20, 768, 576))
    return img


def bars(ctx, t, i):
    img = draw.blank()
    n = ctx.numbers
    rows = [dict(r) for r in sorted(n["rows"], key=lambda r: -r["test"])]
    for r in rows:
        if "Tetris" in r["label"] and "no Tetris" not in r["label"]:
            r["colour"] = draw.REGION["wall"]
    shown = min(len(rows), 1 + int(t / 1.6))
    draw.text(img, (40, 20), "jumps started when the teacher started one, on 4-1", 22, draw.DIM)
    draw.bar_chart(img, (40, 60, 1200, 520), rows, "test", "label", shown=shown, se=n["test_se"], show_se=t >= 20)
    return img


def clock(ctx, t, i):
    img = draw.blank()
    draw.nes(img, ctx.takes["unseen_4_1"].frames("clock")[FADE_IN + i], (256, 20, 768, 576))
    return img


CHAPTER = Chapter(7, "A level it never saw", "World 4-1", [
    Shot("why", 12, why, [
        Caption(0, "Doing well on a level you trained on proves little. You might have memorised it."),
        Caption(6, "So we trained on two levels and tested on a third: World 4-1."),
    ]),
    Shot("teacher_4_1", 12, teacher, [
        Caption(0, "The teacher plays 4-1. Every brain has to guess when it would jump."),
    ], takes=("unseen_4_1",)),
    Shot("bars", 30, bars, [
        Caption(0, "How often each brain started a jump when the teacher did, on the level it never saw:"),
        Caption(10, ""),
        Caption(20, "With only {numbers.test_onsets} jumps to go on, the top four are a tie."),
    ]),
    Shot("clock_joke", 10, clock, [
        Caption(0, "And the clock? It reached x {unseen_4_1.clock_max_x} by jumping at the wrong times. 4-1 is forgiving."),
    ], takes=("unseen_4_1",)),
])
