"""Chapter 3: the teacher, who cheats with save states, dodges and then wins."""

from reservoir_film import draw
from reservoir_film.model import Caption, Chapter, Shot

from .common import diagram_shot, fight, fight_panel, frame_index, knockdown_stings

# The knockout fight plays at double speed.
KO_SPEED = 2


def defence(ctx, t, i):
    img = draw.blank()
    take = ctx.takes["p1-teacher-glass-joe"]
    f = frame_index(take, i)
    fight(img, take, f, "the teacher vs Glass Joe, Defence")
    fight_panel(img, take, f, "the teacher")
    return img


def knockout(ctx, t, i):
    img = draw.blank()
    take = ctx.takes["p3-teacher-von-kaiser"]
    f = frame_index(take, i, speed=KO_SPEED)
    fight(img, take, f, "the teacher vs Von Kaiser, Fight, double speed")
    fight_panel(img, take, f, "the teacher")
    return img


def ko_seconds(ctx):
    """The shot ends a few seconds after the knockout."""
    steps = ctx.takes["p3-teacher-von-kaiser"].meta["score"]["steps"]
    return steps * 4 / 60 / KO_SPEED + 4


def ko_bell(ctx):
    """The bell and a roar at the knockout, and a roar at each knockdown."""
    ko = ko_seconds(ctx) - 4
    return [(ko, "bell"), (ko, "roar")] + knockdown_stings(ctx.takes["p3-teacher-von-kaiser"], KO_SPEED)


CHAPTER = Chapter(3, "The teacher", "Who cheats with save states", [
    Shot("swap", 14, diagram_shot(3, swaps=[("learn", 5, 3)], focus="learn"), [
        Caption(0, "Mainstream agents learn from a reward. Ours learns by copying a teacher."),
        Caption(6, "The teacher cheats. Before every move it saves the game, tries each move 60 frames ahead, and keeps the best."),
    ]),
    Shot("defence", 18, defence, [
        Caption(0, "In the first phase, Defence, Mac may only dodge and duck."),
        Caption(5, "Against Glass Joe the teacher takes {numbers.teacher.phase1.glass-joe.hits} hits."),
        Caption(10, "Its favourite move, by far, is to do nothing: {numbers.teacher_idle_share.phase1:.0%} of its answers are Idle."),
    ], takes=("p1-teacher-glass-joe",)),
    Shot("knockout", ko_seconds, knockout, [
        Caption(0, "In the third phase, Fight, Mac may punch too. Here is the teacher against Von Kaiser."),
        Caption(7, "Three knockdowns, a knockout in {numbers.teacher.phase3.von-kaiser.steps} moves, and not one hit taken."),
        Caption(14, "Every brain in this film learns by copying this teacher."),
        Caption(20, "Copying it turns out to be harder than it looks."),
    ], takes=("p3-teacher-von-kaiser",), stings=ko_bell),
])
