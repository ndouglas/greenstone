"""Build 'chase', the NES ROM behind 001's "who" beat: 8-bit Nate walks on, sees the paperclips, leaps, and runs,
the paperclips hopping after him. No music: Nate talks over it.

The art is Nate's own (nate.png, drawn on pixel-editor.com) and the paperclip below. Python turns them into
CHR tiles and metasprites, and works out every actor's place on every frame; chase.s plays that table back.

Usage: cd tools && series/.venv/bin/python -m series.homebrew.chase.make [OUT_DIR] [--phases NAME LOOK CHASE]
(the three beats' lengths in seconds, voice and stillness)"""

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

# Nate's sheet: three colours, one sprite palette. Frames by their box in nate.png (x0, y0, x1, y1).
NATE_COLOURS = {(124, 124, 124): 1, (255, 160, 68): 2, (248, 248, 248): 3}
NATE_FRAMES = {"stride_a": (117, 1, 134, 23), "stride_b": (139, 1, 156, 23), "pass": (101, 0, 111, 23),
               "front": (82, 0, 96, 23), "front_b": (60, 0, 74, 23), "look_back": (45, 0, 55, 23),
               "leap": (69, 47, 87, 70)}  # front: the sheet's fifth frame, his standing pose; front_b, the fourth
RUN = ("stride_a", "pass", "stride_b", "pass")  # his run to the right, a beat per pose
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
SKY = 0x22
PALETTES = bytes([SKY, 0x17, 0x07, 0x27,  # ground: brown, its dark edges, its lit edges
                  SKY, 0x2A, 0x1A, 0x0F,  # hills and bushes: green, darker green, black outline
                  SKY, 0x30, 0x21, 0x0F,  # clouds: white, pale blue, black outline
                  SKY, 0x30, 0x31, 0x0F,
                  SKY, 0x00, 0x27, 0x30,  # Nate: grey, orange, white
                  SKY, 0x30, 0x00, 0x0F,  # paperclip: white wire, grey, black
                  SKY, 0x00, 0x10, 0x30,
                  SKY, 0x00, 0x10, 0x30])
NATE, CLIP_PAL = 0, 1
GROUND_PAL, HILLS_PAL, CLOUDS_PAL = 0, 1, 2
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


def ground_block() -> np.ndarray:
    """One 16x16 ground block: brown, lit along its top and left, dark along its bottom and right."""
    b = np.ones((16, 16), np.uint8)
    b[0, :], b[:, 0] = 3, 3
    b[15, :], b[:, 15] = 2, 2
    b[0, 15], b[15, 0] = 2, 2
    return b


def scenery() -> tuple[np.ndarray, np.ndarray]:
    """An original backdrop in the Mario spirit: outlined clouds, hills lit from the upper left, bushes, and a
    floor of ground blocks. Colour indices for every pixel (240 x 256), and the background palette of each
    16x16 block (15 x 16): clouds in the top eight block rows, hills and bushes in the next four, ground below,
    so no block needs more than its three colours."""
    pixels = np.zeros((240, 256), np.uint8)
    clouds = [_lobes([(58, 44, 16, 10), (44, 50, 11, 8), (74, 50, 11, 8), (58, 53, 24, 6)]),
              _lobes([(176, 30, 13, 9), (164, 36, 9, 7), (190, 36, 10, 7), (177, 39, 19, 5)]),
              _lobes([(226, 78, 10, 7), (216, 82, 7, 5), (236, 82, 7, 5), (226, 85, 14, 4)])]
    for cloud in clouds:
        pixels[cloud] = 1                       # white
        pixels[_shade(cloud, 3)] = 2            # pale blue away from the light
        pixels[_edge(cloud)] = 3                # outline
    y, x = np.mgrid[0:240, 0:256]
    for cx, rx, ry in ((70, 52, 50), (196, 34, 30)):
        hill = _round((((x - cx) / rx) ** 2 + ((y - GROUND) / ry) ** 2 <= 1) & (y < GROUND))
        pixels[hill] = 1                        # green
        pixels[_shade(hill | (y >= GROUND), 6) & hill] = 2  # dark green away from the light; it goes on below
        pixels[_edge(hill) & (y < GROUND - 1)] = 3
    for lobes in ([(130, GROUND - 6, 10, 9), (144, GROUND - 9, 11, 11), (158, GROUND - 6, 10, 9)],
                  [(236, GROUND - 5, 8, 7), (247, GROUND - 7, 9, 9)]):
        bush = _lobes(lobes) & (y < GROUND)
        pixels[bush] = 1
        pixels[_shade(bush | (y >= GROUND), 3) & bush] = 2
        pixels[_edge(bush) & (y < GROUND - 1)] = 3
    block = ground_block()
    for by in range(GROUND, 240, 16):
        for bx in range(0, 256, 16):
            pixels[by:by + 16, bx:bx + 16] = block[:240 - by]
    blocks = np.full((15, 16), CLOUDS_PAL, np.uint8)
    blocks[GROUND // 16 - 4:GROUND // 16] = HILLS_PAL
    blocks[GROUND // 16:] = GROUND_PAL
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
            entries.append((row * 8 - h, len(chr_tiles), palette, col * 8 + (BOX - w) // 2))
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
    walk = (stop + 24) / (0.8 * name)  # pixels a frame: about one, a walking pace
    speed = (stop - 20 + 16) / (arrive - first_clip)  # to arrive on time, from just off the left edge
    frames = []
    for f in range(name + engineer + chase):
        y = GROUND
        if f < name:
            x = min(-24 + round(walk * f), stop)  # a walk, arriving four-fifths into his name
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
    lines = [f"ACTORS = {len(frames[0])}", f"FRAME_COUNT = {len(frames)}", "", ".segment \"RODATA\"", "palettes:",
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
