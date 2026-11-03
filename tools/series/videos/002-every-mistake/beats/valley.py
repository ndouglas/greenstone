"""The square trick's valley, for chapter 3: the total squared miss as the position and speed knobs turn about
where the square trick rests (the bias held still), computed exactly from the moments, and each moment's real
nudge on those two knobs.

Both knobs are drawn in the same units (a knob turn is as long across as it is deep), so the added-up nudge,
which points straight down the steepest way, crosses the contours square on."""

import numpy as np

from series.terrain import DEEP, Ground, View, WIDE

from beats.common import FILM

RATE = 0.1  # the batch square trick's step (exp002's BATCH_RATE)
X, Y = FILM["x"], FILM["y"]
T = np.where(Y == 1, 1.0, -1.0)
XB = np.c_[X, np.ones(len(X))]
MOMENTS = len(T)
REST = FILM["batch_square"][-1]
SPAN = 1.0  # how far either side of rest each knob turns, as the U does (film002's U_SPAN)
ACROSS = (0.5 - DEEP / WIDE / 2, 0.5 + DEEP / WIDE / 2)  # the valley's share of the sheet's width: as wide as deep
UP = (0.0, 1.0)
CONTOUR = 1.0  # a contour at every whole total squared miss

_A = XB.T @ XB / MOMENTS  # the total squared miss is a bowl in the knobs: c − 2 w·b + w·A w, exactly
_B = XB.T @ T / MOMENTS
_C = float(T @ T / MOMENTS)


def miss(w0, w1) -> np.ndarray:
    """The total squared miss (averaged over the moments) with the position and speed knobs at w0 and w1 and
    the bias held at rest."""
    w0, w1 = np.broadcast_arrays(np.asarray(w0, float), np.asarray(w1, float))
    w = np.stack([w0, w1, np.full(w0.shape, REST[2])], -1)
    return _C - 2 * w @ _B + np.einsum("...i,ij,...j->...", w, _A, w)


def shares(w0, w1) -> tuple[np.ndarray, np.ndarray]:
    """Where knob settings sit on the drawn sheet, as its shares across and up."""
    a = ACROSS[0] + (np.asarray(w0) - REST[0] + SPAN) / (2 * SPAN) * (ACROSS[1] - ACROSS[0])
    return a, UP[0] + (np.asarray(w1) - REST[1] + SPAN) / (2 * SPAN) * (UP[1] - UP[0])


def ground(cells: int = 121, rows: int | None = None) -> Ground:
    """The valley, or (with `rows`) the slice of it `rows` cells deep about the speed knob's resting place."""
    turns = np.linspace(-SPAN, SPAN, cells)
    d = (turns[1] - turns[0]) / 2
    keep = slice(None) if rows is None else slice((cells - rows) // 2, (cells + rows) // 2)
    deep = turns[keep]
    v = miss(REST[0] + turns[:, None], REST[1] + deep[None, :])
    across, up = shares(REST[0] + np.array([-SPAN - d, SPAN + d]), REST[1] + np.array([deep[0] - d, deep[-1] + d]))
    return Ground(v, TOP, smooth=True, across=tuple(across), up=tuple(up), contour=CONTOUR, colours=(0.0, HIGHEST))


_corners = np.array([-SPAN, SPAN])
HIGHEST = float(miss(REST[0] + _corners[:, None], REST[1] + _corners[None, :]).max())  # the valley's highest corner
TOP = HIGHEST  # drawn TALL high: chapter 3's one height scale
_a, _u = shares(REST[0] + _corners, REST[1] + _corners)
FLOOR = (*_a, *_u, 4.0, 4.0)  # the floor under the valley alone, a grid line every half a knob turn
SIDE = View(yaw=0.0, pitch=0.02, distance=16.0, target=(0.0, 0.0, 0.55), fov=24)  # side on: the U
VALLEY = View(yaw=-0.5, pitch=0.62, distance=13.0, target=(0.0, 0.1, 0.45), fov=28)


def pulls(w0: float, w1: float) -> np.ndarray:
    """Each moment's own pull on the position and speed knobs (its miss times its input), in recording order:
    the square trick nudges by these, times its step."""
    w = np.array([w0, w1, REST[2]])
    return (T - XB @ w)[:, None] * X


def chain(w0: float, w1: float) -> np.ndarray:
    """Every moment's nudge, laid tip to tail from (w0, w1), each drawn at 1/MOMENTS of its size: the chain ends
    where one move of the added-up square trick goes."""
    steps = RATE * pulls(w0, w1) / MOMENTS
    return np.array([w0, w1]) + np.vstack([[0.0, 0.0], np.cumsum(steps, 0)])
