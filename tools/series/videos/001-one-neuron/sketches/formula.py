"""The formula, neutral until filled: w·x + b > 100, then 001's numbers, then the threshold carried into the
bias."""

from manim import Scene

from series.ember import backdrop
from series.formula import Formula


class Sketch(Scene):
    def construct(self):
        self.add(backdrop())
        formula = Formula(["w·x", "+ b"], "> 100").scale(2.2)
        self.add(formula)
        self.wait(1)
        self.play(formula.fill("w", -1), run_time=1)
        self.play(formula.fill("b", 176), run_time=1)
        self.wait(0.5)
        self.play(formula.move_into_bias(100), run_time=1.0)
        self.wait(1.5)
