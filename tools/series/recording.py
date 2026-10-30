"""A game's footage beside the RAM that produced it, lined up frame for frame."""

from pathlib import Path
from typing import Callable

import numpy as np

from reservoir_film.model import Takes
from series.build import OUT
from series.lab.data import RAM_BYTES

DATA = OUT / "001-one-neuron" / "data"

# How many frames the picture trails the RAM written for the same frame
# index, measured against the player's sprite (see measure_lag).
LAG = {"smb1": 1}
# Bytes whose lag differs from their game's, each measured against the sprite the same way. SMB1 writes
# Mario's screen x ($03AD) before the picture of the same frame is drawn.
BYTE_LAG = {"smb1": {0x03AD: 0}}

MARIO_RED = (152, 34, 32)
MARIO_SHOE = (84, 90, 0)


class Recording:
    """The footage take and seed 1's RAM of one game."""

    def __init__(self, game: str, root: Path = DATA):
        self.game = game
        root = Path(root)
        self.frames = Takes(root / game, "record the footage take")["footage"].frames("game")
        ram_path = root / game / "1" / "ram.u8"
        if not ram_path.is_file():
            raise FileNotFoundError(
                f"{game}: no RAM at {ram_path} to go with the footage's {len(self.frames)} frames; record it with: "
                f"cargo run --release --example series_record -- {game} PLAYER FRAMES OUT --footage")
        self.ram = np.fromfile(ram_path, dtype=np.uint8).reshape(-1, RAM_BYTES)
        if len(self.ram) < len(self.frames):
            raise ValueError(f"{game}: the footage has {len(self.frames)} frames but the RAM only {len(self.ram)}")

    def __len__(self) -> int:
        return len(self.frames)

    def ram_at(self, frame: int) -> np.ndarray:
        """The RAM the picture of `frame` shows."""
        if self.game not in LAG:
            raise KeyError(f"{self.game}'s lag hasn't been measured; find a sprite rule and run "
                           f"series.recording.measure_lag against its footage, then add it to LAG")
        return self.ram[max(frame - LAG[self.game], 0)]

    def byte_at(self, frame: int, address: int) -> int:
        """The value of one byte as the picture of `frame` shows it, at that byte's own lag if it has one."""
        lag = BYTE_LAG.get(self.game, {}).get(address)
        if lag is None:
            return int(self.ram_at(frame)[address])
        return int(self.ram[max(frame - lag, 0)][address])


def measure_lag(frames, ram: np.ndarray, sprite_bottom: Callable[[np.ndarray], int | None],
                byte: int, offset: int, max_lag: int = 3, span: range | None = None) -> tuple[int, float]:
    """The lag (in frames) at which the sprite's bottom row best equals
    `ram[f - lag][byte] + offset`, and the share of frames where it does
    exactly. Only frames where the byte is moving count, and only `span`
    if given."""
    span = span if span is not None else range(max_lag + 1, min(len(frames), len(ram)))
    if span.start < max_lag or span.stop > min(len(frames), len(ram)):
        raise ValueError(f"span {span} must lie within frames {max_lag} to {min(len(frames), len(ram))}: "
                         f"each frame reads up to {max_lag} frames back and the frames and RAM run out there")
    hits = np.zeros(max_lag + 1)
    total = 0
    for f in span:
        if abs(int(ram[f][byte]) - int(ram[f - 1][byte])) <= 1:
            continue
        row = sprite_bottom(frames[f])
        if row is None:
            continue
        total += 1
        for lag in range(max_lag + 1):
            hits[lag] += row == int(ram[f - lag][byte]) + offset
    if not total:
        raise ValueError("no moving frames with a sprite to measure")
    lag = int(np.argmax(hits))
    return lag, float(hits[lag] / total)


def smb1_sprite_bottom(frame: np.ndarray) -> int | None:
    """The row of Mario's lowest shoe pixel, or None when he isn't clearly there."""
    red = np.argwhere((frame == MARIO_RED).all(axis=2))
    if len(red) < 10:
        return None
    cy, cx = np.median(red, axis=0).astype(int)
    r0, c0 = max(cy - 4, 0), max(cx - 10, 0)
    window = frame[r0:cy + 24, c0:cx + 10]
    shoes = np.flatnonzero((window == MARIO_SHOE).all(axis=2).any(axis=1))
    return int(r0 + shoes[-1]) if len(shoes) else None
