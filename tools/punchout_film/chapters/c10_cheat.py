"""Chapter 10: the cheat reads the game's memory, and does worse than the
brains that read the screen."""

from reservoir_film import draw
from reservoir_film.model import Caption, Chapter, Shot

from .common import diagram_shot, pair

SPEED = 2


def versus(ctx, t, i):
    img = draw.blank()
    pair(img, [ctx.takes["p3-cheat-von-kaiser"], ctx.takes["p3-stack4-von-kaiser"]], i * SPEED,
         ["the cheat: reads RAM", "stack4: reads the screen"])
    return img


CHAPTER = Chapter(10, "The cheat", "Who reads the game's mind", [
    Shot("yardstick", 14, diagram_shot(10, swaps=[("yardstick", 2, 3)], focus="yardstick"), [
        Caption(0, "One more brain: the cheat. It skips the screen and reads all 2,048 bytes of the game's memory."),
        Caption(7, "Everything the game knows is in there. It should be the best any brain can do."),
    ]),
    Shot("versus", 28, versus, [
        Caption(0, "Fight, against Von Kaiser, at double speed. Left, the cheat. Right, stack4."),
        Caption(6, "The cheat takes {numbers.phase3.cheat.r4.von-kaiser.hpm:.1f} hits a minute. stack4 takes {numbers.phase3.stack4.r4.von-kaiser.hpm:.1f}."),
        Caption(13, "Raw bytes under one linear readout take more hits than stack4 and echo0.5 from every trained opponent, in every phase."),
        Caption(21, "The yardstick turned out shorter than the things it measured."),
    ], takes=("p3-cheat-von-kaiser", "p3-stack4-von-kaiser")),
])
