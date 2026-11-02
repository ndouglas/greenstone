"""Mario's map drawn by point: the game keeps position and speed in whole steps, so many moments share one
point. Each point is a dot, bigger and brighter the more moments it holds, warm where the screen moved soon,
cool where it didn't, and split into both, in proportion, where both happened. A border set by (w, b) splits
the map: total = w·p + b, yes where the total is above zero, its side faintly warm, the other faintly cool.

The dots are one image, redrawn only when what they show changes, so a map of a thousand points stays quick."""

import math

import numpy as np
from manim import DOWN, LEFT, RIGHT, UP, Animation, Group, ImageMobject, Line, ManimColor, Polygon, UpdateFromAlphaFunc, VGroup, config, interpolate_color

from series import style
from series.bordermap import clip
from series.ember import numeral, rgb, serif

MOVED, STILL = style.WARM[0], style.COOL[1]
SIDE, NEAR, FADE = 0.10, 0.015, 12  # the sides' tint, as the bordermap's
STEPS = 32
SMALLEST, LARGEST = 0.022, 0.06  # a dot's radius, for one moment and for the most crowded point
DIM = 0.55  # a single moment's brightness; the most crowded point's is 1


def dot_sprite(radius_px: float, share: float, warm: np.ndarray, cool: np.ndarray, level: float) -> np.ndarray:
    """An anti-aliased disc as RGBA floats, warm over `share` of its turn (from twelve o'clock, clockwise),
    cool over the rest."""
    n = int(math.ceil(radius_px + 1.5))
    yy, xx = np.mgrid[-n:n + 1, -n:n + 1].astype(float)
    alpha = np.clip(radius_px + 0.5 - np.hypot(xx, yy), 0, 1) * level
    turn = (np.arctan2(xx, -yy) / (2 * np.pi)) % 1.0  # 0 at the top, rising clockwise (rows run down)
    colour = np.where((turn < share)[..., None], warm, cool)
    if share >= 1:
        colour = np.broadcast_to(warm, colour.shape)
    elif share <= 0:
        colour = np.broadcast_to(cool, colour.shape)
    return np.dstack([colour, alpha])


class PointMap(Group):
    """The map, `x_range` blocks across and `y_range` pixels a frame up. `points` (k, 2); `moved`, `still`:
    each point's moments of each answer. set_line(w, b) places the border; follow() runs it along a path."""

    def __init__(self, points: np.ndarray, moved: np.ndarray, still: np.ndarray, x_range=(0.0, 5.0),
                 y_range=(0.0, 2.6), width: float = 7.6, height: float = 4.6, at: np.ndarray = None,
                 line: np.ndarray | None = None, x_label: str = "blocks from the left edge",
                 y_label: str = "pixels a frame, right", mark: bool = True):
        super().__init__()
        self.where = np.asarray(points, float)
        self.moved, self.still = np.asarray(moved, int), np.asarray(still, int)
        self.x_range, self.y_range = x_range, y_range
        self.size, self.at = (width, height), np.zeros(3) if at is None else np.asarray(at, float)
        self.level = np.ones(len(self.where))  # each point's brightness, 0 to 1, for fading some out
        self.ring = np.zeros(len(self.where), bool)  # points circled in parchment
        self.v = np.zeros(3) if line is None else np.asarray(line, float)
        self.bands = [Polygon(*[self.at] * 3, stroke_width=0, fill_opacity=0) for _ in range(2 * FADE)]
        self.pieces = [Line(self.at, self.at, stroke_width=1.4) for _ in range(STEPS)]
        self.border = VGroup(*self.pieces)
        self.dots = ImageMobject(np.zeros((2, 2, 4), np.uint8))
        self._shown = None
        self.add(*self.bands, self._frame(x_label, y_label, mark), self.dots, self.border)
        self.redraw()
        self.dots.add_updater(lambda m: self.redraw_dots())

    # where things are
    def page(self, p) -> np.ndarray:
        p = np.asarray(p, float)
        (x0, x1), (y0, y1) = self.x_range, self.y_range
        fx, fy = (p[0] - x0) / (x1 - x0) - 0.5, (p[1] - y0) / (y1 - y0) - 0.5
        return self.at + np.array([fx * self.size[0], fy * self.size[1], 0.0])

    def corners(self) -> list:
        (x0, x1), (y0, y1) = self.x_range, self.y_range
        return [np.array(c, float) for c in ((x0, y0), (x1, y0), (x1, y1), (x0, y1))]

    def total(self, p) -> float:
        return float(self.v[:2] @ np.asarray(p, float) + self.v[2])

    # the frame
    def _frame(self, x_label: str, y_label: str, mark: bool) -> VGroup:
        (x0, x1), (y0, y1) = self.x_range, self.y_range
        corner = self.page((x0, y0))
        axes = VGroup(Line(corner, self.page((x0, y1)), stroke_color=style.ASH, stroke_width=1.2),
                      Line(corner, self.page((x1, y0)), stroke_color=style.ASH, stroke_width=1.2))
        ticks = VGroup()
        for b in range(int(math.ceil(x0)), int(x1) + 1):
            p = self.page((b, y0))
            ticks.add(Line(p, p + DOWN * 0.07, stroke_color=style.ASH, stroke_width=1.0),
                      numeral(str(b), style.ASH, 0.26).next_to(p, DOWN, buff=0.12))
        for v, text in ((1.5, "1.5"), (2.5, "2.5")):
            if y0 <= v <= y1:
                p = self.page((x0, v))
                ticks.add(Line(p, p + LEFT * 0.07, stroke_color=style.ASH, stroke_width=1.0),
                          numeral(text, style.ASH, 0.26).next_to(p, LEFT, buff=0.12))
        self.x_text = serif(x_label, style.ASH, 0.3).next_to(axes[1], DOWN, buff=0.45)
        self.y_text = serif(y_label, style.ASH, 0.3).rotate(math.pi / 2).next_to(axes[0], LEFT, buff=0.5)
        frame = VGroup(axes, ticks, self.x_text, self.y_text)
        if mark:  # the map ends where the screen starts to follow him
            top, bottom = self.page((x1, y1)), self.page((x1, y0))
            self.mark = Line(bottom, top, stroke_color=style.WARM[2], stroke_width=1.6)
            self.mark_label = serif("the mark", style.WARM[2], 0.3).next_to(top, UP, buff=0.12)
            frame.add(self.mark, self.mark_label)
        return frame

    # the border
    def set_line(self, v) -> "PointMap":
        self.v = np.asarray(v, float).copy()
        self._lay()
        return self

    def border_ends(self) -> tuple:
        """Where the border crosses the map's edges; both the same point if it doesn't cross."""
        w, b = self.v[:2], self.v[2]
        (x0, x1), (y0, y1) = self.x_range, self.y_range
        found = []
        for axis, (lo, hi), (olo, ohi) in ((0, (x0, x1), (y0, y1)), (1, (y0, y1), (x0, x1))):
            other = 1 - axis
            if abs(w[other]) < 1e-12:
                continue
            for edge in (lo, hi):
                u = -(b + w[axis] * edge) / w[other]
                if olo - 1e-9 <= u <= ohi + 1e-9:
                    p = np.zeros(2)
                    p[axis], p[other] = edge, u
                    found.append(p)
        if len(found) < 2:
            return self.corners()[0], self.corners()[0]
        found.sort(key=lambda p: (p[0], p[1]))
        return found[0], found[-1]

    def _lay(self) -> None:
        box = self.corners()
        w, b = self.v[:2], self.v[2]
        top = max(abs(self.total(c)) for c in box) or 1.0
        for k, band in enumerate(self.bands):
            yes, i = k < FADE, k % FADE
            lo, hi = top * i / FADE, top * (i + 1) / FADE
            poly = clip(box, w, b, *((lo, hi) if yes else (-hi, -lo))) if np.any(w) else []
            if len(poly) < 3:
                band.set_points_as_corners([self.at] * 3).set_fill(opacity=0)
                continue
            band.set_points_as_corners([self.page(p) for p in poly] + [self.page(poly[0])])
            band.set_fill(MOVED if yes else style.COOL[0], opacity=NEAR + (SIDE - NEAR) * (i + 0.5) / FADE)
        a, z = self.border_ends()
        drawn = np.linalg.norm(a - z) > 1e-9
        a, z = self.page(a), self.page(z)
        for i, piece in enumerate(self.pieces):
            p, q = a + (z - a) * i / STEPS, a + (z - a) * (i + 1) / STEPS
            piece.put_start_and_end_on(p, q if drawn else p + RIGHT * 1e-6)
            colour = interpolate_color(ManimColor(style.WARM[2]), ManimColor(style.TEXT), (i + 0.5) / STEPS)
            piece.set_stroke(color=colour, width=1.6, opacity=1 if drawn else 0)

    def follow(self, path: np.ndarray, rate_func=lambda t: t) -> Animation:
        """Run the border through a path of (w, b) rows, one after another, evenly in time."""
        path = np.asarray(path, float)

        def update(_, alpha: float) -> None:
            u = rate_func(alpha) * (len(path) - 1)
            i = min(int(u), len(path) - 2) if len(path) > 1 else 0
            f = u - i
            self.set_line(path[i] if len(path) == 1 else path[i] * (1 - f) + path[i + 1] * f)
        return UpdateFromAlphaFunc(self.border, update)

    def jump(self, path: np.ndarray) -> Animation:
        """Step the border through a path of (w, b) rows, each held for an equal share of the time: for runs
        whose borders jump (a nudge is a jump, not a glide)."""
        path = np.asarray(path, float)

        def update(_, alpha: float) -> None:
            self.set_line(path[min(int(alpha * len(path)), len(path) - 1)])
        return UpdateFromAlphaFunc(self.border, update)

    def turn_to(self, v) -> Animation:
        """Glide the border from where it is to (w, b)."""
        end = np.asarray(v, float)

        class Glide(UpdateFromAlphaFunc):
            def begin(inner):
                inner.start = self.v.copy()
                super().begin()
        glide = Glide(self.border, lambda _, a: self.set_line(glide.start * (1 - a) + end * a))
        return glide

    # the dots
    def radius(self) -> np.ndarray:
        n = self.moved + self.still
        top = max(int(n.max()), 2)
        return SMALLEST + (LARGEST - SMALLEST) * np.log(n) / math.log(top)

    def redraw(self) -> None:
        self._lay()
        self.redraw_dots()

    def redraw_dots(self, force: bool = False) -> None:
        key = (self.level.tobytes(), self.ring.tobytes())
        if key == self._shown and not force:
            return
        self._shown = key
        px = config.pixel_width / config.frame_width  # pixels per unit
        w, h = int(round(self.size[0] * px)) + 16, int(round(self.size[1] * px)) + 16
        canvas = np.zeros((h, w, 4))
        warm, cool, ring = rgb(MOVED) / 255, rgb(STILL) / 255, rgb(style.TEXT) / 255
        n = self.moved + self.still
        light = DIM + (1 - DIM) * np.log(n) / math.log(max(int(n.max()), 2))
        origin = self.page(self.corners()[0])
        for p, r, m, k, lv, rl, on in zip(self.where, self.radius(), self.moved, n, self.level, light, self.ring):
            if lv <= 0.003:
                continue
            q = (self.page(p) - origin) * px
            cx, cy = q[0] + 8, h - 8 - q[1]
            sprite = dot_sprite(r * px, m / k, warm, cool, lv * rl)
            if on:
                rr = r * px + 2.5 * px / 108  # a hairline ring just outside the dot
                s = int(math.ceil(rr + 2))
                yy, xx = np.mgrid[-s:s + 1, -s:s + 1].astype(float)
                a = np.clip(1 - np.abs(np.hypot(xx, yy) - rr) / max(px / 108 * 1.3, 0.7), 0, 1) * lv
                self._stamp(canvas, np.dstack([np.broadcast_to(ring, a.shape + (3,)), a]), cx, cy)
            self._stamp(canvas, sprite, cx, cy)
        image = np.dstack([canvas[..., :3] * 255, canvas[..., 3] * 255]).round().astype(np.uint8)
        fresh = ImageMobject(image).set(width=w / px)
        fresh.move_to(self.page(((self.x_range[0] + self.x_range[1]) / 2, (self.y_range[0] + self.y_range[1]) / 2)))
        self.dots.pixel_array = fresh.pixel_array
        self.dots.orig_alpha_pixel_array = fresh.pixel_array[:, :, 3].copy()
        self.dots.points = fresh.points

    @staticmethod
    def _stamp(canvas: np.ndarray, sprite: np.ndarray, cx: float, cy: float) -> None:
        """Lay a sprite over the canvas, centred at (cx, cy) in pixels, alpha over."""
        n = sprite.shape[0] // 2
        x, y = int(round(cx)) - n, int(round(cy)) - n
        h, w = canvas.shape[:2]
        x0, y0, x1, y1 = max(x, 0), max(y, 0), min(x + sprite.shape[1], w), min(y + sprite.shape[0], h)
        if x0 >= x1 or y0 >= y1:
            return
        s = sprite[y0 - y:y1 - y, x0 - x:x1 - x]
        c = canvas[y0:y1, x0:x1]
        a = s[..., 3:4]
        out_a = a + c[..., 3:4] * (1 - a)
        c[..., :3] = np.where(out_a > 0, (s[..., :3] * a + c[..., :3] * c[..., 3:4] * (1 - a)) / np.maximum(out_a, 1e-9), 0)
        c[..., 3:4] = out_a

    def fade_points(self, which: np.ndarray, to: float) -> Animation:
        """Fade the chosen points to a brightness."""
        which = np.asarray(which, bool)

        class Fade(UpdateFromAlphaFunc):
            def begin(inner):
                inner.start = self.level.copy()
                super().begin()
        fade = Fade(self.dots, lambda _, a: self.level.__setitem__(which, fade.start[which] * (1 - a) + to * a))
        return fade
