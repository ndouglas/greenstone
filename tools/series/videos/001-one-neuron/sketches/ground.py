"""The Ember ground: the backdrop, a warm and a cool glow easing between values, and serif
numbers counting through zero with a true minus."""

import numpy as np
from manim import LEFT, RIGHT, Circle, Scene, ValueTracker, smooth

from series import style
from series.ember import backdrop, fade_image, glow, live, serif

NEURON = RIGHT * 3


class Sketch(Scene):
    def construct(self):
        self.add(backdrop(NEURON))
        value = ValueTracker(-60.0)
        warm = glow(style.WARM[1], 1.6).move_to(NEURON)
        cool = glow(style.COOL[0], 1.6).move_to(NEURON)
        warm_level, cool_level = ValueTracker(0.0), ValueTracker(0.0)
        fade_image(warm, warm_level)
        fade_image(cool, cool_level)

        def lights(_):
            v = value.get_value()
            warm_level.set_value(style.light(v) if v > 0 else 0.0)
            cool_level.set_value(style.light(v) if v < 0 else 0.0)

        ring = Circle(radius=0.55, stroke_width=1.4, color=style.ASH).move_to(NEURON)
        shown = ValueTracker(1.0)

        def place(m):
            v = round(value.get_value())
            m.become(serif(style.number(v), style.temperature(v), 0.6, italic=False)).move_to(NEURON)

        total = live(serif("0", style.TEXT, 0.6, italic=False), place, shown)
        caption = serif("a total", style.ASH, 0.42).move_to(LEFT * 3)
        self.add(warm, cool, ring, total, caption)
        warm.add_updater(lights)
        for target in (60.0, -30.0, 0.0):
            self.play(value.animate.set_value(target), run_time=2.5, rate_func=smooth)
            self.wait(0.5)
