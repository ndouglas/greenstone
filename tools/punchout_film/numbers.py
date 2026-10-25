"""Write numbers.json, the film's numbers, from the Punch-Out!! training
results under target/punchout/.

    cd tools && python3 -m punchout_film.numbers
"""

import json

from reservoir_film.model import REPO

from .film import PUNCHOUT

ROOT = REPO / "target" / "punchout"
# The game's opponent names in the teacher's records, and the film's.
OPPONENTS = {"GlassJoe": "glass-joe", "VonKaiser": "von-kaiser", "PistonHonda": "piston-honda",
             "DonFlamenco": "don-flamenco", "BaldBull": "bald-bull"}


def key(kind):
    """A brain's name as a numbers key: caption references split on dots, so echo0.5 is echo0_5."""
    return kind.replace(".", "_")


def fight(entry):
    return {"hpm": entry["hits_per_minute"], "dodge": entry.get("dodge_rate"), "dealt": entry["damage_dealt"],
            "knockdowns": entry["knockdowns_dealt"], "won": entry["won"], "hits": entry["hits"]}


def rounds(by_round):
    """Round 0 as r0 and the last round as r4, each opponent's fight."""
    last = max(by_round, key=lambda k: int(k.split()[1]))
    return {name: {o: fight(e) for o, e in by_round[key].items() if "error" not in e}
            for name, key in (("r0", "round 0"), ("r4", last))}


def teacher(phase):
    """The pure teacher's fights (index 0), and the share of its labels that are Idle."""
    fights, idle, labels = {}, 0, 0
    for line in (ROOT / f"phase{phase}" / "teacher.jsonl").open():
        r = json.loads(line)
        idle += sum(m == "Idle" for _, m in r["labels"])
        labels += len(r["labels"])
        if r["index"] == 0:
            s = r["score"]
            fights[OPPONENTS[r["opponent"]]] = {"hits": s["hits"], "won": s["won"], "steps": s["steps"],
                                               "knockdowns": s["knockdowns_dealt"], "dealt": s["damage_dealt"]}
    return fights, idle / labels


def collect():
    out = {"teacher": {}, "teacher_idle_share": {}}
    for phase in (1, 2, 3):
        results = json.loads((ROOT / f"phase{phase}" / "results.json").read_text())
        if phase == 1:
            out["idle"] = {o: {"hpm": e["hits_per_minute"]} for o, e in results["idle"].items()}
        out[f"phase{phase}"] = {key(kind): rounds(r) for kind, r in results["kinds"].items()
                                if "error" not in r}
        out["teacher"][f"phase{phase}"], out["teacher_idle_share"][f"phase{phase}"] = teacher(phase)
    sweep = json.loads((ROOT / "phase1" / "sweep.json").read_text())
    out["sweep"] = {key(kind): rounds(r)["r4"] for kind, r in sweep["kinds"].items()}
    return out


if __name__ == "__main__":
    PUNCHOUT.numbers.write_text(json.dumps(collect(), indent=1, sort_keys=True) + "\n")
    print(f"wrote {PUNCHOUT.numbers}")
