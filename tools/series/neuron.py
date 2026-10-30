"""The neuron, drawn from what it measures: a hairline ring with its running total inside and its glow
behind; a path into it from each thing it reads (a mark on the game, a square of memory, or a bare number);
the bias as a path from its place, or a bare number; and the answer, a dot in a ring the output runs into,
with its verdict beneath."""

import math
from dataclasses import dataclass
from typing import Callable

import numpy as np
from manim import (DOWN, LEFT, RIGHT, UP, Animation, AnimationGroup, Circle, Group, Line, Mobject,
                   ValueTracker, VGroup, VMobject)

from series import style
from series.ember import fade_image, glow, serif, swap

RADIUS = 0.55
STROKE = 1.4
PATH_STROKE = 1.4
GLOW_RADIUS = 1.25
EASE = 9.0  # the glow closes this share of its distance to the total per second (≈1/9 s to settle)
TOTAL_SIZE = 0.46
VERDICT_SIZE = 0.5
ANSWER_GAP = 2.45  # from the ring's right to the answer's centre
SOCKET = 0.2
DOT = 0.12
QUIET = "#3A3034"  # an output line before its colour arrives
COOLING = "#7A6A58"  # the midpoint of a warm path cooling, so it never passes through green
END_HANDLE = 1.5  # how far out from the ring a path turns in
LABEL_AT = 0.55  # how far along a path's turn its label sits
LABEL_GAP = 0.12
BARE_LEFT = 2.4  # a bare number's distance left of the ring's centre
BARE_STEP = 0.6
# Paths run straight right from their sources until level with the rightmost one, then turn into the ring's
# lower-left quarter, from its left (the highest source) to its bottom (the lowest), so they never cross. As
# one source passes another the two swap places, smoothly but fast (over about SOFT); a later path counts as
# TIE lower, so a tie (Mario standing on the setting) keeps the inputs above the bias. Only while a source
# is within TIE below a later one do their paths cross, by a hair.
TIE = 0.03
SOFT = 0.008


@dataclass
class Input:
    """One thing the neuron reads: its value now, its weight, where its path starts (None for a bare number),
    and its name (shown beside a bare number)."""
    value: Callable[[], float]
    weight: float
    source: Callable[[], np.ndarray] | None
    label: str


def figure(value: float) -> str:
    """A number as the neuron writes it: whole numbers whole, true minus."""
    return style.number(value, 0 if float(value).is_integer() else 2)


def path_colours(start: str, contribution: float) -> list[str]:
    """Warm as it leaves its source, taking the temperature of what it contributes as it arrives."""
    if contribution < 0:
        return [start, COOLING, style.COOL[0]]
    return [start, style.WARM[1] if contribution > 0 else style.ASH]


def soft_ranks(keys: list[float]) -> list[float]:
    """How many keys lie above each one, counted smoothly: continuous in the keys, and in their order."""
    def above(d: float) -> float:
        return 1 / (1 + math.exp(-max(min(d / SOFT, 60), -60)))
    return [sum(above(b - a) for j, b in enumerate(keys) if j != i) for i, a in enumerate(keys)]


class Neuron(Group):
    """inputs weighted and summed with the bias; yes when the total is above zero. Everything is drawn from
    the start; arrive() hides it and returns the animations that bring it in."""

    def __init__(self, inputs: list[Input], bias: float, bias_source: Callable[[], np.ndarray] | None,
                 centre: np.ndarray, yes: str, no: str, scale: float = style.LIGHT_SCALE,
                 bias_name: str | None = None):
        super().__init__()
        self.inputs, self.bias, self.bias_source = inputs, bias, bias_source
        self.centre, self.yes, self.no, self.light_scale = np.asarray(centre, float), yes, no, scale
        self.drawn = ValueTracker(1.0)  # the input paths
        self.body = ValueTracker(1.0)  # ring, total, glow
        self.bias_drawn = ValueTracker(1.0)
        self.out_drawn = ValueTracker(1.0)
        self.eased = ValueTracker(self.total())

        self.warm_glow = glow(style.WARM[1], GLOW_RADIUS).move_to(self.centre)
        self.cool_glow = glow(style.COOL[0], GLOW_RADIUS).move_to(self.centre)
        self.warm_level, self.cool_level = ValueTracker(0.0), ValueTracker(0.0)
        fade_image(self.warm_glow, self.warm_level)
        fade_image(self.cool_glow, self.cool_level)
        self.ring = Circle(radius=RADIUS, stroke_width=STROKE, color=style.ASH).move_to(self.centre)

        bare = [i for i, put in enumerate(inputs) if put.source is None]
        self.bare: list[VGroup | None] = [None] * len(inputs)
        self.bare_spots = {}
        for k, i in enumerate(bare):
            y = self.centre[1] + BARE_STEP * ((len(bare) - 1) / 2 - k)
            self.bare_spots[i] = np.array([self.centre[0] - BARE_LEFT, y, 0])
            self.bare[i] = VGroup()
        self.bare_shown: list[str | None] = [None] * len(inputs)
        self.paths = [VMobject() for _ in inputs]
        self.weights = [serif(f"× {figure(put.weight)}", style.temperature(put.weight, 1), italic=False)
                        for put in inputs]
        self.bias_path = VMobject() if bias_source else None
        colour = style.temperature(bias, 2)
        sign, size = "−" if bias < 0 else "+", figure(abs(bias)) + ("," if bias_name else "")
        self.bias_label = VGroup(serif(sign, colour, italic=False), serif(size, colour, italic=False))
        if bias_name:
            self.bias_label.add(serif(bias_name, colour))
        self.bias_label.arrange(RIGHT, buff=0.1)
        if not bias_source:  # a number with no place: above the ring, clear of the paths below it
            self.bias_label.next_to(self.ring, UP, buff=0.2)

        # The live texts are rebuilt only when they change (building text is slow), inside fixed boxes.
        self.total_box, self.verdict_box = VGroup(), VGroup()
        self.total_text = self.verdict = None
        self.shown_total, self.shown_answer = None, None
        self.answer_at = self.centre + RIGHT * (RADIUS + ANSWER_GAP)
        self.output = Line(self.centre + RIGHT * RADIUS, self.answer_at + LEFT * SOCKET, stroke_width=PATH_STROKE)
        self.output_full = self.output.copy()
        self.socket = Circle(radius=SOCKET, stroke_width=PATH_STROKE).move_to(self.answer_at)
        self.dot = Circle(radius=DOT, stroke_width=0, fill_color=style.WARM[0], fill_opacity=0).move_to(self.answer_at)

        self.add(self.warm_glow, self.cool_glow, *self.paths, *([self.bias_path] if self.bias_path is not None else []),
                 self.ring, self.total_box, *[b for b in self.bare if b is not None], *self.weights,
                 self.bias_label, self.output, self.socket, self.dot, self.verdict_box)
        self.tick(0)
        self.add_updater(self.tick)

    def total(self) -> float:
        return sum(put.weight * put.value() for put in self.inputs) + self.bias

    def answer(self) -> bool:
        return self.total() > 0

    def arrive(self) -> list[Animation]:
        """Hide everything, and return its arrival in order: the input paths (drawn from their sources), the
        ring and its total, the bias, the output and answer."""
        trackers = [self.drawn, self.body, self.bias_drawn, self.out_drawn]
        for t in trackers:
            t.set_value(0.0)
        self.tick(0)
        times = (0.9, 0.5, 0.8, 0.6)
        return [AnimationGroup(t.animate.set_value(1.0), run_time=r) for t, r in zip(trackers, times)]

    # Drawing, once a frame (the neuron's own updater runs before its parts', so the glows read this frame's
    # levels).

    def tick(self, _: Mobject, dt: float = 0.0) -> None:
        total = self.total()
        e = self.eased.get_value()
        self.eased.set_value(e + (total - e) * min(1.0, dt * EASE))
        e, body = self.eased.get_value(), self.body.get_value()
        self.warm_level.set_value(body * style.light(e, self.light_scale) if e > 0 else 0.0)
        self.cool_level.set_value(body * style.light(e, self.light_scale) if e < 0 else 0.0)
        self.ring.set_stroke(opacity=body)
        if (text := figure(round(total))) != self.shown_total:
            self.total_text = serif(text, style.TEXT, TOTAL_SIZE, italic=False).move_to(self.centre)
            swap(self.total_box, self.total_text)
            self.shown_total = text
        self.total_text.set_opacity(body)
        self.draw_bare()
        self.draw_paths()
        self.draw_answer(total > 0)

    def draw_bare(self) -> None:
        for i, group in enumerate(self.bare):
            if group is None:
                continue
            put = self.inputs[i]
            text = figure(put.value())
            if text != self.bare_shown[i]:
                swap(group, serif(put.label, style.WARM[0]), serif(text, style.WARM[0], italic=False))
                group.arrange(RIGHT, buff=0.12)
                self.bare_shown[i] = text
            group.next_to(self.bare_spots[i], LEFT, buff=0).set_opacity(self.drawn.get_value())

    def sources(self) -> list[np.ndarray]:
        points = []
        for i, put in enumerate(self.inputs):
            points.append(np.asarray(put.source(), float) if put.source else self.bare_spots[i] + RIGHT * 0.1)
        if self.bias_source:
            points.append(np.asarray(self.bias_source(), float))
        return points

    def entries(self, sources: list[np.ndarray]) -> list[tuple[float, float]]:
        """Where each path enters the ring (an angle), in the order of the sources' heights, and which side of
        its path its label leans to: 1 above, but −1 below for the lowest path (passing through 0 only while
        paths swap places)."""
        if len(sources) == 1:
            return [(math.pi, 1.0)]
        keys = [s[1] - i * TIE for i, s in enumerate(sources)]
        last = len(sources) - 1
        return [(math.pi + r * (math.pi / 2) / last, min(max(2 * (last - r) - 1, -1.0), 1.0))
                for r in soft_ranks(keys)]

    def curve(self, start: np.ndarray, gather: float, angle: float) -> VMobject:
        """Straight right from the source to x = gather (level with every other path's turn), then turning
        into the ring at `angle`."""
        normal = np.array([math.cos(angle), math.sin(angle), 0])
        end = self.centre + normal * RADIUS
        turn = np.array([max(gather, start[0]), start[1], 0])
        reach = max(end[0] - turn[0], 0.5)
        path = VMobject()
        path.start_new_path(start)
        if turn[0] - start[0] > 1e-6:
            path.add_line_to(turn)
        path.add_cubic_bezier_curve_to(turn + RIGHT * 0.6 * reach, end + normal * min(END_HANDLE, 0.45 * reach), end)
        return path

    def draw_paths(self) -> None:
        sources = self.sources()
        entries = self.entries(sources)
        gather = max(p[0] for p in sources)
        drawn, bias_drawn = self.drawn.get_value(), self.bias_drawn.get_value()
        for i, put in enumerate(self.inputs):
            full = self.curve(sources[i], gather, entries[i][0])
            self.trace(self.paths[i], full, drawn, path_colours(style.WARM[0], put.weight * put.value()))
            self.place_label(self.weights[i], full, entries[i][1], drawn)
        if self.bias_path is not None:
            full = self.curve(sources[-1], gather, entries[-1][0])
            self.trace(self.bias_path, full, bias_drawn, path_colours(style.WARM[2], self.bias))
            self.place_label(self.bias_label, full, entries[-1][1], bias_drawn)
        else:
            self.bias_label.set_opacity(bias_drawn)

    @staticmethod
    def place_label(label: VGroup, path: VMobject, lean: float, shown: float) -> None:
        """Beside the path as it turns into the ring: above it (lean 1), below it (−1), or on it (0), clear
        of it by LABEL_GAP."""
        turn = path.points[-4:]  # the last curve: the turn into the ring

        def at(t: float) -> np.ndarray:
            return ((1 - t) ** 3 * turn[0] + 3 * (1 - t) ** 2 * t * turn[1] + 3 * (1 - t) * t ** 2 * turn[2]
                    + t ** 3 * turn[3])
        point, ahead = at(LABEL_AT), at(LABEL_AT + 0.01)
        tangent = (ahead - point) / (np.linalg.norm(ahead - point) or 1)
        left = np.array([-tangent[1], tangent[0], 0])  # to the left of travel: above a path heading right
        half = abs(left[0]) * label.width / 2 + abs(left[1]) * label.height / 2
        label.move_to(point + left * lean * (LABEL_GAP + half))
        label.set_opacity(shown)

    @staticmethod
    def trace(path: VMobject, full: VMobject, drawn: float, colours: list[str]) -> None:
        """Draw `drawn` of the path from its source, always from where the source is now."""
        path.set_points(full.points)
        if drawn < 1:
            path.pointwise_become_partial(full, 0, max(drawn, 1e-3))
        start, end = path.get_start(), path.get_end()
        path.set_stroke(color=colours, width=PATH_STROKE, opacity=1.0 if drawn > 0 else 0.0)
        path.set_sheen_direction([np.sign(end[0] - start[0]) or 1, np.sign(end[1] - start[1]), 0])

    def draw_answer(self, yes: bool) -> None:
        shown = self.out_drawn.get_value()
        colour = style.WARM[1] if yes else style.COOL[0]
        if shown < 1:
            self.output.pointwise_become_partial(self.output_full, 0, max(shown, 1e-3))
        else:
            self.output.set_points(self.output_full.points)
        self.output.set_stroke(color=colour, opacity=1.0 if shown > 0 else 0.0)
        self.socket.set_stroke(color=colour, opacity=shown)
        self.dot.set_fill(opacity=shown if yes else 0.0)
        if yes != self.shown_answer:
            self.verdict = (serif(self.yes, style.TEXT, VERDICT_SIZE) if yes
                            else serif(self.no, style.COOL[1], VERDICT_SIZE)).next_to(self.answer_at, DOWN, buff=0.45)
            swap(self.verdict_box, self.verdict)
            self.shown_answer = yes
        self.verdict.set_opacity(shown)
