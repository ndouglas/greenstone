"""Video 001's experiments: one neuron, its setting chosen by trying every
value on recordings 1-2 and tested on recording 3, answering "is the player
in the air?" from his height number; and a two-number neuron for "will he
land within 10 frames?".

Usage: cd tools && series/.venv/bin/python -m series.lab.exp001
"""

import json
import math
from pathlib import Path

import numpy as np

from series.lab.data import check_rate, dataset, load
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
    for r in recs:
        check_recording(game, r)
    x, y = dataset(recs, addresses, label(game))
    play = np.concatenate([in_play(game, r.ram) for r in recs])
    x, y = x[play], y[play]
    check_rate(y)
    for i, (_, _, signed) in enumerate(INPUTS[game]):
        if signed:
            x[:, i] = np.where(x[:, i] > 127, x[:, i] - 256, x[:, i])
    return x, y


def accuracy(w: np.ndarray, b: float, x: np.ndarray, y: np.ndarray) -> float:
    """How often the line's answer (yes when w·x + b > 0) matches the answer key."""
    return float(((x @ w + b > 0).astype(int) == y).mean())


def percent(a: float) -> tuple[str, str]:
    """93% and "ninety-three percent", for the screen and the narrator."""
    n = math.floor(a * 100 + 0.5)
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


def false_alarm_kind(air: np.ndarray, vy: np.ndarray, depth: np.ndarray, normal: np.ndarray,
                     restarts: list[int], t: int) -> str:
    """Why the border said "lands soon" at frame t and he didn't: he started
    rising again before landing ("bounce", off an enemy), fell below the
    screen ("pit") or died some other way ("died") before the next restart,
    or simply landed later ("late"). `normal` is false while the game, not
    the player, is moving Mario (dying pops him upward). Nothing after the
    next restart counts: that's another life."""
    restarts = set(restarts)
    for k in range(t + 1, len(air)):
        if k in restarts or not normal[k]:
            return "pit" if (depth[t:k] >= 200).any() else "died"
        if not air[k]:
            return "late"
        if vy[k] < 0:
            return "bounce"
    return "pit" if (depth[t:] >= 200).any() else "died"


# The worked examples' made-up moments: a depth for the first neuron, and a
# depth with a falling and a rising speed for the landing neuron.
EXAMPLE_AIR_DEPTH = 150
LAND_EXAMPLE_DEPTH, LAND_EXAMPLE_FALLING, LAND_EXAMPLE_RISING = 140, 4, -2


def worked_examples(cut: int, k: int, threshold: int) -> dict[str, int]:
    """The script's worked sums, from the numbers the experiments found.
    The script says each example's answer (in the air; lands soon; not yet)
    and writes the negative ones as "minus", so refuse numbers that would
    change an answer or a sign."""
    air = cut - EXAMPLE_AIR_DEPTH
    speed_yes = k * LAND_EXAMPLE_FALLING
    yes = LAND_EXAMPLE_DEPTH + speed_yes - threshold
    speed_no = k * LAND_EXAMPLE_RISING
    no = LAND_EXAMPLE_DEPTH + speed_no - threshold
    if not (air > 0 and yes > 0 and no < 0 and speed_no < 0):
        raise ValueError(f"the worked examples' answers no longer hold (air {air}, lands soon {yes}, not yet {no})")
    return {"example_air_total": air, "land_example_speed_yes": speed_yes, "land_example_yes": yes,
            "land_example_speed_no": -speed_no, "land_example_no": -no,
            "land_balance_yes_pan": LAND_EXAMPLE_DEPTH + speed_yes,
            "land_balance_no_pan": -speed_no + threshold,
            "example_air_depth": EXAMPLE_AIR_DEPTH, "land_example_depth": LAND_EXAMPLE_DEPTH,
            "land_example_falling": LAND_EXAMPLE_FALLING, "land_example_rising": -LAND_EXAMPLE_RISING}


# Each game's ROM, as the recorder names it (src/nes/gym/series/ram.rs).
ROM_SHA1 = {
    "smb1": "ea343f4e445a9050d4b4fbac2c77d0693b1d0922",
    "smb3": "a03e7e526e79df222e048ae22214bca2bc49c449",
    "castlevania": "7a20c44f302fb2f1b7adffa6b619e3e1cae7b546",
}


def check_recording(game: str, rec) -> None:
    """Refuse a recording that doesn't match this game's answer key as the
    recorder saw it: another ROM, another answer byte, a different labelling,
    or a truncated file."""
    meta = rec.meta
    problems = []
    if meta.get("sha1") != ROM_SHA1[game]:
        problems.append(f"ROM {meta.get('sha1')}")
    if meta.get("airborne_byte") != ANSWER_KEYS[game][0]:
        problems.append(f"answer byte {meta.get('airborne_byte')}")
    if not len(rec.ram) == len(rec.buttons) == meta.get("frames"):
        problems.append(f"{len(rec.ram)} RAM frames, {len(rec.buttons)} button frames, meta says {meta.get('frames')}")
    elif abs(label(game)(rec.ram).mean() - meta.get("label_rate", -1)) > 1e-9:
        problems.append(f"labels {label(game)(rec.ram).mean():.4f} here, {meta.get('label_rate')} when recorded")
    if problems:
        raise ValueError(f"{game} recording doesn't match: {'; '.join(problems)}")


def in_play(game: str, ram: np.ndarray) -> np.ndarray:
    """Frames where the player is playing. Super Mario Bros. says so in its
    player state ($000E): 08 is play. The rest are the game moving Mario:
    dying, growing or shrinking (a freeze), the flagpole, walking into the
    castle, pipes; there the answer key can mislead (dying sets the
    in-the-air byte while he stands on the floor). The other games' frames
    all count."""
    if game == "smb1":
        return ram[:, 0x000E] == 0x08
    return np.ones(len(ram), dtype=bool)


def seconds_between_deaths(frames: list[int], restarts: list[int]) -> int:
    """How often the player died, from the recordings' lengths (60 frames a second)."""
    return int(round(np.mean(frames) / 60 / max(round(np.mean(restarts)), 1)))


def recording_minutes(frames: list[int]) -> int:
    return int(round(np.mean(frames) / 3600))


def whole_number_border(w: np.ndarray, b: float) -> tuple[int, int]:
    """The border as depth + k × speed > threshold: everything divided by the
    depth's weight, which must be positive or the comparison would flip."""
    if w[0] <= 0:
        raise ValueError(f"the depth's weight is {w[0]}; dividing by it would flip the border")
    return int(round(w[1] / w[0])), int(round(-b / w[0]))


def check_claims(cut: int, standing: int, smb3_cut: int, smb3_standing: int, train: float, test: float,
                 depth_alone: float, not_yet: float, best_line: float,
                 raised_depth: float, floor_depth: float, falling_depth: float, falling_not_yet: float) -> None:
    """The script's plain-word claims about the numbers: refuse a run that
    makes any of them false."""
    problems = []
    if cut != standing:
        problems.append(f"the setting ({cut}) isn't Mario's standing depth ({standing})")
    if smb3_cut != smb3_standing:
        problems.append(f"Super Mario Bros. 3's setting ({smb3_cut}) isn't its floor ({smb3_standing})")
    if test < train:
        problems.append(f"the new session scored worse ({test:.3f}) than the ones it was chosen on ({train:.3f})")
    if best_line > test + 0.01:
        problems.append(f"adding falling speed helps a straight line ({best_line:.3f} against {test:.3f})")
    if raised_depth >= floor_depth:
        problems.append(f"landings on raised things ({raised_depth}) don't sit left of floor landings ({floor_depth})")
    if falling_depth < falling_not_yet + 0.1:
        problems.append(f"among falling moments depth alone ({falling_depth:.3f}) barely beats always 'not yet' "
                        f"({falling_not_yet:.3f}), so rising and falling aren't what defeats it")
    if abs(depth_alone - not_yet) >= 0.01:
        problems.append(f"depth alone ({depth_alone:.3f}) isn't about as good as always 'not yet' ({not_yet:.3f})")
    if problems:
        raise ValueError("; ".join(problems))


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
    raw: dict[str, dict[str, float]] = {}

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
        raw[game] = {"test": accuracy(np.array([-1.0]), cut, xt[:, :1], yt),
                     "train": accuracy(np.array([-1.0]), cut, x[:, :1], y)}
        put(f"{game}_height_accuracy", raw[game]["test"])
        # On the sessions it was chosen on: a lucky fit would score worse on the new one.
        put(f"{game}_height_train_accuracy", raw[game]["train"])
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
            raw[game]["best_line"] = accuracy(w, b, xt, yt)
            put(f"{game}_best_line_accuracy", raw[game]["best_line"])
        numbers[f"{game}_test_frames"] = int(len(yt))
        numbers[f"{game}_restarts_per_recording"] = int(round(np.mean([r.meta["restarts"] for r in train + test])))
        put_int(f"{game}_seconds_between_deaths", seconds_between_deaths(
            [r.meta["frames"] for r in train + test], [r.meta["restarts"] for r in train + test]))
        # Where the one-cut neuron goes wrong, on the test frames.
        wrong = (xt[:, 0] < cut).astype(int) != yt
        on_something = wrong & (yt == 0)
        put(f"{game}_mistakes_on_ground_above_floor", on_something.sum() / max(wrong.sum(), 1))
        ground = x[y == 0, 0]
        put_int(f"{game}_standing_height", int(np.median(ground)))
        put_int(f"{game}_standing_height_low", int(np.percentile(ground, 1)))
        put_int(f"{game}_standing_height_high", int(np.percentile(ground, 99)))
    numbers["test_minutes"] = recording_minutes([r.meta["frames"] for r in test])

    # Two numbers that both matter: Mario is in the air; will he land in
    # the next LAND_WITHIN frames? (Low and falling fast means soon.)
    def land_moments(r):
        """The moments in the air, during play, and whether each lands soon."""
        air = label("smb1")(r.ram)
        keep, lands = landing(air, r.meta.get("restart_frames", []), LAND_WITHIN)
        play = in_play("smb1", r.ram)[keep]
        return air, keep[play], lands[play]

    def land_data(recs):
        xs, ys = [], []
        for r in recs:
            check_recording("smb1", r)
            air, keep, lands = land_moments(r)
            h = r.ram[keep, 0x00CE].astype(float)
            vy = r.ram[keep, 0x009F].astype(float)
            xs.append(np.stack([h, np.where(vy > 127, vy - 256, vy)], 1))
            ys.append(lands)
        check_rate(np.concatenate(ys))
        return np.concatenate(xs), np.concatenate(ys)

    train = [load(data_root / "smb1" / s) for s in ("1", "2")]
    test = [load(data_root / "smb1" / "3")]
    x, y = land_data(train)
    xt, yt = land_data(test)
    put("smb1_land_rate", yt.mean())
    raw["land_baseline"] = max(yt.mean(), 1 - yt.mean())
    put("smb1_land_baseline", raw["land_baseline"])
    for i, name in ((0, "height"), (1, "speed")):
        best = max(((side * (x[:, i] - t) > 0).astype(int) == y).mean().item() for t in np.unique(x[:, i]) for side in (1, -1))
        t, side = next((t, side) for t in np.unique(x[:, i]) for side in (1, -1)
                       if ((side * (x[:, i] - t) > 0).astype(int) == y).mean() == best)
        raw[f"land_{name}"] = ((side * (xt[:, i] - t) > 0).astype(int) == yt).mean()
        put(f"smb1_land_{name}_cut_accuracy", raw[f"land_{name}"])
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
        air, keep, lands = land_moments(r)
        for t, l in zip(keep, lands):
            if l:
                first = next(k for k in range(1, LAND_WITHIN + 1) if not air[t + k])
                land_heights.append(int(r.ram[t + first, 0x00CE]))
            else:
                land_heights.append(-1)
    land_heights = np.array(land_heights)
    raised = missed & (land_heights >= 0) & (land_heights < numbers["smb1_height_cut"])
    # Where the landings sit on the map: moments before a landing on something
    # raised against before one on the floor, by depth.
    raw["raised_depth"] = float(np.median(xt[(land_heights >= 0) & (land_heights < numbers["smb1_height_cut"]), 0]))
    raw["floor_depth"] = float(np.median(xt[land_heights >= numbers["smb1_height_cut"], 0]))
    # Depth alone, among falling moments only: if it does well there, what
    # defeats it overall is not knowing rising from falling.
    fall, fall_t = x[:, 1] > 0, xt[:, 1] > 0
    scores = [(((side * (x[fall, 0] - t) > 0).astype(int) == y[fall]).mean(), t, side)
              for t in np.unique(x[fall, 0]) for side in (1, -1)]
    _, t, side = max(scores, key=lambda s: s[0])
    raw["falling_depth"] = ((side * (xt[fall_t, 0] - t) > 0).astype(int) == yt[fall_t]).mean()
    raw["falling_not_yet"] = max(yt[fall_t].mean(), 1 - yt[fall_t].mean())
    put("smb1_land_misses_on_raised", raised.sum() / max(missed.sum(), 1))
    put("smb1_land_raised_share_of_all_mistakes", raised.sum() / max((said != yt).sum(), 1))
    # The same border in whole numbers: height + k × speed against a threshold.
    k, threshold = whole_number_border(w, b)
    put_int("smb1_land_speed_weight", k)
    put_int("smb1_land_threshold", threshold)
    for name, value in worked_examples(numbers["smb1_height_cut"], k, threshold).items():
        put_int(f"smb1_{name}", value)
    put("smb1_land_whole_accuracy", accuracy(np.array([1.0, float(k)]), -float(threshold), xt, yt))
    # The same breakdown for the whole-number border the viewer is shown.
    said_whole = (xt @ np.array([1.0, float(k)]) - threshold > 0).astype(int)
    missed_whole = (said_whole == 0) & (yt == 1)
    raised_whole = missed_whole & (land_heights >= 0) & (land_heights < numbers["smb1_height_cut"])
    put("smb1_land_whole_misses_on_raised", raised_whole.sum() / max(missed_whole.sum(), 1))
    # And its false alarms: "lands soon" when he didn't, by why.
    kinds = []
    for r in test:
        air, keep, _ = land_moments(r)
        restarts = r.meta.get("restart_frames", [])
        vy = r.ram[:, 0x009F].astype(int)
        vy = np.where(vy > 127, vy - 256, vy)
        # Whether he lands: the freeze while the game shrinks a hurt Mario
        # (0x0A) doesn't end his life, so it doesn't end the look-ahead.
        normal = np.isin(r.ram[:, 0x000E], (0x08, 0x0A))
        kinds += [false_alarm_kind(air, vy, r.ram[:, 0x00CE].astype(int), normal, restarts, t) for t in keep]
    kinds = np.array(kinds)
    alarms = (said_whole == 1) & (yt == 0)
    put("smb1_land_false_alarms_bounce_or_pit", np.isin(kinds[alarms], ("bounce", "pit")).mean())
    numbers["smb1_land_false_alarm_kinds"] = {k: int((kinds[alarms] == k).sum()) for k in ("bounce", "pit", "died", "late")}
    put("smb1_land_whole_raised_share_of_all_mistakes", (missed_whole & (land_heights >= 0) & (land_heights < numbers["smb1_height_cut"])).sum() / max((said_whole != yt).sum(), 1))

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
    check_claims(cut=numbers["smb1_height_cut"], standing=numbers["smb1_standing_height"],
                 smb3_cut=numbers["smb3_height_cut"], smb3_standing=numbers["smb3_standing_height"],
                 train=raw["smb1"]["train"], test=raw["smb1"]["test"],
                 depth_alone=raw["land_height"], not_yet=raw["land_baseline"],
                 best_line=raw["smb1"]["best_line"],
                 raised_depth=raw["raised_depth"], floor_depth=raw["floor_depth"],
                 falling_depth=raw["falling_depth"], falling_not_yet=raw["falling_not_yet"])
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(numbers, indent=2, sort_keys=True) + "\n")
    return numbers


if __name__ == "__main__":
    for k, v in run().items():
        if not k.endswith("_spoken"):
            print(f"{k}: {v}")
