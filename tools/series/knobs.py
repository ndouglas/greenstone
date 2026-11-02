"""The neuron's knobs: one dial for each weight and one for the bias, each needle turned by its value (zero
straight up, more turning it clockwise, less anticlockwise), lit by its sign; and the total, a bar up from
zero or down, with optional targets marked."""

import math

import numpy as np
from manim import DOWN, LEFT, RIGHT, UP, Animation, Arc, Circle, DashedLine, Dot, Line, Rectangle, UpdateFromAlphaFunc, VGroup

from series import style
from series.ember import numeral, serif, swap

RADIUS = 0.42
SWEEP = math.radians(150)  # the furthest a needle turns either way


def reading(value: float) -> str:
    return style.number(value, 2)


class Dial(VGroup):
    def __init__(self, label: str, scale: float, at: np.ndarray, value: float = 0.0):
        super().__init__()
        self.scale_, self.at, self.value = scale, np.asarray(at, float), float(value)
        self.ring = Circle(radius=RADIUS, stroke_color=style.ASH, stroke_width=1.2).move_to(self.at)
        self.zero = Line(self.at + UP * (RADIUS - 0.08), self.at + UP * (RADIUS + 0.04), stroke_color=style.ASH,
                         stroke_width=1.2)
        self.arc = Arc(radius=RADIUS, arc_center=self.at, start_angle=math.pi / 2, angle=0.0, stroke_width=3.2)
        self.needle = Line(self.at, self.at + UP * RADIUS * 0.82, stroke_width=1.8)
        self.hub = Dot(self.at, radius=0.035, color=style.TEXT)
        self.name = serif(label, style.ASH, 0.3).next_to(self.ring, DOWN, buff=0.36)
        self.number = VGroup()
        self.shown = None
        self.add(self.ring, self.zero, self.arc, self.needle, self.hub, self.name, self.number)
        self.set_value(value)

    def turn(self) -> float:
        """The needle's angle from straight up, clockwise, for the current value."""
        return SWEEP * math.tanh(self.value / self.scale_)

    def set_value(self, value: float) -> None:
        self.value = float(value)
        colour = style.temperature(self.value)
        t = self.turn()
        tip = self.at + RADIUS * 0.82 * np.array([math.sin(t), math.cos(t), 0.0])
        self.needle.put_start_and_end_on(self.at, tip if abs(t) > 1e-9 else self.at + UP * RADIUS * 0.82)
        self.needle.set_stroke(color=colour if self.value else style.TEXT)
        new = Arc(radius=RADIUS, arc_center=self.at, start_angle=math.pi / 2, angle=-t if abs(t) > 1e-4 else -1e-4,
                  stroke_width=3.2, stroke_color=colour, stroke_opacity=0.9 if abs(t) > 1e-4 else 0)
        self.arc.become(new)
        text = reading(self.value)
        if text != self.shown:
            swap(self.number, numeral(text, colour if self.value else style.TEXT, 0.32))
            self.shown = text
        self.number.next_to(self.ring, DOWN, buff=0.06)


class Total(VGroup):
    """A bar from zero, up for a positive total and down for a negative one, its value beside it; `targets`
    are marked as short ticks."""

    def __init__(self, at: np.ndarray, scale: float = 1.0, half: float = 1.1, targets: tuple = (),
                 label: str = "total"):
        super().__init__()
        self.at, self.scale_, self.half = np.asarray(at, float), scale, half
        self.value = 0.0
        self.axis = Line(self.at + DOWN * half, self.at + UP * half, stroke_color="#4A3C34", stroke_width=1.0)
        self.zero = Line(self.at + LEFT * 0.22, self.at + RIGHT * 0.22, stroke_color=style.ASH, stroke_width=1.4)
        self.zero_text = numeral("0", style.ASH, 0.26).next_to(self.zero, LEFT, buff=0.08)
        self.bar = Rectangle(width=0.22, height=1e-3, stroke_width=0, fill_opacity=0.85)
        self.name = serif(label, style.ASH, 0.3).next_to(self.axis, DOWN, buff=0.2)
        self.number = VGroup()
        self.ticks = VGroup()
        for t in targets:
            y = self.y(t)
            self.ticks.add(DashedLine(self.at + LEFT * 0.34 + UP * y, self.at + RIGHT * 0.34 + UP * y,
                                      stroke_color=style.TEXT, stroke_width=1.2, dash_length=0.05),
                           numeral(("+" if t > 0 else "") + style.number(t), style.TEXT, 0.26)
                           .next_to(self.at + RIGHT * 0.34 + UP * y, RIGHT, buff=0.08))
        self.shown = None
        self.add(self.axis, self.ticks, self.bar, self.zero, self.zero_text, self.name, self.number)
        self.set_value(0.0)

    def y(self, value: float) -> float:
        """Height above zero on the page: in proportion near zero, squeezed far from it so big totals fit."""
        return self.half * math.tanh(value / (2 * self.scale_))

    def set_value(self, value: float) -> None:
        self.value = float(value)
        y = self.y(self.value)
        colour = style.temperature(self.value)
        self.bar.stretch_to_fit_height(max(abs(y), 1e-3)).move_to(self.at + UP * y / 2)
        self.bar.set_fill(colour, opacity=0.85 if self.value else 0)
        text = reading(self.value)
        if text != self.shown:
            swap(self.number, numeral(text, colour if self.value else style.TEXT, 0.32))
            self.shown = text
        self.number.next_to(self.at + UP * y, LEFT, buff=0.3)


class Knobs(VGroup):
    """Dials for (position, speed, bias), side by side, and optionally the total for a moment."""

    def __init__(self, at: np.ndarray, values=(0.0, 0.0, 0.0), scales=(1.0, 1.0, 2.0),
                 labels=("position knob", "speed knob", "bias"), gap: float = 1.35):
        super().__init__()
        at = np.asarray(at, float)
        self.dials = [Dial(l, s, at + RIGHT * gap * (i - 1), v) for i, (l, s, v) in enumerate(zip(labels, scales, values))]
        self.add(*self.dials)

    @property
    def values(self) -> np.ndarray:
        return np.array([d.value for d in self.dials])

    def set_values(self, v) -> None:
        for d, x in zip(self.dials, v):
            d.set_value(x)

    def turn_to(self, v, total: Total | None = None, moment: np.ndarray | None = None) -> Animation:
        """Turn every dial from where it is to v; a total, if given, follows the moment's total."""
        end = np.asarray(v, float)
        knobs = self

        class Turn(UpdateFromAlphaFunc):
            def begin(inner):
                inner.start = knobs.values.copy()
                super().begin()

        def update(_, a: float) -> None:
            now = turn.start * (1 - a) + end * a
            knobs.set_values(now)
            if total is not None and moment is not None:
                total.set_value(float(now[:2] @ moment + now[2]))
        turn = Turn(self, update)
        return turn
