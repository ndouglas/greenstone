"""Chapter 11: the finished diagram, the verdict, what we'd try next, and
credits."""

from reservoir_film import draw
from reservoir_film.model import Caption, Chapter, H, Shot, W

from .common import diagram_shot

CREDITS = [
    ("Made with", "Greenstone, an NES emulator written in Rust."),
    ("Music", "Played on Greenstone's own sound chip."),
    ("Korobeiniki", "Traditional."),
    ("Minuet in G", "Christian Petzold, long attributed to Bach."),
    ("Everything else", "Original."),
    ("Mike Tyson's Punch-Out!! and Tetris", "Nintendo's. We only read their screens and poked their memory."),
]


def verdict(ctx, t, i):
    return draw.text_card(["Can a dice roll and a weighted sum box?",
                           "It dodges. It lands knockdowns. It never wins.",
                           "Its teacher wins every Fight."], kicker="Verdict")


def next_card(ctx, t, i):
    return draw.text_card(["Widen the bank's window, so Tetris gets a fair test.",
                           "Feed every frame of a step, not one in four."], kicker="Next time")


def credits(ctx, t, i):
    img = draw.blank()
    y0 = H - t * 60
    for k, (head, body) in enumerate(CREDITS):
        y = y0 + k * 110
        if -60 < y < H:
            draw.text(img, (W // 2, y), head, 24, draw.GOLD, anchor="ma")
            draw.text(img, (W // 2, y + 34), body, 32, draw.INK, anchor="ma")
    return img


CHAPTER = Chapter(11, "Verdict", "What each strange part did", [
    Shot("diagram", 30, diagram_shot(11, focus=[(0, "see"), (6, "remember"), (14, "decide"), (20, "learn"),
                                                (26, None)]), [
        Caption(0, "Seeing by dice roll worked. A random projection was enough to dodge."),
        Caption(6, "For memory, a short crisp stack beat long blurry echoes. The Tetris bank never got a fair test."),
        Caption(14, "One ridge solve came close to perfect in Counter."),
        Caption(20, "In Fight, the teacher knocks out everyone it meets. Its students never win."),
        Caption(26, "Odd, mostly ineffective, and fun."),
    ]),
    Shot("verdict", 12, verdict),
    Shot("next", 10, next_card),
    Shot("credits", 16, credits),
])
