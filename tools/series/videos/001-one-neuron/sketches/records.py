"""001's records: three tapes, the third sliding apart with its dying stretches greyed; then the score bars,
the baseline always on the ground, the neuron's result, and the score still to beat."""

import json
from pathlib import Path

from manim import FadeIn, FadeOut, Scene

from series.ember import backdrop
from series.records import ScoreBars, Tapes

N = json.loads((Path(__file__).parents[1] / "numbers.json").read_text())


def percent(text: str) -> float:
    return float(text.rstrip("%"))


class Sketch(Scene):
    def construct(self):
        self.add(backdrop())
        tapes = Tapes(3, ["1", "2", "3"])
        self.play(FadeIn(tapes), run_time=1)
        self.wait(1)
        self.play(tapes.apart(2), run_time=1.2)
        self.play(tapes.grey(2, [(0.18, 0.26), (0.55, 0.61), (0.82, 0.9)]), tapes.grey(0, [(0.4, 0.46)]), run_time=1)
        self.wait(2)
        self.play(FadeOut(tapes), run_time=0.6)

        base, result = percent(N["smb1_baseline"]), percent(N["smb1_height_accuracy"])
        bars = ScoreBars(base)
        self.play(FadeIn(bars), run_time=0.6)
        self.wait(1)
        self.play(bars.bar(result, "the neuron"), run_time=1.4)
        self.wait(2)
