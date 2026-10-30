"""001's map: every moment in the air a dot (depth across, falling speed up); the border where the total
is zero, then the bias slid, then the weights leaned."""

import json

import numpy as np
from manim import DOWN, UP, Create, FadeIn, Scene

from series.bordermap import BorderMap
from series.ember import backdrop
from series.lab.data import load
from series.lab.exp001 import LAND_WITHIN, in_play, label, landing
from series.script import REPO, VIDEOS

VIDEO = VIDEOS / "001-one-neuron"
DATA = REPO / "target/series/001-one-neuron/data/smb1/1"
FLOOR = 176  # depth at the floor


def moments(n: int = 300):
    """Subsample of 001's moments in the air: (depth, speed) points and what each lands on soon."""
    r = load(DATA)
    air = label("smb1")(r.ram)
    keep, lands = landing(air, r.meta.get("restart_frames", []), LAND_WITHIN)
    play = in_play("smb1", r.ram)[keep]
    keep, lands = keep[play], lands[play]
    depth = r.ram[keep, 0x00CE].astype(float)
    vy = r.ram[keep, 0x009F].astype(float)
    pts = np.stack([depth, np.where(vy > 127, vy - 256, vy)], 1)
    down = [next((t + k for k in range(1, LAND_WITHIN + 1) if not air[t + k]), None) for t in keep]
    outcomes = [None if not soon else ("floor" if r.ram[t, 0x00CE] >= FLOOR else "blocks") for t, soon in zip(down, lands)]
    pick = np.random.default_rng(3).choice(len(pts), min(n, len(pts)), replace=False)
    return pts[pick], [outcomes[i] for i in pick]


class Sketch(Scene):
    def construct(self):
        self.add(backdrop())
        w0, w1, b = json.loads((VIDEO / "numbers.json").read_text())["smb1_land_weights"]
        points, outcomes = moments()
        m = BorderMap(points, outcomes, (w0, w1), b, "depth", "falling speed", "lands soon", "not yet", width=9.5, height=5.4, at=DOWN * 0.1)
        self.play(FadeIn(m, run_time=1.2))
        self.wait(0.5)
        self.play(m.slide_bias(b - 4), run_time=2)
        self.play(m.slide_bias(b), run_time=2)
        self.play(m.lean((w0 * 2.2, w1 * 0.6)), run_time=2.5)
        self.wait(1)
