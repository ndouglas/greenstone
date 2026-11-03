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


def train(rule: str, x: np.ndarray, y: np.ndarray, rate: float, passes: int, seed: int,
          skip: np.ndarray | None = None) -> list[Line]:
    """Start at zero; each pass visits every moment once, in a shuffled
    order. Moments marked in `skip` are left out without changing the
    order the others come in. Returns the line after each pass."""
    rng = np.random.default_rng(seed)
    line = Line(np.zeros(x.shape[1]), 0.0)
    after = []
    for _ in range(passes):
        for i in rng.permutation(len(y)):
            if skip is None or not skip[i]:
                line = step(rule, line, x[i], int(y[i]), rate)
        after.append(line)
    return after


def nudges(x: np.ndarray, y: np.ndarray, rate: float, passes: int, seed: int, count_from: int = 1) -> np.ndarray:
    """How many times each moment moved the perceptron's line, over a run
    like `train`'s, counting from pass `count_from` (1 is the first)."""
    rng = np.random.default_rng(seed)
    line, counts = Line(np.zeros(x.shape[1]), 0.0), np.zeros(len(y), dtype=int)
    for p in range(1, passes + 1):
        for i in rng.permutation(len(y)):
            after = step("perceptron", line, x[i], int(y[i]), rate)
            if p >= count_from:
                counts[i] += after is not line
            line = after
    return counts


def sizes_before_nudges(x: np.ndarray, y: np.ndarray, rate: float, passes: int, seed: int,
                        count_from: int = 1) -> np.ndarray:
    """The size of the perceptron's weights and bias, in steps, just
    before each nudge from pass `count_from` on, over a run like `train`'s."""
    rng = np.random.default_rng(seed)
    line, sizes = Line(np.zeros(x.shape[1]), 0.0), []
    for p in range(1, passes + 1):
        for i in rng.permutation(len(y)):
            after = step("perceptron", line, x[i], int(y[i]), rate)
            if after is not line and p >= count_from:
                sizes.append(np.linalg.norm(np.r_[line.w, line.b]) / rate)
            line = after
    return np.array(sizes)


def weights_around_nudges(x: np.ndarray, y: np.ndarray, rate: float, passes: int, seed: int,
                          count_from: int = 1) -> tuple[np.ndarray, np.ndarray]:
    """The perceptron's weights and bias (as rows: weights, then bias) just
    before and just after each nudge from pass `count_from` on, over a run
    like `train`'s."""
    rng = np.random.default_rng(seed)
    line, before, after = Line(np.zeros(x.shape[1]), 0.0), [], []
    for p in range(1, passes + 1):
        for i in rng.permutation(len(y)):
            moved = step("perceptron", line, x[i], int(y[i]), rate)
            if moved is not line and p >= count_from:
                before.append(np.r_[line.w, line.b])
                after.append(np.r_[moved.w, moved.b])
            line = moved
    return np.array(before), np.array(after)


def averaged_perceptron(x: np.ndarray, y: np.ndarray, rate: float, passes: int, seed: int) -> Line:
    """The perceptron, answering with the average of every line it passed
    through (one after each moment visited), not the last."""
    rng = np.random.default_rng(seed)
    line, total, n = Line(np.zeros(x.shape[1]), 0.0), np.zeros(x.shape[1] + 1), 0
    for _ in range(passes):
        for i in rng.permutation(len(y)):
            line = step("perceptron", line, x[i], int(y[i]), rate)
            total, n = total + np.r_[line.w, line.b], n + 1
    return Line(total[:-1] / n, float(total[-1] / n))


def pocket(lines: list[Line], x: np.ndarray, y: np.ndarray) -> Line:
    """Of the lines a run passed through, the one with fewest mistakes on
    the moments learned from."""
    return max(lines, key=lambda l: accuracy(l, x, y))


def batch_perceptron(x: np.ndarray, y: np.ndarray, rate: float, steps: int) -> list[Line]:
    """The perceptron on the whole batch: every mistake's nudge averaged
    into one step. Returns the line after each step."""
    sign = np.where(y == 1, 1.0, -1.0)
    xb, v, after = _with_bias(x), np.zeros(x.shape[1] + 1), []
    for _ in range(steps):
        wrong = (xb @ v > 0).astype(int) != y
        v = v + rate * (xb[wrong] * sign[wrong, None]).sum(0) / len(y)
        after.append(Line(v[:-1], float(v[-1])))
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


def batch_square_step_sizes(x: np.ndarray, y: np.ndarray, rate: float, steps: int) -> list[float]:
    """The batch square trick from zero: how far each step moves the
    weights and bias."""
    t = np.where(y == 1, CODES[1], CODES[0])
    xb, v, sizes = _with_bias(x), np.zeros(x.shape[1] + 1), []
    for _ in range(steps):
        move = rate * xb.T @ (t - xb @ v) / len(t)
        sizes.append(float(np.linalg.norm(move)))
        v = v + move
    return sizes


def batch_square_misses(x: np.ndarray, y: np.ndarray, rate: float, steps: int) -> list[float]:
    """The batch square trick from zero, returning the total squared miss
    (averaged over the moments) before the first step and after each."""
    t = np.where(y == 1, CODES[1], CODES[0])
    xb, v = _with_bias(x), np.zeros(x.shape[1] + 1)
    misses = [float((t ** 2).mean())]
    for _ in range(steps):
        v = v + rate * xb.T @ (t - xb @ v) / len(t)
        misses.append(float(((t - xb @ v) ** 2).mean()))
    return misses


def batch_squared_hinge(x: np.ndarray, y: np.ndarray, rate: float, steps: int, start: Line | None = None) -> Line:
    """The batch square trick, except that a moment already past its
    target pulls nothing: no pull for being right by too much."""
    t = np.where(y == 1, CODES[1], CODES[0])
    xb = _with_bias(x)
    v = np.zeros(x.shape[1] + 1) if start is None else np.r_[start.w, start.b]
    for _ in range(steps):
        out = xb @ v
        short = t * out < 1
        v = v + rate * xb[short].T @ (t[short] - out[short]) / len(t)
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


def landscape(x: np.ndarray, y: np.ndarray, angles: np.ndarray, offsets: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    """Every border of two inputs, as (angle, offset): w = (cos, sin), and
    the border where w·x + offset = 0. Scores each by its share of
    mistakes and by the square trick's error at its best scale (a border
    doesn't fix how big the weights are; the square trick's error does
    depend on it, so each border gets the size that suits it best)."""
    t = np.where(y == 1, CODES[1], CODES[0])
    mistakes = np.empty((len(angles), len(offsets)))
    squared = np.empty_like(mistakes)
    for i, a in enumerate(angles):
        f = (x @ np.array([np.cos(a), np.sin(a)]))[None, :] + offsets[:, None]
        mistakes[i] = ((f > 0).astype(int) != y).mean(1)
        scale = np.maximum((f * t).sum(1) / (f * f).sum(1), 0.0)
        squared[i] = ((t - scale[:, None] * f) ** 2).mean(1)
    return mistakes, squared


def mistakes_at(x: np.ndarray, y: np.ndarray, angles: np.ndarray, offsets: np.ndarray) -> np.ndarray:
    """The share of mistakes of every border, as `landscape` scores them, but by sorting: for each angle,
    the moments are sorted along it once, and every offset's count is then a lookup. Cheap enough to look
    at the staircase as finely as its real ledges."""
    out = np.empty((len(angles), len(offsets)))
    yes = y == 1
    for i, a in enumerate(angles):
        p = x @ np.array([np.cos(a), np.sin(a)])
        py, pn = np.sort(p[yes]), np.sort(p[~yes])
        # says yes where p + offset > 0: a yes at or below -offset is missed, a no above it is a false yes
        missed = np.searchsorted(py, -offsets, side="right")
        false_yes = len(pn) - np.searchsorted(pn, -offsets, side="right")
        out[i] = (missed + false_yes) / len(y)
    return out


def right_by_too_much(line: Line, x: np.ndarray, y: np.ndarray) -> np.ndarray:
    """Moments the line answers right, but whose sum overshoots the square
    trick's target: the square trick still pulls the line toward them."""
    out = x @ line.w + line.b
    return (says(line, x) == y) & (np.abs(out) > max(abs(c) for c in CODES))


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
