"""Chapter 1, "Building one neuron": its beats, each a function that says its own line given the scene."""

import numpy as np
from manim import (DOWN, LEFT, RIGHT, UP, AnimationGroup, Circle, Create, Dot, FadeIn, FadeOut, Group, Line,
                   ManimColor, ParametricFunction, ReplacementTransform, Rotate, RoundedRectangle, Succession,
                   UpdateFromAlphaFunc, ValueTracker, VGroup, VMobject, Wait, interpolate_color, linear, smooth)

from series import style
from series.ember import fade_image, glow, numeral, serif, swap

RING = 0.55
CENTRE = np.array([0.0, 0.3, 0.0])
SETTING = 20  # the thermostat's setting, degrees
LOW, HIGH = 10, 30  # the thermometer's scale


def tween(tracker: ValueTracker, start: float, end: float, run_time: float, rate_func=smooth) -> UpdateFromAlphaFunc:
    """Move a tracker from start to end. (Chained .animate calls on one tracker in a Succession don't all play.)"""
    return UpdateFromAlphaFunc(tracker, lambda m, a: m.set_value(start + (end - start) * a), run_time=run_time,
                               rate_func=rate_func)


class Thermometer(VGroup):
    """A hairline thermometer, its amber column at `level` degrees with the reading beside it, and its setting
    as a gold mark across the tube."""

    def __init__(self, at: np.ndarray, level: float):
        super().__init__()
        self.at, self.tall = np.asarray(at, float), 4.4  # not .height: that is Manim's, and rescales
        self.level = ValueTracker(level)
        self.tube = RoundedRectangle(corner_radius=0.16, width=0.36, height=self.tall, stroke_color=style.TEXT,
                                     stroke_width=1.2).move_to(self.at + UP * 0.2)
        self.bulb = Circle(radius=0.32, stroke_color=style.TEXT, stroke_width=1.2, fill_color=style.WARM[0],
                           fill_opacity=0.9).move_to(self.tube.get_bottom() + DOWN * 0.18)
        self.column = Line(ORIGIN3, ORIGIN3, stroke_color=style.WARM[0], stroke_width=9)
        self.mark = Line(LEFT * 0.34, RIGHT * 0.34, stroke_color=style.WARM[2], stroke_width=2.2).move_to(
            self.y_of(SETTING) * UP + self.at[0] * RIGHT)
        self.mark_label = VGroup(serif("setting", style.WARM[2], 0.36), numeral(f"{SETTING}°", style.WARM[2], 0.36)
                                 ).arrange(RIGHT, buff=0.1).next_to(self.mark, LEFT, buff=0.18)
        self.reading = VGroup()
        self.shown = None
        self.mark.set_stroke(opacity=0)
        self.mark_label.set_opacity(0)
        self.add(self.column, self.tube, self.bulb, self.reading, self.mark, self.mark_label)
        self.add_updater(lambda m: m.draw())
        self.draw()

    def y_of(self, degrees: float) -> float:
        bottom = self.tube.get_bottom()[1] + 0.2
        return bottom + (degrees - LOW) / (HIGH - LOW) * (self.tall - 0.45)

    def draw(self) -> None:
        level = self.level.get_value()
        x = self.at[0]
        self.column.put_start_and_end_on(np.array([x, self.tube.get_bottom()[1], 0]),
                                         np.array([x, self.y_of(level), 0]))
        text = f"{round(level)}°"
        if text != self.shown:
            swap(self.reading, numeral(text, style.TEXT, 0.4))
            self.shown = text
        self.reading.next_to(np.array([x + 0.2, self.y_of(level), 0]), RIGHT, buff=0.12)


ORIGIN3 = np.zeros(3)


def flame(at: np.ndarray) -> VMobject:
    """A heater's flame: a hairline teardrop, point up."""
    def point(t: float) -> np.ndarray:
        return at + np.array([0.32 * np.sin(t) * np.sin(t / 2), -0.42 * np.cos(t), 0])
    return ParametricFunction(point, t_range=(0, 2 * np.pi), stroke_color=style.ASH, stroke_width=1.6)


def building_intro(scene) -> None:
    """A dim coal in the dark opens into a neuron: a ring with a question in it, one path in, an empty answer."""
    coal = Dot(CENTRE, radius=0.06, color=style.ASH)
    ring = Circle(radius=RING, stroke_color=style.ASH, stroke_width=1.4).move_to(CENTRE)
    unknown = serif("?", style.ASH, 0.5).move_to(CENTRE)
    path = Line(CENTRE + LEFT * 3.2, CENTRE + LEFT * RING, stroke_color=style.WARM[0], stroke_width=1.4)
    socket = Circle(radius=0.2, stroke_color=style.ASH, stroke_width=1.4).move_to(CENTRE + RIGHT * 2.45)
    out = Line(CENTRE + RIGHT * RING, socket.get_left(), stroke_color=style.ASH, stroke_width=1.4)
    scene.add(coal)
    scene.say("building-intro", Succession(
        FadeIn(coal, run_time=1.0),
        AnimationGroup(FadeOut(coal), Create(ring), FadeIn(unknown), run_time=1.2),
        Create(path, run_time=1.0, rate_func=linear),
        AnimationGroup(Create(out), Create(socket), run_time=1.0),
        Wait(1.0)))
    scene.kept = {"intro": VGroup(ring, unknown, path, out, socket)}


def thermostat(scene) -> None:
    """A thermometer: its column rises to 23°, then the setting is marked at 20°."""
    scene.play(FadeOut(scene.kept.pop("intro")), run_time=0.5)
    therm = Thermometer(LEFT * 2.6, LOW)
    scene.kept["therm"] = therm
    scene.add(therm)
    scene.say("thermostat", Succession(
        FadeIn(therm, run_time=0.8),
        tween(therm.level, LOW, 23, 2.0),
        AnimationGroup(UpdateFromAlphaFunc(therm.mark, lambda m, a: m.set_stroke(opacity=a)),
                       UpdateFromAlphaFunc(therm.mark_label, lambda m, a: m.set_opacity(a)), run_time=1.0),
        Wait(1.0)))


def thermostat_2(scene) -> None:
    """Below the setting the heat comes on (the flame lights, warm); above it, it goes dark."""
    therm = scene.kept["therm"]
    at = RIGHT * 2.2 + UP * 0.2
    heat = flame(at)
    level = ValueTracker(0.0)
    light = glow(style.WARM[1], 1.1).move_to(at)
    fade_image(light, level)
    word = serif("heat", style.ASH, 0.4).next_to(heat, DOWN, buff=0.3)

    def heater(m: VMobject, dt: float) -> None:
        on = therm.level.get_value() < SETTING
        level.set_value(level.get_value() + ((1.0 if on else 0.0) - level.get_value()) * min(1.0, dt * 8))
        m.set_stroke(color=interpolate_color(ManimColor(style.ASH), ManimColor(style.WARM[1]), level.get_value()))
        m.set_fill(style.WARM[1], opacity=0.25 * level.get_value())
    heat.add_updater(heater)
    scene.say("thermostat-2", Succession(
        AnimationGroup(FadeIn(heat), FadeIn(word), FadeIn(light), run_time=0.6),
        tween(therm.level, 23, 16, 1.4),
        Wait(1.0),
        tween(therm.level, 16, 24, 1.4),
        Wait(0.8)))
    heat.clear_updaters()
    scene.kept["heater"] = Group(heat, word, light)


def simplest_neuron(scene) -> None:
    """Numbers flow into the ring and add up; the total meets the setting; the answer says yes."""
    scene.play(FadeOut(scene.kept.pop("heater")), FadeOut(scene.kept["therm"]), run_time=0.6)
    centre = RIGHT * 1.0 + UP * 0.3
    values = (3, 4, 2)
    sums = np.cumsum(values)
    total = ValueTracker(0)
    ring = Circle(radius=RING, stroke_color=style.TEXT, stroke_width=1.4).move_to(centre)
    starts = [centre + LEFT * 4.2 + UP * dy for dy in (1.1, 0.0, -1.1)]
    ends = [centre + RING * np.array([-np.cos(a), np.sin(a), 0]) for a in (0.5, 0.0, -0.5)]
    paths = VGroup(*[Line(a, b, stroke_color=style.WARM[0], stroke_width=1.4) for a, b in zip(starts, ends)])
    numbers = VGroup(*[numeral(str(v), style.WARM[0], 0.46).next_to(a, LEFT, buff=0.15) for v, a in zip(values, starts)])
    shown = VGroup(numeral("0", style.TEXT, 0.46).move_to(centre))
    setting = VGroup(serif("setting", style.WARM[2], 0.38), numeral("6", style.WARM[2], 0.38)).arrange(
        RIGHT, buff=0.1).next_to(ring, DOWN, buff=0.3)
    socket = Circle(radius=0.2, stroke_color=style.ASH, stroke_width=1.4).move_to(centre + RIGHT * 2.45)
    out = Line(centre + RIGHT * RING, socket.get_left(), stroke_color=style.ASH, stroke_width=1.4)
    dot = Dot(socket.get_center(), radius=0.12, color=style.WARM[1]).set_opacity(0)
    verdict = serif("yes", style.WARM[1], 0.5).next_to(socket, DOWN, buff=0.3).set_opacity(0)

    def count(m: VGroup) -> None:
        text = str(round(total.get_value()))
        if m[0].text != text:
            swap(m, numeral(text, style.TEXT, 0.46).move_to(centre))
    shown.add_updater(count)

    def flow(i: int) -> Succession:
        """A copy of the number runs along its path into the ring, and the total takes it in."""
        travel = numeral(str(values[i]), style.WARM[1], 0.46)

        def along(m, a: float) -> None:
            m.move_to(starts[i] + (ends[i] - starts[i]) * a).set_opacity(min(1.0, 4 * (1 - a)))
        return Succession(UpdateFromAlphaFunc(travel, along, run_time=0.8, rate_func=smooth),
                          tween(total, int(sums[i] - values[i]), int(sums[i]), 0.3))
    scene.say("simplest-neuron", Succession(
        AnimationGroup(Create(ring), FadeIn(shown), *[Create(p) for p in paths], FadeIn(numbers), run_time=1.0),
        *[flow(i) for i in range(3)],
        AnimationGroup(FadeIn(setting), Create(out), Create(socket), run_time=0.7),
        AnimationGroup(dot.animate.set_opacity(1), verdict.animate.set_opacity(1), run_time=0.6),
        Wait(0.8)))
    shown.clear_updaters()
    scene.kept["neuron"] = VGroup(ring, shown, setting, socket, out, dot, verdict, paths, numbers)


def simplest_neuron_2(scene) -> None:
    """With one number it's a thermostat: the thermometer lies down and becomes a number line, the setting
    marked, no below it (cool), yes above (warm)."""
    therm = scene.kept["therm"]
    therm.level.set_value(23)
    therm.draw()
    therm.clear_updaters()
    therm.shift(UP * 0.2 - therm.tube.get_center())
    axis = Line(LEFT * 5 + UP * 0.2, RIGHT * 5 + UP * 0.2, stroke_color=style.TEXT, stroke_width=1.2)
    no = serif("no", style.COOL[1], 0.5).move_to(LEFT * 2.8 + UP * 0.75)
    yes = serif("yes", style.WARM[1], 0.5).move_to(RIGHT * 2.8 + UP * 0.75)
    setting = serif("setting", style.WARM[2], 0.38).set_opacity(0)
    scene.add(setting)
    scene.say("simplest-neuron-2", Succession(
        AnimationGroup(FadeOut(scene.kept.pop("neuron")), FadeIn(therm), run_time=0.7),
        AnimationGroup(FadeOut(therm.reading), FadeOut(therm.mark_label), run_time=0.3),
        Rotate(VGroup(therm.tube, therm.column, therm.bulb, therm.mark), -np.pi / 2, about_point=UP * 0.2,
               run_time=1.0),
        AnimationGroup(ReplacementTransform(therm.tube, axis), FadeOut(therm.column), FadeOut(therm.bulb),
                       run_time=0.8),
        AnimationGroup(Create(Line(axis.get_start(), therm.mark.get_center(), stroke_color=style.COOL[0], stroke_width=1.8)),
                       Create(Line(therm.mark.get_center(), axis.get_end(), stroke_color=style.WARM[1], stroke_width=1.8)),
                       FadeIn(no), FadeIn(yes), run_time=0.8),
        UpdateFromAlphaFunc(setting, lambda m, a: m.next_to(therm.mark, DOWN, buff=0.2).set_opacity(a),
                            run_time=0.5)))


BEATS = {"building-intro": building_intro, "thermostat": thermostat, "thermostat-2": thermostat_2,
         "simplest-neuron": simplest_neuron, "simplest-neuron-2": simplest_neuron_2}
