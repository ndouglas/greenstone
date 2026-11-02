"""What video 002's animations show, computed from exp002's moments by the
same rules that gave its numbers, and cached for the beats to draw.

Every border, dot and score on screen comes from here; the tests check
that it agrees with numbers.json.

Usage: cd tools && series/.venv/bin/python -m series.lab.film002
"""

from pathlib import Path

import numpy as np

from series.lab import rules
from series.lab.data import load
from series.lab.exp001 import DATA, best_line
from series.lab.exp002 import (BATCH_RATE, BIGGER, CONTROL_PASSES, DROPS, LAST, LINE, ORDERS, PASSES, RATES,
                               SIXTEENTHS, TEST, TOO_BIG, TOO_BIG_STEPS, TRAIN, scroll_data)
from series.script import REPO

FILM = REPO / "target" / "series" / "002-every-mistake" / "film.npz"
BATCH_STEPS = 2000  # the batch square trick's moves, as exp002 runs it
BATCH_PERCEPTRON_STEPS = 300
# The U and the valley: how far either side of the square trick's resting
# weights each one is turned.
U_SPAN = 1.0
VALLEY_N = 61


def vec(line: rules.Line) -> np.ndarray:
    return np.r_[line.w, line.b]


def logged_run(x: np.ndarray, y: np.ndarray, rate: float, passes: int, seed: int,
               skip: np.ndarray | None = None) -> dict:
    """The perceptron from zero, as rules.train runs it, keeping every
    nudge: which moment caused it, the line after it, and its pass."""
    rng = np.random.default_rng(seed)
    line = rules.Line(np.zeros(x.shape[1]), 0.0)
    moment, after, when, ends = [], [], [], []
    for p in range(passes):
        for i in rng.permutation(len(y)):
            if skip is not None and skip[i]:
                continue
            new = rules.step("perceptron", line, x[i], int(y[i]), rate)
            if new is not line:
                moment.append(i)
                after.append(vec(new))
                when.append(p)
            line = new
        ends.append(vec(line))
    return {"moment": np.array(moment, int), "after": np.array(after).reshape(-1, 3),
            "pass": np.array(when, int), "ends": np.array(ends)}


def square_misses(v: np.ndarray, xb: np.ndarray, t: np.ndarray) -> np.ndarray:
    """The total squared miss (averaged over the moments) for each row of v."""
    return ((t[None, :] - v @ xb.T) ** 2).mean(1)


def compute(data_root: Path = DATA) -> dict:
    x, y = scroll_data([load(data_root / "smb1" / s) for s in TRAIN])
    xt, yt = scroll_data([load(data_root / "smb1" / s) for s in TEST])
    film: dict = {"x": x, "y": y, "xt": xt, "yt": yt, "mark": LINE / SIXTEENTHS}

    spots, owner, per_spot = np.unique(x, axis=0, return_inverse=True, return_counts=True)
    owner = owner.ravel()
    moved = np.bincount(owner, weights=y, minlength=len(spots)).astype(int)
    film.update(spots=spots, spot_of=owner, spot_moved=moved, spot_still=per_spot - moved)

    w, b, _ = best_line(x, y)
    search = rules.Line(w, b)
    film["search"] = vec(search)
    film["search_wrong"] = rules.says(search, x) != y

    rate = RATES["perceptron"]
    shown = logged_run(x, y, rate, PASSES, 0)
    film.update({f"run_{k}": v for k, v in shown.items()})
    film["run_scores"] = np.array([rules.accuracy(rules.Line(e[:2], e[2]), xt, yt) for e in shown["ends"]])
    film["run_bigger_ends"] = np.array([vec(l) for l in rules.train("perceptron", x, y, BIGGER * rate, PASSES, 0)])
    film["bigger"] = BIGGER

    runs = [rules.train("perceptron", x, y, rate, PASSES, seed) for seed in range(ORDERS)]
    film["order_ends"] = np.array([[vec(l) for l in r] for r in runs])
    film["order_scores"] = np.array([[rules.accuracy(l, xt, yt) for l in r] for r in runs])
    pockets = [rules.pocket(r, x, y) for r in runs]
    film["pocket"] = np.array([vec(l) for l in pockets])
    film["pocket_scores"] = np.array([rules.accuracy(l, xt, yt) for l in pockets])
    film["pocket_pass"] = np.array([next(i for i, l in enumerate(r) if l is p) for r, p in zip(runs, pockets)])

    dropped, dropped_scores = [], []
    for k in range(ORDERS):
        skip = np.zeros(len(y), bool)
        skip[np.random.default_rng(100 + k).choice(len(y), DROPS, replace=False)] = True
        end = rules.train("perceptron", x, y, rate, PASSES, 0, skip=skip)[-1]
        dropped.append(vec(end))
        dropped_scores.append(rules.accuracy(end, xt, yt))
    film["dropped"], film["dropped_scores"] = np.array(dropped), np.array(dropped_scores)

    film["late_nudges"] = rules.nudges(x, y, rate, PASSES, 0, count_from=PASSES - LAST + 1)

    split = rules.says(search, x) == y
    control = logged_run(x[split], y[split], rate, CONTROL_PASSES, 0)
    film["control_split"] = split
    film.update({f"control_{k}": v for k, v in control.items()})

    square_runs = [rules.train("square", x, y, RATES["square"], PASSES, seed) for seed in range(ORDERS)]
    film["square_scores"] = np.array([[rules.accuracy(l, xt, yt) for l in r] for r in square_runs])
    film["square_ends"] = np.array([[vec(l) for l in r] for r in square_runs])

    t = np.where(y == 1, rules.CODES[1], rules.CODES[0])
    xb = np.c_[x, np.ones(len(x))]
    v, path = np.zeros(3), [np.zeros(3)]
    for _ in range(BATCH_STEPS):
        v = v + BATCH_RATE * xb.T @ (t - xb @ v) / len(t)
        path.append(v.copy())
    path = np.array(path)
    film["batch_square"] = path
    film["batch_square_misses"] = square_misses(path, xb, t)
    film["batch_square_scores"] = np.array([rules.accuracy(rules.Line(p[:2], p[2]), xt, yt) for p in path[::10]])
    v, big = np.zeros(3), [np.zeros(3)]
    for _ in range(TOO_BIG_STEPS):
        v = v + TOO_BIG * BATCH_RATE * xb.T @ (t - xb @ v) / len(t)
        big.append(v.copy())
    film["too_big"] = np.array(big)
    film["too_big_misses"] = square_misses(np.array(big), xb, t)
    film["too_big_factor"] = TOO_BIG

    lines = rules.batch_perceptron(x, y, rate=0.1, steps=BATCH_PERCEPTRON_STEPS)
    film["batch_perceptron"] = np.r_[[np.zeros(3)], [vec(l) for l in lines]]
    film["batch_perceptron_scores"] = np.array([rules.accuracy(l, xt, yt) for l in lines])

    film["hinge"] = vec(rules.batch_squared_hinge(x, y, rate=0.1, steps=5000))

    # The U: every weight held at the square trick's resting place but one.
    rest = path[-1]
    turns = np.linspace(-U_SPAN, U_SPAN, 201)
    for k, name in enumerate(("position", "speed", "bias")):
        probe = np.repeat(rest[None], len(turns), 0)
        probe[:, k] += turns
        film[f"u_{name}"] = square_misses(probe, xb, t)
    film["u_turns"] = turns
    # The valley: both weights turned, the bias held.
    grid = np.linspace(-U_SPAN, U_SPAN, VALLEY_N)
    gw, gs = np.meshgrid(grid, grid, indexing="ij")
    probe = np.stack([rest[0] + gw.ravel(), rest[1] + gs.ravel(), np.full(gw.size, rest[2])], 1)
    film["valley"] = square_misses(probe, xb, t).reshape(VALLEY_N, VALLEY_N)
    film["valley_turns"] = grid
    return film


def save(film: dict, path: Path = FILM) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    np.savez_compressed(path, **film)


def load_film(path: Path = FILM) -> dict:
    """The cached film data; compute it first with `python -m series.lab.film002`."""
    if not path.exists():
        raise FileNotFoundError(f"{path} doesn't exist; run: cd tools && series/.venv/bin/python -m series.lab.film002")
    with np.load(path) as f:
        return {k: f[k] for k in f.files}


if __name__ == "__main__":
    save(compute())
    print(FILM)
