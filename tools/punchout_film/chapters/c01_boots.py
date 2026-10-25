"""Chapter 1: Punch-Out!! boots on Greenstone, the mapper built for it, and
the mainstream pipeline we're about to take apart."""

from reservoir_film import draw
from reservoir_film.model import Caption, Chapter, Shot

from .common import diagram_shot

# Skip the black frames before the first screen.
BOOT_START = 200


def boot(ctx, t, i):
    img = draw.blank()
    frames = ctx.takes["boot"].frames("punchout")
    draw.nes(img, frames[min(BOOT_START + i, frames.n - 1)], (352, 20, 576, 540))
    return img


CHAPTER = Chapter(1, "The cartridge boots", "And the box diagram we'll take apart", [
    Shot("boot", 24, boot, [
        Caption(0, "This is Mike Tyson's Punch-Out!!, running on Greenstone, an NES emulator written in Rust."),
        Caption(6, "To run it at all, Greenstone needed a new mapper chip: the MMC2."),
        Caption(12, "It swaps in a different bank of picture tiles mid-frame, whenever the screen draws one of two trigger tiles."),
        Caption(19, "We built it for this experiment."),
    ], takes=("boot",)),
    Shot("usual", 20, diagram_shot(1, focus=[(5, "see"), (7, "remember"), (8.5, "decide"), (10, "learn"),
                                            (15, None)]), [
        Caption(0, "Here is roughly how a mainstream agent would learn to box."),
        Caption(5, "A conv net reads the pixels, a recurrent net remembers, and a policy picks the move."),
        Caption(10, "Every part is trained from a reward, over millions of frames."),
        Caption(15, "We're going to swap each box for something stranger, and see what happens."),
    ]),
])
