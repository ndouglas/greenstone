"""The sheet of borders, for chapters 4 and 5: exp002's two maps of every border (mistakes, a staircase; the
total squared miss at each border's best size, a bowl), and how a spot on the sheet is a ruler on Mario's map."""

import numpy as np
from manim import ImageMobject, UpdateFromAlphaFunc, config, smooth

from series.lab import rules
from series.lab.exp002 import MAPS
from series.sheet import Camera, Sheet
from series.terrain import Ground, View, render

from beats.common import FILM

with np.load(MAPS) as _maps:
    ANGLES, OFFSETS = _maps["angles"], _maps["offsets"]
    MISTAKES, SQUARED, MIDDLE = _maps["mistakes"], _maps["squared"], _maps["middle"]

SIZE = (7.4, 4.6)  # the sheet's picture, in Manim units
TURN = 2 * np.pi


def border(angle: float, offset: float) -> np.ndarray:
    """The (w, b) of the ruler facing `angle`, slid `offset` from the middle of Mario's map."""
    w = np.array([np.cos(angle), np.sin(angle)])
    return np.r_[w, offset - w @ MIDDLE]


def spot(v) -> tuple[float, float]:
    """Where a border (w, b) sits on the sheet, as shares across (its angle) and up (its offset)."""
    w, b = np.asarray(v[:2], float), float(v[2])
    size = np.linalg.norm(w)
    angle = np.arctan2(w[1], w[0])
    angle = ANGLES[0] + (angle - ANGLES[0]) % TURN
    offset = (b + w @ MIDDLE) / size
    return (angle - ANGLES[0]) / TURN, (offset - OFFSETS[0]) / (OFFSETS[-1] - OFFSETS[0])


def at_share(across: float, up: float) -> tuple[float, float]:
    """The angle and offset at shares across and up the sheet."""
    return ANGLES[0] + (across % 1.0) * TURN, OFFSETS[0] + up * (OFFSETS[-1] - OFFSETS[0])


def lowest(height: np.ndarray) -> tuple[float, float]:
    i, j = np.unravel_index(np.argmin(height), height.shape)
    return (i + 0.5) / len(ANGLES), (j + 0.5) / len(OFFSETS)


class SheetPicture(ImageMobject):
    """A sheet drawn as an image, redrawn when its lift, flatness or glow change."""

    def __init__(self, height: np.ndarray, smooth: bool, camera: Camera | None = None, size=SIZE, lift: float = 1.0,
                 flat: float = 0.0, **sheet_kwargs):
        self.sheet = Sheet(height, **sheet_kwargs)
        self.camera = camera or Camera(pitch=0.9)
        self.size_, self.smooth, self.lift, self.flat, self.glow = size, smooth, lift, flat, None
        super().__init__(self.picture())
        self.set(width=size[0])

    def picture(self) -> np.ndarray:
        return self.sheet.render(self.camera, self.lift, self.size_, flat=self.flat, glow=self.glow, smooth=self.smooth)

    def redraw(self) -> None:
        centre, width = self.get_center(), self.width
        self.pixel_array = self.picture()
        self.set(width=width).move_to(centre)

    def point(self, across: float, up: float, height: float | None = None) -> np.ndarray:
        """Where a spot on the sheet shows on the page."""
        return self.sheet.at(across, up, self.lift, self.camera, height) * (self.width / self.size_[0]) + self.get_center()

    def rise(self, start: float, end: float, flat_from: float | None = None, flat_to: float | None = None):
        def update(m, a: float) -> None:
            m.lift = start + (end - start) * a
            if flat_from is not None:
                m.flat = flat_from + (flat_to - flat_from) * a
            m.redraw()
        return UpdateFromAlphaFunc(self, update)


# The new ground (series.terrain): true heights, one scale for the chapter (all of the moments wrong stands
# TALL high), and the real ledges, computed exactly wherever we look closely.
MOMENTS = len(FILM["y"])
TOP = 1.0  # shares of the moments wrong: the staircase's scale, and the bowl's (its shelf is at 1)
_X = FILM["x"] - MIDDLE


def across_of(angle) -> np.ndarray:
    return ((np.asarray(angle) - ANGLES[0]) % TURN) / TURN


def up_of(offset) -> np.ndarray:
    return (np.asarray(offset) - OFFSETS[0]) / (OFFSETS[-1] - OFFSETS[0])


def _extent(shares: np.ndarray) -> tuple[float, float]:
    """The patch evenly spaced samples cover, each sample in the middle of its cell."""
    d = (shares[-1] - shares[0]) / (len(shares) - 1)
    return float(shares[0] - d / 2), float(shares[-1] + d / 2)


def exact(angles, offsets) -> np.ndarray:
    """The share of moments each border gets wrong, exactly (any spacing)."""
    return rules.mistakes_at(_X, FILM["y"], np.asarray(angles), np.asarray(offsets))


def _lowest_ledge() -> tuple[float, float]:
    i, j = np.unravel_index(np.argmin(MISTAKES), MISTAKES.shape)
    fa = ANGLES[i] + np.radians(np.linspace(-3, 3, 601))
    fo = OFFSETS[j] + np.linspace(-0.1, 0.1, 801)
    k, l = np.unravel_index(np.argmin(exact(fa, fo)), (601, 801))
    return float(fa[k]), float(fo[l])


LEDGE = _lowest_ledge()  # a border on the lowest ledge: 835 mistakes (the sheet's samples miss it, at 838)
LEDGE_MISTAKES = int(round(exact([LEDGE[0]], [LEDGE[1]])[0, 0] * MOMENTS))


def staircase_ground(step: int = 2, **kwargs) -> Ground:
    """The whole staircase, from the sheet's samples (every `step`th)."""
    a, o = ANGLES[::step], OFFSETS[::step]
    return Ground(MISTAKES[::step, ::step], TOP, across=(-(a[1] - a[0]) / TURN / 2, 1 - (a[1] - a[0]) / TURN / 2),
                  up=_extent(up_of(o)), drop=50 / MOMENTS, **kwargs)


def bowl_ground(step: int = 2, **kwargs) -> Ground:
    a, o = ANGLES[::step], OFFSETS[::step]
    return Ground(SQUARED[::step, ::step], TOP, smooth=True, contour=0.05,
                  across=(-(a[1] - a[0]) / TURN / 2, 1 - (a[1] - a[0]) / TURN / 2), up=_extent(up_of(o)), **kwargs)


def ledge_ground(half_angle: float = np.radians(2), half_offset: float = 0.05, cells: int = 400,
                 stretch: float = 2.0, centre=None) -> Ground:
    """A close patch around the lowest ledge, computed exactly, heights stretched above its lowest by
    `stretch` (said on screen), coloured from its lowest to 125 moments higher."""
    ca, co = centre or LEDGE
    fa = ca + np.linspace(-half_angle, half_angle, cells)
    fo = co + np.linspace(-half_offset, half_offset, cells)
    v = exact(fa, fo)
    low = float(v.min())
    g = Ground(v, TOP, across=_extent(across_of(fa)), up=_extent(up_of(fo)), drop=4.5 / MOMENTS,
               contour=25 / MOMENTS, base=low - 6 / MOMENTS, stretch=stretch, colours=(low, low + 125 / MOMENTS))
    g.glow = np.isclose(v, low)
    g.angles, g.offsets = fa, fo
    return g


INK = "#120A08"  # what marks on the ground sit on: dark, so they read on gold and plum alike


def tag(text: str, colour: str, size: float = 0.28):
    """Words on the ground: on a dark rounded pill."""
    from manim import RoundedRectangle, VGroup
    from series.ember import serif
    words = serif(text, colour, size)
    pill = RoundedRectangle(corner_radius=0.08, width=words.width + 0.24, height=words.height + 0.16, stroke_width=0,
                            fill_color=INK, fill_opacity=0.82).move_to(words)
    return VGroup(pill, words)


def pin(colour: str, radius: float = 0.085):
    """A spot on the ground: a coloured dot in a dark ring."""
    from manim import Dot, VGroup
    return VGroup(Dot(radius=radius + 0.035, color=INK), Dot(radius=radius, color=colour))


def height_key(low: float, high: float, ticks: list[tuple[float, str]], title: str, height: float = 3.2):
    """The ramp as a key: gold at `low`, plum ash at `high`, real values marked."""
    from manim import LEFT, RIGHT, UP, Group, Line, VGroup
    from series import style
    from series.ember import serif
    from series.terrain import ramp
    n = 256
    bar = np.repeat(ramp(np.linspace(1, 0, n))[:, None, :], 8, 1)
    image = ImageMobject(np.dstack([bar, np.full((n, 8), 255)]).astype(np.uint8))
    image.stretch_to_fit_height(height).stretch_to_fit_width(0.22)
    marks = VGroup()
    for value, words in ticks:
        y = image.get_bottom()[1] + (value - low) / (high - low) * height
        tick = Line([image.get_right()[0], y, 0], [image.get_right()[0] + 0.12, y, 0], stroke_color=style.ASH, stroke_width=1.5)
        marks.add(tick, serif(words, style.ASH, 0.24).next_to(tick, RIGHT, buff=0.08))
    name = serif(title, style.ASH, 0.26).next_to(image, UP, buff=0.18).align_to(image, LEFT)
    from manim import RoundedRectangle
    body = Group(image, marks, name)
    panel = RoundedRectangle(corner_radius=0.12, width=body.width + 0.4, height=body.height + 0.4, stroke_width=0,
                             fill_color=INK, fill_opacity=0.8).move_to(body)
    return Group(panel, body)


class GroundPicture(ImageMobject):
    """Grounds seen from a view, drawn as an image `size` Manim units; redrawn as the view, lift or flatness
    change. `point` puts page marks where the ground is drawn."""

    def __init__(self, grounds: list[Ground], view: View, size=SIZE, lift: float = 1.0, flat: float = 0.0,
                 floor: bool = True, fog: tuple[float, float] | None = None):
        self.grounds, self.view, self.size_ = list(grounds), view, size
        self.lift, self.flat, self.floor, self.fog = lift, flat, floor, fog
        super().__init__(self.picture())
        self.set(width=size[0])

    def pixels(self) -> tuple[int, int]:
        px = config.pixel_width / config.frame_width
        return int(round(self.size_[0] * px)), int(round(self.size_[1] * px))

    def picture(self) -> np.ndarray:
        d = self.view.distance  # fog in step with the camera: far ground fades, however close it comes
        return render(self.grounds, self.view, self.pixels(), self.lift, self.flat, self.floor,
                      self.fog or (d * 1.3, d * 2.8))

    def redraw(self) -> None:
        centre, width = self.get_center(), self.width
        self.pixel_array = self.picture()
        self.orig_alpha_pixel_array = self.pixel_array[:, :, 3].copy()
        self.set(width=width).move_to(centre)

    def point(self, across, up, value=None, ground: int = 0) -> np.ndarray:
        """Where a spot on the sheet shows on the page: on that ground, or at a true value."""
        g = self.grounds[ground]
        w = g.world(across, up, value, self.lift)
        return self.view.project(w, self.size_) * (self.width / self.size_[0]) + self.get_center()

    def fly(self, to: View | None = None, lift: tuple[float, float] | None = None,
            flat: tuple[float, float] | None = None, stretch: tuple[float, float] | None = None,
            colours: tuple[tuple, tuple] | None = None, place: tuple | None = None, ground: int = -1,
            run_time: float = 2.0, rate_func=smooth) -> UpdateFromAlphaFunc:
        """Move the camera to another view, raising, unflattening, or stretching and recolouring a ground (the
        last, unless `ground` says) on the way; `place` (centre, size) moves and resizes the picture."""
        began: dict = {}

        def update(m, a: float) -> None:
            if not began:  # where it is when the move begins, not when it was planned
                began.update(view=m.view, here=m.get_center(), size=m.size_)
            start, here, size = began["view"], began["here"], began["size"]
            if to is not None:
                m.view = start.between(to, a)
            if lift is not None:
                m.lift = lift[0] + (lift[1] - lift[0]) * a
            if flat is not None:
                m.flat = flat[0] + (flat[1] - flat[0]) * a
            if stretch is not None:
                m.grounds[ground].stretch = stretch[0] + (stretch[1] - stretch[0]) * a
            if colours is not None:
                m.grounds[ground].colours = tuple(p + (q - p) * a for p, q in zip(*colours))
            if place is not None:
                m.size_ = tuple(p + (q - p) * a for p, q in zip(size, place[1]))
                m.pixel_array = m.picture()
                m.orig_alpha_pixel_array = m.pixel_array[:, :, 3].copy()
                m.set(width=m.size_[0]).move_to(here + (np.asarray(place[0]) - here) * a)
                return
            m.redraw()
        return UpdateFromAlphaFunc(self, update, run_time=run_time, rate_func=rate_func)


BOX = [np.array(c, float) for c in ((0, 0), (5, 0), (5, 2.6), (0, 2.6))]  # Mario's map: blocks across, speed up


class Tiles:
    """The sheet of borders as what it is: a grid of tiny Mario maps laid on a flat sheet picture, each map one
    real border (the ruler at its column's turn and its row's slide), its yes side washed warm and its no side
    cool. Each tile is a VGroup in `tiles[column][row]`, drawn exactly where the sheet shows that cell."""

    def __init__(self, sheet: "GroundPicture", columns: int = 12, rows: int = 7, gap: float = 0.12):
        from manim import Polygon, VGroup
        from series import style
        from series.bordermap import clip
        self.sheet, self.columns, self.rows = sheet, columns, rows
        self.group = VGroup()
        self.tiles = []
        for i in range(columns):
            column = []
            for j in range(rows):
                a0, a1 = (i + gap / 2) / columns, (i + 1 - gap / 2) / columns
                u0, u1 = (j + gap / 2) / rows, (j + 1 - gap / 2) / rows
                v = border(*at_share((i + 0.5) / columns, (j + 0.5) / rows))

                def page(p, a0=a0, a1=a1, u0=u0, u1=u1):
                    return sheet.point(a0 + (a1 - a0) * p[0] / 5, u0 + (u1 - u0) * p[1] / 2.6, 0.0)
                tile = VGroup()
                for lo, hi, colour in ((0, 1e9, style.WARM[1]), (-1e9, 0, style.COOL[1])):
                    side = clip(BOX, v[:2], v[2], lo, hi)
                    if len(side) >= 3:
                        tile.add(Polygon(*[page(p) for p in side], stroke_width=0, fill_color=colour, fill_opacity=0.5))
                tile.frame = Polygon(*[page(p) for p in BOX], stroke_color=style.ASH, stroke_width=0.8, fill_opacity=0)
                tile.add(tile.frame)
                along = np.array([-v[1], v[0]])
                ends = sorted(clip(BOX, v[:2], v[2], -1e-9, 1e-9), key=lambda p: p @ along)
                if len(ends) >= 2:
                    from manim import Line
                    tile.add(Line(page(ends[0]), page(ends[-1]), stroke_color=style.TEXT, stroke_width=1.4))
                tile.cell = (i, j)
                column.append(tile)
                self.group.add(tile)
            self.tiles.append(column)

    def at(self, across: float, up: float):
        """The tile holding a spot on the sheet."""
        i = int(np.floor((across % 1.0) * self.columns)) % self.columns
        j = min(max(int(np.floor(up * self.rows)), 0), self.rows - 1)
        return self.tiles[i][j]
