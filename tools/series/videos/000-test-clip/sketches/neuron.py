"""The perceptron as a balance scale: each input's weighted value sits in
the left pan, the bias is a counterweight in the right, and the beam tips
to "yes" when the left outweighs the right (w1*x1 + w2*x2 + b > 0)."""

import numpy as np
from manim import (DOWN, UP, Create, FadeIn, Line, MathTex, Polygon, Scene, Square, Text, ValueTracker, VGroup,
                   always_redraw)

from series import style

HALF_BEAM = 4.2
PIVOT = np.array([0.0, 1.4, 0.0])
STRING = 1.8
PAN = 1.2  # half a pan's width
DROP = 2.5  # how far above its pan a block starts


class Sketch(Scene):
    def construct(self):
        self.camera.background_color = style.BACKGROUND
        total = ValueTracker(0.0)
        pans = {-1: [], 1: []}

        def angle():
            # Positive totals lower the left end.
            return np.clip(total.get_value(), -3, 3) * 0.08

        def end(side):
            a = angle()
            return PIVOT + side * HALF_BEAM * np.array([np.cos(a), np.sin(a), 0.0])

        def block(label, colour, mass, drop):
            box = Square(side_length=0.8 + 0.3 * mass, fill_color=colour, fill_opacity=0.95, stroke_width=0)
            tex = MathTex(label, color=style.BACKGROUND).scale(1.1).move_to(box)
            fade = 1 - drop.get_value()
            return VGroup(box, tex).set_opacity(fade)

        def scale():
            beam = Line(end(-1), end(1), color=style.TEXT, stroke_width=6)
            parts = [beam]
            for side in (-1, 1):
                top = end(side)
                pan_y = top + STRING * DOWN
                left, right = pan_y + PAN * np.array([-1, 0, 0]), pan_y + PAN * np.array([1, 0, 0])
                # Strings run to the pan's ends, leaving its middle clear for the stack.
                parts += [Line(top, left, color=style.TEXT, stroke_width=2),
                          Line(top, right, color=style.TEXT, stroke_width=2),
                          Line(left, right, color=style.TEXT, stroke_width=5)]
                y = pan_y[1] + 0.04
                for b in pans[side]:
                    blk = block(*b)
                    blk.move_to(np.array([pan_y[0], y + blk[0].height / 2, 0]) + b[3].get_value() * DROP * UP)
                    y += blk[0].height + 0.06
                    parts.append(blk)
                answer = "yes" if side < 0 else "no"
                parts.append(Text(answer, font=style.FONT, color=style.WARM[2] if side < 0 else style.TEXT)
                             .scale(0.6).next_to(pan_y, DOWN, buff=0.3))
            return VGroup(*parts)

        stand = Polygon(PIVOT, PIVOT + np.array([-0.6, -4.2, 0]), PIVOT + np.array([0.6, -4.2, 0]),
                        color=style.TEXT, fill_opacity=0.25, stroke_width=2)
        rig = always_redraw(scale)
        self.play(Create(stand), FadeIn(rig), run_time=1.0)
        self.wait(0.5)

        for side, label, colour, mass, delta in [
            (-1, "w_1 x_1", style.WARM[0], 1.5, 1.5),
            (-1, "w_2 x_2", style.WARM[0], 1.0, 1.0),
            (1, "b", style.WARM[1], 2.0, -2.0),
        ]:
            drop = ValueTracker(1.0)
            pans[side].append((label, colour, mass, drop))
            self.play(drop.animate.set_value(0.0), run_time=0.8)
            self.play(total.animate.increment_value(delta), run_time=1.2)
            self.wait(0.5)

        verdict = MathTex("w_1 x_1 + w_2 x_2 + b > 0", color=style.WARM[2]).scale(1.0).to_edge(UP)
        self.play(FadeIn(verdict), run_time=0.75)
        self.wait(1.0)
