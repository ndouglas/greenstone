"""The map: every moment a dot, one input across and the other up. A two-input neuron's border is the
line where w·p + b = 0; the yes side is faintly warm, the no side faintly cool, as the graph tints its sides."""

import numpy as np
from manim import DOWN, LEFT, RIGHT, UP, Animation, Dot, Line, ManimColor, Polygon, UpdateFromAlphaFunc, VGroup, interpolate_color

from series import style
from series.ember import serif

DOT = "#6B5B52"  # a moment with no outcome: dim ash
DOT_RADIUS = 0.032
LIT_RADIUS = 0.05
HALO = 0.15  # the small glow round a lit dot
HUES = {"floor": style.WARM[0], "blocks": style.WARM[3]}
SIDE = 0.11  # how faintly each side is tinted, at its far edge
NEAR = 0.02  # and at the border
FADE = 14  # bands across each side
STEPS = 32  # short pieces along the border


def clip(poly: list, w: np.ndarray, b: float, lo: float, hi: float) -> list:
    """The part of a convex polygon (data coordinates) where lo <= w·p + b <= hi."""
    for sign, edge in ((1.0, lo), (-1.0, -hi)):
        out = []
        for i, p in enumerate(poly):
            q = poly[(i + 1) % len(poly)]
            fp, fq = sign * (w @ p + b) - edge, sign * (w @ q + b) - edge
            if fp >= 0:
                out.append(p)
            if (fp >= 0) != (fq >= 0):
                out.append(p + (q - p) * fp / (fp - fq))
        poly = out
        if not poly:
            break
    return poly


class BorderMap(VGroup):
    """Dots at `points` (n, 2), `outcomes` naming the lit ones. total = w·p + b; the border is where it is zero.
    slide_bias() moves the border parallel to itself; lean() turns it about its own middle."""

    def __init__(self, points: np.ndarray, outcomes: list, weights: tuple, bias: float, x_label: str,
                 y_label: str, yes: str = "yes", no: str = "no", width: float = 7.0, height: float = 4.4,
                 at: np.ndarray = None, hues: dict = HUES, x_hue: str = style.WARM[0], y_hue: str = style.WARM[3]):
        super().__init__()
        pts = np.asarray(points, float)
        self.data, self.outcomes = pts, list(outcomes)
        self.weights, self.bias = np.asarray(weights, float), float(bias)
        self.size, self.at = (width, height), np.zeros(3) if at is None else np.asarray(at, float)
        lo, hi = pts.min(0), pts.max(0)
        pad = (hi - lo) * 0.06
        self.lo, self.hi = lo - pad, hi + pad
        self.x_hue, self.y_hue = x_hue, y_hue
        self.bands = [Polygon(*[self.at] * 3, stroke_width=0, fill_opacity=0) for _ in range(2 * FADE)]
        self.pieces = [Line(self.at, self.at, stroke_width=1.4) for _ in range(STEPS)]
        self.dots = []
        halos, cores = VGroup(), VGroup()
        for p, o in zip(pts, self.outcomes):
            where = self.page(p)
            if o in hues:
                halos.add(Dot(where, radius=HALO, color=hues[o], fill_opacity=0.16, stroke_width=0))
                d = Dot(where, radius=LIT_RADIUS, color=hues[o], fill_opacity=1, stroke_width=0)
            else:
                d = Dot(where, radius=DOT_RADIUS, color=DOT, fill_opacity=0.7, stroke_width=0)
            self.dots.append(d)
            cores.add(d)
        corner = self.page(self.lo)
        axes = VGroup(
            Line(corner, self.page([self.lo[0], self.hi[1]]), stroke_color=style.ASH, stroke_width=1.2),
            Line(corner, self.page([self.hi[0], self.lo[1]]), stroke_color=style.ASH, stroke_width=1.2),
        )
        x_text = serif(x_label, style.ASH, 0.3).next_to(axes[1].get_end(), DOWN, buff=0.12).align_to(axes[1].get_end(), RIGHT)
        y_text = serif(y_label, style.ASH, 0.3).next_to(axes[0].get_end(), RIGHT, buff=0.12).shift(DOWN * 0.1)
        self.yes_label = serif(yes, style.WARM[1], 0.32).set_opacity(0.85)
        self.no_label = serif(no, style.COOL[1], 0.32).set_opacity(0.85)
        top = self.page(self.hi)[1] + 0.3  # outside the plot, above its edge, each on its own side
        self.yes_label.move_to([self.page(self.hi)[0], top, 0]).align_to(self.page(self.hi), RIGHT)
        self.no_label.move_to([self.page(self.lo)[0], top, 0]).align_to(self.page(self.lo), LEFT)
        self._lay()
        self.border = VGroup(*self.pieces)
        self.add(*self.bands, axes, x_text, y_text, self.yes_label, self.no_label, halos, cores, self.border)

    def page(self, p) -> np.ndarray:
        """Where the data point p falls on the page."""
        p = np.asarray(p, float)
        frac = (p - self.lo) / (self.hi - self.lo) - 0.5
        return self.at + np.array([frac[0] * self.size[0], frac[1] * self.size[1], 0.0])

    def total(self, p) -> float:
        return float(self.weights @ np.asarray(p, float) + self.bias)

    def border_points(self) -> tuple:
        """Where the border meets the edges of the map; both the same point when there is no border to draw."""
        w, b, lo, hi = self.weights, self.bias, self.lo, self.hi
        found = []
        for axis in (0, 1):
            other = 1 - axis
            if w[other] == 0:
                continue
            for edge in (lo[axis], hi[axis]):
                v = -(b + w[axis] * edge) / w[other]
                if lo[other] - 1e-9 <= v <= hi[other] + 1e-9:
                    p = np.zeros(2)
                    p[axis], p[other] = edge, v
                    found.append(p)
        if len(found) < 2:
            return lo.copy(), lo.copy()
        found.sort(key=lambda p: (p[0], p[1]))
        return found[0], found[-1]

    def _tint(self) -> None:
        box = [self.lo, np.array([self.hi[0], self.lo[1]]), self.hi, np.array([self.lo[0], self.hi[1]])]
        top = max(abs(self.total(c)) for c in box) or 1.0
        for k, band in enumerate(self.bands):
            yes = k < FADE
            i = k % FADE
            a, z = top * i / FADE, top * (i + 1) / FADE
            poly = clip(box, self.weights, self.bias, *((a, z) if yes else (-z, -a)))
            colour, strength = (style.WARM[0], (i + 0.5) / FADE) if yes else (style.COOL[0], (i + 0.5) / FADE)
            if len(poly) < 3:
                band.set_points_as_corners([self.at] * 3).set_fill(opacity=0)
                continue
            band.set_points_as_corners([self.page(p) for p in poly] + [self.page(poly[0])])
            band.set_fill(colour, opacity=NEAR + (SIDE - NEAR) * strength).set_stroke(width=0)

    def _lay(self) -> None:
        self._tint()
        a, z = self.border_points()
        drawn = np.linalg.norm(a - z) > 1e-9
        a, z = self.page(a), self.page(z)
        for i, piece in enumerate(self.pieces):
            p, q = a + (z - a) * i / STEPS, a + (z - a) * (i + 1) / STEPS
            piece.put_start_and_end_on(p, q if drawn else p + RIGHT * 1e-6)
            colour = interpolate_color(ManimColor(self.x_hue), ManimColor(self.y_hue), (i + 0.5) / STEPS)
            piece.set_stroke(color=colour, width=1.4, opacity=1 if drawn else 0)

    def slide_bias(self, new: float) -> Animation:
        """Move the border parallel to itself, until the bias is `new`."""
        def update(b0: float, alpha: float) -> None:
            self.bias = b0 + (float(new) - b0) * alpha
            self._lay()
        return _AtBegin(self, lambda: self.bias, update)

    def lean(self, weights: tuple) -> Animation:
        """Turn the border to the new weights, about the middle of where it crosses the map as the turn begins."""
        target = np.asarray(weights, float)

        def start() -> tuple:
            a, z = self.border_points()
            return self.weights.copy(), (a + z) / 2 if np.linalg.norm(a - z) > 1e-9 else (self.lo + self.hi) / 2

        def update(state: tuple, alpha: float) -> None:
            w0, pivot = state
            self.weights = w0 + (target - w0) * alpha
            self.bias = -float(self.weights @ pivot)
            self._lay()
        return _AtBegin(self, start, update)


class _AtBegin(UpdateFromAlphaFunc):
    """An alpha update whose starting state is taken as the animation begins, not when it was made."""

    def __init__(self, mobject, start, update):
        self._start = start
        self._state = None
        super().__init__(mobject, lambda _, alpha: update(self._state, alpha))

    def begin(self) -> None:
        self._state = self._start()
        super().begin()
