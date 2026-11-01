"""Video 002's experiments: 001's landing neuron (depth and vertical speed;
will Mario land within 10 frames?) learning its own border, by each rule
in `rules.py`, against the border 001 found by search. Learned on
recordings 1-2, scored on recording 3.

Usage: cd tools && series/.venv/bin/python -m series.lab.exp002
"""

import json
from pathlib import Path

import numpy as np

from series.lab import rules
from series.lab.data import load
from series.lab.exp001 import DATA, best_line, land_data, percent, spoken_number
from series.script import REPO, VIDEOS

OUT = VIDEOS / "002-every-mistake" / "numbers.json"
# The two maps of every border, for the animations (large; not committed).
MAPS = REPO / "target" / "series" / "002-every-mistake" / "maps.npz"
ANGLES = np.radians(np.arange(0, 360, 0.5))
OFFSETS = np.linspace(-3, 3, 601)

PASSES = 30
ORDERS = 10  # each rule is run with this many shuffled orders
LAST = 10  # how settled a rule is: its swing over the last passes
DROPS = 30  # moments left out, to see how much a tiny change moves the end
RATES = {"simple": 0.01, "perceptron": 0.01, "square": 0.001}


def check_claims(perceptron_rate_free: bool, batch_is_least_squares: bool, codes_agree: bool,
                 bowl_is_least_squares: bool = True) -> None:
    """Things the script will rest on; refuse to write numbers if any fails."""
    if not perceptron_rate_free:
        raise AssertionError("the perceptron's border changed with its step size")
    if not batch_is_least_squares:
        raise AssertionError("the batch square trick didn't come to rest at least squares")
    if not codes_agree:
        raise AssertionError("the square trick's border depends on how the answers are coded")
    if not bowl_is_least_squares:
        raise AssertionError("the bowl's lowest point isn't the least-squares border")


def run(data_root: Path = DATA, out: Path = OUT, maps: Path = MAPS) -> dict:
    numbers: dict = {}

    def put(name: str, value: float) -> None:
        numbers[name], numbers[name + "_spoken"] = percent(value)

    def put_int(name: str, value: float) -> None:
        """A whole number (halves round up), and how it's said."""
        n = int(np.floor(value + 0.5))
        numbers[name], numbers[name + "_spoken"] = n, spoken_number(n)

    x, y = land_data([load(data_root / "smb1" / s) for s in ("1", "2")])
    xt, yt = land_data([load(data_root / "smb1" / "3")])
    # Standardized on the moments learned from, so one step size suits both inputs.
    mean, scale = x.mean(0), x.std(0)
    z, zt = (x - mean) / scale, (xt - mean) / scale
    put_int("train_moments", len(y))
    put_int("test_moments", len(yt))
    put("lands_soon_rate", yt.mean())

    w, b, _ = best_line(x, y)
    search = rules.Line(w * scale, float(b + w @ mean))
    put("search", rules.accuracy(search, zt, yt))

    for rule, rate in RATES.items():
        runs = [rules.train(rule, z, y, rate, PASSES, seed) for seed in range(ORDERS)]
        scores = [[rules.accuracy(l, zt, yt) for l in r] for r in runs]
        finals = [s[-1] for s in scores]
        put(f"{rule}_lowest", min(finals))
        put(f"{rule}_highest", max(finals))
        put(f"{rule}_swing", float(np.median([rules.swing(s, LAST) for s in scores])))
        # The same swing in percentage points, to be said as "points".
        put_int(f"{rule}_swing_points", 100 * float(np.median([rules.swing(s, LAST) for s in scores])))
        put_int(f"{rule}_turn_degrees", float(np.median([rules.turn(r[-LAST:]) for r in runs])))
        numbers[f"{rule}_one_run"] = [percent(s)[0] for s in scores[0][-LAST:]]
        for n, score in enumerate(scores[0][-LAST:]):
            put(f"{rule}_one_run_{PASSES - LAST + n + 1}", score)

    # A tiny change: leave out a few moments, keep everything else the same.
    finals = []
    for k in range(ORDERS):
        keep = np.ones(len(y), bool)
        keep[np.random.default_rng(100 + k).choice(len(y), DROPS, replace=False)] = False
        finals.append(rules.accuracy(rules.train("perceptron", z[keep], y[keep], RATES["perceptron"], PASSES, 0)[-1], zt, yt))
    put("perceptron_dropped_lowest", min(finals))
    put("perceptron_dropped_highest", max(finals))
    put_int("dropped_moments", DROPS)

    small = rules.train("perceptron", z, y, 0.01, 3, 0)
    large = rules.train("perceptron", z, y, 1.0, 3, 0)
    rate_free = all((rules.says(a, zt) == rules.says(c, zt)).all() for a, c in zip(small, large))

    batch = rules.batch_square(z, y, rate=0.1, steps=2000)
    exact = rules.least_squares(z, y)
    zero_one = rules.least_squares(z, y, codes=(0.0, 1.0))
    logistic = rules.batch_logistic(z, y, rate=0.5, steps=5000)
    put("batch_square", rules.accuracy(batch, zt, yt))
    put("logistic", rules.accuracy(logistic, zt, yt))
    # How differently the settled borders answer from search's, and how far they lean from it.
    put("batch_square_disagrees_with_search", rules.disagreement(batch, search, zt))
    put("logistic_disagrees_with_search", rules.disagreement(logistic, search, zt))
    put_int("batch_square_lean_from_search_degrees", rules.turn([batch, search]))
    put_int("logistic_lean_from_search_degrees", rules.turn([logistic, search]))
    # Does the square trick's shortfall depend on the step size chosen?
    for rate in (0.0003, 0.003):
        finals = [rules.accuracy(rules.train("square", z, y, rate, PASSES, s)[-1], zt, yt) for s in range(3)]
        put(f"square_rate_{rate}_lowest", min(finals))
        put(f"square_rate_{rate}_highest", max(finals))

    # Batching doesn't stop the perceptron wobbling: every mistake's nudge
    # at once, averaged into one step.
    batched = [rules.accuracy(l, zt, yt) for l in rules.batch_perceptron(z, y, rate=0.1, steps=300)]
    put("batch_perceptron_lowest", min(batched[-100:]))
    put("batch_perceptron_highest", max(batched[-100:]))
    # Remembering instead of settling: the best line a run passed through
    # (judged on the moments learned from), or the average of all of them.
    runs = [rules.train("perceptron", z, y, RATES["perceptron"], PASSES, seed) for seed in range(ORDERS)]
    pockets = [rules.accuracy(rules.pocket(r, z, y), zt, yt) for r in runs]
    put("pocket_lowest", min(pockets))
    put("pocket_highest", max(pockets))
    averaged = [rules.accuracy(rules.averaged_perceptron(z, y, RATES["perceptron"], PASSES, seed), zt, yt)
                for seed in range(3)]
    put("averaged_lowest", min(averaged))
    put("averaged_highest", max(averaged))
    # Which moments keep the perceptron moving: the share of all nudges
    # made by the busiest tenth of moments.
    counts = np.sort(rules.nudges(z, y, RATES["perceptron"], PASSES, 0))[::-1]
    put("busiest_tenth_share_of_nudges", counts[: len(counts) // 10].sum() / counts.sum())
    put("never_nudged", (counts == 0).mean())
    # The square trick listens to moments it already gets right, by too much.
    far = rules.right_by_too_much(exact, z, y)
    put("right_by_too_much", far.mean())
    numbers["right_by_too_much_median_depth"] = float(np.median(x[far, 0]))
    numbers["right_by_too_much_median_speed"] = float(np.median(x[far, 1]))
    numbers["median_depth"], numbers["median_speed"] = float(np.median(x[:, 0])), float(np.median(x[:, 1]))
    put("right_by_too_much_rising", (x[far, 1] < 0).mean())

    # The two maps of every border: mistakes (a staircase), squared error (a bowl).
    mistakes, squared = rules.landscape(z, y, ANGLES, OFFSETS)
    i, j = np.unravel_index(np.argmin(mistakes), mistakes.shape)
    k, l = np.unravel_index(np.argmin(squared), squared.shape)
    stairs_bottom = rules.Line(np.array([np.cos(ANGLES[i]), np.sin(ANGLES[i])]), float(OFFSETS[j]))
    bowl_bottom = rules.Line(np.array([np.cos(ANGLES[k]), np.sin(ANGLES[k])]), float(OFFSETS[l]))
    put("stairs_bottom", rules.accuracy(stairs_bottom, zt, yt))
    put("bowl_bottom", rules.accuracy(bowl_bottom, zt, yt))
    put_int("bottoms_apart_degrees", rules.turn([stairs_bottom, bowl_bottom]))
    # How flat the staircase is: the share of neighbouring borders (one
    # offset step apart) with exactly the same number of mistakes.
    put("stairs_flat", (np.diff(mistakes, axis=1) == 0).mean())
    maps.parent.mkdir(parents=True, exist_ok=True)
    np.savez_compressed(maps, angles=ANGLES, offsets=OFFSETS, mistakes=mistakes, squared=squared,
                        mean=mean, scale=scale)

    check_claims(
        bowl_is_least_squares=rules.turn([bowl_bottom, exact]) < 1.0,
        perceptron_rate_free=rate_free,
        batch_is_least_squares=bool(np.allclose(np.r_[batch.w, batch.b], np.r_[exact.w, exact.b], atol=1e-6)),
        codes_agree=bool((rules.says(exact, zt) == rules.says(zero_one, zt)).all()),
    )
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(numbers, indent=2, sort_keys=True) + "\n")
    return numbers


if __name__ == "__main__":
    for k, v in run().items():
        if not k.endswith("_spoken"):
            print(f"{k}: {v}")
