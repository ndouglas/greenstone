"""How big the search gets: the borders the same search would try with more numbers, as a heap of sand (one
1 mm grain a border) beside things whose sizes are known, and as the time it would take on this computer.

Every count is the search's own (NUMBERS: its borders and seconds, and the 180 settings each extra dial adds);
the heaps are cones at sand's angle of repose, so a heap's volume is its grains' and its height is real."""

import math

import numpy as np
from manim import DOWN, LEFT, RIGHT, UP, Line, Polygon, VGroup, VMobject

from series import style
from series.ember import serif

from beats.common import NUMBERS

BORDERS, SECONDS, DIAL = NUMBERS["search_borders"], NUMBERS["search_seconds"], NUMBERS["extra_dial_settings"]
GRAIN = 1e-9  # one grain of sand, a millimetre a side, in cubic metres
REPOSE = math.radians(30)  # the steepest a heap of dry sand stands
YEAR = 365.25 * 24 * 3600

# Known sizes, in metres (public figures: see citations "scale-references").
EVEREST = 8_849
EARTH = 12_742e3  # across
SUN = 1_391_400e3  # across
UNIVERSE = 8.8e26  # the observable universe, across
AGE = 13.8e9  # the universe's age, in years


def borders(n: float) -> float:
    """The borders the same search tries with n numbers read."""
    return BORDERS * DIAL ** (n - 2)


def seconds(n: float) -> float:
    return SECONDS * borders(n) / BORDERS


def heap(n: float) -> tuple[float, float]:
    """The heap of one grain per border: its radius and height, in metres."""
    volume = borders(n) * GRAIN
    r = (3 * volume / (math.pi * math.tan(REPOSE))) ** (1 / 3)
    return r, r * math.tan(REPOSE)


def power(x: float) -> str:
    """A big number as a × 10 to a power, with the power raised."""
    e = int(math.floor(math.log10(x)))
    return f"{x / 10 ** e:.2g} × 10{str(e).translate(str.maketrans('0123456789', '⁰¹²³⁴⁵⁶⁷⁸⁹'))}"


def count(x: float) -> str:
    """A count as words a viewer reads at a glance."""
    if x < 1e6:
        return f"{x:,.0f}"
    for size, word in ((1e12, "trillion"), (1e9, "billion"), (1e6, "million")):
        if size <= x < size * 1000:
            return f"{x / size:.3g} {word}"
    return power(x)


def span(s: float) -> str:
    """A length of time in the largest unit that fits."""
    for size, one, many in ((YEAR * 1e9, "billion years", "billion years"), (YEAR * 1e6, "million years", "million years"),
                            (YEAR, "year", "years"), (86400, "day", "days"), (3600, "hour", "hours"),
                            (60, "minute", "minutes"), (1, "second", "seconds")):
        if s >= size:
            v = s / size
            return f"{v:,.2g} {one if round(v, 1) == 1 else many}" if v < 100 else f"{v:,.0f} {many}"
    return f"{s:.1f} seconds"


# The things the heap is set beside, as (name, how many numbers until the heap passes it, outline in metres).
def _bucket():
    return [(-0.14, 0), (0.14, 0), (0.17, 0.3), (-0.17, 0.3)]


def _house():
    return [(-4, 0), (4, 0), (4, 5.5), (0, 9), (-4, 5.5)]


def _pyramid():
    return [(-115, 0), (115, 0), (0, 139)]


def _mountain():
    return [(-11_000, 0), (-4_000, 5_200), (-1_200, 7_400), (0, EVEREST), (2_400, 6_500), (6_500, 3_200), (12_000, 0)]


def _ball(across: float):
    return [(across / 2 * math.cos(t), across / 2 + across / 2 * math.sin(t)) for t in np.linspace(0, 2 * math.pi, 64)]


THINGS = [("a bucket", _bucket()), ("a house", _house()), ("the Great Pyramid", _pyramid()), ("Mount Everest", _mountain()),
          ("the Earth", _ball(EARTH)), ("the Sun", _ball(SUN)), ("the observable universe", _ball(UNIVERSE))]


class Heap(VGroup):
    """The heap for `n` numbers read (a float, so it grows smoothly), zoomed so it stays the same size on screen,
    beside whichever known thing is nearest its size. Redraw with `show(n)`."""

    def __init__(self, at: np.ndarray, width: float = 7.0, height: float = 4.6):
        super().__init__()
        self.at, self.w, self.h = np.asarray(at, float), width, height
        self.ground = Line(self.at + LEFT * width / 2, self.at + RIGHT * width / 2, stroke_color=style.ASH, stroke_width=1.2)
        self.pile = VMobject()
        self.shapes = VGroup(*[VMobject() for _ in THINGS])
        self.names = VGroup(*[serif(name, style.TEXT, 0.28) for name, _ in THINGS])
        self.ruler, self.ruler_words = VMobject(), VGroup()
        self.add(self.ground, self.shapes, self.pile, self.names, self.ruler, self.ruler_words)
        self.show(2.0)

    def show(self, n: float) -> None:
        r, h = heap(n)
        scale = self.w * 0.16 / r  # Manim units per metre: the heap about a third of the width across
        base = self.at + LEFT * self.w * 0.2
        self.pile.become(Polygon(base + LEFT * r * scale, base + RIGHT * r * scale, base + UP * h * scale,
                                 stroke_width=0, fill_color=style.WARM[2], fill_opacity=0.9))
        beside = base + RIGHT * (r * scale + self.w * 0.2)
        for (name, outline), shape, words in zip(THINGS, self.shapes, self.names):
            pts = np.array(outline) * scale
            size = max(np.ptp(pts[:, 0]), np.ptp(pts[:, 1]))
            # shown while it's within about eight times the heap's height, either way, brightest at the same size
            near = max(0.0, 1 - abs(math.log(size / (h * scale))) / math.log(8))
            if near == 0:  # not shown: left alone, never drawn at sizes far off the screen
                shape.set_stroke(opacity=0)
                words.set_opacity(0)
                continue
            shape.become(Polygon(*[beside + np.array([x - pts[:, 0].min(), y, 0]) for x, y in pts], stroke_color=style.TEXT,
                                 stroke_width=1.8, stroke_opacity=near, fill_opacity=0))
            words.set_opacity(near).next_to(shape, UP, buff=0.12)
        # a scale bar under the ground: the round length (a power of ten metres) between 0.6 and 6 units long
        k = math.ceil(math.log10(0.6 / scale))
        unit = 1.0
        if 10.0 ** k >= 9.4607e15:  # past a light year, a round number of light years
            unit = 9.4607e15
            k = math.ceil(math.log10(0.6 / scale / unit))
        length = 10.0 ** k * unit * scale
        start = self.at + LEFT * self.w * 0.45 + DOWN * 0.35
        self.ruler.become(Line(start, start + RIGHT * length, stroke_color=style.ASH, stroke_width=2.5))
        text = metres(10.0 ** k * unit)
        if getattr(self.ruler_words, "text_", None) != text:
            self.ruler_words.become(serif(text, style.ASH, 0.24))
            self.ruler_words.text_ = text
        self.ruler_words.next_to(self.ruler, DOWN, buff=0.08).align_to(self.ruler, LEFT)


def metres(m: float) -> str:
    """A power-of-ten length, as a reader says it."""
    LIGHT_YEAR = 9.4607e15
    if m >= LIGHT_YEAR:
        return f"{count(m / LIGHT_YEAR)} light years" if m >= LIGHT_YEAR * 1e3 else f"{m / LIGHT_YEAR:,.0f} light years"
    if m >= 1e3:
        return f"{count(m / 1e3)} km"
    if m >= 1:
        return f"{m:,.0f} m"
    if m >= 0.01:
        return f"{m * 100:,.0f} cm"
    return f"{m * 1000:,.0f} mm"
