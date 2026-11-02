"""A line chart in Ember: hairline axes, a few labelled values on each, series drawn as hairlines that can
draw themselves on, and reference levels (the best border's score, always yes) as dashed lines."""

import numpy as np
from manim import DOWN, LEFT, RIGHT, UP, Animation, DashedLine, Line, UpdateFromAlphaFunc, VGroup, VMobject

from series import style
from series.ember import numeral, serif


class Chart(VGroup):
    def __init__(self, x_range: tuple, y_range: tuple, x_label: str, y_label: str, width: float = 6.0,
                 height: float = 3.4, at: np.ndarray = None, x_ticks: list = (), y_ticks: list = (),
                 y_format=lambda v: f"{v:.0f}"):
        super().__init__()
        self.x_range, self.y_range = x_range, y_range
        self.size, self.at = (width, height), np.zeros(3) if at is None else np.asarray(at, float)
        (x0, x1), (y0, y1) = x_range, y_range
        corner = self.point(x0, y0)
        self.axes = VGroup(Line(corner, self.point(x0, y1), stroke_color=style.ASH, stroke_width=1.2),
                           Line(corner, self.point(x1, y0), stroke_color=style.ASH, stroke_width=1.2))
        ticks = VGroup()
        for x in x_ticks:
            p = self.point(x, y0)
            ticks.add(Line(p, p + DOWN * 0.07, stroke_color=style.ASH, stroke_width=1.0),
                      numeral(f"{x:g}", style.ASH, 0.26).next_to(p, DOWN, buff=0.1))
        for y in y_ticks:
            p = self.point(x0, y)
            ticks.add(Line(p, p + LEFT * 0.07, stroke_color=style.ASH, stroke_width=1.0),
                      numeral(y_format(y), style.ASH, 0.26).next_to(p, LEFT, buff=0.1))
        self.x_text = serif(x_label, style.ASH, 0.3).next_to(self.axes[1], DOWN, buff=0.42)
        self.y_text = serif(y_label, style.ASH, 0.3).next_to(self.axes[0].get_end(), UP, buff=0.14).align_to(self.axes[0], LEFT)
        self.add(self.axes, ticks, self.x_text, self.y_text)

    def point(self, x: float, y: float) -> np.ndarray:
        (x0, x1), (y0, y1) = self.x_range, self.y_range
        return self.at + np.array([((x - x0) / (x1 - x0) - 0.5) * self.size[0],
                                   ((min(max(y, y0), y1) - y0) / (y1 - y0) - 0.5) * self.size[1], 0.0])

    def series(self, xs, ys, colour: str, width: float = 1.6) -> VMobject:
        line = VMobject(stroke_color=colour, stroke_width=width)
        line.set_points_as_corners([self.point(x, y) for x, y in zip(xs, ys)])
        return line

    def level(self, y: float, label: str, colour: str) -> VGroup:
        (x0, x1) = self.x_range
        a, z = self.point(x0, y), self.point(x1, y)
        dash = DashedLine(a, z, stroke_color=colour, stroke_width=1.2, dash_length=0.07)
        return VGroup(dash, serif(label, colour, 0.28).next_to(z, RIGHT, buff=0.12))

    def grow(self, xs, ys, colour: str, width: float = 1.6) -> tuple[VMobject, Animation]:
        """A series that draws itself left to right; the animation and the line."""
        line = VMobject(stroke_color=colour, stroke_width=width)
        pts = [self.point(x, y) for x, y in zip(xs, ys)]
        line.set_points_as_corners([pts[0], pts[0] + RIGHT * 1e-6])

        def update(m, a: float) -> None:
            k = max(int(round(a * (len(pts) - 1))), 1)
            m.set_points_as_corners(pts[:k + 1])
        return line, UpdateFromAlphaFunc(line, update)
