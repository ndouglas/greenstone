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
# The perceptron's step is a power of two, so that a step 8 times bigger
# scales every weight exactly, with no rounding, and the claim that the
# step doesn't matter can be checked exactly over a whole run.
RATES = {"simple": 0.01, "perceptron": 2.0 ** -7, "square": 0.001}
BIGGER = 8


def check_claims(perceptron_rate_free: bool, batch_is_least_squares: bool, codes_agree: bool,
                 bowl_is_least_squares: bool = True, control_settles: bool = True,
                 one_hollow: bool = True) -> None:
    """Things the script will rest on; refuse to write numbers if any fails."""
    if not perceptron_rate_free:
        raise AssertionError("the perceptron's border changed with its step size")
    if not batch_is_least_squares:
        raise AssertionError("the batch square trick didn't come to rest at least squares")
    if not codes_agree:
        raise AssertionError("the square trick's border depends on how the answers are coded")
    if not bowl_is_least_squares:
        raise AssertionError("the bowl's lowest point isn't the least-squares border")
    if not control_settles:
        raise AssertionError("the perceptron didn't stop on moments a line can split")
    if not one_hollow:
        raise AssertionError("the square trick's map has more than one low point where it slopes")


def local_lows(height: np.ndarray, flat: np.ndarray) -> int:
    """Spots on a map (angle across, wrapping round; offset up) lower than
    or level with all eight neighbours, leaving out the flat part."""
    low = np.ones(height.shape, bool)
    padded = np.pad(height, ((0, 0), (1, 1)), mode="edge")
    for da in (-1, 0, 1):
        for do in (-1, 0, 1):
            if da or do:
                low &= height <= np.roll(padded, da, axis=0)[:, 1 + do : padded.shape[1] - 1 + do]
    return int((low & ~flat).sum())


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

    # Why no border can be perfect: the game keeps depth and speed as whole
    # numbers, and many spots on Mario's map hold moments with both answers.
    spots, owner, per_spot = np.unique(x, axis=0, return_inverse=True, return_counts=True)
    owner = owner.ravel()
    landed = np.bincount(owner, weights=y, minlength=len(spots))
    both = (landed > 0) & (landed < per_spot)
    put_int("spots", len(spots))
    put_int("spots_with_both", both.sum())
    put("in_spots_with_both", both[owner].mean())
    # The best any answer could do on these moments: each spot's commoner answer.
    put("best_possible", np.maximum(landed, per_spot - landed).sum() / len(y))
    put_int("most_in_one_spot", per_spot.max())
    put("in_crowded_spots", (per_spot[owner] >= 50).mean())
    # Crossing one spot changes a border's mistakes by the difference
    # between that spot's landings and non-landings.
    put_int("spot_biggest_change", np.abs(2 * landed - per_spot).max())

    for rule, rate in RATES.items():
        runs = [rules.train(rule, z, y, rate, PASSES, seed) for seed in range(ORDERS)]
        scores = [[rules.accuracy(l, zt, yt) for l in r] for r in runs]
        finals = [s[-1] for s in scores]
        put(f"{rule}_lowest", min(finals))
        put(f"{rule}_highest", max(finals))
        put(f"{rule}_swing", float(np.median([rules.swing(s, LAST) for s in scores])))
        # The same swing in percentage points, to be said as "points".
        put_int(f"{rule}_swing_points", 100 * float(np.median([rules.swing(s, LAST) for s in scores])))
        # One run's last passes, to show (the first order; its swing is
        # checked against the median above in the notes).
        numbers[f"{rule}_one_run"] = [percent(s)[0] for s in scores[0][-LAST:]]
        put(f"{rule}_one_run_lowest", min(scores[0][-LAST:]))
        put(f"{rule}_one_run_highest", max(scores[0][-LAST:]))
        # Every order, every one of the last passes.
        put(f"{rule}_late_lowest", min(min(s[-LAST:]) for s in scores))
        put(f"{rule}_late_highest", max(max(s[-LAST:]) for s in scores))

    # A tiny change: leave out a few moments, and keep everything else,
    # the order of the rest included, the same.
    put("perceptron_undropped", rules.accuracy(rules.train("perceptron", z, y, RATES["perceptron"], PASSES, 0)[-1], zt, yt))
    finals = []
    for k in range(ORDERS):
        skip = np.zeros(len(y), bool)
        skip[np.random.default_rng(100 + k).choice(len(y), DROPS, replace=False)] = True
        finals.append(rules.accuracy(rules.train("perceptron", z, y, RATES["perceptron"], PASSES, 0, skip=skip)[-1], zt, yt))
    put("perceptron_dropped_lowest", min(finals))
    put("perceptron_dropped_highest", max(finals))
    put_int("dropped_moments", DROPS)

    # The control: on only the moments search's border gets right, a line
    # does split them, and the rule should stop.
    split = rules.says(search, z) == y
    control = rules.train("perceptron", z[split], y[split], RATES["perceptron"], PASSES, 0)
    put("control_score", rules.accuracy(control[-1], z[split], y[split]))
    control_settles = bool(np.array_equal(control[-1].w, control[-2].w) and control[-1].b == control[-2].b)

    # A step BIGGER times bigger: the same mistakes, so the same nudges,
    # bigger; every weight exactly BIGGER times, after every pass.
    small = rules.train("perceptron", z, y, RATES["perceptron"], PASSES, 0)
    large = rules.train("perceptron", z, y, BIGGER * RATES["perceptron"], PASSES, 0)
    rate_free = all(np.array_equal(BIGGER * a.w, c.w) and BIGGER * a.b == c.b for a, c in zip(small, large))
    put_int("bigger_step", BIGGER)

    batch = rules.batch_square(z, y, rate=0.1, steps=2000)
    exact = rules.least_squares(z, y)
    zero_one = rules.least_squares(z, y, codes=(0.0, 1.0))
    logistic = rules.batch_logistic(z, y, rate=0.5, steps=5000)
    put("batch_square", rules.accuracy(batch, zt, yt))
    put("logistic", rules.accuracy(logistic, zt, yt))
    # How differently the settled borders answer from search's, and how far they lean from it.
    put("batch_square_disagrees_with_search", rules.disagreement(batch, search, zt))
    put("logistic_disagrees_with_search", rules.disagreement(logistic, search, zt))
    # Every smooth score lands near search's border but not on it: even one
    # that stops pulling once a moment is past its target.
    hinge = rules.batch_squared_hinge(z, y, rate=0.1, steps=5000)
    put("hinge", rules.accuracy(hinge, zt, yt))
    put("hinge_disagrees_with_search", rules.disagreement(hinge, search, zt))
    # Does the square trick's shortfall depend on the step size chosen?
    for rate in (0.0003, 0.003):
        finals = [rules.accuracy(rules.train("square", z, y, rate, PASSES, s)[-1], zt, yt) for s in range(3)]
        put(f"square_rate_{rate}_lowest", min(finals))
        put(f"square_rate_{rate}_highest", max(finals))

    # Batching doesn't stop the perceptron wobbling: every mistake's nudge
    # at once, averaged into one step.
    lines = rules.batch_perceptron(z, y, rate=0.1, steps=300)
    batched = [rules.accuracy(l, zt, yt) for l in lines[-100:]]
    put("batch_perceptron_lowest", min(batched))
    put("batch_perceptron_highest", max(batched))
    put("batch_perceptron_median", float(np.median(batched)))
    put("batch_perceptron_low_tenth", float(np.percentile(batched, 10)))
    # Its weights stay near zero, where its own score (how far wrong the
    # wrong moments are) is lowest when no line splits the moments.
    numbers["batch_perceptron_largest_weights"] = round(max(float(np.linalg.norm(l.w)) for l in lines), 4)
    # The same, in its own nudges: a direction-only rule's weights mean
    # something only against the size of the steps it takes.
    v = np.array([np.r_[l.w, l.b] for l in lines])
    numbers["batch_perceptron_weights_in_nudges"] = round(float(
        np.median(np.linalg.norm(v[1:], axis=1)) / np.median(np.linalg.norm(np.diff(v, axis=0), axis=1))), 1)
    # Remembering instead of settling: the best line a run passed through
    # (judged on the moments learned from), or the average of all of them.
    runs = [rules.train("perceptron", z, y, RATES["perceptron"], PASSES, seed) for seed in range(ORDERS)]
    pockets = [rules.accuracy(rules.pocket(r, z, y), zt, yt) for r in runs]
    put("pocket_lowest", min(pockets))
    put("pocket_highest", max(pockets))
    averaged = [rules.accuracy(rules.averaged_perceptron(z, y, RATES["perceptron"], PASSES, seed), zt, yt)
                for seed in range(ORDERS)]
    put("averaged_lowest", min(averaged))
    put("averaged_highest", max(averaged))
    # Which moments keep the perceptron moving, over the last passes
    # (after it's had time to settle, if it could), for every order: the
    # busiest tenth's share of the nudges, and how many of them search's
    # border gets wrong too.
    shares, wrong_too, never = [], [], []
    wrong = rules.says(search, z) != y
    for seed in range(ORDERS):
        counts = rules.nudges(z, y, RATES["perceptron"], PASSES, seed, count_from=PASSES - LAST + 1)
        busiest = np.argsort(-counts, kind="stable")[: len(counts) // 10]
        shares.append(counts[busiest].sum() / counts.sum())
        wrong_too.append(wrong[busiest].mean())
        never.append((counts == 0).mean())
    put("busiest_tenth_share_of_nudges", float(np.median(shares)))
    put("busiest_tenth_wrong_for_search", float(np.median(wrong_too)))
    put("wrong_for_search", wrong.mean())
    put("never_nudged", float(np.median(never)))
    # How close to zero the perceptron's weights stay, in steps, measured
    # just before each nudge of its last passes: near zero, a full-size
    # nudge swings the border a long way.
    sizes = rules.sizes_before_nudges(z, y, RATES["perceptron"], PASSES, 0, count_from=PASSES - LAST + 1)
    put_int("perceptron_steps_from_zero", float(np.median(sizes)))
    # A nudge's own length, in steps, is the length of its moment's numbers
    # with the bias's 1: so the weights are this many nudges from zero.
    counts = rules.nudges(z, y, RATES["perceptron"], PASSES, 0, count_from=PASSES - LAST + 1)
    lengths = np.repeat(np.linalg.norm(np.c_[z, np.ones(len(z))], axis=1), counts)
    numbers["perceptron_nudge_in_steps"] = round(float(np.median(lengths)), 2)
    put_int("perceptron_nudges_from_zero", float(np.median(sizes)) / float(np.median(lengths)))
    # For comparison: the settled square trick's weights, in the perceptron's steps.
    numbers["square_steps_from_zero"] = round(float(np.linalg.norm(np.r_[exact.w, exact.b]) / RATES["perceptron"]), 1)
    # The square trick listens to moments it already gets right, by too much.
    far = rules.right_by_too_much(exact, z, y)
    put("right_by_too_much", far.mean())
    numbers["right_by_too_much_median_depth"] = float(np.median(x[far, 0]))
    numbers["right_by_too_much_median_speed"] = float(np.median(x[far, 1]))
    numbers["median_depth"], numbers["median_speed"] = float(np.median(x[:, 0])), float(np.median(x[:, 1]))
    put("right_by_too_much_rising", (x[far, 1] < 0).mean())
    put("rising", (x[:, 1] < 0).mean())

    # The two maps of every border: mistakes (a staircase), squared error (a bowl).
    mistakes, squared = rules.landscape(z, y, ANGLES, OFFSETS)
    i, j = np.unravel_index(np.argmin(mistakes), mistakes.shape)
    k, l = np.unravel_index(np.argmin(squared), squared.shape)
    stairs_bottom = rules.Line(np.array([np.cos(ANGLES[i]), np.sin(ANGLES[i])]), float(OFFSETS[j]))
    bowl_bottom = rules.Line(np.array([np.cos(ANGLES[k]), np.sin(ANGLES[k])]), float(OFFSETS[l]))
    put("stairs_bottom", rules.accuracy(stairs_bottom, zt, yt))
    put("bowl_bottom", rules.accuracy(bowl_bottom, zt, yt))
    # Big drops: neighbouring borders (one offset step apart) whose
    # mistakes differ by fifty moments or more.
    drops = np.abs(np.diff(mistakes, axis=1)) * len(y)
    put("big_drops", (drops >= 50).mean())
    put_int("biggest_drop", drops.max())
    flat = np.isclose(squared, 1.0)
    put("bowl_flat", flat.mean())
    lows = local_lows(squared, flat)
    maps.parent.mkdir(parents=True, exist_ok=True)
    np.savez_compressed(maps, angles=ANGLES, offsets=OFFSETS, mistakes=mistakes, squared=squared,
                        mean=mean, scale=scale)

    check_claims(
        bowl_is_least_squares=rules.turn([bowl_bottom, exact]) < 1.0,
        control_settles=control_settles,
        one_hollow=lows == 1,
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
