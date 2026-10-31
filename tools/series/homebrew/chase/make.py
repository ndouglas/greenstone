"""Build 'chase', the NES ROM behind 001's "who" beat: 8-bit Nate walks on, sees the paperclips, leaps, and runs,
the paperclips hopping after him. No music: Nate talks over it.

The art is Nate's own (nate.png, drawn on pixel-editor.com) and the paperclip below. Python turns them into
CHR tiles and metasprites, and works out every actor's place on every frame; chase.s plays that table back.

Usage: cd tools && series/.venv/bin/python -m series.homebrew.chase.make [OUT_DIR] [--phases NAME LOOK CHASE]
(the three beats' lengths in seconds, voice and stillness)"""

import math
import subprocess
import sys
from pathlib import Path

import numpy as np
from PIL import Image
from scipy import ndimage


HERE = Path(__file__).parent
FPS = 60.0988
# Seconds for each of the three beats it plays under, each line's voice and its stillness: "My name is Nathan
# Douglas." (he walks in), "I'm a software engineer." (he looks around), "And recently, I've begun a move into
# AI safety and research." (the chase). Estimates until Nate's takes exist; then the build passes the real ones.
PHASES = (2.4, 2.0, 5.5)
# The first frame the scene is fully drawn on, filmed from power-on: before it the console is starting up (black
# while the palettes load, then the sky alone until the picture is switched on between frames). Footage starts here.
FIRST_DRAWN = 4

# Nate's sheet: three colours, one sprite palette. Frames by their box in nate.png (x0, y0, x1, y1).
NATE_COLOURS = {(124, 124, 124): 1, (255, 160, 68): 2, (248, 248, 248): 3}
NATE_FRAMES = {"stride_a": (117, 1, 134, 23), "stride_b": (139, 1, 156, 23), "pass": (101, 0, 111, 23),
               "front": (82, 0, 96, 23), "front_b": (60, 0, 74, 23), "look_back": (45, 0, 55, 23),
               "leap": (69, 47, 87, 70)}  # front: the sheet's fifth frame, his standing pose; front_b, the fourth
RUN = ("stride_a", "pass", "stride_b", "pass")  # his run to the right, a beat per pose
NUDGE = {"stride_b": 1}  # pixels right: in the sheet this frame's whole figure sits a pixel left of the others
CLIP_ART = [  # 10 x 22: X the white wire, x the grey inner loop, W the eyes' whites, K brows and pupils
    "...XXXX...",
    "..X....X..",
    ".X..xx..X.",
    ".XK....KX.",
    ".XWKxKWxX.",
    ".WKWxWKWX.",
    ".WWWxWWWX.",
] + [".X.x..x.X."] * 6 + [".X....x.X."] * 3 + [".X...x..X.", ".X..x...X.", ".X......X.", "..X....X..", "...XXXX...",
                                                  ".........."]
CLIP = [row.translate(str.maketrans("XWKx", "1132")) for row in CLIP_ART]  # colour indices in its palette

# NES palettes: four for the background, then four for sprites; entry 0 of each is the shared sky.
SKY = 0x0F  # black: the universal backdrop, under everything
PALETTES = bytes([SKY, 0x07, 0x17, 0x27,  # the night: dark brown (the glow's dither), brown, amber
                  SKY, 0x07, 0x17, 0x27,  # the same, for blocks with embers: its amber is cycled to twinkle
                  SKY, 0x07, 0x17, 0x27,
                  SKY, 0x07, 0x17, 0x27,
                  SKY, 0x00, 0x27, 0x30,  # Nate: grey, orange, white
                  SKY, 0x30, 0x00, 0x0F,  # paperclip: white wire, grey, black
                  SKY, 0x00, 0x10, 0x30,
                  SKY, 0x00, 0x10, 0x30])
NATE, CLIP_PAL = 0, 1
GROUND_PAL, EMBER_PAL = 0, 1
FLOOR_ROW = 24  # the ground's first nametable row, on a 16-pixel colour-block boundary; feet stand on it
GROUND = 8 * FLOOR_ROW
BOX = 24  # every sprite frame is centred in a box this wide; an actor's x is the box's left edge


def tiles_of(grid: np.ndarray) -> list[np.ndarray]:
    """A grid of colour indices, padded to whole 8x8 tiles, as tiles row by row."""
    h, w = grid.shape
    padded = np.zeros(((h + 7) // 8 * 8, (w + 7) // 8 * 8), np.uint8)
    padded[:h, :w] = grid
    return [padded[y:y + 8, x:x + 8] for y in range(0, padded.shape[0], 8) for x in range(0, padded.shape[1], 8)]


def chr_bytes(tile: np.ndarray) -> bytes:
    """One 8x8 tile in the NES's two bit planes."""
    low = bytes(int("".join(str(v & 1) for v in row), 2) for row in tile)
    high = bytes(int("".join(str(v >> 1 & 1) for v in row), 2) for row in tile)
    return low + high


def nate_grid(box) -> np.ndarray:
    sheet = np.array(Image.open(HERE / "nate.png").convert("RGBA"))
    x0, y0, x1, y1 = box
    part = sheet[y0:y1, x0:x1]
    grid = np.zeros(part.shape[:2], np.uint8)
    for colour, index in NATE_COLOURS.items():
        grid[(part[..., :3] == colour).all(-1) & (part[..., 3] > 0)] = index
    return grid


def clip_grid() -> np.ndarray:
    return np.array([[0 if c == "." else int(c) for c in row] for row in CLIP], np.uint8)


def _round(mask: np.ndarray) -> np.ndarray:
    """A shape cleaned of stray pixels and notches, as pixel art would be drawn."""
    return ndimage.binary_closing(ndimage.binary_opening(mask, iterations=2), iterations=2) & mask.any()


def _edge(mask: np.ndarray) -> np.ndarray:
    """A shape's one-pixel outline, inside it."""
    return mask & ~ndimage.binary_erosion(mask, structure=[[0, 1, 0], [1, 1, 1], [0, 1, 0]])


def _shade(mask: np.ndarray, depth: int) -> np.ndarray:
    """The band of a shape away from the light (the light comes from the upper left)."""
    return mask & ~np.roll(np.roll(mask, -depth, axis=0), -depth, axis=1)


def _lobes(lobes, size=(240, 256)) -> np.ndarray:
    y, x = np.mgrid[0:size[0], 0:size[1]]
    shape = np.zeros(size, bool)
    for cx, cy, rx, ry in lobes:
        shape |= ((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2 <= 1
    return _round(shape)


BAYER = np.array([[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]]) / 16  # ordered dither
EMBERS = ((30, 20), (58, 46), (92, 14), (140, 38), (171, 22), (214, 52), (238, 18), (120, 70), (20, 84),
          (196, 88), (74, 100))
TWINKLE = (0x27, 0x37, 0x27, 0x17, 0x07, 0x17)  # the embers' amber, a step every TWINKLE_FRAMES
TWINKLE_FRAMES = 6
GLOW = (176, 170, 150)  # the fire's centre (x, y) and reach, off to the right: the dark warms toward it


def scenery() -> tuple[np.ndarray, np.ndarray]:
    """An original night scene in the series' Ember look: a black sky dithered warm toward a fire off to the
    right, embers, hills and bushes as black silhouettes rimmed in amber where the firelight catches them,
    and an amber floor over dim blocks. All in one palette (black, dark brown, brown, amber), but the blocks
    with embers take the second, whose amber is cycled so they twinkle. Colour indices for every pixel
    (240 x 256), and each 16x16 block's palette (15 x 16)."""
    y, x = np.mgrid[0:240, 0:256]
    cx, cy, reach = GLOW
    warmth = np.clip(1 - np.hypot(x - cx, (y - cy) * 1.4) / reach, 0, 1) ** 1.8 * 0.55
    pixels = np.where(warmth > BAYER[y % 4, x % 4], 1, 0).astype(np.uint8)  # dark brown, dithered
    blocks = np.full((15, 16), GROUND_PAL, np.uint8)
    for ex, ey in EMBERS:
        pixels[ey, ex] = 3
        pixels[ey, ex + 1] = pixels[ey + 1, ex] = 2
        blocks[ey // 16, ex // 16] = EMBER_PAL

    def silhouette(shape: np.ndarray) -> None:
        pixels[shape] = 0
        lit = shape & ~np.roll(np.roll(shape, 2, axis=0), -2, axis=1)  # the edge facing the fire, up and right
        pixels[lit] = 2
        pixels[shape & ~np.roll(np.roll(shape, 1, axis=0), -1, axis=1)] = 3
    for hx, rx, ry in ((70, 52, 50), (196, 34, 30)):
        silhouette(_round((((x - hx) / rx) ** 2 + ((y - GROUND) / ry) ** 2 <= 1) & (y < GROUND)))
    for lobes in ([(130, GROUND - 6, 10, 9), (144, GROUND - 9, 11, 11), (158, GROUND - 6, 10, 9)],
                  [(236, GROUND - 5, 8, 7), (247, GROUND - 7, 9, 9)]):
        silhouette(_lobes(lobes) & (y < GROUND))
    ground = y >= GROUND
    shine = np.clip(1 - np.abs(x - cx) / reach, 0, 1) * np.clip(1 - (y - GROUND) / 40, 0, 1) * 0.45
    pixels[ground] = np.where(shine[ground] > BAYER[y % 4, x % 4][ground], 1, 0)  # the fire, reflected
    pixels[GROUND] = 3  # the floor: an amber hairline
    pixels[GROUND + 1] = 2
    return pixels, blocks


def nametable(pixels: np.ndarray, blocks: np.ndarray, chr_tiles: list[np.ndarray]) -> bytes:
    """The screen's 960 tile numbers (adding new tiles to chr_tiles, reusing identical ones) and its 64
    attribute bytes (each a 32x32 area: four 16x16 blocks, two bits each)."""
    index = {t.tobytes(): i for i, t in enumerate(chr_tiles)}
    names = bytearray()
    for ty in range(30):
        for tx in range(32):
            tile = pixels[ty * 8:ty * 8 + 8, tx * 8:tx * 8 + 8]
            key = tile.tobytes()
            if key not in index:
                index[key] = len(chr_tiles)
                chr_tiles.append(tile.copy())
            names.append(index[key])
    padded = np.vstack([blocks, blocks[-1:]])  # the last attribute row covers half a block row past the screen
    attributes = bytes(int(padded[2 * ay, 2 * ax] | padded[2 * ay, 2 * ax + 1] << 2 | padded[2 * ay + 1, 2 * ax] << 4
                           | padded[2 * ay + 1, 2 * ax + 1] << 6) for ay in range(8) for ax in range(8))
    return bytes(names) + attributes


def art():
    """CHR (8 KB: tile 0 blank, then the sprites, then the scenery), the metasprites (per sprite frame, its tiles
    as (dy, tile, attribute, dx) from its bottom-left corner) and the nametable."""
    chr_tiles = [np.zeros((8, 8), np.uint8)]
    metas = {}
    sources = {name: (nate_grid(box), NATE) for name, box in NATE_FRAMES.items()}
    sources["clip"] = (clip_grid(), CLIP_PAL)
    for name, (grid, palette) in sources.items():
        h, w = grid.shape
        cols = (w + 7) // 8
        entries = []
        for i, tile in enumerate(tiles_of(grid)):
            if not tile.any():
                continue
            row, col = divmod(i, cols)
            # From the art's bottom edge, centred in a 24-pixel box, so poses share feet and middle.
            entries.append((row * 8 - h, len(chr_tiles), palette, col * 8 + (BOX - w) // 2 + NUDGE.get(name, 0)))
            chr_tiles.append(tile)
        metas[name] = entries
    names = nametable(*scenery(), chr_tiles)
    if len(chr_tiles) > 256:
        raise ValueError(f"{len(chr_tiles)} tiles; a pattern table holds 256")
    data = b"".join(chr_bytes(t) for t in chr_tiles)
    return data + bytes(8192 - len(data)), metas, names


def frame_counts(phases: tuple[float, float, float] = PHASES) -> tuple[int, int, int]:
    return tuple(round(seconds * FPS) for seconds in phases)


def choreography(phases: tuple[float, float, float] = PHASES) -> list[list[tuple[int, int, str | None]]]:
    """Every frame: each actor's (x, y of its feet, sprite frame or None). Actor 0 is Nate, the rest clips.
    In the first phase he walks in and stops; in the second he looks around; in the third the clips hop in
    behind him, he sees them, leaps, and runs off, and they chase him."""
    name, engineer, chase = frame_counts(phases)
    stop, clips = 96, 2  # two clips: with him, seven sprites a line at most
    arrive = name + engineer + round(0.35 * chase)  # the lead clip reaches him
    notice, leap, run = arrive - 14, arrive, arrive + 32
    first_clip = name + engineer
    walk = 1 if stop + 24 <= name - 6 else 2  # pixels a frame, whole so every step is the same; two if his name is short
    arrive_walk = math.ceil((stop + 24) / walk)
    speed = (stop - 20 + 16) / (arrive - first_clip)  # to arrive on time, from just off the left edge
    frames = []
    for f in range(name + engineer + chase):
        y = GROUND
        if f < name:
            x = stop - walk * max(0, arrive_walk - f)  # whole pixels a frame, arriving before his name ends
            pose = "front" if x == stop else RUN[f // 8 % 4]
        elif f < notice:
            x = stop
            pose = "front"
            if f < name + engineer:  # in the second beat he looks around, between his two front frames
                pose = ("front", "front_b")[(f - name) // 24 % 2]
        elif f < leap:
            x, pose = stop, "look_back"  # he hears them
        elif f < run:
            x, pose = stop, "leap"
            y = GROUND - round(14 * np.sin(np.pi * (f - leap) / (run - leap)))
        else:
            x = stop + round(2.5 * (f - run))
            pose = RUN[(f - run) // 5 % 4]
        actors = [(x, y, pose if x < 256 else None)]
        for k in range(clips):
            start = first_clip + 22 * k
            t = f - start
            travelled = speed * max(0, min(f, run) - start) + 2.6 * max(0, f - max(run, start))
            # Follow the leader: never closer than a gap behind him, nor behind the clip ahead.
            cx = min(-16 + round(travelled), x - 20 - 14 * k)
            hop = round(abs(np.sin(np.pi * (t + 7 * k) / 18)) * 6)
            actors.append((cx, GROUND - hop, "clip" if t >= 0 and cx < 256 else None))
        frames.append(actors)
    return frames


def table(frames, names: list[str]) -> bytes:
    """Per frame, per actor: x low, x high, y, sprite frame number (0xFF: hidden)."""
    out = bytearray()
    for actors in frames:
        for x, y, name in actors:
            out += bytes([x & 0xFF, (x >> 8) & 0xFF, y & 0xFF, 0xFF if name is None else names.index(name)])
    return bytes(out)


def source(metas, frames, screen: bytes) -> str:
    names = list(metas)
    lines = [f"ACTORS = {len(frames[0])}", f"FRAME_COUNT = {len(frames)}", f"TWINKLE_COUNT = {len(TWINKLE)}", f"TWINKLE_FRAMES = {TWINKLE_FRAMES}", "",
             ".segment \"RODATA\"", "twinkle:", "  .byte " + ", ".join(f"${b:02X}" for b in TWINKLE), "palettes:",
             "  .byte " + ", ".join(f"${b:02X}" for b in PALETTES), "metas_lo:",
             "  .byte " + ", ".join(f"<meta_{n}" for n in names), "metas_hi:",
             "  .byte " + ", ".join(f">meta_{n}" for n in names)]
    for n in names:
        lines.append(f"meta_{n}:")
        for dy, tile, attr, dx in metas[n]:
            lines.append(f"  .byte ${dy & 0xFF:02X}, ${tile:02X}, ${attr:02X}, ${dx:02X}")
        lines.append("  .byte $80")
    lines.append("nametable:")
    for i in range(0, len(screen), 32):
        lines.append("  .byte " + ", ".join(f"${b:02X}" for b in screen[i:i + 32]))
    lines.append("choreography:")
    data = table(frames, names)
    for i in range(0, len(data), 16):
        lines.append("  .byte " + ", ".join(f"${b:02X}" for b in data[i:i + 16]))
    return "\n".join(lines) + "\n"


def build(out: Path, phases: tuple[float, float, float] = PHASES) -> Path:
    """Assemble chase.nes into `out`, with ca65 and ld65, its three phases lasting `phases` seconds."""
    out.mkdir(parents=True, exist_ok=True)
    chr_data, metas, names = art()
    (out / "chase.chr").write_bytes(chr_data)
    (out / "data.inc").write_text(source(metas, choreography(phases), names))
    obj, rom = out / "chase.o", out / "chase.nes"
    subprocess.run(["ca65", str(HERE / "chase.s"), "-I", str(out), "--bin-include-dir", str(out), "-o", str(obj)],
                   check=True)
    subprocess.run(["ld65", "-C", str(HERE / "nrom.cfg"), str(obj), "-o", str(rom)], check=True)
    return rom


if __name__ == "__main__":
    from series.build import OUT
    args = sys.argv[1:]
    phases = PHASES
    if "--phases" in args:
        i = args.index("--phases")
        phases, args = tuple(float(a) for a in args[i + 1:i + 4]), args[:i] + args[i + 4:]
    print(build(Path(args[0]) if args else OUT / "homebrew" / "chase", phases))
