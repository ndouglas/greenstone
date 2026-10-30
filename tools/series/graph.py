"""The graph: a neuron's total plotted against one input. The line is lit by its own sign, warm while the
total is above zero, dark exactly at the crossing, cool below; the crossing is the setting."""

import numpy as np
from manim import DOWN, RIGHT, UP, Animation, Dot, Line, ManimColor, Rectangle, UpdateFromAlphaFunc, VGroup, interpolate_color

from series import style
from series.ember import numeral, serif

DARK = "#2A2226"  # the line where the total is zero
GRID = "#2E2628"
SIDE = 0.11  # how faintly each side is tinted, at its far edge
NEAR = 0.02  # and at the setting
FADE = 24  # strips across each side
SPAN = 0.4  # how far from zero, as a share of the page's height, the line reaches its full colour
STEPS = 64  # short pieces per side of the crossing


class NeuronGraph(VGroup):
    """total = weight * x + bias against x. scale_weights() steepens the line about the setting, which
    stays where it is, as do the tinted sides."""

    def __init__(self, weight: float, bias: float, x_range: tuple, y_range: tuple, x_label: str,
                 y_label: str = "total", yes: str = "yes", no: str = "no", width: float = 7.0,
                 height: float = 4.0, at: np.ndarray = None):
        super().__init__()
        self.weight, self.bias = float(weight), float(bias)
        self.x_range, self.y_range = x_range, y_range
        self.size, self.at = (width, height), np.zeros(3) if at is None else np.asarray(at, float)
        self._build_frame(x_label, y_label, yes, no)
        self.segments = [Line(self.at, self.at, stroke_width=1.4) for _ in range(2 * STEPS)]
        self.line = VGroup(*self.segments)
        self._draw_line()
        self.add(self.line)
        if self._on_page(self.setting()):  # off the page there is no setting to mark
            self.add(self.mark, self.mark_label, self.mark_value)

    def setting(self) -> float:
        """Where the total crosses zero; a flat line (weight 0) never does, so its setting is infinite."""
        return -self.bias / self.weight if self.weight else float("inf")

    def total(self, x: float) -> float:
        return self.weight * x + self.bias

    def point(self, x: float, y: float) -> np.ndarray:
        """Where (x, y) falls on the page."""
        (x0, x1), (y0, y1) = self.x_range, self.y_range
        w, h = self.size
        return self.at + np.array([(x - x0) / (x1 - x0) - 0.5, (y - y0) / (y1 - y0) - 0.5, 0.0]) * [w, h, 1]

    def _on_page(self, x: float) -> bool:
        return self.x_range[0] <= x <= self.x_range[1]

    def _build_frame(self, x_label: str, y_label: str, yes: str, no: str) -> None:
        (x0, x1), (y0, y1) = self.x_range, self.y_range
        base = max(y0, min(0.0, y1))  # the horizontal axis sits at zero if zero is on the page
        ground = self.setting()
        if self.weight:
            cut = min(max(ground, x0), x1)
        else:  # flat: the whole page is on one side
            cut = x1 if self.bias > 0 else x0
        yes_left = self.weight <= 0
        left, right = (style.WARM[0], style.COOL[0]) if yes_left else (style.COOL[0], style.WARM[0])
        zones = [self._side(x0, cut, left, False), self._side(cut, x1, right, True)]
        self.yes_zone, self.no_zone = (zones[0], zones[1]) if yes_left else (zones[1], zones[0])
        grid = VGroup()
        for fx in (0.25, 0.5, 0.75):
            x = self.point(x0 + fx * (x1 - x0), 0)[0]
            grid.add(Line([x, self.point(0, y0)[1], 0], [x, self.point(0, y1)[1], 0], stroke_color=GRID, stroke_width=0.5))
        for fy in (0.25, 0.5, 0.75):
            y = self.point(0, y0 + fy * (y1 - y0))[1]
            grid.add(Line([self.point(x0, 0)[0], y, 0], [self.point(x1, 0)[0], y, 0], stroke_color=GRID, stroke_width=0.5))
        axis_y = self.point(0, base)[1]
        axes = VGroup(
            Line(self.point(x0, y0), self.point(x0, y1), stroke_color=style.ASH, stroke_width=1.2),
            Line([self.point(x0, 0)[0], axis_y, 0], [self.point(x1, 0)[0], axis_y, 0], stroke_color=style.ASH, stroke_width=1.2),
        )
        depth = serif(x_label, style.ASH, 0.3).next_to(axes[1].get_end(), DOWN, buff=0.12).align_to(axes[1].get_end(), RIGHT)
        total = serif(y_label, style.ASH, 0.3).next_to(axes[0].get_end(), RIGHT, buff=0.12).shift(DOWN * 0.1)
        self.yes_label = serif(yes, style.WARM[1], 0.32).set_opacity(0.85)
        self.no_label = serif(no, style.COOL[1], 0.32).set_opacity(0.85)
        edges = self.point(x0, 0)[0], self.point(x1, 0)[0]
        for label, zone in ((self.yes_label, self.yes_zone), (self.no_label, self.no_zone)):
            label.move_to(zone.get_bottom() + UP * 0.35)  # an empty side is a sliver at the page's edge: keep the word on the page
            label.shift(RIGHT * (max(edges[0] - label.get_left()[0], 0) - max(label.get_right()[0] - edges[1], 0)))
        self.mark = Dot(self.point(ground, base), radius=0.06, color=style.TEXT)
        self.mark_label = serif("the setting", "#D8CFC8", 0.3).next_to(self.mark, UP + RIGHT, buff=0.1)
        self.mark_value = numeral(style.number(ground) if self.weight else "", style.TEXT, 0.3).next_to(self.mark, DOWN, buff=0.12)
        if not self._on_page(ground):  # hidden, and left out of the group so nothing fades it back in
            for m in (self.mark, self.mark_label, self.mark_value):
                m.set_opacity(0)
        self.add(self.yes_zone, self.no_zone, grid, axes, depth, total, self.yes_label, self.no_label)

    def _side(self, a: float, b: float, colour: str, rising: bool) -> VGroup:
        """One tinted side: strongest away from the setting, fading toward it."""
        left, right = self.point(a, 0)[0], self.point(b, 0)[0]
        side = VGroup()
        for i in range(FADE):
            lo, hi = left + (right - left) * i / FADE, left + (right - left) * (i + 1) / FADE
            t = (i + 0.5) / FADE
            strength = t if rising else 1 - t
            strip = Rectangle(width=max(hi - lo, 1e-6), height=self.size[1], stroke_width=0, fill_color=colour,
                              fill_opacity=NEAR + (SIDE - NEAR) * strength)
            side.add(strip.move_to([(lo + hi) / 2, self.at[1], 0]))
        return side

    def _colour(self, y: float):
        """Dark at zero, then the sign's colour, its far shade as the line runs on."""
        f = min(abs(y) / (SPAN * (self.y_range[1] - self.y_range[0])), 1.0)
        near, far = (style.WARM[1], style.WARM[2]) if y > 0 else (style.COOL[0], style.COOL[1])
        shade = interpolate_color(ManimColor(near), ManimColor(far), f)
        return interpolate_color(ManimColor(DARK), shade, f ** 0.6)

    def _draw_line(self) -> None:
        """Lay the line as short pieces, each in its own sign's colour, the crossing between two halves."""
        (x0, x1), (y0, y1) = self.x_range, self.y_range
        w, b = self.weight, self.bias
        lo, hi = x0, x1
        if w:  # keep only the part of the line inside the page's height
            xa, xb = sorted(((y0 - b) / w, (y1 - b) / w))
            lo, hi = max(x0, xa), min(x1, xb)
        elif not y0 <= b <= y1:
            hi = lo
        cross = min(max(self.setting(), lo), hi) if w else lo
        halves = [(lo, cross), (cross, hi)]
        for h, (a, z) in enumerate(halves):
            xs = np.linspace(a, z, STEPS + 1)
            for i in range(STEPS):
                seg = self.segments[h * STEPS + i]
                if z - a <= 1e-9 or hi - lo <= 1e-9:
                    seg.set_stroke(opacity=0)
                    seg.put_start_and_end_on(self.at, self.at + RIGHT * 1e-6)
                    continue
                p, q = xs[i], xs[i + 1]
                seg.put_start_and_end_on(self.point(p, self.total(p)), self.point(q, self.total(q)))
                seg.set_stroke(color=self._colour(self.total((p + q) / 2)), opacity=1, width=1.4)

    def scale_weights(self, k: float) -> Animation:
        """Multiply the weight and the bias by k: the line steepens and pivots about the setting."""
        w0, b0 = self.weight, self.bias

        def update(_, alpha: float) -> None:
            f = 1 + (k - 1) * alpha
            self.weight, self.bias = w0 * f, b0 * f
            self._draw_line()
        return UpdateFromAlphaFunc(self.line, update)

