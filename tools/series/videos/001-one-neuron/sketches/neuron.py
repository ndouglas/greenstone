"""The neuron, drawn from what it measures (the spike's thirteenth pass, rebuilt from the pieces): depth's
ruler under Mario's feet and the setting at the floor run off the screen into one neuron, the gap between
them shaded; it answers "in the air" until the run ends in the pit."""

import numpy as np
from manim import RIGHT, UP, Create, FadeIn, ValueTracker, config

from series.ember import backdrop
from series.exact import ExactScene
from series.neuron import Input, Neuron
from series.overlays import gap_band, feet_line, setting_line
from series.recording import Recording
from series.screen import Screen

START, END = 1247, 1943  # seed 1's first run, ending in a pit
SETTING = 176  # 001's cut: the floor
CENTRE = RIGHT * 2.3 + UP * 0.2


def drawn_along(line, drawn: ValueTracker) -> None:
    """Draw a self-placing line from its left end as `drawn` rises, and fade its label in with it, after
    its own updater has placed it this frame (so it stays on the moving feet)."""
    def draw(m) -> None:
        d = drawn.get_value()
        if d < 1:
            start, end = m.line.get_start(), m.line.get_end()
            m.line.put_start_and_end_on(start, start + (end - start) * max(d, 1e-3))
        m.label.set_opacity(d)
    draw(line)
    line.add_updater(draw)


def faded(band, shown: ValueTracker) -> None:
    band.add_updater(lambda m: m.set_fill(opacity=m.get_fill_opacity() * shown.get_value()))


class Sketch(ExactScene):
    def construct(self):
        self.add(backdrop())
        screen = Screen(Recording("smb1"), START, left_px=36 * config.pixel_height // 1080)
        feet = feet_line(screen)
        setting = setting_line(screen, SETTING, "")
        band = gap_band(screen, feet, setting)
        right = screen.col_x(256)
        neuron = Neuron([Input(feet.value, -1, lambda: np.array([right, feet.y(), 0]), "depth")], SETTING,
                        lambda: np.array([right, setting.y(), 0]), CENTRE, "in the air", "on the ground",
                        bias_name="the setting")
        ruler, gap = ValueTracker(0.0), ValueTracker(0.0)
        drawn_along(feet, ruler)
        faded(band, gap)
        paths, body, bias, output = neuron.arrive()

        self.play(screen.arrive())
        self.add(band, feet, neuron)
        self.play(ruler.animate.set_value(1.0), run_time=0.5)
        self.play(paths)
        self.play(body)
        self.play(Create(setting), run_time=0.6)
        self.play(bias, gap.animate.set_value(1.0))
        self.play(output)
        self.wait((END - screen.frame()) / 60)  # to the last frame, whatever the arrivals took
