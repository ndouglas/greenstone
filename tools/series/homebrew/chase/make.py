"""Build 'chase', the NES ROM behind 001's "who" beat: 8-bit Nate walks on, sees the paperclips, leaps, and runs,
the swarm hopping after him.

The art is Nate's own (nate.png, drawn on pixel-editor.com) and the paperclip below. Python turns them into
CHR tiles and metasprites, and works out every actor's place on every frame; chase.s plays that table back.

Usage: cd tools && series/.venv/bin/python -m series.homebrew.chase.make [OUT_DIR]"""

import subprocess
import sys
from pathlib import Path

import numpy as np
from PIL import Image

HERE = Path(__file__).parent
FRAMES = 420  # 7 s at 60 fps

# Nate's sheet: three colours, one sprite palette. Frames by their box in nate.png (x0, y0, x1, y1).
NATE_COLOURS = {(124, 124, 124): 1, (255, 160, 68): 2, (248, 248, 248): 3}
NATE_FRAMES = {"stride_a": (117, 1, 134, 23), "stride_b": (139, 1, 156, 23), "pass": (101, 0, 111, 23),
               "front": (82, 0, 96, 23), "look_back": (45, 0, 55, 23), "leap": (69, 47, 87, 70)}
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

# NES palettes: background, then four sprite palettes (entry 0 of each is the shared backdrop).
PALETTES = bytes([0x0F, 0x00, 0x10, 0x30] * 4 +  # background: black, greys
                 [0x0F, 0x00, 0x27, 0x30,       # Nate: grey, orange, white
                  0x0F, 0x30, 0x00, 0x0F,       # paperclip: white wire, grey, black
                  0x0F, 0x00, 0x10, 0x30,
                  0x0F, 0x00, 0x10, 0x30])
NATE, CLIP_PAL = 0, 1
FLOOR_ROW = 19  # nametable row of the floor line; feet stand just above it
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


def art():
    """CHR (8 KB: tile 0 blank, tile 1 the floor line, then the sprites) and the metasprites: per sprite frame,
    its tiles as (dy, tile, attribute, dx) from its bottom-left corner."""
    chr_tiles = [np.zeros((8, 8), np.uint8), np.zeros((8, 8), np.uint8)]
    chr_tiles[1][0] = 2  # the floor: a grey line along the top of the row
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
    data = b"".join(chr_bytes(t) for t in chr_tiles)
    return data + bytes(8192 - len(data)), metas


def choreography() -> list[list[tuple[int, int, str | None]]]:
    """Every frame: each actor's (x, y of its feet, sprite frame or None). Actor 0 is Nate, the rest clips."""
    stop, turn, look, leap, run, clips = 96, 196, 212, 230, 262, 5
    frames = []
    for f in range(FRAMES):
        # Nate walks in (a pixel a frame) and stands; the swarm reaches him; he leaps, lands, and runs off.
        y = GROUND
        if f < turn:
            x = min(-24 + f, stop)
            pose = "pass" if x == stop else RUN[f // 8 % 4]
        elif f < leap:
            x, pose = stop, "front" if f < look else "look_back"  # he turns to us, then looks back at them
        elif f < run:
            x, pose = stop, "leap"
            y = GROUND - round(14 * np.sin(np.pi * (f - leap) / (run - leap)))
        else:
            x = stop + round(2.5 * (f - run))
            pose = RUN[(f - run) // 5 % 4]
        actors = [(x, y, pose if x < 256 else None)]
        nate_x = x
        # The clips hop in from the left, one after another, then chase him at his pace.
        for k in range(clips):
            start = 140 + 22 * k
            t = f - start
            travelled = max(0, min(f, run) - start) + 2.6 * max(0, f - max(run, start))
            # Follow the leader: never closer than a gap behind him, nor behind the clip ahead.
            x = min(-16 + round(travelled), nate_x - 20 - 14 * k)
            hop = round(abs(np.sin(np.pi * (t + 7 * k) / 18)) * 6)
            actors.append((x, GROUND - hop, "clip" if t >= 0 and x < 256 else None))
        frames.append(actors)
    return frames


def table(frames, names: list[str]) -> bytes:
    """Per frame, per actor: x low, x high, y, sprite frame number (0xFF: hidden)."""
    out = bytearray()
    for actors in frames:
        for x, y, name in actors:
            out += bytes([x & 0xFF, (x >> 8) & 0xFF, y & 0xFF, 0xFF if name is None else names.index(name)])
    return bytes(out)


def source(metas, frames) -> str:
    names = list(metas)
    lines = [f"ACTORS = {len(frames[0])}", f"FRAME_COUNT = {len(frames)}", f"FLOOR_ROW = {FLOOR_ROW}", "", ".segment \"RODATA\"", "palettes:",
             "  .byte " + ", ".join(f"${b:02X}" for b in PALETTES), "metas_lo:",
             "  .byte " + ", ".join(f"<meta_{n}" for n in names), "metas_hi:",
             "  .byte " + ", ".join(f">meta_{n}" for n in names)]
    for n in names:
        lines.append(f"meta_{n}:")
        for dy, tile, attr, dx in metas[n]:
            lines.append(f"  .byte ${dy & 0xFF:02X}, ${tile:02X}, ${attr:02X}, ${dx:02X}")
        lines.append("  .byte $80")
    lines.append("choreography:")
    data = table(frames, names)
    for i in range(0, len(data), 16):
        lines.append("  .byte " + ", ".join(f"${b:02X}" for b in data[i:i + 16]))
    return "\n".join(lines) + "\n"


def build(out: Path) -> Path:
    """Assemble chase.nes into `out`, with ca65 and ld65."""
    out.mkdir(parents=True, exist_ok=True)
    chr_data, metas = art()
    (out / "chase.chr").write_bytes(chr_data)
    (out / "data.inc").write_text(source(metas, choreography()))
    obj, rom = out / "chase.o", out / "chase.nes"
    subprocess.run(["ca65", str(HERE / "chase.s"), "-I", str(out), "--bin-include-dir", str(out), "-o", str(obj)],
                   check=True)
    subprocess.run(["ld65", "-C", str(HERE / "nrom.cfg"), str(obj), "-o", str(rom)], check=True)
    return rom


if __name__ == "__main__":
    from series.build import OUT
    print(build(Path(sys.argv[1]) if len(sys.argv) > 1 else OUT / "homebrew" / "chase"))
