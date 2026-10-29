from manim import DOWN, UP, FadeIn, FadeOut, MathTex, Text, Write

from series import style
from series.footage import GameFootage
from series.scene import NarratedScene
from punchout_film.film import PUNCHOUT


class VideoScene(NarratedScene):
    def construct(self):
        title = Text("Inside the Machine", font=style.FONT, color=style.TEXT)
        self.say("opening", FadeIn(title))
        self.play(FadeOut(title), run_time=0.5)

        formula = MathTex("w", r"\cdot", "x", "+", "b").scale(2)
        formula[0].set_color(style.WEIGHT)
        formula[2].set_color(style.INPUT)
        formula[4].set_color(style.BIAS)
        self.say("neuron", Write(formula))
        self.play(FadeOut(formula), run_time=0.5)

        game = GameFootage("p3-teacher-von-kaiser", "punchout", start_frame=600,
                           root=PUNCHOUT.takes_dir, hint=PUNCHOUT.record_hint).scale_to_fit_height(6)
        self.add(game)
        self.say("footage")
        self.remove(game)

        self.say("closing")
        self.roll_credits()
