"""Video 001's experiments: one neuron, its setting chosen by trying every
value on recordings 1-2 and tested on recording 3, answering "is the player
in the air?" from his height number; and a two-number neuron for "will he
land within 10 frames?".

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
# which of its values mean "yes" (None: any non-zero value). These match
# src/nes/gym/series/ram.rs; see docs/series/games/001.md.
ANSWER_KEYS = {
    "smb1": (0x001D, (1, 2)),
    "smb3": (0x00D8, None),
    # Jumping, knocked back (he flies backwards through the air), dropping.
    "castlevania": (0x046C, (0x01, 0x05, 0x07)),
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
    if yes is None:
        return lambda ram: ram[:, address] != 0
    return lambda ram: np.isin(ram[:, address], yes)


def landing(air: np.ndarray, restarts: list[int], within: int) -> tuple[np.ndarray, np.ndarray]:
    """Frames where the player is in the air, and whether he's on the ground
    at some frame in the next `within`. Frames whose look-ahead crosses a
    restart are left out: a fresh start isn't a landing."""
    n = len(air)
    crossed = np.zeros(n, dtype=bool)
    for r in restarts:
        crossed[max(0, r - within + 1) : r + 1] = True
    keep = np.array([t for t in range(n - within) if air[t] and not crossed[t]], dtype=int)
    lands = np.array([int((~air[t + 1 : t + 1 + within]).any()) for t in keep], dtype=int)
    return keep, lands


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
    if n < 1_000_000:
        rest = n % 1000
        joiner = " and " if 0 < rest < 100 else " "
        return spoken_number(n // 1000) + " thousand" + ("" if rest == 0 else joiner + spoken_number(rest))
    raise ValueError(n)


LAND_WITHIN = 10


def best_cut(v: np.ndarray, y: np.ndarray) -> tuple[float, float]:
    """The height below which "in the air" is most often right: every value tried."""
    best = (0.0, 0.0)
    for t in np.unique(v):
        a = float(((v < t).astype(int) == y).mean())
        if a > best[0]:
            best = (a, float(t))
    return best[1], best[0]


def agreed_cut(cuts: list[float]) -> int | None:
    """The setting each session chose on its own, if they all chose the same one."""
    return int(cuts[0]) if len(set(cuts)) == 1 else None


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
        # A fluke of the first two sessions would show up as a session that,
        # on its own, chooses a different setting. Missing if any does.
        each = []
        for r in train + test:
            xs, ys = features(game, [r])
            each.append(best_cut(xs[:, 0], ys)[0])
        if (agreed := agreed_cut(each)) is not None:
            put_int(f"{game}_every_session_cut", agreed)
        # A steadier player, playing by fixed rules: does it pick the same setting?
        steady = data_root / game / "rule"
        if steady.exists():
            xs, ys = features(game, [load(steady)])
            if (agreed := agreed_cut([cut, best_cut(xs[:, 0], ys)[0]])) is not None:
                put_int(f"{game}_steady_player_cut", agreed)
        put(f"{game}_height_accuracy", accuracy(np.array([-1.0]), cut, xt[:, :1], yt))
        # On the sessions it was chosen on: a lucky fit would score worse on the new one.
        put(f"{game}_height_train_accuracy", accuracy(np.array([-1.0]), cut, x[:, :1], y))
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
        numbers[f"{game}_restarts_per_recording"] = int(round(np.mean([r.meta["restarts"] for r in train + test])))
        put_int(f"{game}_seconds_between_deaths", int(round(18000 / 60 / max(numbers[f"{game}_restarts_per_recording"], 1))))
        # Where the one-cut neuron goes wrong, on the test frames.
        wrong = (xt[:, 0] < cut).astype(int) != yt
        on_something = wrong & (yt == 0)
        put(f"{game}_mistakes_on_ground_above_floor", on_something.sum() / max(wrong.sum(), 1))
        ground = x[y == 0, 0]
        put_int(f"{game}_standing_height", int(np.median(ground)))
        put_int(f"{game}_standing_height_low", int(np.percentile(ground, 1)))
        put_int(f"{game}_standing_height_high", int(np.percentile(ground, 99)))
    numbers["test_minutes"] = 5

    # Two numbers that both matter: Mario is in the air; will he land in
    # the next LAND_WITHIN frames? (Low and falling fast means soon.)
    def land_data(recs):
        xs, ys = [], []
        for r in recs:
            air = label("smb1")(r.ram)
            keep, lands = landing(air, r.meta.get("restart_frames", []), LAND_WITHIN)
            h = r.ram[keep, 0x00CE].astype(float)
            vy = r.ram[keep, 0x009F].astype(float)
            xs.append(np.stack([h, np.where(vy > 127, vy - 256, vy)], 1))
            ys.append(lands)
        return np.concatenate(xs), np.concatenate(ys)

    train = [load(data_root / "smb1" / s) for s in ("1", "2")]
    test = [load(data_root / "smb1" / "3")]
    x, y = land_data(train)
    xt, yt = land_data(test)
    put("smb1_land_rate", yt.mean())
    put("smb1_land_baseline", max(yt.mean(), 1 - yt.mean()))
    for i, name in ((0, "height"), (1, "speed")):
        best = max(((side * (x[:, i] - t) > 0).astype(int) == y).mean().item() for t in np.unique(x[:, i]) for side in (1, -1))
        t, side = next((t, side) for t in np.unique(x[:, i]) for side in (1, -1)
                       if ((side * (x[:, i] - t) > 0).astype(int) == y).mean() == best)
        put(f"smb1_land_{name}_cut_accuracy", ((side * (xt[:, i] - t) > 0).astype(int) == yt).mean())
    w, b, _ = best_line(x, y)
    put("smb1_land_line_accuracy", accuracy(w, b, xt, yt))
    numbers["smb1_land_within"] = LAND_WITHIN
    numbers["smb1_land_within_spoken"] = spoken_number(LAND_WITHIN)
    # The line, as "lands soon when speed > slope × height + intercept".
    numbers["smb1_land_weights"] = [float(w[0]), float(w[1]), float(b)]
    # Where the landing border goes wrong: missed landings onto something
    # raised (heights above the floor), versus onto the floor.
    said = (xt @ w + b > 0).astype(int)
    missed = (said == 0) & (yt == 1)
    land_heights = []
    for r in test:
        air = label("smb1")(r.ram)
        keep, lands = landing(air, r.meta.get("restart_frames", []), LAND_WITHIN)
        for t, l in zip(keep, lands):
            if l:
                first = next(k for k in range(1, LAND_WITHIN + 1) if not air[t + k])
                land_heights.append(int(r.ram[t + first, 0x00CE]))
            else:
                land_heights.append(-1)
    land_heights = np.array(land_heights)
    raised = missed & (land_heights >= 0) & (land_heights < numbers["smb1_height_cut"])
    put("smb1_land_misses_on_raised", raised.sum() / max(missed.sum(), 1))
    put("smb1_land_raised_share_of_all_mistakes", raised.sum() / max((said != yt).sum(), 1))
    # The same border in whole numbers: height + k × speed against a threshold.
    k = int(round(w[1] / w[0]))
    threshold = int(round(-b / w[0]))
    put_int("smb1_land_speed_weight", k)
    put_int("smb1_land_threshold", threshold)
    put("smb1_land_whole_accuracy", accuracy(np.array([1.0, float(k)]), -float(threshold), xt, yt))
    # The same breakdown for the whole-number border the viewer is shown.
    said_whole = (xt @ np.array([1.0, float(k)]) - threshold > 0).astype(int)
    missed_whole = (said_whole == 0) & (yt == 1)
    raised_whole = missed_whole & (land_heights >= 0) & (land_heights < numbers["smb1_height_cut"])
    put("smb1_land_whole_misses_on_raised", raised_whole.sum() / max(missed_whole.sum(), 1))

    # Castlevania keeps where Simon is in his jump, not a speed.
    cv = [load(data_root / "castlevania" / s) for s in ("1", "2", "3")]
    ram = np.concatenate([r.ram for r in cv])
    jumping = ram[:, 0x046C] == 0x01
    put_int("castlevania_speed_slot_nonzero_frames", int((ram[:, 0x03C4] != 0).sum()))
    put_int("castlevania_jump_steps", int(ram[jumping, 0x0488].max()))

    # Mario's score in these recordings (six digit bytes, millions to tens).
    s1 = np.concatenate([r.ram for r in train + test])
    digits = (s1[:, 0x07DD:0x07E3] & 0x0F).astype(int)
    put_int("smb1_top_score", int((digits * np.array([100000, 10000, 1000, 100, 10, 1])).sum(1).max() * 10))
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(numbers, indent=2, sort_keys=True) + "\n")
    return numbers


if __name__ == "__main__":
    for k, v in run().items():
        if not k.endswith("_spoken"):
            print(f"{k}: {v}")
