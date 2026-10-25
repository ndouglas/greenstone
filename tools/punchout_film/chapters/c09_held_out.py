"""Chapter 9: Bald Bull, whom no brain trained on. More teaching made stack4
worse against him."""

from reservoir_film import draw
from reservoir_film.model import Caption, Chapter, Shot

from .common import pair

SPEED = 2


def rounds(ctx, t, i):
    img = draw.blank()
    pair(img, [ctx.takes["p2-stack4-r0-bald-bull"], ctx.takes["p2-stack4-bald-bull"]], i * SPEED,
         ["round 0: the teacher alone", "round 4: after DAgger"])
    return img


CHAPTER = Chapter(9, "Held out", "An opponent nobody trained on", [
    Shot("rounds", 32, rounds, [
        Caption(0, "Held out means never trained on. No brain fought Bald Bull until the test."),
        Caption(6, "stack4 in Counter, side by side. Left, fitted to the teacher alone. Right, after four rounds of DAgger."),
        Caption(13, "Round 0 takes {numbers.phase2.stack4.r0.bald-bull.hpm:.2f} hits a minute. Round 4 takes {numbers.phase2.stack4.r4.bald-bull.hpm:.2f}."),
        Caption(20, "More teaching made it worse. Its rounds only ever showed it Glass Joe, Von Kaiser and Piston Honda."),
        Caption(27, "echo0.5 went the other way against him: {numbers.phase2.echo0_5.r0.bald-bull.hpm:.1f} to {numbers.phase2.echo0_5.r4.bald-bull.hpm:.1f}."),
    ], takes=("p2-stack4-r0-bald-bull", "p2-stack4-bald-bull")),
])
