"""Chapter 7: Counter. DAgger joins the diagram, and stack4 against Von
Kaiser comes close to perfect."""

from reservoir_film import draw
from reservoir_film.model import Caption, Chapter, Shot

from .common import diagram_shot, fight, fight_panel, frame_index

TAKE = "p2-stack4-von-kaiser"
SPEED = 3


def counter(ctx, t, i):
    img = draw.blank()
    take = ctx.takes[TAKE]
    f = frame_index(take, i, speed=SPEED)
    fight(img, take, f, "stack4 vs Von Kaiser, Counter, three times speed")
    fight_panel(img, take, f, "stack4")
    return img


CHAPTER = Chapter(7, "Counter", "And a teacher who checks your work", [
    Shot("dagger", 18, diagram_shot(7, swaps=[("learn", 9, 4, "dagger")], focus="learn"), [
        Caption(0, "Copying a teacher has a trap. The student makes mistakes the teacher never made."),
        Caption(6, "Then it's somewhere the teacher never was, with no idea what to do."),
        Caption(11, "DAgger: the student fights, the teacher labels the student's own moments, and the readout is refitted. Four rounds."),
    ]),
    Shot("counter", 30, counter, [
        Caption(0, "Counter adds left and right jabs. Here is stack4 against Von Kaiser."),
        Caption(6, "{numbers.phase2.stack4.r4.von-kaiser.hpm:.2f} hits a minute, where standing still takes {numbers.idle.von-kaiser.hpm:.1f}."),
        Caption(13, "Counter is where the brains come closest to perfect."),
        Caption(19, "Four glimpses of a screen, a random matrix, and one weighted sum per move."),
    ], takes=(TAKE,)),
])
