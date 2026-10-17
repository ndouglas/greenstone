"""Chapter 6: four Tetris games, one per part of Mario's world, drive him;
then we shake their inputs."""

from .. import draw
from ..model import Caption, Chapter, Shot, FPS

REGIONS = ["mario", "enemies", "floor", "wall"]
REGION_NAMES = {"mario": "Mario", "enemies": "the enemies", "floor": "the floor ahead", "wall": "the wall ahead"}
FADE_IN = 40
# Where the echo state network died on 1-1 (run 30: its best run, 1525, and
# the Goomba pair near x 1800); the bank passing 1850 is past it.
GOOMBA_PAIR_X = 1850


def bank(ctx):
    return ctx.takes["bank_drive"]


def step_at(ctx, i):
    steps = bank(ctx).meta["best"]["steps"]
    return steps[min(len(steps) - 1, i // 4)]


def four_boards(img, ctx, i, box=(700, 40), size=(110, 220)):
    step = step_at(ctx, i)
    for k, (cells, region) in enumerate(zip(step["boards"], REGIONS)):
        x, y = box[0] + (k % 2) * (size[0] + 140), box[1] + (k // 2) * (size[1] + 50)
        draw.board(img, cells, (x, y, size[0], size[1]), colour=draw.REGION[region])
        draw.text(img, (x + size[0] / 2, y + size[1] + 8), REGION_NAMES[region], 18, draw.REGION[region], anchor="ma")


def regions(ctx, t, i):
    img = draw.blank()
    walk = ctx.takes["matrix_walk"].meta["steps"][120]
    w = bytes.fromhex(walk["window"])
    # Colour the bytes each board reads, roughly: Mario's own bytes, the enemy
    # slots, the floor row and the rows above it in the terrain ahead.
    colours = {}
    for a in (0x1D, 0x9F, 0xCE, 0x57):
        colours[a] = draw.REGION["mario"]
    for a in list(range(0x0F, 0x14)) + list(range(0x87, 0x8C)) + list(range(0x6E, 0x73)):
        colours[a] = draw.REGION["enemies"]
    shown = REGIONS[: 1 + int(t >= 6) + int(t >= 9) + int(t >= 11)]
    if "floor" not in shown:
        pass
    draw.byte_grid(img, w[:256], (60, 60, 352, 352), 16, colours={a: c for a, c in colours.items()
                   if (c == draw.REGION["mario"] and "mario" in shown) or (c == draw.REGION["enemies"] and "enemies" in shown)})
    terrain = w[256:]
    rows = 13
    cols = len(terrain) // rows
    as_rows = bytes(terrain[c * rows + r] for r in range(rows) for c in range(cols))
    tc = {}
    for k, v in enumerate(as_rows):
        r = k // cols
        if r == 11 and "floor" in shown:
            tc[k] = draw.REGION["floor"]
        elif 8 <= r <= 10 and "wall" in shown:
            tc[k] = draw.REGION["wall"] if v else tuple(c // 3 for c in draw.REGION["wall"])
        else:
            tc[k] = draw.PANEL if not v else (70, 80, 90)
    draw.byte_grid(img, as_rows, (60, 430, 352, 150), cols, colours=tc)
    step = bank(ctx).meta["best"]["steps"][40]
    for k, region in enumerate(REGIONS):
        if region not in shown:
            continue
        x, y = 700 + (k % 2) * 250, 40 + (k // 2) * 270
        draw.board(img, step["boards"][k], (x, y, 110, 220), colour=draw.REGION[region])
        draw.text(img, (x + 55, y + 228), REGION_NAMES[region], 18, draw.REGION[region], anchor="ma")
        draw.arrow(img, (430, 200 if region in ("mario", "enemies") else 500), (x - 20, y + 110), draw.REGION[region], 2)
    return img


def tidy(ctx, t, i):
    """The two banks' average distance on 1-1, as bank_drive measured them."""
    img = draw.blank()
    names = {"bank-tetris": "four games", "bank-tidy": "four games, plus a tidying piece"}
    rows = [{"label": names.get(c["name"], c["name"]), "v": c["mean_x"]} for c in bank(ctx).meta["conditions"]]
    draw.text(img, (60, 60), "how far each gets on 1-1, on average", 24, draw.DIM)
    draw.bar_chart(img, (60, 120, 1150, 200), rows, "v", "label", shown=len(rows), fmt="x {:.0f}", max_value=3300)
    return img


def tidy_caption(ctx):
    means = {c["name"]: c["mean_x"] for c in bank(ctx).meta["conditions"]}
    if means["bank-tidy"] > means["bank-tetris"]:
        return "A fifth piece that tidies each board, filling its lowest gap, helped the games last."
    return "We also tried a fifth piece that tidies each board. Here, the plain four drove further."


def drive_seconds(ctx):
    n = bank(ctx).meta["streams"]["smb"]
    return min(48.0, max(30.0, (n - FADE_IN) / FPS))


def drive(ctx, t, i):
    img = draw.blank()
    i += FADE_IN
    draw.nes(img, bank(ctx).frames("smb")[i], (40, 40, 512, 480))
    four_boards(img, ctx, i)
    x = step_at(ctx, i)["x"]
    d = draw.ImageDraw.Draw(img)
    x0, x1, y = 40, 552, 560
    d.line((x0, y, x1, y), fill=draw.DIM, width=2)
    px = x0 + (x1 - x0) * min(1.0, x / 3300)
    d.ellipse((px - 7, y - 7, px + 7, y + 7), fill=draw.GOLD)
    gx = x0 + (x1 - x0) * GOOMBA_PAIR_X / 3300
    d.line((gx, y - 14, gx, y + 14), fill=draw.RED, width=2)
    draw.text(img, (gx, y + 18), "where the echo state died", 16, draw.RED, anchor="ma")
    return img


def drive_stings(ctx):
    steps = bank(ctx).meta["best"]["steps"]
    for k, s in enumerate(steps):
        if s["x"] > GOOMBA_PAIR_X:
            return [(max(0.0, (k * 4 - FADE_IN) / FPS), "fanfare")]
    return []


def drive_caption(ctx):
    if bank(ctx).meta["passed_goomba_pair"]:
        return "Past the two Goombas where the echo state network died."
    return "It doesn't get past the two Goombas. Not this time."


def noise(ctx, t, i):
    img = draw.blank()
    meta = ctx.takes["bank_noise"].meta
    rows = [{"label": f"shaken by up to {lv['jitter']}", "v": lv["mean_x"]} for lv in meta["levels"]]
    top = max(r["v"] for r in rows) or 1
    draw.bar_chart(img, (40, 60, 760, 300), rows, "v", "label", shown=len(rows), fmt="x {:.0f}", max_value=top)
    draw.text(img, (40, 30), f"{shaken_bank(ctx)}: how far it gets, on average", 22, draw.DIM)
    draw.nes(img, ctx.takes["bank_noise"].frames("jitter32")[FADE_IN + i], (860, 60, 384, 360))
    draw.text(img, (1052, 430), "shaken by up to 32", 20, draw.DIM, anchor="ma")
    return img


def shaken_bank(ctx):
    return "the tidy bank" if ctx.takes["bank_noise"].meta["condition"] == "bank-tidy" else "the four games"


def noise_intro(ctx):
    return f"Now shake the numbers going into {shaken_bank(ctx)}."


def noise_caption(ctx):
    levels = ctx.takes["bank_noise"].meta["levels"]
    clean, worst = levels[0]["mean_x"], levels[-1]["mean_x"]
    if worst < clean / 2:
        return "Shake them hard, and it falls apart."
    if min(lv["mean_x"] for lv in levels) < clean / 2:
        return "It bounces around, but even shaken hard it keeps going. Three runs each is a small sample."
    return "Even shaken hard, it keeps going."


def best_card(ctx, t, i):
    return draw.text_card(["Its best run on 1-1 reached x {}.".format(bank(ctx).meta["best_x"])])


CHAPTER = Chapter(6, "Four Tetris games", "A bank of boards", [
    Shot("regions", 18, regions, [
        Caption(0, "One Tetris game wasn't enough, so we gave Mario four."),
        Caption(6, "One watches Mario. One the enemies. One the floor ahead. One the wall ahead."),
        Caption(13, "Each picks four pieces for its own board, every step."),
    ], takes=("bank_drive", "matrix_walk")),
    Shot("tidy", 12, tidy, [
        Caption(0, tidy_caption),
    ], takes=("bank_drive",)),
    Shot("drive", drive_seconds, drive, [
        Caption(0, "Now the four games drive."),
        Caption(6, ""),
        Caption(22, drive_caption),
        Caption(28, ""),
    ], takes=("bank_drive",), stings=drive_stings),
    Shot("noise", 16, noise, [
        Caption(0, noise_intro),
        Caption(8, noise_caption),
    ], takes=("bank_noise",)),
    Shot("best", 8, best_card, [
        Caption(0, "No position, no clock: just four games of Tetris and a weighted sum."),
    ], takes=("bank_drive",)),
])
