"""Helpers the chapters share: a fight on the left with a panel of what
happened so far on the right, and the build-up diagram."""

import numpy as np

from reservoir_film import draw
from reservoir_film.model import H, W

from .. import diagram
from ..insides import MOVE_FRAMES, step_at

# The fight's screen, 256x240 scaled up, beside a panel of facts.
FIGHT_BOX = (40, 24, 600, 562)
PANEL_BOX = (680, 24, 560, 562)
# The diagram's place on a full screen, clear of the caption bar.
DIAGRAM_RECT = (40, 60, 1200, 520)
MOVE_NAMES = {"Idle": "stand still", "DodgeLeft": "dodge left", "DodgeRight": "dodge right", "Duck": "duck",
              "JabLeft": "left jab", "JabRight": "right jab", "Uppercut": "star uppercut"}


_LAST_LIT = {}


def last_lit(take):
    """The take's last frame before the game fades to black, so a finished
    fight holds on its knockout rather than on a black screen."""
    if take.path not in _LAST_LIT:
        frames = take.frames("punchout")
        k = frames.n - 1
        # A frame well inside the fight sets what lit looks like.
        lit = np.asarray(frames[max(0, k - 200)]).mean()
        while k > 0 and np.asarray(frames[k]).mean() < 0.9 * lit:
            k -= 1
        _LAST_LIT[take.path] = k
    return _LAST_LIT[take.path]


def frame_index(take, i, speed=1, start=0):
    """The take's frame for output frame `i`, played at `speed` from `start`
    and held on its last lit frame once the take runs out."""
    return min(start + int(i * speed), last_lit(take))


def hits_so_far(take, frame):
    steps = take.meta["steps"]
    return sum(s["hit"] for s in steps[:frame // MOVE_FRAMES + 1])


def fight(img, take, frame, label, box=FIGHT_BOX):
    draw.nes(img, take.frames("punchout")[frame], box)
    draw.text(img, (box[0], box[1] + box[3] + 6), label, 20, draw.DIM)


def fight_panel(img, take, frame, title, box=PANEL_BOX, show_move=True):
    """What the fighter is doing and what it has cost so far."""
    x, y, w, h = box
    draw.panel(img, box)
    draw.text(img, (x + 24, y + 20), title, 30, draw.INK, bold=True)
    step = step_at(take, frame)
    if show_move:
        draw.text(img, (x + 24, y + 90), "latest move", 20, draw.DIM)
        move = recent_move(take, frame)
        draw.text(img, (x + 24, y + 116), MOVE_NAMES[move] if move else "nothing", 40,
                  draw.GOLD if move else draw.DIM, bold=True)
    draw.text(img, (x + 24, y + 210), "hits taken", 20, draw.DIM)
    draw.text(img, (x + 24, y + 236), str(hits_so_far(take, frame)), 56, draw.RED, bold=True)
    draw.text(img, (x + 24, y + 340), "opponent's health", 20, draw.DIM)
    health = step["opponent_health"]
    d = draw.ImageDraw.Draw(img)
    d.rectangle((x + 24, y + 372, x + w - 24, y + 400), outline=draw.DIM, width=2)
    d.rectangle((x + 26, y + 374, x + 26 + (w - 52) * health / 96, y + 398), fill=draw.GREEN)
    return step


def ramp(t, start, length):
    """0 before `start`, rising to 1 over `length` seconds."""
    return min(1.0, max(0.0, (t - start) / length))


def diagram_shot(chapter, swaps=(), focus=None):
    """A shot drawing the diagram as it stands when `chapter` begins, with
    each (piece, start, length[, target]) in `swaps` swapping in over time.
    `focus` is a piece, or a list of (from second, piece or None)."""
    def shot(ctx, t, i):
        img = draw.blank()
        states = dict(diagram.pieces_by_chapter(chapter))
        current = None
        for s in swaps:
            piece, start, length = s[:3]
            target = s[3] if len(s) > 3 else "ours"
            p = ramp(t, start, length)
            if p >= 1:
                states[piece] = target
            elif p > 0:
                current = (piece, p, target)
        lit = focus
        if isinstance(focus, list):
            lit = None
            for start, piece in focus:
                if t >= start:
                    lit = piece
        diagram.draw_diagram(img, DIAGRAM_RECT, states, swap=current, focus=lit)
        return img
    return shot


__all__ = ["H", "W"]


def plural(n, word):
    return f"{n} {word}" if n == 1 else f"{n} {word}s"


def recent_move(take, frame, hold=8):
    """The last move other than Idle within `hold` steps, so a 4-frame dodge
    stays readable."""
    steps = take.meta["steps"]
    k = min(frame // MOVE_FRAMES, len(steps) - 1)
    for s in reversed(steps[max(0, k - hold + 1):k + 1]):
        if s["move"] != "Idle":
            return s["move"]
    return None


def pair(img, takes, frame, labels):
    """Two fights side by side at the same frame, each with its hits so far
    and its latest move."""
    for k, (take, label) in enumerate(zip(takes, labels)):
        x = 40 + k * 620
        n = take.frames("punchout").n
        f = min(frame, n - 1)
        draw.nes(img, take.frames("punchout")[f], (x, 20, 560, 525))
        draw.text(img, (x, 552), label, 22, draw.INK, bold=True)
        draw.text(img, (x + 560, 552), plural(hits_so_far(take, f), "hit"), 22, draw.RED, anchor="ra", bold=True)
        if frame >= n:
            draw.text(img, (x + 280, 260), "fight over" if take.meta["score"]["won"] else "Mac is down", 36,
                      draw.RED, anchor="ma", bold=True)
            continue
        move = recent_move(take, f)
        if move:
            draw.text(img, (x, 584), MOVE_NAMES[move], 22, draw.GOLD, bold=True)


def proj_grid(img, values, box):
    """The 256 projection numbers as a 16x16 grid: gold above zero, blue below."""
    x, y, w, h = box
    d = draw.ImageDraw.Draw(img)
    cw, ch = w / 16, h / 16
    for k, v in enumerate(values[:256]):
        a = min(1.0, abs(v) / 2.0)
        base = draw.GOLD if v > 0 else diagram.FROZEN
        colour = tuple(round(b * a + g * (1 - a)) for b, g in zip(base, draw.PANEL))
        cx, cy = x + (k % 16) * cw, y + (k // 16) * ch
        d.rectangle((cx + 1, cy + 1, cx + cw - 1, cy + ch - 1), fill=colour)


def score_bars(img, step, moves, box):
    """The readout's score for each move this step; the pressed one in gold."""
    x, y, w, h = box
    d = draw.ImageDraw.Draw(img)
    scores = step["scores"]
    best = max(range(len(scores)), key=lambda k: scores[k])
    bh = h / len(scores)
    zero = x + 200
    for k, (name, v) in enumerate(zip(moves, scores)):
        by = y + k * bh
        draw.text(img, (zero - 12, by + bh / 2), MOVE_NAMES[name], 22, draw.INK, anchor="rm")
        length = max(-1.0, min(1.5, v)) * (w - 220) / 1.5
        colour = draw.GOLD if k == best else draw.DIM
        d.rectangle((min(zero, zero + length), by + bh * 0.25, max(zero, zero + length), by + bh * 0.75), fill=colour)
    d.line((zero, y, zero, y + h), fill=draw.DIM, width=1)
