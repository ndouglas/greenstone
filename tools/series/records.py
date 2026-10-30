"""The records: tapes (three recordings as hairline strips, one sliding apart, its dying stretches greyed) and
the score bars (a baseline in ash beside the neuron's result in amber)."""

import numpy as np
from manim import DOWN, LEFT, RIGHT, UP, Animation, Line, Rectangle, UpdateFromAlphaFunc, VGroup

from series import style
from series.ember import serif

TAPE_W, TAPE_H, TAPE_GAP = 8.0, 0.55, 0.45
TICKS = 40
APART = np.array([0.9, -0.55, 0.0])  # where a tape slides to
GREY = 0.35

BAR_W, BAR_GAP, BAR_HEIGHT = 1.3, 0.8, 4.0


class Tape(VGroup):
    """One strip of recording, its italic label to its left."""

    def __init__(self, label: str):
        super().__init__()
        self.strip = Rectangle(width=TAPE_W, height=TAPE_H, stroke_color=style.ASH, stroke_width=1.2, fill_opacity=0)
        step = TAPE_W / TICKS
        self.ticks = VGroup(*[Line(self.strip.get_left() + RIGHT * step * k + DOWN * TAPE_H / 2,
                                   self.strip.get_left() + RIGHT * step * k + DOWN * TAPE_H / 2 + UP * TAPE_H * 0.25,
                                   stroke_color=style.ASH, stroke_width=0.5, stroke_opacity=0.5) for k in range(1, TICKS)])
        self.label = serif(label, style.TEXT, 0.45).next_to(self.strip, LEFT, buff=0.4)
        self.greys = VGroup()
        self.add(self.strip, self.ticks, self.greys, self.label)


class Tapes(VGroup):
    """n labelled strips, stacked top to bottom."""

    def __init__(self, n: int, labels: list[str]):
        super().__init__()
        if len(labels) != n:
            raise ValueError(f"{n} tapes need {n} labels, not {len(labels)}")
        self.tapes = [Tape(label) for label in labels]
        for i, tape in enumerate(self.tapes):
            tape.shift(DOWN * (i - (n - 1) / 2) * (TAPE_H + TAPE_GAP))
        self.add(*self.tapes)

    def apart(self, i: int) -> Animation:
        """Tape i slides forward and away from the rest."""
        return self.tapes[i].animate.shift(APART).build()

    def grey(self, i: int, spans: list[tuple[float, float]]) -> Animation:
        """The given stretches (fractions of tape i's length) go grey: the player was dying there."""
        strip = self.tapes[i].strip
        shades = [Rectangle(width=(b - a) * strip.width, height=strip.height, stroke_width=0,
                            fill_color=style.ASH, fill_opacity=0)
                  .move_to([strip.get_left()[0] + (a + b) / 2 * strip.width, strip.get_center()[1], 0])
                  for a, b in spans]
        self.tapes[i].greys.add(*shades)

        def update(_, alpha: float) -> None:
            for shade in shades:
                shade.set_fill(opacity=GREY * alpha)
        return UpdateFromAlphaFunc(self.tapes[i].greys, update)


class Bar(VGroup):
    """One bar: a hairline-outlined box standing on the line, its number above and its name below."""

    def __init__(self, value: float, label: str, colour: str, x: float, floor: float, unit: float, faint: bool, mark: str):
        super().__init__()
        self.value, self.colour, self.faint = value, colour, faint
        self.x, self.floor, self.rise = x, floor, value * unit
        self.strength = 0.3 if faint else 1.0
        self.box = Rectangle(width=BAR_W, height=max(self.rise, 1e-3), stroke_color=colour, stroke_width=1.2,
                             fill_color=colour, fill_opacity=0.0)
        self.number = serif(mark, colour, 0.5, italic=False)
        self.label = serif(label, style.TEXT if not faint else style.ASH, 0.38).move_to([x, floor - 0.4, 0])
        self.add(self.box, self.number, self.label)
        self.grow(0.0)

    def grow(self, level: float) -> None:
        h = max(self.rise * level, 1e-3)
        a, b = self.x - BAR_W / 2, self.x + BAR_W / 2
        self.box.set_points_as_corners([[a, self.floor, 0], [a, self.floor + h, 0], [b, self.floor + h, 0],
                                        [b, self.floor, 0], [a, self.floor, 0]])
        self.box.set_stroke(opacity=self.strength * min(1.0, level * 8)).set_fill(opacity=0.12 * self.strength * level)
        self.number.move_to([self.x, self.floor + h + 0.4, 0]).set_opacity(self.strength * level)
        self.label.set_opacity(self.strength * min(1.0, level * 4))


class ScoreBars(VGroup):
    """Bars standing on one line, heights proportional to their value; the first, the baseline, is ash."""

    def __init__(self, baseline: float, baseline_label: str = "always on the ground", top: float = 100.0,
                 slots: int = 3, unit: str = "%", at: np.ndarray = (0, -0.4, 0)):
        super().__init__()
        self.strength, self.slots, self.unit_mark, self.at = top, slots, unit, np.asarray(at, float)
        self.bars: list[Bar] = []
        self._place(baseline, baseline_label, style.ASH, False)
        self.bars[0].grow(1.0)

    def _place(self, value: float, label: str, colour: str, faint: bool) -> Bar:
        k = len(self.bars)
        if k >= self.slots:
            raise ValueError(f"the bars have {self.slots} places and all are taken")
        x = self.at[0] + (k - (self.slots - 1) / 2) * (BAR_W + BAR_GAP)
        bar = Bar(value, label, colour, x, self.at[1], BAR_HEIGHT / self.strength, faint, style.number(value) + self.unit_mark)
        self.bars.append(bar)
        super().add(bar)
        return bar

    def bar(self, value: float, label: str, faint: bool = False) -> Animation:
        """A bar rises for `value` (the neuron's result is warm; faint is the score still to beat)."""
        bar = self._place(float(value), label, style.WARM[0], faint)
        return UpdateFromAlphaFunc(bar, lambda m, alpha: m.grow(alpha))
