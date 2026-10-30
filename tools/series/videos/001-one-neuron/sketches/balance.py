"""001's "Weighing it": the land example weighed. The depth's 140 falls into the yes pan, then the speed's 10
and the bias's 152 into the no pan; it tips to no, and the total, −22, is shown beneath."""

import json
from pathlib import Path

from manim import FadeIn, Scene

from series import style
from series.balance import Balance
from series.ember import backdrop

N = json.loads((Path(__file__).parents[1] / "numbers.json").read_text())


class Sketch(Scene):
    def construct(self):
        self.add(backdrop())
        scale = Balance("yes", "no", at=[0, 0.1, 0])
        self.play(FadeIn(scale), run_time=1)
        self.wait(0.5)
        self.play(scale.drop(N["smb1_land_example_depth"], str(N["smb1_land_example_depth"]), 0), run_time=1.4)
        self.wait(1)
        self.play(scale.drop(-N["smb1_land_example_speed_no"], str(N["smb1_land_example_speed_no"]), 0), run_time=1.2)
        self.play(scale.drop(-N["smb1_land_threshold"], str(N["smb1_land_threshold"]), 3), run_time=1.4)
        self.wait(1)
        self.play(scale.announce(), run_time=1)
        self.wait(2)
