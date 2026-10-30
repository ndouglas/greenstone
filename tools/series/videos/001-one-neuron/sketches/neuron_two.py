"""The two-input neuron: depth's ruler and the falling-speed arrow at Mario, weighed by 001's land-soon
weights (total = 1 × depth + speed weight × falling speed − threshold), answering "lands soon"."""

import json
from pathlib import Path

import numpy as np
from manim import RIGHT, UP, Scene, ValueTracker, config

from series.ember import backdrop
from series.neuron import Input, Neuron
from series.overlays import MARIO_HALF, feet_line, signed, speed_arrow
from series.recording import Recording
from series.screen import Screen

NUMBERS = json.loads((Path(__file__).parent.parent / "numbers.json").read_text())
SPEED_WEIGHT, THRESHOLD = NUMBERS["smb1_land_speed_weight"], NUMBERS["smb1_land_threshold"]
X, DEPTH, FALLING = 0x0086, 0x00CE, 0x009F
START, END = 1247, 1943
CENTRE = RIGHT * 2.3 + UP * 0.2


class Sketch(Scene):
    def construct(self):
        self.add(backdrop())
        screen = Screen(Recording("smb1"), START, left_px=36 * config.pixel_height // 1080)
        feet = feet_line(screen)
        arrow = speed_arrow(screen, X, DEPTH, FALLING)
        right = screen.col_x(256)

        def mario() -> np.ndarray:  # where the arrow starts: his middle
            ram = screen.ram()
            return np.array([screen.col_x(int(ram[X]) + MARIO_HALF[0]),
                             screen.row_y(int(ram[DEPTH]) + MARIO_HALF[1]), 0])

        neuron = Neuron([Input(feet.value, 1, lambda: np.array([right, feet.y(), 0]), "depth"),
                         Input(lambda: signed(int(screen.ram()[FALLING])), SPEED_WEIGHT, mario, "falling")],
                        -THRESHOLD, None, CENTRE, "lands soon", "not yet")
        steps = neuron.arrive()
        shown = ValueTracker(0.0)
        for mark in (feet, arrow):
            mark.add_updater(lambda m: m.set_opacity(shown.get_value()))

        self.play(screen.arrive())
        self.add(feet, arrow, neuron)
        self.play(shown.animate.set_value(1.0), run_time=0.5)
        for step in steps:
            self.play(step)
        self.wait((END - screen.frame()) / 60)  # to the last frame, whatever the arrivals took
