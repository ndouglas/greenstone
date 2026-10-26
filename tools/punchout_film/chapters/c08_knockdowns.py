"""Chapter 8: Fight. stack4 knocks Glass Joe down twice; no student wins."""

from reservoir_film import draw
from reservoir_film.model import Caption, Chapter, Shot

from ..insides import MOVE_FRAMES
from .common import downs, fight, fight_panel, frame_index, knockdown_stings, step_seconds

TAKE = "p3-stack4-glass-joe"
SPEED = 4


def knockdowns(ctx, t, i):
    img = draw.blank()
    take = ctx.takes[TAKE]
    f = frame_index(take, i, speed=SPEED)
    fight(img, take, f, "stack4 vs Glass Joe, Fight, four times speed")
    fight_panel(img, take, f, "stack4")
    return img


def bells(ctx):
    """A bell and a roar each time Glass Joe's health reaches zero."""
    take = ctx.takes[TAKE]
    return [(step_seconds(k, SPEED), "bell") for k in downs(take, "opponent_health")] + knockdown_stings(take, SPEED)


def seconds(ctx):
    return ctx.takes[TAKE].meta["score"]["steps"] * MOVE_FRAMES / SPEED / 60


CHAPTER = Chapter(8, "Knockdowns", "And no wins", [
    Shot("knockdowns", seconds, knockdowns, [
        Caption(0, "Fight adds the star uppercut. stack4 against Glass Joe."),
        Caption(5, "That's a knockdown."),
        Caption(10, "Down twice in the fight, {numbers.phase3.stack4.r4.glass-joe.dealt} damage dealt, and {numbers.phase3.stack4.r4.glass-joe.hits} hits taken."),
        Caption(20, "But Glass Joe gets up every time, and the fight runs out the clock at 2,400 moves."),
        Caption(29, "No student wins a fight. The teacher it copied won every Fight, in {numbers.teacher.phase3.von-kaiser.steps} to {numbers.teacher.phase3.glass-joe.steps:,} moves."),
    ], takes=(TAKE,), stings=bells),
])
