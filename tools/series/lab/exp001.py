"""Video 001's experiments: one neuron, placed by hand, answering "is the
player in the air?" from two numbers in each game's memory.

Usage: cd tools && series/.venv/bin/python -m series.lab.exp001
"""

import json
from pathlib import Path

import numpy as np

from series.lab.data import dataset, load
from series.lab.perceptron import Perceptron
from series.script import REPO, VIDEOS

DATA = REPO / "target" / "series" / "001-one-neuron" / "data"
OUT = VIDEOS / "001-one-neuron" / "numbers.json"

# The answer key: the byte that says whether the player is in the air, and
# which of its values mean "yes" (see docs/series/games/001.md).
ANSWER_KEYS = {
    "smb1": (0x001D, (1, 2)),
    "smb3": (0x00D8, (1,)),
    "castlevania": (0x046C, (0x01, 0x07)),
}

# What the neuron reads: (name, address, signed). Never the answer key.
INPUTS = {
    "smb1": [("height", 0x00CE, False), ("vertical speed", 0x009F, True)],
    "smb3": [("height", 0x00A2, False), ("vertical speed", 0x00CF, True)],
    "castlevania": [("height", 0x003F, False)],
}


def check_inputs(game: str, inputs: list[int]) -> None:
    """Refuse to let the neuron read the answer it's being asked for."""
    if ANSWER_KEYS[game][0] in inputs:
        raise ValueError(f"{game}: the answer key ${ANSWER_KEYS[game][0]:04X} can't be one of the inputs")


def label(game: str):
    address, yes = ANSWER_KEYS[game]
    return lambda ram: np.isin(ram[:, address], yes)


def features(game: str, recs) -> tuple[np.ndarray, np.ndarray]:
    addresses = [a for _, a, _ in INPUTS[game]]
    check_inputs(game, addresses)
    x, y = dataset(recs, addresses, label(game))
    for i, (_, _, signed) in enumerate(INPUTS[game]):
        if signed:
            x[:, i] = np.where(x[:, i] > 127, x[:, i] - 256, x[:, i])
    return x, y


def accuracy(w: np.ndarray, b: float, x: np.ndarray, y: np.ndarray) -> float:
    """How often the line's answer (yes when w·x + b > 0) matches the answer key."""
    return float(((x @ w + b > 0).astype(int) == y).mean())


def percent(a: float) -> tuple[str, str]:
    """93% and "ninety-three percent", for the screen and the narrator."""
    n = round(a * 100)
    return f"{n}%", f"{spoken_number(n)} percent"


def spoken_number(n: int) -> str:
    ones = "zero one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen".split()
    tens = "_ _ twenty thirty forty fifty sixty seventy eighty ninety".split()
    if n < 20:
        return ones[n]
    if n < 100:
        return tens[n // 10] + ("" if n % 10 == 0 else "-" + ones[n % 10])
    if n < 1000:
        rest = n % 100
        return ones[n // 100] + " hundred" + ("" if rest == 0 else " and " + spoken_number(rest))
    raise ValueError(n)


def best_cut(v: np.ndarray, y: np.ndarray) -> tuple[float, float]:
    """The height below which "in the air" is most often right: every value tried."""
    best = (0.0, 0.0)
    for t in np.unique(v):
        a = float(((v < t).astype(int) == y).mean())
        if a > best[0]:
            best = (a, float(t))
    return best[1], best[0]


def best_line(x: np.ndarray, y: np.ndarray, angles: int = 360) -> tuple[np.ndarray, float, float]:
    """The best straight line through two inputs: every direction tried, and
    for each, every cut along it. Returns w, b (in the inputs' own units)
    and its accuracy."""
    mean, scale = x.mean(0), x.std(0) + 1e-9
    z = (x - mean) / scale
    best = (0.0, np.zeros(2), 0.0)
    for theta in np.linspace(0, 2 * np.pi, angles, endpoint=False):
        d = np.array([np.cos(theta), np.sin(theta)])
        s = z @ d
        for cut in np.quantile(s, np.linspace(0.005, 0.995, 199)):
            a = float(((s > cut).astype(int) == y).mean())
            if a > best[0]:
                best = (a, d, float(cut))
    a, d, cut = best
    w = d / scale
    return w, float(-cut - w @ mean), a


def run(data_root: Path = DATA, out: Path = OUT) -> dict:
    numbers: dict[str, str | int | float] = {}

    def put(name: str, value: float) -> None:
        numbers[name], numbers[name + "_spoken"] = percent(value)

    def put_int(name: str, value: int) -> None:
        numbers[name], numbers[name + "_spoken"] = int(value), spoken_number(int(value))

    for game in ANSWER_KEYS:
        train = [load(data_root / game / s) for s in ("1", "2")]
        test = [load(data_root / game / "3")]
        x, y = features(game, train)
        xt, yt = features(game, test)
        # Always answering the commoner answer: the score to beat.
        put(f"{game}_baseline", max(yt.mean(), 1 - yt.mean()))
        cut, _ = best_cut(x[:, 0], y)
        put_int(f"{game}_height_cut", int(cut))
        put(f"{game}_height_accuracy", accuracy(np.array([-1.0]), cut, xt[:, :1], yt))
        put(f"{game}_air_rate", yt.mean())
        if x.shape[1] == 2:
            # The second number, and whether a tilted line does better.
            not_zero = ((xt[:, 1] != 0).astype(int) == yt).mean()
            put(f"{game}_speed_not_zero_accuracy", not_zero)
            p = Perceptron(2)
            scale = x.std(0) + 1e-9
            p.fit((x - x.mean(0)) / scale, y, epochs=20, rate=0.01, seed=0)
            put(f"{game}_learned_line_accuracy", float((p.predict((xt - x.mean(0)) / scale) == yt).mean()))
            numbers[f"{game}_learned_speed_share"] = round(abs(p.w[1]) / (abs(p.w[0]) + abs(p.w[1])), 3)
            w, b, _ = best_line(x, y)
            put(f"{game}_best_line_accuracy", accuracy(w, b, xt, yt))
        numbers[f"{game}_test_frames"] = int(len(yt))
        ground = x[y == 0, 0]
        put_int(f"{game}_standing_height", int(np.median(ground)))
        put_int(f"{game}_standing_height_low", int(np.percentile(ground, 1)))
        put_int(f"{game}_standing_height_high", int(np.percentile(ground, 99)))
    numbers["test_minutes"] = 5
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(numbers, indent=2, sort_keys=True) + "\n")
    return numbers


if __name__ == "__main__":
    for k, v in run().items():
        if not k.endswith("_spoken"):
            print(f"{k}: {v}")
