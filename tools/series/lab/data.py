"""Recordings of simulated play, as written by Greenstone's series recorder."""

import json
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

import numpy as np

RAM_BYTES = 2048


@dataclass
class Recording:
    ram: np.ndarray  # (frames, 2048) uint8: the console's memory after each frame
    buttons: np.ndarray  # (frames,) uint8: what was pressed during each frame
    meta: dict


def load(path: Path) -> Recording:
    path = Path(path)
    ram = np.fromfile(path / "ram.u8", dtype=np.uint8)
    return Recording(
        ram=ram.reshape(-1, RAM_BYTES),
        buttons=np.fromfile(path / "buttons.u8", dtype=np.uint8),
        meta=json.loads((path / "meta.json").read_text()),
    )


def dataset(recs: list[Recording], inputs: list[int], label: Callable[[np.ndarray], np.ndarray],
            min_rate: float = 0.05) -> tuple[np.ndarray, np.ndarray]:
    """The chosen bytes of every frame, and the label for each frame.

    Refuses a dataset where either answer is rarer than `min_rate`: a line
    can't be judged on a question whose answer almost never changes."""
    ram = np.concatenate([r.ram for r in recs])
    x = ram[:, inputs].astype(float)
    y = label(ram).astype(int)
    check_rate(y, min_rate)
    return x, y


def check_rate(y: np.ndarray, min_rate: float = 0.05) -> None:
    """Refuse labels where either answer is rarer than `min_rate`."""
    rate = np.mean(y)
    if not min_rate <= rate <= 1 - min_rate:
        raise ValueError(f"the label is true on {rate:.1%} of frames; need between {min_rate:.0%} and {1 - min_rate:.0%}")
