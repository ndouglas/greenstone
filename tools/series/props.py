"""Small drawings more than one video uses: the console, a check mark, and the controller."""

from pathlib import Path

import numpy as np
from manim import DOWN, LEFT, RIGHT, UP, Circle, Line, Rectangle, RoundedRectangle, Square, ValueTracker, VGroup, VMobject

from series import style
from series.ember import serif
from series.recording import LAG
from series.screen import Screen

BUTTON_A, BUTTON_B, BUTTON_UP, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT = 0x01, 0x02, 0x10, 0x20, 0x40, 0x80  # src/nes/bus


def nes(at: np.ndarray, width: float = 3.2) -> VGroup:
    """The console in hairlines: its body, the darker front stripe, the cartridge door, two buttons, two ports."""
    k = width / 3.2
    body = RoundedRectangle(corner_radius=0.06 * k, width=3.2 * k, height=1.15 * k, stroke_color=style.ASH,
                            stroke_width=1.4)
    stripe = Line(LEFT * 1.6 * k, RIGHT * 1.6 * k, stroke_color=style.ASH, stroke_width=1.0).shift(DOWN * 0.18 * k)
    door = Rectangle(width=1.7 * k, height=0.42 * k, stroke_color=style.ASH, stroke_width=1.2).shift(
        UP * 0.2 * k + RIGHT * 0.15 * k)
    buttons = VGroup(*[Rectangle(width=0.22 * k, height=0.09 * k, stroke_color=style.WARM[0], stroke_width=1.2).shift(
        LEFT * (1.15 - 0.32 * i) * k + DOWN * 0.38 * k) for i in range(2)])
    ports = VGroup(*[Rectangle(width=0.18 * k, height=0.2 * k, stroke_color=style.ASH, stroke_width=1.0).shift(
        RIGHT * (0.3 + 0.35 * i) * k + DOWN * 0.38 * k) for i in range(2)])
    return VGroup(body, stripe, door, buttons, ports).move_to(at)


def tick(at: np.ndarray, size: float = 0.2) -> VMobject:
    """A check mark."""
    return VMobject(stroke_color=style.WARM[1], stroke_width=2.0).set_points_as_corners(
        [at + size * np.array(p) for p in ((-0.5, 0.0, 0), (-0.15, -0.4, 0), (0.55, 0.45, 0))])


class Pad(VGroup):
    """The controller as hairlines, the d-pad a cross and B and A two rings, each key lighting while the
    program holds it down for the picture on screen."""

    def __init__(self, screen: Screen, at: np.ndarray, buttons: Path, key: float = 0.44):
        super().__init__()
        self.screen = screen
        self.buttons = np.fromfile(buttons, dtype=np.uint8)
        self.level = ValueTracker(1.0)
        cross = at + LEFT * 1.3
        self.keys = {bit: Square(key).move_to(cross + direction * key)
                     for bit, direction in ((BUTTON_UP, UP), (BUTTON_DOWN, DOWN), (BUTTON_LEFT, LEFT), (BUTTON_RIGHT, RIGHT))}
        self.hub = Square(key, stroke_width=0, fill_opacity=0).move_to(cross)
        for bit, x in ((BUTTON_B, 0.65), (BUTTON_A, 1.75)):
            self.keys[bit] = Circle(radius=0.36).move_to(at + RIGHT * x)
        self.names = VGroup(*[serif(name, style.ASH, 0.36).next_to(self.keys[bit], DOWN, buff=0.18)
                              for name, bit in (("B", BUTTON_B), ("A", BUTTON_A))])
        self.add(self.hub, *self.keys.values(), self.names)
        self.add_updater(lambda m: m.draw())
        self.draw()

    def held(self) -> int:
        """The buttons pressed during the frame whose RAM the picture shows (the picture trails it by LAG)."""
        return int(self.buttons[max(self.screen.frame() - LAG["smb1"], 0)])

    def draw(self) -> None:
        held, level = self.held(), self.level.get_value()
        for bit, key in self.keys.items():
            on = bool(held & bit)
            key.set_stroke(style.WARM[1] if on else style.ASH, width=1.4, opacity=level)
            key.set_fill(style.WARM[1], opacity=0.75 * level if on else 0)
        self.names.set_opacity(level)
