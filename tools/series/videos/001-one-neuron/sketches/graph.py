"""001's graph: the total against depth, weight −1 and bias 176, then "× 3" and the pivot about the setting."""

from manim import DOWN, RIGHT, UP, FadeIn, FadeOut, Scene

from series import style
from series.ember import backdrop, numeral, serif
from series.graph import NeuronGraph


class Sketch(Scene):
    def construct(self):
        self.add(backdrop())
        graph = NeuronGraph(-1, 176, (0, 300), (-150, 250), "depth", "total", "in the air", "on the ground",
                            width=9.0, height=5.2)
        self.play(FadeIn(graph), run_time=1)
        self.wait(1)
        times = numeral("× 3", style.TEXT, 0.7).to_corner(UP + RIGHT).shift(DOWN * 0.3)
        self.play(FadeIn(times), run_time=0.4)
        self.play(graph.scale_weights(3), run_time=2.5)
        self.wait(1.5)
