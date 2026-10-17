"""Chapter 9: the verdict, what we'd try with more hardware, and credits."""

from .. import draw
from ..model import Caption, Chapter, H, Shot, W

CREDITS = [
    ("Made with", "Greenstone, an NES emulator written in Rust."),
    ("Music", "Played on Greenstone's own sound chip."),
    ("Found along the way", "One of the chip's two square-wave channels had been silent."),
    ("Korobeiniki", "Traditional."),
    ("Minuet in G", "Christian Petzold, long attributed to Bach."),
    ("Everything else", "Original."),
    ("Super Mario Bros. and Tetris", "Nintendo's. We only poked their memory."),
]


def verdict(ctx, t, i):
    return draw.text_card(["Can Tetris play Mario? Sort of.",
                           "Four games of Tetris learn to make Mario jump about as well as a 2,048-unit echo state network.",
                           "But they don't remember anything a brain can use."])


def next_card(ctx, t, i):
    return draw.text_card(["With more hardware, we'd try:", "1. A task that needs memory.",
                           "2. Hundreds of Tetris games at once.", "3. More levels to test on."])


def credits(ctx, t, i):
    img = draw.blank()
    y0 = H - t * 60
    for k, (head, body) in enumerate(CREDITS):
        y = y0 + k * 110
        if -60 < y < H:
            draw.text(img, (W // 2, y), head, 24, draw.GOLD, anchor="ma")
            draw.text(img, (W // 2, y + 34), body, 32, draw.INK, anchor="ma")
    return img


CHAPTER = Chapter(9, "Verdict", "And what we'd do next", [
    Shot("verdict", 12, verdict),
    Shot("next", 14, next_card),
    Shot("credits", 22, credits),
])
