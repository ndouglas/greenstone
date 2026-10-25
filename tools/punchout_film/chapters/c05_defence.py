"""Chapter 5: Defence. echo0.5 sidesteps where standing still gets hit; the
bank stands still, as chapter 4 predicted."""

from reservoir_film import draw
from reservoir_film.model import Caption, Chapter, Shot

from .common import fight, fight_panel, frame_index, pair

# Both fights from step 200, just before Glass Joe's first punches.
START = 200 * 4


def sidestep(ctx, t, i):
    img = draw.blank()
    pair(img, [ctx.takes["p1-idle-glass-joe"], ctx.takes["p1-echo0.5-glass-joe"]], START + i,
         ["always-Idle", "echo0.5"])
    return img


def bank(ctx, t, i):
    img = draw.blank()
    take = ctx.takes["p1-bank-glass-joe"]
    f = frame_index(take, i, speed=4, start=START)
    fight(img, take, f, "the bank vs Glass Joe, four times speed")
    fight_panel(img, take, f, "the bank")
    return img


CHAPTER = Chapter(5, "Defence", "Dodge, duck, or stand there", [
    Shot("sidestep", 24, sidestep, [
        Caption(0, "Each brain is fitted to the teacher's Defence answers, then fights on its own."),
        Caption(6, "Same fight, same moments. Left, standing still. Right, echo0.5."),
        Caption(12, "It dodges a few frames before each punch lands."),
        Caption(17, "Against Glass Joe: {numbers.phase1.echo0_5.r4.glass-joe.hpm:.1f} hits a minute, where standing still takes {numbers.idle.glass-joe.hpm:.1f}."),
    ], takes=("p1-idle-glass-joe", "p1-echo0.5-glass-joe")),
    Shot("bank", 16, bank, [
        Caption(0, "The bank, against the same opponent: {numbers.phase1.bank.r4.glass-joe.hpm:.2f} hits a minute."),
        Caption(6, "Identical to standing still, as chapter 4 predicted. Its readout found nothing in those boards worth acting on."),
        Caption(12, "These are the trained brains. How the training goes round, we'll come to."),
    ], takes=("p1-bank-glass-joe",)),
])
