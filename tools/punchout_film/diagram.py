"""The build-up diagram: a brain as a row of boxes, which starts as the
mainstream deep-RL pipeline and has each box swapped for this experiment's
stranger version over the film."""

from collections import namedtuple

from PIL import ImageDraw

from reservoir_film import draw

PIECES = ["see", "remember", "decide", "learn", "yardstick"]
LEARNED = draw.GOLD
FROZEN = (90, 150, 250)
# The learning signal is neither a learned part nor a frozen one.
SIGNAL = draw.DIM

# `learned` is True for a trained part, False for a frozen one, and None
# for the learning signal.
Box = namedtuple("Box", "label sub learned")

MAINSTREAM = {
    "see": Box("conv net", "learned end to end", True),
    "remember": Box("recurrent net", "learns what to keep", True),
    "decide": Box("policy", "millions of gradient steps", True),
    "learn": Box("reward", "about 10^7 frames", None),
}
OURS = {
    "see": Box("random projection", "5,712 pixels to 256 numbers, rolled once", False),
    "remember": Box("a memory I picked", "stack, echo, or four Tetris games", False),
    "decide": Box("one linear readout", "one ridge solve", True),
    "learn": Box("a teacher with save states", "tries every move, 60 frames ahead", None),
    "yardstick": Box("the cheat", "reads all 2,048 bytes of RAM", False),
}
DAGGER = Box("a teacher with save states", "tries every move, 60 frames ahead, then DAgger", None)

ROW = ["see", "remember", "decide", "learn"]
GAP = 60


def pieces_by_chapter(n):
    """Each piece's state when chapter `n` begins: the teacher arrives in
    chapter 3, seeing, remembering and deciding in 4, DAgger in 7, and the
    cheat in 10."""
    states = {p: "mainstream" for p in ROW}
    states["yardstick"] = "hidden"
    if n >= 4:
        states["learn"] = "ours"
    if n >= 5:
        states.update(see="ours", remember="ours", decide="ours")
    if n >= 8:
        states["learn"] = "dagger"
    if n >= 11:
        states["yardstick"] = "ours"
    return states


def box_rects(rect, states):
    """Where each visible piece's box sits inside `rect` (x, y, w, h)."""
    x, y, w, h = rect
    with_yardstick = states.get("yardstick", "hidden") != "hidden"
    slot = (w - GAP * (len(ROW) - 1)) // len(ROW)
    row_h = int(h * (0.5 if with_yardstick else 0.7))
    top = y + int(h * 0.08)
    rects = {p: (x + k * (slot + GAP), top, slot, row_h) for k, p in enumerate(ROW)}
    if with_yardstick:
        rects["yardstick"] = (x, top + row_h + int(h * 0.14), slot, int(h * 0.3))
    return rects


def _box_of(piece, state):
    if state == "hidden":
        return None
    if state == "mainstream":
        return MAINSTREAM.get(piece)
    if state == "dagger":
        return DAGGER
    return OURS.get(piece)


def _colour(box):
    return SIGNAL if box.learned is None else (LEARNED if box.learned else FROZEN)


def _dim(colour, amount):
    """`colour` faded toward the background by `amount` (0 none, 1 gone)."""
    return tuple(round(c * (1 - amount) + b * amount) for c, b in zip(colour, draw.BG))


def _dashed(d, box, colour, width=3, dash=14, space=9):
    x0, y0, x1, y1 = box
    for (ax, ay), (bx, by) in [((x0, y0), (x1, y0)), ((x1, y0), (x1, y1)), ((x1, y1), (x0, y1)), ((x0, y1), (x0, y0))]:
        length = abs(bx - ax) + abs(by - ay)
        sx, sy = (bx - ax) / length, (by - ay) / length
        t = 0
        while t < length:
            e = min(t + dash, length)
            d.line((ax + sx * t, ay + sy * t, ax + sx * e, ay + sy * e), fill=colour, width=width)
            t += dash + space


def _draw_box(img, rect, box, fade=0.0, struck=False):
    x, y, w, h = rect
    d = ImageDraw.Draw(img)
    colour = _dim(_colour(box), fade)
    d.rounded_rectangle((x, y, x + w, y + h), radius=8, fill=_dim(draw.PANEL, fade))
    if box.learned is False:
        _dashed(d, (x, y, x + w, y + h), colour)
    else:
        d.rounded_rectangle((x, y, x + w, y + h), radius=8, outline=colour, width=3)
    label_font, sub_font = draw.font(22, bold=True), draw.font(16)
    lines = draw.wrap(box.label, label_font, w - 24)
    subs = draw.wrap(box.sub, sub_font, w - 24)
    ty = y + h // 2 - (len(lines) * 28 + len(subs) * 20) // 2
    for line in lines:
        d.text((x + w // 2, ty), line, font=label_font, fill=_dim(draw.INK, fade), anchor="ma")
        ty += 28
    for line in subs:
        d.text((x + w // 2, ty), line, font=sub_font, fill=_dim(draw.DIM, fade), anchor="ma")
        ty += 20
    if struck:
        d.line((x + 8, y + h - 8, x + w - 8, y + 8), fill=draw.RED, width=4)


def _draw_piece(img, rect, piece, state, swap, fade):
    if swap and swap[0] == piece:
        p, target = swap[1], (swap[2] if len(swap) > 2 else "ours")
        old, new = _box_of(piece, state), _box_of(piece, target)
        if old is None or old == new:
            # Nothing to swap from: the new box fades in.
            _draw_box(img, rect, new, fade=max(fade, 1 - p))
        elif p < 1 / 3:
            _draw_box(img, rect, old, fade)
        elif p < 2 / 3:
            # A moment with both: the old box struck out beside the new one.
            x, y, w, h = rect
            half = (w - 8) // 2
            _draw_box(img, (x, y, half, h), old, fade=max(fade, 0.4), struck=True)
            _draw_box(img, (x + half + 8, y, half, h), new, fade)
        else:
            _draw_box(img, rect, new, fade)
        return
    _draw_box(img, rect, _box_of(piece, state), fade)


def draw_diagram(img, rect, states, swap=None, focus=None):
    """Draw the diagram in `rect` with each piece in its state. `swap` is
    (piece, progress 0..1) or (piece, progress, target state) for a piece
    changing from its state to the target ("ours" by default); a hidden piece
    fades in. `focus` dims every other piece."""
    if swap and states.get(swap[0]) == "hidden":
        states = {**states, swap[0]: swap[2] if len(swap) > 2 else "ours"}
    rects = box_rects(rect, states)
    fade_of = {p: (0.0 if focus in (None, p) else 0.65) for p in rects}
    for piece, r in rects.items():
        _draw_piece(img, r, piece, states[piece], swap, fade_of[piece])
    arrow_colour = draw.DIM
    see, remember, decide, learn = (rects[p] for p in ROW)
    for a, b in [(see, remember), (remember, decide)]:
        my = a[1] + a[3] // 2
        draw.arrow(img, (a[0] + a[2] + 8, my), (b[0] - 8, my), colour=arrow_colour)
    my = decide[1] + decide[3] // 2
    # The learning signal points back at the part it trains.
    draw.arrow(img, (learn[0] - 8, my), (decide[0] + decide[2] + 8, my), colour=arrow_colour)
    if "yardstick" in rects:
        ys = rects["yardstick"]
        draw.arrow(img, (ys[0] + ys[2] + 8, ys[1] + ys[3] // 2), (decide[0] + decide[2] // 2, decide[1] + decide[3] + 8),
                   colour=arrow_colour)
    x, y, w, h = rect
    draw.text(img, (x + w, y + h), "gold: learned     dashed blue: frozen at birth", 16, draw.DIM, anchor="rd")
