"""Chapter 2: a Mac who never presses anything, and what that costs."""

from reservoir_film import draw
from reservoir_film.model import Caption, Chapter, Shot

from .common import downs, fight, fight_panel, frame_index, step_seconds

# The whole fight, at four times speed.
IDLE_SPEED = 4

OPPONENTS = [("glass-joe", "Glass Joe"), ("von-kaiser", "Von Kaiser"), ("piston-honda", "Piston Honda"),
             ("don-flamenco", "Don Flamenco"), ("bald-bull", "Bald Bull")]


def idle(ctx, t, i):
    img = draw.blank()
    take = ctx.takes["p1-idle-glass-joe"]
    f = frame_index(take, i, speed=IDLE_SPEED)
    fight(img, take, f, "always-Idle vs Glass Joe, four times speed")
    fight_panel(img, take, f, "always-Idle", show_move=False)
    return img


def funeral_offset(ctx):
    """Seconds into the idle shot of Mac's last knockdown, where the funeral
    march begins. Not the take's mac_down_at, which is his first."""
    take = ctx.takes["p1-idle-glass-joe"]
    return step_seconds(downs(take, "mac_health")[-1], IDLE_SPEED)


def floor(ctx, t, i):
    img = draw.blank()
    rows = [{"label": name, "v": ctx.numbers["idle"][key]["hpm"]} for key, name in OPPONENTS]
    draw.text(img, (60, 60), "hits a minute, standing still", 26, draw.DIM)
    draw.bar_chart(img, (60, 110, 1160, 440), rows, "v", "label", len(rows), fmt="{:.1f}", max_value=50)
    return img


CHAPTER = Chapter(2, "Always-Idle", "The floor", [
    Shot("idle", 22, idle, [
        Caption(0, "First, a brain that never presses anything."),
        Caption(5, "Glass Joe is the easiest opponent in the game. Standing still, Mac takes {numbers.idle.glass-joe.hpm:.1f} hits a minute."),
        Caption(12, "That's the floor: every brain has to do better than this."),
        Caption(17, "And down goes Mac, for good."),
    ], takes=("p1-idle-glass-joe",)),
    Shot("floor", 14, floor, [
        Caption(0, "The others hit harder. Bald Bull lands {numbers.idle.bald-bull.hpm:.1f} a minute."),
        Caption(6, "Don Flamenco never attacks a Mac who stands still, so he scores zero here."),
    ]),
])
