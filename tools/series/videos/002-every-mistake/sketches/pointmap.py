"""002's map, by point: the best border, then the run shown's first nudges, then its last pass."""

import numpy as np
from manim import DOWN, FadeIn, Scene

from series.ember import backdrop
from series.lab.film002 import load_film
from series.pointmap import PointMap


class Sketch(Scene):
    def construct(self):
        f = load_film()
        self.add(backdrop())
        m = PointMap(f["spots"], f["spot_moved"], f["spot_still"], at=DOWN * 0.1, line=f["search"])
        self.play(FadeIn(m, run_time=1.0))
        self.wait(0.5)
        first = f["run_after"][:40]
        self.play(m.jump(first), run_time=4)
        last = f["run_after"][f["run_pass"] == 29]
        self.play(m.jump(last[::10]), run_time=4)
        both = (f["spot_moved"] > 0) & (f["spot_still"] > 0)
        m.ring[:] = both
        self.play(m.fade_points(~both, 0.15), run_time=1.5)
        self.wait(1)
