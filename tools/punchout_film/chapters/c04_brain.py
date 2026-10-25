"""Chapter 4: building a brain. Seeing is a random projection, remembering
is one of three hand-picked memories (the bank of Tetris games gets the
close-up, and its honest flaw), and deciding is one ridge solve."""

from reservoir_film import draw
from reservoir_film.model import Caption, Chapter, Shot

from ..insides import step_at
from .common import diagram_shot, fight, frame_index, proj_grid, score_bars

ECHO = "p1-echo0.5-glass-joe"
BANK = "p1-bank-glass-joe"
# Start the inset clips where Glass Joe starts swinging.
ECHO_START = 200 * 4
BANK_START = 200 * 4


def see(ctx, t, i):
    img = draw.blank()
    take = ctx.takes[ECHO]
    f = frame_index(take, i, start=ECHO_START)
    fight(img, take, f, "what Mac's brain is shown: the screen below the score")
    draw.panel(img, (680, 24, 560, 562), "the 256 numbers, this step")
    proj_grid(img, step_at(take, f)["proj"], (720, 80, 480, 480))
    return img


MEMORIES = [
    ("stack", ["The last few glimpses,", "side by side.", "", "stack4 spans 12 frames."]),
    ("echo", ["2,048 units wired at random,", "each a leaky mix of its past.", "", "Higher leak, faster forgetting."]),
    ("bank", ["Four games of Tetris.", "", "Yes, really."]),
]


def memories(ctx, t, i):
    img = draw.blank()
    for k, (name, lines) in enumerate(MEMORIES):
        if t < 1 + k * 5.5:
            break
        x = 40 + k * 410
        draw.panel(img, (x, 60, 380, 500))
        draw.text(img, (x + 190, 100), name, 44, draw.GOLD, anchor="ma", bold=True)
        for n, line in enumerate(lines):
            draw.text(img, (x + 190, 200 + n * 40), line, 24, draw.INK, anchor="ma")
    return img


def boards(img, step, x0, y0, highlight=False):
    """The bank's four boards; with `highlight`, columns 3 to 6 shaded."""
    for k, cells in enumerate(step["boards"]):
        x, y = x0 + (k % 2) * 270, y0 + (k // 2) * 280
        if highlight:
            d = draw.ImageDraw.Draw(img)
            d.rectangle((x + 3 * 11, y, x + 7 * 11, y + 240), fill=(60, 50, 20))
        draw.board(img, cells, (x, y, 110, 240))


def bank(ctx, t, i):
    img = draw.blank()
    take = ctx.takes[BANK]
    f = frame_index(take, i, start=BANK_START)
    fight(img, take, f, "the bank vs Glass Joe")
    boards(img, step_at(take, f), 720, 40)
    return img


def honest(ctx, t, i):
    img = draw.blank()
    take = ctx.takes[BANK]
    f = frame_index(take, i, start=BANK_START + 3000)
    boards(img, step_at(take, f), 100, 40, highlight=t >= 6)
    d = draw.ImageDraw.Draw(img)
    for c in range(10):
        draw.text(img, (100 + c * 11 + 5, 316), str(c), 10, draw.DIM, anchor="ma")
    draw.text(img, (700, 120), "each byte fed to Tetris", 28, draw.INK)
    draw.text(img, (700, 160), "= the mean of 16 numbers", 28, draw.INK)
    if t >= 6:
        draw.text(img, (700, 260), "so pieces only land", 28, draw.GOLD)
        draw.text(img, (700, 300), "in columns 3 to 6", 28, draw.GOLD, bold=True)
    del d
    return img


def decide(ctx, t, i):
    img = draw.blank()
    take = ctx.takes[ECHO]
    f = frame_index(take, i, start=ECHO_START)
    fight(img, take, f, "echo0.5 vs Glass Joe")
    draw.panel(img, (680, 24, 560, 562), "the readout's score for each move")
    score_bars(img, step_at(take, f), take.meta["moves"], (700, 120, 520, 360))
    return img


CHAPTER = Chapter(4, "Building a brain", "Out of things that never learn", [
    Shot("see_swap", 12, diagram_shot(4, swaps=[("see", 4, 4)], focus="see"), [
        Caption(0, "Seeing. A mainstream agent trains a conv net to see."),
        Caption(5, "Ours multiplies the screen by one random matrix, rolled once from a seed and never trained."),
    ]),
    Shot("see", 14, see, [
        Caption(0, "The screen, shrunk to 84 by 84 with the score cropped off, is 5,712 pixels. The matrix turns them into 256 numbers."),
        Caption(8, "Nobody chose what any of these numbers means."),
    ], takes=(ECHO,)),
    Shot("remember_swap", 10, diagram_shot(4, swaps=[("see", -2, 1), ("remember", 3, 4)], focus="remember"), [
        Caption(0, "Remembering. A mainstream agent learns what to keep."),
        Caption(4, "Ours gets one of three memories, each picked by hand, none of them trained."),
    ]),
    Shot("memories", 18, memories, [
        Caption(0, "A stack keeps the last few glimpses. A brain moves every 4 frames, so stack4 spans 12 frames."),
        Caption(6, "An echo state network is 2,048 randomly wired units that keep a leaky mix of their past."),
        Caption(12, "And the bank."),
    ]),
    Shot("bank", 22, bank, [
        Caption(0, "Each step, numbers from the projection choose where pieces drop, in four running games of Tetris."),
        Caption(7, "The boards are the memory. The brain reads all four of them every step."),
        Caption(14, "A lumpy, rule-bound, slightly ridiculous memory. That's the appeal."),
    ], takes=(BANK,)),
    Shot("honest", 20, honest, [
        Caption(0, "Here's the catch. Each byte we feed Tetris is an average of 16 projection numbers."),
        Caption(6, "Averaging squeezes them toward the middle, so every piece lands in columns 3 to 6."),
        Caption(12, "The boards change every step, but look alike whatever Glass Joe does. That was our plumbing, not the idea."),
    ], takes=(BANK,)),
    Shot("decide_swap", 10, diagram_shot(4, swaps=[("see", -2, 1), ("remember", -2, 1), ("decide", 3, 4)],
                                         focus="decide"), [
        Caption(0, "Deciding. A mainstream policy takes millions of gradient steps."),
        Caption(5, "Ours is one weighted sum per move, fitted by ridge regression in a single solve."),
    ]),
    Shot("decide", 18, decide, [
        Caption(0, "The weighted sum is the readout. Its weights are the only thing in the brain that learns."),
        Caption(6, "Every step, each move gets a score, and the highest is pressed."),
        Caption(11, "A frozen random memory with a trained readout on top is called a reservoir computer."),
    ], takes=(ECHO,)),
])
