"""Small drawings more than one video uses: the console, a check mark, and the controller."""

from pathlib import Path

import numpy as np
from manim import DOWN, LEFT, RIGHT, UP, Circle, Line, Polygon, Rectangle, RoundedRectangle, Square, ValueTracker, VGroup, VMobject

from series import style
from series.ember import serif
from series.recording import LAG
from series.screen import Screen

BUTTON_A, BUTTON_B, BUTTON_UP, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT = 0x01, 0x02, 0x10, 0x20, 0x40, 0x80  # src/nes/bus


def nes(at: np.ndarray, width: float = 3.2) -> VGroup:
    """The front-loading console (NES-001) in hairlines, seen from in front and a little above and to the right, as
    in its photographs: a light upper half with the cartridge door's lip across most of it, a darker lower band,
    tucked in toward the bottom at the front and the side, with the power light and the POWER and RESET buttons, the dark strip with the two controller ports, and on top the
    door and the ridged strip behind its end caps."""
    k = width / 3.2
    w, h = 3.2 * k, 1.12 * k  # the front: 254 by 89 mm
    depth = np.array([0.62, 0.5, 0.0]) * k  # the top's front-to-back run, foreshortened up and to the right
    split = 0.45 * h  # the dark lower band's height
    strip = (0.635 * w, 0.775 * w)  # the dark strip, across the front from the left
    o = np.array([-w / 2, -h / 2, 0.0])  # the front's bottom-left corner

    def p(x: float, y: float, d: float = 0.0) -> np.ndarray:
        """A point x across the front from its left, y up from its bottom, d of the way back."""
        return o + np.array([x, y, 0.0]) + depth * d

    def shape(points, colour=style.ASH, width_=1.2, fill=None, opacity=0.0) -> Polygon:
        return Polygon(*points, stroke_color=colour, stroke_width=width_, fill_color=fill or colour, fill_opacity=opacity)

    dark, black = "#2C2427", "#151112"
    # Below the light upper half the body tucks in: the dark band slopes back and in, at the front and the side,
    # so the bottom is smaller than the top.
    inset, back = 0.035 * w, 0.1  # how far the bottom edge is in from each side, and back from the front

    def low(x: float, y: float, side: bool = False) -> np.ndarray:
        """A point on the sloping band, x across the front (or, on the side, x of the way back), y up from the
        bottom: where the slope from the band's top edge down to the tucked-in bottom edge is at that height."""
        t = y / split
        if side:
            return p(w, split, x) * t + p(w - inset, 0, back + x * (1 - 2 * back)) * (1 - t)
        return p(x, split) * t + p(inset + x * (w - 2 * inset) / w, 0, back) * (1 - t)

    upper = shape([p(0, split), p(w, split), p(w, h), p(0, h)], width_=1.4)
    band = shape([low(0, split), low(w, split), low(w, 0), low(0, 0)], fill=dark, opacity=0.9, width_=1.2)
    front = VGroup(upper, band)
    top = shape([p(0, h), p(w, h), p(w, h, 1), p(0, h, 1)], width_=1.4)
    side = shape([p(w, split), p(w, split, 1), p(w, h, 1), p(w, h)], width_=1.4)
    side_band = shape([low(0, split, True), low(1, split, True), low(1, 0, True), low(0, 0, True)], fill=dark, opacity=0.9,
                      width_=1.2)
    front_strip = VGroup(shape([p(strip[0], split), p(strip[1], split), p(strip[1], h), p(strip[0], h)], fill=black, opacity=0.95, width_=1.0),
                         shape([low(strip[0], split), low(strip[1], split), low(strip[1], 0), low(strip[0], 0)], fill=black, opacity=0.95, width_=1.0))
    # On top: the door over the front of the left part; the strip running back, dark caps at each end, ridged between.
    door = shape([p(0.03 * w, h, 0.0), p(0.62 * w, h, 0.0), p(0.62 * w, h, 0.45), p(0.03 * w, h, 0.45)], width_=1.0)
    lip = Line(p(0.03 * w, h - 0.06 * h), p(0.62 * w, h - 0.06 * h), stroke_color=style.ASH, stroke_width=0.9)
    caps = VGroup(shape([p(strip[0], h, 0), p(strip[1], h, 0), p(strip[1], h, 0.16), p(strip[0], h, 0.16)], fill=black, opacity=0.95, width_=1.0),
                  shape([p(strip[0], h, 0.8), p(strip[1], h, 0.8), p(strip[1], h, 1), p(strip[0], h, 1)], fill=black, opacity=0.95, width_=1.0))
    ridges = VGroup(*[Line(p(strip[0], h, d), p(strip[1], h, d), stroke_color=style.ASH, stroke_width=0.7)
                      for d in np.linspace(0.2, 0.76, 12)])
    led = Rectangle(width=0.035 * k, height=0.035 * k, stroke_width=0, fill_color=style.WARM[4], fill_opacity=0.9).move_to(low(0.075 * w, 0.28 * h))
    buttons = VGroup(*[Rectangle(width=0.075 * w, height=0.12 * h, stroke_color=style.WARM[0], stroke_width=1.2).move_to(low(x * w, 0.25 * h))
                       for x in (0.13, 0.215)])
    ports = VGroup(*[RoundedRectangle(corner_radius=0.02 * k, width=0.042 * w, height=0.22 * h, stroke_color=style.ASH, stroke_width=1.0)
                     .move_to(low(x * w, 0.2 * h)) for x in (0.672, 0.738)])
    return VGroup(top, side, side_band, front, band, front_strip, door, lip, caps, ridges, led, buttons, ports).move_to(at)


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
