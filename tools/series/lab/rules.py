"""Rules for moving a neuron's line: 002's experiments.

A line says yes (1) where w·x + b > 0. The per-moment rules look at one
moment at a time and nudge the line; the batch rules average every
moment's nudge and take one step. The names follow Nate's
`ndouglas/linear-regression` (after Serrano, *Grokking Machine Learning*),
translated from fitting a line through points to drawing one between them.
"""

from dataclasses import dataclass

import numpy as np

# The square trick's targets for "no" and "yes".
CODES = (-1.0, 1.0)


@dataclass(frozen=True)
class Line:
    w: np.ndarray
    b: float


def says(line: Line, x: np.ndarray) -> np.ndarray:
    """The line's answer for each moment: 1 on the side w points to."""
    return (x @ line.w + line.b > 0).astype(int)


def accuracy(line: Line, x: np.ndarray, y: np.ndarray) -> float:
    return float((says(line, x) == y).mean())


def step(rule: str, line: Line, x: np.ndarray, y: int, rate: float) -> Line:
    """One nudge, after looking at one moment.

    perceptron: only if wrong, move by the input, times a fixed step.
    simple:     only if wrong, move each weight a fixed step by its input's sign.
    square:     always, move by the input times the error (Widrow-Hoff).
    """
    sign = 1.0 if y else -1.0
    if rule == "square":
        error = CODES[y] - (x @ line.w + line.b)
        return Line(line.w + rate * error * x, line.b + rate * error)
    if says(line, x[None])[0] == y:
        return line
    if rule == "perceptron":
        return Line(line.w + rate * sign * x, line.b + rate * sign)
    if rule == "simple":
        return Line(line.w + rate * sign * np.sign(x), line.b + rate * sign)
    raise ValueError(rule)


def train(rule: str, x: np.ndarray, y: np.ndarray, rate: float, passes: int, seed: int) -> list[Line]:
    """Start at zero; each pass visits every moment once, in a shuffled
    order. Returns the line after each pass."""
    rng = np.random.default_rng(seed)
    line = Line(np.zeros(x.shape[1]), 0.0)
    after = []
    for _ in range(passes):
        for i in rng.permutation(len(y)):
            line = step(rule, line, x[i], int(y[i]), rate)
        after.append(line)
    return after


def _with_bias(x: np.ndarray) -> np.ndarray:
    return np.c_[x, np.ones(len(x))]


def batch_square(x: np.ndarray, y: np.ndarray, rate: float, steps: int) -> Line:
    """The square trick, every moment's nudge averaged into one step."""
    t = np.where(y == 1, CODES[1], CODES[0])
    xb, v = _with_bias(x), np.zeros(x.shape[1] + 1)
    for _ in range(steps):
        v = v + rate * xb.T @ (t - xb @ v) / len(t)
    return Line(v[:-1], float(v[-1]))


def least_squares(x: np.ndarray, y: np.ndarray, codes: tuple[float, float] = CODES) -> Line:
    """Where the batch square trick comes to rest, solved directly. The
    border sits halfway between the two codes."""
    t = np.where(y == 1, codes[1], codes[0])
    v = np.linalg.lstsq(_with_bias(x), t, rcond=None)[0]
    return Line(v[:-1], float(v[-1]) - (codes[0] + codes[1]) / 2)


def batch_logistic(x: np.ndarray, y: np.ndarray, rate: float, steps: int) -> Line:
    """The logistic trick on the whole batch: each moment pulls by how far
    its squashed output (between 0 and 1) is from its answer."""
    xb, v = _with_bias(x), np.zeros(x.shape[1] + 1)
    for _ in range(steps):
        p = 1 / (1 + np.exp(-(xb @ v)))
        v = v + rate * xb.T @ (y - p) / len(y)
    return Line(v[:-1], float(v[-1]))


def swing(scores: list[float], last: int) -> float:
    """How far the score moves over the last passes: highest minus lowest."""
    tail = scores[-last:]
    return float(max(tail) - min(tail))


def turn(lines: list[Line]) -> float:
    """The widest angle, in degrees, between any two of the lines' directions."""
    u = np.array([l.w / np.linalg.norm(l.w) for l in lines])
    return float(np.degrees(np.arccos(np.clip(u @ u.T, -1, 1))).max())


def disagreement(a: Line, b: Line, x: np.ndarray) -> float:
    """The share of moments the two lines answer differently."""
    return float((says(a, x) != says(b, x)).mean())
