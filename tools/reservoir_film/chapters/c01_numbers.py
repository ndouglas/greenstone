"""Chapter 1: the rain resolves into Mario's memory; a few bytes get names;
then we change the numbers and the world changes, and finally the brain's
view: 360 numbers and no picture."""

from .. import draw
from ..model import Caption, Chapter, Shot
from .c00_cold_open import tiles

ZERO_PAGE = 256
TERRAIN_ROWS = 13


def walk_step(ctx, i):
    steps = ctx.takes["matrix_walk"].meta["steps"]
    return steps[min(i // 4, len(steps) - 1)]


def window(ctx, i):
    return bytes.fromhex(walk_step(ctx, i)["window"])


def rain_resolves(ctx, t, i):
    img = draw.blank()
    draw.rain(img, tiles(ctx), t, seed=1, speed=max(0.1, 1.0 - t / 20), density=max(0.15, 1.0 - t / 12))
    if t > 6:
        layer = draw.blank()
        draw.hex_grid(layer, window(ctx, i)[:ZERO_PAGE], (240, 40, 800, 520), 16)
        img = draw.fade(img, layer, min(1.0, (t - 6) / 5))
    return img


# Labels in the order the captions introduce them: (window index, text, span, colour).
LABELS = [(0x86, "Mario's x", 1, draw.GOLD), (0x1D, "ground / jump / fall", 1, draw.RED),
          (0x0F, "enemy slots", 5, draw.REGION["enemies"]), (0x87, "enemy x", 5, draw.REGION["wall"])]
LABEL_TIMES = [0, 5, 11, 11]
TERRAIN_TIME = 17


def window_panel(img, ctx, i, t, hexes=False, smb=True, smb_alpha=1.0):
    """Mario on the left, his window on the right: the zero page as a 16x16
    grid, the terrain ahead as an 8-column map beneath it."""
    w = window(ctx, i)
    prev = window(ctx, max(0, i - 4))
    if smb:
        frame = ctx.takes["matrix_walk"].frames("smb")[i]
        layer = draw.blank()
        draw.nes(layer, frame, (40, 60, 512, 480))
        img.paste(draw.fade(img, layer, smb_alpha))
    shown = [k for k, at in enumerate(LABEL_TIMES) if t >= at]
    highlight = {}
    for k in shown:
        a, _, span, colour = LABELS[k]
        for j in range(span):
            highlight[a + j] = colour
    box = (600, 40, 352, 352)
    if hexes:
        draw.hex_grid(img, w[:ZERO_PAGE], box, 16, highlight=highlight)
    else:
        changed = {j for j in range(ZERO_PAGE) if w[j] != prev[j]}
        draw.byte_grid(img, w[:ZERO_PAGE], box, 16, highlight=highlight, changed=changed)
    for n, k in enumerate(shown):
        a, label, _, colour = LABELS[k]
        x, y = box[0] + box[2] + 30, box[1] + 10 + n * 40
        cw = box[2] / 16
        cx, cy = box[0] + (a % 16 + 1) * cw, box[1] + (a // 16 + 0.5) * cw
        draw.ImageDraw.Draw(img).line((cx, cy, x - 6, y + 12), fill=colour, width=2)
        draw.text(img, (x, y), label, 22, colour)
    terrain = w[ZERO_PAGE:]
    tbox = (600, 410, 352, 150)
    # The terrain buffer is stored column by column, 13 rows each; draw it as a map.
    cols = len(terrain) // TERRAIN_ROWS
    as_rows = bytes(terrain[c * TERRAIN_ROWS + r] for r in range(TERRAIN_ROWS) for c in range(cols))
    colours = {k: (draw.REGION["floor"] if v else draw.PANEL) for k, v in enumerate(as_rows)}
    draw.byte_grid(img, as_rows, tbox, cols, colours=colours)
    if t >= TERRAIN_TIME:
        draw.text(img, (tbox[0] + tbox[2] + 20, tbox[1] + 60), "the ground ahead", 22, draw.GOLD)


def labels(ctx, t, i):
    img = draw.blank()
    window_panel(img, ctx, int(i * 0.8), t)
    return img


def matrix(ctx, t, i):
    img = draw.blank()
    window_panel(img, ctx, 1150 + i // 4, 30, hexes=True, smb_alpha=max(0.0, 1 - t / 3))
    return img


def poke_panel(img, ctx, take, stream, i, slow, pokes, label):
    frames = ctx.takes[take].frames(stream)
    k = i // slow
    draw.nes(img, frames[k], (40, 60, 512, 480))
    done = [p for p in pokes if p["frame"] <= k] if "frame" in (pokes[0] if pokes else {}) else (pokes if k > 0 else [])
    draw.text(img, (620, 80), label, 26, draw.DIM)
    for n, p in enumerate(done[-4:]):
        y = 140 + n * 90
        draw.text(img, (620, y), f"${p['address']:04X}", 40, draw.GOLD, face=draw.mono(40))
        draw.text(img, (800, y), f"{p['before']:02X} → {p['after']:02X}", 40, draw.INK, face=draw.mono(40))


def poke_enemy(ctx, t, i):
    img = draw.blank()
    pokes = ctx.takes["poke_enemy"].meta["pokes"]
    poke_panel(img, ctx, "poke_enemy", "smb", i, 3, pokes, "the Goomba's x, rewritten")
    return img


def terrain_scene(scene, label):
    def shot(ctx, t, i):
        img = draw.blank()
        meta = ctx.takes["poke_terrain"].meta[scene]
        poke_panel(img, ctx, "poke_terrain", scene, i, 3, meta["pokes"], label)
        return img
    return shot


def died_frame(ctx):
    meta = ctx.takes["poke_terrain"].meta["floor"]
    xs = meta["x"]
    return None if not meta["died"] else len(xs)


def floor_caption(ctx):
    if ctx.takes["poke_terrain"].meta["floor"]["died"]:
        return "He falls through. The picture is a drawing; the numbers are what's real."
    return "The picture is a drawing; the numbers are what's real."


def noise(ctx, t, i):
    img = draw.blank()
    take = ctx.takes["noise_bytes"]
    for k in range(4):
        x, y = 64 + (k % 2) * 600, 20 + (k // 2) * 290
        draw.nes(img, take.frames(f"seed{k + 1}")[i // 2], (x + 140, y, 288, 270))
    return img


def brain_view(ctx, t, i):
    img = draw.blank()
    alpha = 1.0 if t < 6 else max(0.0, 1 - (t - 6) / 4)
    window_panel(img, ctx, i, 30, smb_alpha=alpha)
    return img


CHAPTER = Chapter(1, "Mario is just numbers", "Two thousand and forty-eight bytes", [
    Shot("rain_resolves", 20, rain_resolves, [
        Caption(0, "The NES has 2,048 bytes of memory, and Super Mario Bros. keeps its whole world in them."),
        Caption(8, "Most of them change every frame. It looks like noise."),
        Caption(14, "But some of them are things."),
    ], takes=("lives_glyphs", "matrix_walk")),
    Shot("labels", 24, labels, [
        Caption(0, "This byte is how far Mario has walked."),
        Caption(5, "This one says whether he's on the ground, jumping, or falling."),
        Caption(11, "These five are the enemies on screen, and these say where they are."),
        Caption(17, "And this block is a map of the ground ahead: pits, pipes and bricks."),
    ], takes=("matrix_walk",)),
    Shot("matrix", 8, matrix, [
        Caption(1, "After a while you don't see the code. You see Goomba, pipe, pit."),
    ], takes=("matrix_walk",)),
    Shot("poke_enemy", 16, poke_enemy, [
        Caption(0, "If the numbers are the world, changing a number should change the world."),
        Caption(6, "So we moved a Goomba by writing a new x into its byte."),
        Caption(11, ""),
    ], takes=("poke_enemy",)),
    Shot("wall", 12, terrain_scene("wall", "bricks written into the map"), [
        Caption(0, "Here we wrote bricks into the map, but not onto the screen."),
        Caption(6, "Mario runs into a wall nobody can see."),
    ], takes=("poke_terrain",)),
    Shot("floor", 12, terrain_scene("floor", "ground erased from the map"), [
        Caption(0, "And here we erased the ground from the map, but not from the picture."),
        Caption(6, floor_caption),
    ], takes=("poke_terrain",), stings=lambda ctx: [(6.0, "death")] if ctx.takes["poke_terrain"].meta["floor"]["died"] else []),
    Shot("noise", 14, noise, [
        Caption(0, "Spray random numbers into memory and the world comes apart."),
        Caption(7, ""),
    ], takes=("noise_bytes",)),
    Shot("brain_view", 16, brain_view, [
        Caption(0, "A brain playing Mario never sees the picture. It gets these 360 numbers every step."),
        Caption(8, "Could you tell when to jump from this?"),
    ], takes=("matrix_walk",)),
])
