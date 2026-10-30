"""The balance scale: a neuron's answer weighed instead of summed. Each term is a block of its size; plus
blocks fall into the yes pan, minus blocks into the no pan, and the beam tips to the heavier. A balance says no."""

import numpy as np
from manim import DOWN, UP, Animation, Dot, Line, ManimColor, Square, UpdateFromAlphaFunc, VGroup, interpolate_color, linear, linear

from series import style
from series.ember import serif

HALF_BEAM = 3.0
PIVOT = np.array([0.0, 2.2, 0.0])  # relative to the scale's own centre
STRING = 2.2
PAN = 1.2  # half a pan's width
DROP = 3.0  # how far above its pan a block starts
TILT = 0.24  # the beam's greatest angle, radians
SOFT = 80.0  # the sum at which the beam is most of the way to it
BIGGEST = 160.0  # the value whose block is full size
STEPS = 24  # pieces of the beam
GAP = 0.06
NATURAL = 11.0  # the beam's own swing, radians per second
DAMPING = 0.4  # under 1, so it overshoots and rings down: settled in about a second
SUBSTEP = 1 / 240
IMPACT = 0.25  # radians per second of beam jolt, per unit of (mass / BIGGEST) x (impact speed in units per second)
IMPACT = 0.25  # radians per second of beam jolt, per unit of (mass / BIGGEST) x (impact speed in units per second)


class Block(VGroup):
    """One term: a hairline square with a faint fill, both in its own colour, its number roman inside."""

    def __init__(self, value: float, label: str, hue: int):
        super().__init__()
        self.value = value
        self.colour = style.temperature(value if value else -1, hue)
        self.side = 0.3 + 0.85 * (min(abs(value), BIGGEST) / BIGGEST) ** 0.5
        self.fall = 1.0  # 1 high above its pan, 0 resting on it
        self.landed = False  # its weight counts from the moment it lands
        self.box = Square(self.side, stroke_color=self.colour, stroke_width=1.2, fill_color=self.colour, fill_opacity=0.0)
        self.text = serif(label, style.TEXT, 0.4, italic=False)
        self.add(self.box, self.text)
        self.shown = 0.0

    def show(self, level: float) -> None:
        self.shown = level
        self.box.set_fill(opacity=0.12 * level)
        self.box.set_stroke(opacity=level)
        self.text.set_opacity(level)


class Balance(VGroup):
    """yes on the left pan (warm), no on the right (cool). drop() lets a block fall; verdict() reads the tilt."""

    def __init__(self, yes: str, no: str, at: np.ndarray = np.zeros(3)):
        super().__init__()
        self.words = {"yes": yes, "no": no}
        self.at = np.asarray(at, float)
        self.pans = {"yes": [], "no": []}
        self._theta = 0.0  # the beam's angle now, positive when the yes end is down
        self._omega = 0.0  # and how fast it is turning
        self._announced = 0.0
        self._said = None

        pivot = self.at + PIVOT
        self.post = VGroup(Line(pivot, pivot + DOWN * 4.0, stroke_color=style.ASH, stroke_width=1.0),
                           Line(pivot + DOWN * 4.0 + [-1.0, 0, 0], pivot + DOWN * 4.0 + [1.0, 0, 0],
                               stroke_color=style.ASH, stroke_width=1.0))
        cols = (style.WARM[1], style.COOL[0])
        self.beam = VGroup(*[Line(pivot, pivot, stroke_width=1.6, stroke_color=interpolate_color(
            ManimColor(cols[0]), ManimColor(cols[1]), (i + 0.5) / STEPS)) for i in range(STEPS)])
        self.strings = VGroup(*[Line(pivot, pivot, stroke_color=style.ASH, stroke_width=0.8) for _ in range(4)])
        self.plates = {"yes": Line(pivot, pivot, stroke_color=style.WARM[1], stroke_width=1.6),
                       "no": Line(pivot, pivot, stroke_color=style.COOL[0], stroke_width=1.6)}
        self.hub = Dot(pivot, radius=0.05, color=style.ASH)
        self.labels = {"yes": serif(yes, style.WARM[1], 0.4), "no": serif(no, style.COOL[1], 0.4)}
        self.verdict_label = serif(no, style.COOL[1], 0.55)
        self.total_label = serif("0", style.COOL[1], 0.5, italic=False)
        self.stack = VGroup()  # the blocks, in the pans
        self.add(self.post, self.beam, self.strings, *self.plates.values(), self.hub, *self.labels.values(),
                 self.stack, self.verdict_label, self.total_label)
        self._lay()
        self.add_updater(self._tick)

    def _tick(self, _, dt: float = 0.0) -> None:
        self._swing(dt)
        self._lay()

    def sum(self) -> float:
        return sum(k.value for ks in self.pans.values() for k in ks if k.landed)

    def verdict(self) -> str:
        """The side the beam tips to; balanced is no, since a neuron says yes only above zero."""
        return self.words["yes"] if self.sum() > 0 else self.words["no"]

    def angle(self) -> float:
        """The beam's angle now, positive when the yes (left) end is down."""
        return self._theta

    def rest(self) -> float:
        """The angle the beam settles at for the landed weight; it saturates for a heavy sum."""
        return TILT * float(np.tanh(self.sum() / SOFT))

    def _swing(self, dt: float) -> None:
        """A damped spring toward the resting angle, so a landing makes the beam overshoot and ring down."""
        rest = self.rest()
        left = dt
        while left > 1e-9:
            h = min(SUBSTEP, left)
            self._omega += (-NATURAL ** 2 * (self._theta - rest) - 2 * DAMPING * NATURAL * self._omega) * h
            self._theta += self._omega * h
            left -= h

    def end(self, side: str) -> np.ndarray:
        a = self.angle()
        sign = -1 if side == "yes" else 1
        return self.at + PIVOT + sign * HALF_BEAM * np.array([np.cos(a), np.sin(a), 0.0])

    def pan_centre(self, side: str) -> np.ndarray:
        return self.end(side) + DOWN * STRING

    def drop(self, value: float, label: str, hue: int = 0) -> Animation:
        """A block of |value|'s size falls into its sign's pan while the beam tips to the new sum."""
        block = Block(value, label, hue)
        side = "yes" if value > 0 else "no"
        self.pans[side].append(block)
        self.stack.add(block)
        block.fall = 1.0
        self._lay()

        def update(_, alpha: float) -> None:
            block.fall = 1 - alpha ** 2  # gravity: it speeds up all the way down (the rate is linear, no ease-out)
            block.show(min(1.0, alpha * 4))
            if alpha >= 1.0 and not block.landed:
                block.landed = True
                speed = 2 * DROP / max(animation.run_time, 1e-3)  # the speed it hits the pan at, units per second
                self._omega += (1.0 if value > 0 else -1.0) * IMPACT * min(abs(value), BIGGEST) / BIGGEST * speed
            elif alpha < 1.0:
                block.landed = False
            self._lay()
        animation = UpdateFromAlphaFunc(self, update, rate_func=linear)
        return animation

    def announce(self) -> Animation:
        """Say the verdict beneath the scale, with the total."""
        def update(_, alpha: float) -> None:
            self._announced = alpha
            self._lay()
        return UpdateFromAlphaFunc(self, update)

    def _lay(self) -> None:
        pivot = self.at + PIVOT
        yes_end, no_end = self.end("yes"), self.end("no")
        pts = [yes_end + (no_end - yes_end) * i / STEPS for i in range(STEPS + 1)]
        for seg, a, b in zip(self.beam, pts, pts[1:]):
            seg.put_start_and_end_on(a, b)
        self.hub.move_to(pivot)
        for side, i in (("yes", 0), ("no", 2)):
            top, pan = self.end(side), self.pan_centre(side)
            left, right = pan + [-PAN, 0, 0], pan + [PAN, 0, 0]
            self.strings[i].put_start_and_end_on(top, left)
            self.strings[i + 1].put_start_and_end_on(top, right)
            self.plates[side].put_start_and_end_on(left, right)
            y = pan[1] + GAP
            for k in self.pans[side]:
                k.move_to([pan[0], y + k.side / 2 + k.fall * DROP, 0])
                y += k.side + GAP
            self.labels[side].move_to(pan + DOWN * 0.4)
        self._speak(pivot)

    def _speak(self, pivot: np.ndarray) -> None:
        word = self.verdict()
        colour = style.WARM[1] if word == self.words["yes"] and self.sum() > 0 else style.COOL[1]
        key = (word, colour)
        if key != self._said:
            self._said = key
            self.verdict_label.become(serif(word, colour, 0.55))
        self.total_label.become(serif(style.number(self.sum()), colour, 0.5, italic=False))
        base = self.at + PIVOT + DOWN * 4.9
        self.verdict_label.move_to(base)
        self.total_label.next_to(self.verdict_label, DOWN, buff=0.2)
        self.verdict_label.set_opacity(self._announced)
        self.total_label.set_opacity(self._announced)
