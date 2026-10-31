"""Chapter 1, "Building one neuron": its beats, each a function that says its own line given the scene."""

import json
from pathlib import Path

import numpy as np
from manim import (DOWN, LEFT, RIGHT, UP, Animation, AnimationGroup, Arc, Circle, Create, DashedLine,
                   DashedVMobject, Dot, FadeIn, FadeOut, Group, ImageMobject, Line, ManimColor, ParametricFunction,
                   Rectangle, ReplacementTransform, Rotate, RoundedRectangle, Square, Succession, Transform,
                   UpdateFromAlphaFunc, ValueTracker, VGroup, VMobject, Wait, config, interpolate_color, linear,
                   smooth)
from manim.constants import RESAMPLING_ALGORITHMS
from PIL import Image
from reservoir_film.model import NES_H, NES_W

from series import style
from series.ember import baseline, fade_image, glow, in_line, numeral, rgb, serif, swap
from series.footage import NES_FPS
from series.lab.data import load
from series.lab.exp001 import features, in_play, label
from series.memory import MemoryGrid
from series.overlays import LABEL_INSET, SettingLine, feet_line
from series.recording import DATA, LAG, Recording
from series.records import APART, TAPE_H, TAPE_W, Tapes
from series.screen import Screen

RING = 0.55
CENTRE = np.array([0.0, 0.3, 0.0])
SETTING = 20  # the thermostat's setting, degrees
LOW, HIGH = 10, 30  # the thermometer's scale


def tween(tracker: ValueTracker, start: float, end: float, run_time: float, rate_func=smooth) -> UpdateFromAlphaFunc:
    """Move a tracker from start to end. (Chained .animate calls on one tracker in a Succession don't all play.)"""
    return UpdateFromAlphaFunc(tracker, lambda m, a: m.set_value(start + (end - start) * a), run_time=run_time,
                               rate_func=rate_func)


def wander(tracker: ValueTracker, start: float, waves: list[tuple[float, float, float]], run_time: float,
           whole: bool = False) -> UpdateFromAlphaFunc:
    """Move a tracker by a sum of waves (amplitude, cycles per second, phase), each starting at its phase, so
    the tracker starts at `start` plus the waves' starting values and ends there after whole cycles. With
    `whole`, the value is rounded."""
    def at(t: float) -> float:
        value = start + sum(a * (np.sin(2 * np.pi * f * t + phase) - np.sin(phase)) for a, f, phase in waves)
        return round(value) if whole else value
    return UpdateFromAlphaFunc(tracker, lambda m, a: m.set_value(at(a * run_time)), run_time=run_time,
                               rate_func=linear)


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
        self.mark_label = in_line(serif("setting", style.WARM[2], 0.36), numeral(f"{SETTING}°", style.WARM[2], 0.36),
                                  buff=0.1).next_to(self.mark, LEFT, buff=0.18)
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
        Wait(0.8)), pad=0)
    # Then, after a moment, it wanders back and forth across the setting, the heat following it.
    scene.play(Succession(Wait(1.0), wander(therm.level, 24, [(3.5, 1.0, np.pi / 2)], 3.0), Wait(0.6)))
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
    setting = in_line(serif("setting", style.WARM[2], 0.38), numeral("6", style.WARM[2], 0.38), buff=0.1).next_to(
        ring, DOWN, buff=0.3)
    socket = Circle(radius=0.2, stroke_color=style.ASH, stroke_width=1.4).move_to(centre + RIGHT * 2.45)
    out = Line(centre + RIGHT * RING, socket.get_left(), stroke_color=style.ASH, stroke_width=1.4)
    dot = Dot(socket.get_center(), radius=0.12, color=style.WARM[1]).set_opacity(0)
    verdict = serif("yes", style.WARM[1], 0.5).next_to(socket, DOWN, buff=0.3).set_opacity(0)
    answer = VGroup(verdict)

    def count(m: VGroup) -> None:
        text = str(round(total.get_value()))
        if m[0].text != text:
            swap(m, numeral(text, style.TEXT, 0.46).move_to(centre))
    shown.add_updater(count)

    def flow(i: int) -> Succession:
        """A copy of the number runs along its path into the ring, and the total takes it in."""
        # Hidden at its start until its turn: it's on screen from the line's start, waiting.
        travel = numeral(str(values[i]), style.WARM[1], 0.46).move_to(starts[i]).set_opacity(0)

        def along(m, a: float) -> None:
            m.move_to(starts[i] + (ends[i] - starts[i]) * a).set_opacity(min(1.0, 4 * (1 - a)))
        return Succession(UpdateFromAlphaFunc(travel, along, run_time=0.8, rate_func=smooth),
                          tween(total, int(sums[i] - values[i]), int(sums[i]), 0.3))
    scene.say("simplest-neuron", Succession(
        AnimationGroup(Create(ring), FadeIn(shown), *[Create(p) for p in paths], FadeIn(numbers), run_time=1.0),
        *[flow(i) for i in range(3)],
        AnimationGroup(FadeIn(setting), Create(out), Create(socket), run_time=0.7),
        AnimationGroup(dot.animate.set_opacity(1), verdict.animate.set_opacity(1), run_time=0.6),
        Wait(0.8)), pad=0)
    shown.clear_updaters()

    # Then, after a moment, the inputs change, each at its own pace, and the total and the answer follow.
    inputs = [ValueTracker(v) for v in values]
    said = [True]

    def follow(_) -> None:
        now = [round(v.get_value()) for v in inputs]
        if [int(n.text) for n in numbers] != now:
            swap(numbers, *[numeral(str(v), style.WARM[0], 0.46).next_to(a, LEFT, buff=0.15) for v, a in zip(now, starts)])
        total.set_value(sum(now))
        count(shown)
        yes = sum(now) > 6
        dot.set_opacity(1 if yes else 0)
        if yes != said[0]:
            said[0] = yes
            swap(answer, serif("yes" if yes else "no", style.WARM[1] if yes else style.COOL[1], 0.5)
                 .next_to(socket, DOWN, buff=0.3))
    live = VGroup(numbers, shown, dot, answer)  # on what they draw: a play draws a mobject with no updater once
    live.add_updater(follow)
    scene.add(live)
    # Amplitude, cycles a second, phase: the answer says yes half the time, flipping about seven times.
    pace = ((2, 0.7, np.pi), (2, 1.1, np.pi / 2), (2, 0.45, 0.0))
    scene.play(Succession(Wait(1.0), AnimationGroup(*[wander(v, start, [wave], 4.0, whole=True)
                                                      for v, start, wave in zip(inputs, values, pace)]), Wait(0.6)))
    live.clear_updaters()
    scene.kept["neuron"] = VGroup(ring, shown, setting, socket, out, dot, answer, paths, numbers)


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


START = 1247  # seed 1's footage: Mario running, jumping, landing
AIR, DEPTH = 0x001D, 0x00CE  # SMB1: 0 standing, 1 jumping, 2 falling; Mario's depth (docs/series/games/001.md)


def question(scene) -> None:
    """The game arrives, and the question beside it."""
    scene.clear()
    u = config.pixel_height / 1080
    screen = Screen(Recording("smb1"), START, left_px=round(40 * u))  # 3x at 1080p, vertically centred
    ask = VGroup(serif("Is Mario", style.TEXT, 0.62), serif("in the air?", style.TEXT, 0.62)).arrange(
        DOWN, buff=0.2).move_to(RIGHT * 2.9 + UP * 0.2)
    scene.kept = {"screen": screen}
    scene.say("question", Succession(screen.arrive(run_time=1.0), FadeIn(ask, run_time=0.8), Wait(1.0)))
    scene.kept["ask"] = ask


def answer_key(scene) -> None:
    """The game's memory beside it, and one square lit: 0, 1 or 2 as Mario stands, jumps and falls."""
    screen = scene.kept["screen"]
    grid = MemoryGrid(screen, width=6.9)
    grid.move_to([config.frame_width / 2 - 0.35 - grid.image.width / 2, 1.9, 0])
    scene.kept["grid"] = grid
    grid.set_level(0.0)
    scene.add(grid)
    scene.say("answer-key", Succession(
        AnimationGroup(FadeOut(scene.kept.pop("ask")), grid.arrive(), run_time=1.0),
        Wait(1.2),
        grid.light(AIR, "the answer key", 1, DOWN),
        Wait(2.0)))


def answer_key_2(scene) -> None:
    """0 on the ground; 1 (jumping) and 2 (falling) slide together under 'in the air'."""
    y = -0.9
    digits = VGroup(*[numeral(str(n), style.TEXT, 0.6) for n in (0, 1, 2)])
    for d, x in zip(digits, (2.3, 3.6, 4.9)):
        d.move_to([x, y, 0])
    ground = serif("on the ground", style.COOL[1], 0.42).next_to(digits[0], DOWN, buff=0.3)
    air = serif("in the air", style.WARM[1], 0.42).move_to([4.6, y - 0.62, 0])
    scene.kept["legend"] = VGroup(digits, ground, air)
    scene.say("answer-key-2", Succession(
        FadeIn(digits, run_time=0.6),
        AnimationGroup(digits[1].animate.move_to([4.3, y, 0]).set_color(style.WARM[1]),
                       digits[2].animate.move_to([4.9, y, 0]).set_color(style.WARM[1]),
                       digits[0].animate.set_color(style.COOL[1]), run_time=1.0),
        AnimationGroup(FadeIn(air), FadeIn(ground), run_time=0.6)))


def padlock(at: np.ndarray) -> VGroup:
    body = RoundedRectangle(corner_radius=0.04, width=0.32, height=0.24, stroke_color=style.WARM[2], stroke_width=1.4,
                            fill_color=style.BACKGROUND, fill_opacity=1).move_to(at)
    shackle = Arc(radius=0.1, start_angle=0, angle=np.pi, stroke_color=style.WARM[2], stroke_width=1.4).move_to(
        body.get_top() + UP * 0.07)
    return VGroup(shackle, body)


def never_see(scene) -> None:
    """The answer key is locked: a line from it runs to the scorecard, and the way to the neuron is closed."""
    grid = scene.kept["grid"]
    key = grid.lit[AIR].box
    lock = padlock(key.get_corner(UP + RIGHT) + RIGHT * 0.12)
    card = VGroup(RoundedRectangle(corner_radius=0.08, width=1.5, height=0.8, stroke_color=style.TEXT, stroke_width=1.2),
                  serif("score", style.TEXT, 0.42)).move_to(RIGHT * 4.8 + DOWN * 2.6)
    card[1].move_to(card[0])
    ring = Circle(radius=0.4, stroke_color=style.ASH, stroke_width=1.2).move_to(RIGHT * 2.4 + DOWN * 2.6)
    neuron = VGroup(ring, serif("neuron", style.ASH, 0.36).next_to(ring, DOWN, buff=0.15))
    to_card = Line(key.get_bottom(), card[0].get_top(), stroke_color=style.WARM[1], stroke_width=1.4)
    toward = key.get_bottom() + (ring.get_top() - key.get_bottom()) * 0.55
    blocked = VGroup(DashedLine(key.get_bottom(), toward, stroke_color=style.ASH, stroke_width=1.2),
                     Line(LEFT * 0.16, RIGHT * 0.16, stroke_color=style.ASH, stroke_width=2.0).move_to(toward).rotate(
                         np.arctan2(*(ring.get_top() - key.get_bottom())[:2][::-1]) + np.pi / 2))
    scene.kept["lock"] = VGroup(lock, card, neuron, to_card, blocked)
    scene.say("never-see", Succession(
        FadeOut(scene.kept.pop("legend"), run_time=0.5),
        FadeIn(lock, shift=DOWN * 0.1, run_time=0.6),
        AnimationGroup(FadeIn(card), FadeIn(neuron), run_time=0.6),
        Create(to_card, run_time=0.8),
        Create(blocked, run_time=0.8),
        Wait(1.0)))


def tick(at: np.ndarray, size: float = 0.2) -> VMobject:
    """A check mark."""
    return VMobject(stroke_color=style.WARM[1], stroke_width=2.0).set_points_as_corners(
        [at + size * np.array(p) for p in ((-0.5, 0.0, 0), (-0.15, -0.4, 0), (0.55, 0.45, 0))])


def why_key(scene) -> None:
    """Every answer can be checked: ticks fill under the scorecard. Then a question the memory holds no answer
    to, beside an empty square."""
    lock = scene.kept["lock"]
    card = lock[1][0]
    ticks = VGroup(*[tick(card.get_bottom() + DOWN * 0.38 + RIGHT * (k - 3.5) * 0.3) for k in range(8)])
    for t in ticks:
        t.set_stroke(opacity=0)
    ask = serif("Will Mario land soon?", style.ASH, 0.44)
    empty = VGroup(DashedVMobject(Square(0.42, stroke_color=style.ASH, stroke_width=1.2), num_dashes=16),
                   serif("?", style.ASH, 0.36))
    future = VGroup(ask, empty).arrange(RIGHT, buff=0.35).move_to(RIGHT * 3.6 + DOWN * 0.9)
    empty[1].move_to(empty[0])
    lock.add(ticks, future)
    scene.add(ticks)
    scene.say("why-key", Succession(
        Wait(0.8),
        *[t.animate(run_time=0.22).set_stroke(opacity=1) for t in ticks],
        Wait(2.5),
        FadeIn(ask, run_time=0.8),
        AnimationGroup(Create(empty[0]), FadeIn(empty[1]), run_time=0.8),
        Wait(1.0)))


def dim(lit, level: float) -> UpdateFromAlphaFunc:
    """A lit square dimming to `level` through its own drawing (set_opacity would flatten its glow)."""
    start = lit.level
    return UpdateFromAlphaFunc(lit, lambda m, a: m.show(start + (level - start) * a))


def drawn_feet(screen: Screen) -> tuple:
    """A feet line that draws itself across from the left as its tracker goes from 0 to 1, its label fading up,
    and undraws itself as it goes back to 0 (it's also the line's `level`, so `gone` takes it away that way:
    a FadeOut would fight its label, rebuilt at full strength whenever the reading changes)."""
    feet = feet_line(screen)
    drawn = ValueTracker(0.0)
    feet.level = drawn

    def draw(m) -> None:
        d = drawn.get_value()
        start, end = m.line.get_start(), m.line.get_end()
        m.line.put_start_and_end_on(start, start + (end - start) * max(d, 1e-3))
        if d < 1:  # a rebuilt label comes at full strength
            m.label.set_opacity(d)
    draw(feet)
    feet.add_updater(draw)
    return feet, drawn


def depth(scene) -> None:
    """Instead the neuron reads Mario's depth: a line at his feet, its reading live, and its square lit."""
    screen, grid = scene.kept["screen"], scene.kept["grid"]
    feet, drawn = drawn_feet(screen)
    scene.add(feet)
    scene.kept["feet"] = feet
    scene.say("depth", Succession(
        AnimationGroup(FadeOut(scene.kept.pop("lock")), dim(grid.lit[AIR], 0.35), run_time=0.6),
        tween(drawn, 0.0, 1.0, 1.0),
        Wait(1.0),
        grid.light(DEPTH, "depth", 0, DOWN),
        Wait(2.0)))


SESSIONS = DATA / "smb1"  # sessions 1-3 of the random player; the footage is session 1's
BUTTON_A, BUTTON_B, BUTTON_UP, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT = 0x01, 0x02, 0x10, 0x20, 0x40, 0x80  # src/nes/bus
FAST = 8  # the fast-forward
SESSION_FRAMES = 18000
NUMBERS = json.loads((Path(__file__).parents[1] / "numbers.json").read_text())
CUT = NUMBERS["smb1_height_cut"]
OFFSET = 33  # a depth's row on screen is depth + 33 (see overlays.feet_line)
SWEEP_END = 206  # the depth at the bottom row of the screen
CHOOSE_START = 9680  # session 1: on the floor, jumping often; the next restart is 28 s on
NICK = 0.004  # a left-out moment's least width, as a share of its tape: most are one frame


class Pad(VGroup):
    """The controller as hairlines, the d-pad a cross and B and A two rings, each key lighting while the
    program holds it down for the picture on screen."""

    def __init__(self, screen: Screen, at: np.ndarray, key: float = 0.44):
        super().__init__()
        self.screen = screen
        self.buttons = np.fromfile(SESSIONS / "1" / "buttons.u8", dtype=np.uint8)
        self.level = ValueTracker(1.0)
        cross = at + LEFT * 1.3
        self.keys = {bit: Square(key).move_to(cross + direction * key)
                     for bit, direction in ((BUTTON_UP, UP), (BUTTON_DOWN, DOWN), (BUTTON_LEFT, LEFT), (BUTTON_RIGHT, RIGHT))}
        self.hub = Square(key, stroke_width=0, fill_opacity=0).move_to(cross)
        for bit, x in ((BUTTON_B, 0.65), (BUTTON_A, 1.75)):
            self.keys[bit] = Circle(radius=0.36).move_to(at + RIGHT * x)
        self.names = VGroup(*[serif(name, style.ASH, 0.36).next_to(self.keys[bit], DOWN, buff=0.18)
                              for name, bit in (("B", BUTTON_B), ("A", BUTTON_A))])
        self.add(self.hub, *self.keys.values(), self.names)
        self.add_updater(lambda m: m.draw())
        self.draw()

    def held(self) -> int:
        """The buttons pressed during the frame whose RAM the picture shows (the picture trails it by LAG)."""
        return int(self.buttons[max(self.screen.frame() - LAG["smb1"], 0)])

    def draw(self) -> None:
        held, level = self.held(), self.level.get_value()
        for bit, key in self.keys.items():
            on = bool(held & bit)
            key.set_stroke(style.WARM[1] if on else style.ASH, width=1.4, opacity=level)
            key.set_fill(style.WARM[1], opacity=0.75 * level if on else 0)
        self.names.set_opacity(level)


class Restarts(VGroup):
    """The stretch of play being fast-forwarded through as a hairline, a cursor at the frame on screen, and a
    cool tick at each restart it has passed, counted at the line's end."""

    def __init__(self, screen: Screen, first: int, frames: int, left: float, right: float, y: float):
        super().__init__()
        self.screen, self.first, self.frames, self.left, self.right = screen, first, frames, left, right
        restarts = json.loads((SESSIONS / "1" / "meta.json").read_text())["restart_frames"]
        self.at = [r for r in restarts if first <= r < first + frames]
        self.line = Line([left, y, 0], [right, y, 0], stroke_color=style.ASH, stroke_width=1.2)
        self.cursor = Line(UP * 0.16, DOWN * 0.16, stroke_color=style.TEXT, stroke_width=1.6)
        self.ticks = VGroup(*[Line([self.x(r), y + 0.2, 0], [self.x(r), y - 0.2, 0], stroke_color=style.COOL[1],
                                   stroke_width=2.0) for r in self.at])
        self.name = serif("restarts", style.ASH, 0.38).next_to(self.line, LEFT, buff=0.25)
        self.count, self.shown = VGroup(), None
        self.y = y
        self.level = ValueTracker(1.0)
        self.add(self.line, self.ticks, self.cursor, self.name, self.count)
        self.add_updater(lambda m: m.draw())
        self.draw()

    def x(self, frame: int) -> float:
        return self.left + (self.right - self.left) * min(max((frame - self.first) / self.frames, 0.0), 1.0)

    def draw(self) -> None:
        frame = self.screen.frame()
        self.cursor.move_to([self.x(frame), self.y, 0])
        passed, level = sum(frame >= r for r in self.at), self.level.get_value()
        for r, tick in zip(self.at, self.ticks):
            tick.set_stroke(opacity=level if frame >= r else 0.0)
        if passed != self.shown:
            swap(self.count, numeral(str(passed), style.COOL[1], 0.42).next_to(self.line, RIGHT, buff=0.25))
            self.shown = passed
        for part in (self.line, self.cursor):
            part.set_stroke(opacity=level)
        for part in (self.name, self.count):
            part.set_opacity(level)


def grid_away(grid: MemoryGrid) -> UpdateFromAlphaFunc:
    """The memory grid and its lit squares fading out through their own drawing."""
    lits = {address: lit.level for address, lit in grid.lit.items()}

    def fade(m: MemoryGrid, a: float) -> None:
        m.set_level(1 - a)
        for address, level in lits.items():
            m.lit[address].show(level * (1 - a))
    return UpdateFromAlphaFunc(grid, fade)


def recording(scene) -> None:
    """The memory goes; the controller lights beside the game as the random player presses its buttons."""
    screen, grid = scene.kept["screen"], scene.kept.pop("grid")
    feet = scene.kept.pop("feet")
    pad = Pad(screen, RIGHT * 3.0 + UP * 0.4)
    pad.level.set_value(0)
    scene.add(pad)
    scene.kept["pad"] = pad
    scene.say("recording", Succession(
        AnimationGroup(grid_away(grid), gone(feet), run_time=0.8),
        tween(pad.level, 0.0, 1.0, 0.8),
        Wait(1.0)))
    scene.remove(grid)


def recording_2(scene) -> None:
    """Fast-forward: the player dies again and again, and each time the level starts over, ticked on a line."""
    screen, pad = scene.kept["screen"], scene.kept["pad"]
    voice = scene.voices["recording-2"].seconds + 2
    restarts = Restarts(screen, screen.frame(), round((voice + 1) * FAST * NES_FPS), 0.6, 5.8, -2.0)
    speed = ValueTracker(1.0)
    screen.game.add_updater(lambda m: setattr(m, "speed", speed.get_value()))
    fast = numeral(f"×{FAST}", style.TEXT, 0.5).next_to(screen.game, UP + RIGHT, buff=0.15).shift(LEFT * 0.9)
    fast.set_opacity(0)
    restarts.level.set_value(0)
    scene.add(restarts, fast)
    scene.kept["restarts"], scene.kept["fast"] = restarts, fast
    scene.say("recording-2", Succession(
        AnimationGroup(tween(speed, 1.0, FAST, 1.0), fast.animate.set_opacity(1), tween(restarts.level, 0, 1, 1.0),
                       run_time=1.0),
        Wait(1.0)))


def leave(scene, screen: Screen, *others) -> None:
    """The game departs, with whatever goes with it, before the next line; then they're taken off."""
    scene.play(screen.depart(0.6), *[gone(m) for m in others], run_time=0.6)
    scene.remove(screen, *others)


def gone(m) -> Animation:
    """Fade something out: by its own level if it draws itself by one (a fade's target copy would redraw
    itself at full strength every frame), else by FadeOut."""
    level = getattr(m, "level", None)
    if isinstance(level, ValueTracker):
        return tween(level, level.get_value(), 0.0, 0.6)
    return FadeOut(m)


def jump_second(recording: Recording, after: int) -> int:
    """The first frame from `after` that starts a second of play with a whole jump in it: on the ground, up,
    and down again."""
    for f in range(after, len(recording) - NES_FPS):
        air = [recording.byte_at(g, AIR) for g in range(f, f + NES_FPS)]
        if air[0] == 0 and air[8] and air[-1] == 0 and not any(air[-8:]) and 0 in air[1:12]:
            return f
    raise ValueError(f"no whole jump in a second of play after frame {after}")


def frames(scene) -> None:
    """A second of play as sixty pictures, laid down at the speed the game drew them."""
    screen = scene.kept["screen"]
    first = jump_second(screen.recording, START)
    w = 1.04
    thumbs = []
    for f in range(first, first + NES_FPS):
        picture = Image.fromarray(np.asarray(screen.recording.frames[f])).resize((140, 131), Image.Resampling.BOX)
        thumbs.append(ImageMobject(np.asarray(picture)).set(width=w))
    sheet = Group(*thumbs).arrange_in_grid(5, 12, buff=0.06).move_to(UP * 0.45)
    levels = [ValueTracker(0.0) for _ in thumbs]
    for thumb, level in zip(thumbs, levels):
        fade_image(thumb, level)
    count = VGroup(numeral("0", style.TEXT, 0.5))
    shown = [0]
    caption = in_line(count, serif("frames", style.TEXT, 0.5), serif("in one second", style.ASH, 0.5),
                      buff=0.18).next_to(sheet, DOWN, buff=0.45)
    caption[2].set_opacity(0)

    def lay(_, a: float) -> None:
        n = a * NES_FPS
        for k, level in enumerate(levels):
            level.set_value(min(max(n - k, 0.0), 1.0))
        if round(n) != shown[0]:
            shown[0] = round(n)
            swap(count, numeral(str(shown[0]), style.TEXT, 0.5).move_to(count, aligned_edge=DOWN + RIGHT))
    leave(scene, screen, scene.kept.pop("pad"), scene.kept.pop("restarts"), scene.kept.pop("fast"))
    scene.add(sheet)
    scene.kept["frames"] = Group(sheet, caption)
    scene.kept["levels"] = levels
    scene.say("frames", Succession(
        FadeIn(caption[:2], run_time=0.3),
        UpdateFromAlphaFunc(sheet, lay, run_time=1.0, rate_func=linear),
        caption[2].animate(run_time=0.6).set_opacity(1),
        Wait(1.0)))


def frames_2(scene) -> None:
    """Three tapes, one for each five-minute session, filling as their frames count up."""
    sheet, levels = scene.kept.pop("frames"), scene.kept.pop("levels")
    tapes = Tapes(3, ["1", "2", "3"]).shift(LEFT * 0.9 + UP * 0.3)
    fills, counts = [], []
    for tape in tapes.tapes:
        fill = Rectangle(width=1e-3, height=TAPE_H, stroke_width=0, fill_color=style.WARM[0], fill_opacity=0.2)
        tape.add_to_back(fill)
        fills.append(fill)
        counts.append(VGroup(VGroup(numeral("0", style.TEXT, 0.4)), serif("frames", style.ASH, 0.4)))
    shown = [None]

    def record(_, a: float) -> None:
        for tape, fill, count in zip(tapes.tapes, fills, counts):
            left = tape.strip.get_left()
            fill.stretch_to_fit_width(max(a * TAPE_W, 1e-3)).move_to(left + RIGHT * a * TAPE_W / 2)
        n = round(a * SESSION_FRAMES / 100) * 100
        if n != shown[0]:
            shown[0] = n
            for tape, count in zip(tapes.tapes, counts):
                swap(count[0], numeral(f"{n:,}", style.TEXT, 0.4))
                in_line(*count, buff=0.14).next_to(tape.strip, RIGHT, buff=0.3)
    record(None, 0.0)
    scene.kept["tapes"], scene.kept["counts"] = tapes, VGroup(*counts)
    scene.say("frames-2", Succession(
        AnimationGroup(*[tween(level, 1.0, 0.0, 0.6) for level in levels], FadeOut(sheet[1]), run_time=0.6),
        AnimationGroup(FadeIn(tapes), *[FadeIn(c) for c in counts], run_time=0.6),
        # A tracker, not the tapes: animating them would put them on screen from the line's start, before their fade.
        UpdateFromAlphaFunc(ValueTracker(), record, run_time=3.0, rate_func=linear),
        Wait(0.5)))
    scene.remove(*sheet)


def left_out(session: str) -> list[tuple[float, float]]:
    """The stretches of a session the game was moving Mario, as shares of its length, each at least NICK wide."""
    play = in_play("smb1", load(SESSIONS / session).ram)
    edges = np.flatnonzero(np.diff(np.r_[1, play.astype(int), 1]))
    spans = []
    for a, b in zip(edges[::2] / len(play), edges[1::2] / len(play)):
        middle, width = (a + b) / 2, max(b - a, NICK)
        spans.append((middle - width / 2, middle + width / 2))
    return spans


def frames_3(scene) -> None:
    """The moments the game took Mario out of the player's hands grey out, and the count drops to what's kept."""
    tapes, counts = scene.kept["tapes"], scene.kept["counts"]
    kept = [int(in_play("smb1", load(SESSIONS / s).ram).sum()) for s in ("1", "2", "3")]
    shown = [None]

    def recount(_, a: float) -> None:
        values = [round(SESSION_FRAMES + (k - SESSION_FRAMES) * a) for k in kept]
        if values != shown[0]:
            shown[0] = values
            for tape, count, value in zip(tapes.tapes, counts, values):
                swap(count[0], numeral(f"{value:,}", style.TEXT, 0.4))
                in_line(*count, buff=0.14).next_to(tape.strip, RIGHT, buff=0.3)
    scene.say("frames-3", Succession(
        AnimationGroup(*[tapes.grey(i, left_out(s)) for i, s in enumerate(("1", "2", "3"))], run_time=1.6),
        Wait(0.6),
        UpdateFromAlphaFunc(counts, recount, run_time=1.2),
        Wait(1.0)))


def scores() -> np.ndarray:
    """How often "depth below the setting" matched the answer key on sessions 1 and 2, for every setting."""
    x, y = features("smb1", [load(SESSIONS / s) for s in ("1", "2")])
    found = np.array([((x[:, 0] < t).astype(int) == y).mean() for t in range(256)])
    if int(found.argmax()) != CUT:
        raise ValueError(f"the sweep's best setting is {found.argmax()}, but numbers.json says {CUT}: re-run exp001")
    return found


class Sweep(SettingLine):
    """The setting line, its depth live as it moves, labelled at its right end (a feet line's label is at the
    left)."""

    def parts(self) -> tuple[str, str | None]:
        return self.caption, style.number(self.row)

    def refresh(self, *_) -> None:
        super().refresh()
        self.label.next_to([self.screen.col_x(256) - LABEL_INSET, self.y(), 0], UP + LEFT, buff=0.06)


# Small Mario mid-stride, as the game draws him (taken from session 1's footage, frame 5335, against the sky).
MARIO = """
.....#####......
....#########...
....#######.....
...##########...
...###########..
...##########...
.....#######....
..########......
###############.
###.###########.
##..#######..#..
...###########..
..############..
.#####...#####..
.###............
..###...........
"""
STAMP = np.array([[c == "#" for c in row] for row in MARIO.split()], dtype=float)
TALLY_PX, TALLY_W = 360, 3.6  # the tally's width: pixels and frame units
FRAMES_PER_PX = 2.5  # a bar's length: one pixel for each 2.5 frames
SCREEN_X = 0x03AD  # Mario's x on screen


LANES = 64  # Marios on screen at once, each playing its own stretch of the training frames


def training_marios() -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    """Mario's screen x and depth in every frame of sessions 1 and 2 the setting was chosen on (those in play),
    in order, and whether the answer key said he was in the air."""
    xs, depths, airs = [], [], []
    for session in ("1", "2"):
        ram = load(SESSIONS / session).ram
        play = in_play("smb1", ram)
        xs.append(ram[play, SCREEN_X])
        depths.append(ram[play, DEPTH])
        airs.append(label("smb1")(ram)[play])
    x, depth, air = (np.concatenate(v).astype(int) for v in (xs, depths, airs))
    return x, depth, air.astype(bool)


class Crowd(ImageMobject):
    """The training frames as Marios on the game screen: LANES of them at once, each playing its own stretch of
    the frames in order (so they run and jump), cool while the answer key says on the ground and warm while in
    the air, each leaving a faint trail of where it has been. `shown` (0 to 1) is how far they've played, `level`
    how strongly it's all drawn."""

    def __init__(self, screen: Screen):
        super().__init__(np.zeros((NES_H, NES_W, 4), np.uint8))
        self.set_resampling_algorithm(RESAMPLING_ALGORITHMS["nearest"])
        self.stretch_to_fit_width(screen.game.width).stretch_to_fit_height(screen.game.height).move_to(screen.game)
        self.xs, self.depths, self.airs = training_marios()  # not .depth: that's Manim's, and rescales
        self.starts = np.linspace(0, len(self.xs), LANES + 1).astype(int)
        self.ground_n, self.air_n = np.zeros((NES_H, NES_W)), np.zeros((NES_H, NES_W))
        self.ground_at, self.air_at = np.zeros(256), np.zeros(256)  # frames counted at each depth so far
        self.played = 0  # frames into each lane
        self.shown, self.level = ValueTracker(0.0), ValueTracker(1.0)
        self.add_updater(lambda m: m.draw())

    def place(self, k: int) -> tuple[int, int, int, int, int, int]:
        """Where frame k's Mario is, clipped to the screen: rows r0:r1 and columns c0:c1 of the screen, and the
        stamp's own first row and column."""
        top, left = self.depths[k] + 17, self.xs[k]
        r0, r1, c0, c1 = max(top, 0), min(top + 16, NES_H), max(left, 0), min(left + 16, NES_W)
        return r0, r1, c0, c1, r0 - top, c0 - left

    def draw(self) -> None:
        lane = np.diff(self.starts).min()
        played = round(self.shown.get_value() * lane)
        for start in self.starts[:-1]:
            for k in range(start + self.played, start + played):
                (self.air_at if self.airs[k] else self.ground_at)[self.depths[k]] += 1
                r0, r1, c0, c1, s0, t0 = self.place(k)
                if r0 < r1 and c0 < c1:
                    (self.air_n if self.airs[k] else self.ground_n)[r0:r1, c0:c1] += \
                        STAMP[s0:s0 + r1 - r0, t0:t0 + c1 - c0]
        self.played = max(self.played, played)
        total = self.ground_n + self.air_n
        colour = np.where((self.air_n >= self.ground_n)[..., None], rgb(style.WARM[0]), rgb(style.COOL[1]))
        alpha = np.minimum(1.0, np.log1p(total) / np.log1p(150)) * 0.35  # trails faint, so the Marios lead
        if self.played:  # the Marios themselves, where each lane is now
            for start in self.starts[:-1]:
                k = start + min(self.played, lane) - 1
                r0, r1, c0, c1, s0, t0 = self.place(k)
                if r0 < r1 and c0 < c1:
                    body = STAMP[s0:s0 + r1 - r0, t0:t0 + c1 - c0] > 0
                    colour[r0:r1, c0:c1][body] = rgb(style.WARM[0] if self.airs[k] else style.COOL[1])
                    alpha[r0:r1, c0:c1][body] = 0.95
        self.pixel_array = np.dstack([colour, alpha * 255 * self.level.get_value()]).round().astype(np.uint8)


class Tally(ImageMobject):
    """Beside the screen, row for row with it, a bar for each depth: how many of the crowd's frames had Mario's
    feet there, warm (in the air) then cool (on the ground). A bar too long for the space is broken near its end
    and its count written beside it."""

    def __init__(self, screen: Screen, crowd: Crowd, left: float):
        super().__init__(np.zeros((NES_H, TALLY_PX, 4), np.uint8))
        self.set_resampling_algorithm(RESAMPLING_ALGORITHMS["nearest"])
        self.stretch_to_fit_width(TALLY_W).stretch_to_fit_height(screen.game.height)
        self.move_to([left + TALLY_W / 2, screen.game.get_center()[1], 0])
        self.screen, self.crowd = screen, crowd
        self.level = ValueTracker(1.0)
        self.counts, self.shown = VGroup(), None
        self.add_updater(lambda m: m.draw())

    def draw(self) -> None:
        pixels = np.zeros((NES_H, TALLY_PX, 4))
        level, long = self.level.get_value(), []
        for depth in range(SWEEP_END + 1):
            air, ground = self.crowd.air_at[depth], self.crowd.ground_at[depth]
            if not air + ground:
                continue
            row, a = depth + OFFSET, round(air / FRAMES_PER_PX)
            end = min(round((air + ground) / FRAMES_PER_PX), TALLY_PX)
            pixels[row, :min(a, end), :3] = rgb(style.WARM[0])
            pixels[row, min(a, end):end, :3] = rgb(style.COOL[1])
            pixels[row, :end, 3] = 255 * level
            if end == TALLY_PX:  # broken near its end
                pixels[row, TALLY_PX - 18:TALLY_PX - 12, 3] = 0
                long.append((depth, int(air + ground)))
        self.pixel_array = pixels.round().astype(np.uint8)
        if long != self.shown:
            self.shown = long
            swap(self.counts, *[numeral(f"{n:,}", style.COOL[1], 0.32).next_to(
                [self.get_right()[0], self.screen.row_y(d + OFFSET + 0.5), 0], RIGHT, buff=0.12) for d, n in long])
        self.counts.set_opacity(level)


def choosing(scene) -> None:
    """The third tape set aside; a Mario on the game for every training frame, where he was, and beside it how
    many were at each depth: cool on the ground, nearly all at one depth. Then a setting sweeps down, its score
    against the answer key read as it goes."""
    tapes, counts = scene.kept.pop("tapes"), scene.kept.pop("counts")
    u = config.pixel_height / 1080
    screen = Screen(Recording("smb1"), CHOOSE_START, left_px=round(40 * u))
    score = scores()
    row = ValueTracker(0)
    line = Sweep(screen, 0, "setting", OFFSET)
    line.add_updater(lambda m: (setattr(m, "row", round(row.get_value())), m.refresh()))
    left = screen.game.get_right()[0] + 0.45
    crowd = Crowd(screen)
    tally = Tally(screen, crowd, left)
    top, bottom = screen.row_y(OFFSET), screen.row_y(SWEEP_END + OFFSET + 1)
    axis = Line([left, top, 0], [left, bottom, 0], stroke_color=style.ASH, stroke_width=1.0)
    name = serif("frames at each depth", style.ASH, 0.36).next_to([left, bottom, 0], DOWN + RIGHT, buff=0.2)
    across = Line(LEFT, RIGHT, stroke_color=style.WARM[2], stroke_width=1.2)
    across.add_updater(lambda m: m.put_start_and_end_on([left, line.y(), 0], [left + TALLY_W, line.y(), 0]))
    matched, shown = VGroup(numeral("0%", style.WARM[0], 0.42)), [None]
    reading = in_line(serif("matches the answer key", style.ASH, 0.38), matched, buff=0.15).next_to(
        [left, screen.game.get_top()[1], 0], DOWN + RIGHT, buff=0.05)

    def follow(m: VGroup) -> None:
        depth = round(row.get_value())
        if depth != shown[0]:
            shown[0] = depth
            swap(m, numeral(f"{round(score[depth] * 100)}%", style.WARM[0], 0.42).next_to(reading[0], RIGHT, buff=0.15))
            m.shift(UP * (baseline(reading[0]) - baseline(m)))
    follow(matched)  # drawn before it fades in: a fade copies what's there when it starts
    matched.add_updater(follow)
    dimmed = ValueTracker(1.0)
    # Named as the third slides apart, where the counts were: the first two train, the third tests.
    first, second = tapes.tapes[0].strip, tapes.tapes[1].strip
    edge = first.get_right()[0] + 0.35
    bracket = VMobject(stroke_color=style.ASH, stroke_width=1.2).set_points_as_corners(
        [[edge, first.get_top()[1], 0], [edge + 0.15, first.get_top()[1], 0],
         [edge + 0.15, second.get_bottom()[1], 0], [edge, second.get_bottom()[1], 0]])
    training = serif("training", style.TEXT, 0.42).next_to(bracket, RIGHT, buff=0.2)
    test = serif("test", style.TEXT, 0.42).next_to(tapes.tapes[2].strip.get_right() + APART, RIGHT, buff=0.5)
    small = tapes.copy()  # where the tapes go: small, in the corner, the third still apart
    small.tapes[2].shift(APART)
    small.scale(0.4).move_to(RIGHT * 5.0 + DOWN * 2.9)
    training_small = serif("training", style.TEXT, 0.34).next_to(small.tapes[0].strip, UP, buff=0.12,
                                                                 aligned_edge=LEFT)
    test_small = serif("test", style.TEXT, 0.34).next_to(small.tapes[2].strip, DOWN, buff=0.12)
    scene.add(crowd, tally, tally.counts)
    scene.kept.update(screen=screen, line=line, row=row, crowd=crowd, tally=tally, dimmed=dimmed,
                      chart=VGroup(axis, name, across, reading, training_small, test_small))
    scene.say("choosing", Succession(
        AnimationGroup(FadeOut(counts), tapes.apart(2), run_time=1.0),
        AnimationGroup(Create(bracket), FadeIn(training), FadeIn(test), run_time=0.6),
        Wait(0.6),
        AnimationGroup(Transform(tapes, small), ReplacementTransform(training, training_small),
                       ReplacementTransform(test, test_small), FadeOut(bracket), screen.arrive(run_time=1.0),
                       run_time=1.0),
        AnimationGroup(UpdateFromAlphaFunc(dimmed, lambda m, a: (m.set_value(1 - 0.65 * a), screen.fade(1 - 0.65 * a))),
                       Create(axis), FadeIn(name), run_time=0.6),
        tween(crowd.shown, 0.0, 1.0, 4.0, rate_func=linear),
        AnimationGroup(FadeIn(line), FadeIn(across), FadeIn(reading), run_time=0.5),
        tween(row, 0, SWEEP_END, 4.0, rate_func=linear),
        Wait(0.5)))
    scene.kept["tapes"] = tapes


def choosing_2(scene) -> None:
    """The setting settles at 176, where nearly all the cool is below it and the warm above; the crowd clears,
    the game comes back up, and Mario's feet come down onto the setting."""
    screen, row, crowd = scene.kept["screen"], scene.kept["row"], scene.kept["crowd"]
    feet, drawn = drawn_feet(screen)
    scene.add(feet)
    scene.kept["feet"] = feet
    scene.say("choosing-2", Succession(
        tween(row, SWEEP_END, CUT, 1.4),
        Wait(0.6),
        AnimationGroup(tween(crowd.level, 1.0, 0.0, 0.8),
                       UpdateFromAlphaFunc(scene.kept["dimmed"], lambda m, a: screen.fade(0.35 + 0.65 * a)),
                       run_time=0.8),
        tween(drawn, 0.0, 1.0, 0.8),
        Wait(1.0)))


def the_neuron(scene) -> None:
    """The setting across the game, 'in the air' above it and 'on the ground' below; beside it the neuron,
    reading Mario's depth and answering live."""
    screen, line, feet = scene.kept["screen"], scene.kept["line"], scene.kept["feet"]
    y, right = line.line.get_y(), screen.game.get_right()[0]
    above = serif("in the air", style.WARM[1], 0.4).next_to([right + 0.2, y, 0], UP + RIGHT, buff=0.1)
    below = serif("on the ground", style.COOL[1], 0.4).next_to([right + 0.2, y, 0], DOWN + RIGHT, buff=0.1)
    centre = RIGHT * 3.3 + UP * 0.6
    ring = Circle(radius=RING, stroke_color=style.TEXT, stroke_width=1.4).move_to(centre)
    rule = numeral(f"< {CUT}", style.TEXT, 0.42).move_to(centre)
    name = serif("depth", style.WARM[0], 0.42)
    reading, shown = VGroup(numeral(str(feet.value()), style.WARM[0], 0.42)), [feet.value()]
    source = in_line(name, reading, buff=0.12).move_to(centre + LEFT * 3.0)

    def follow(m: VGroup) -> None:
        if feet.value() != shown[0]:
            shown[0] = feet.value()
            swap(m, numeral(str(shown[0]), style.WARM[0], 0.42).next_to(name, RIGHT, buff=0.12))
            m.shift(UP * (baseline(name) - baseline(m)))
    path = Line(source.get_right() + RIGHT * 0.5, ring.get_left(), stroke_color=style.WARM[0], stroke_width=1.4)
    socket = Circle(radius=0.2, stroke_color=style.ASH, stroke_width=1.4).move_to(centre + RIGHT * 2.0)
    out = Line(ring.get_right(), socket.get_left(), stroke_color=style.ASH, stroke_width=1.4)
    lamp = Dot(socket.get_center(), radius=0.12, color=style.WARM[1])
    answer, said = VGroup(), [None]
    on = ValueTracker(0.0)

    def answering(m: VGroup) -> None:
        """Live once switched on (before, the reading would change under its own fade-in)."""
        if on.get_value() > 0:
            follow(reading)
        air = feet.value() < CUT
        if air != said[0]:
            said[0] = air
            swap(m, serif("in the air" if air else "on the ground", style.WARM[1] if air else style.COOL[1], 0.42)
                 .next_to(socket, DOWN, buff=0.3))
        m.set_opacity(on.get_value())
        lamp.set_opacity(on.get_value() if air else 0)
    answer.add_updater(answering)
    neuron = VGroup(ring, rule, path, out, socket)
    lamp.set_opacity(0)
    scene.add(lamp, answer)
    scene.kept.update(neuron=neuron, source=source, lamp=lamp, answer=answer, sides=VGroup(above, below))
    scene.say("the-neuron", Succession(
        AnimationGroup(FadeOut(scene.kept.pop("chart")), gone(scene.kept.pop("tally")), gone(scene.kept.pop("crowd")),
                       FadeOut(scene.kept.pop("tapes")), run_time=0.6),
        AnimationGroup(FadeIn(above), FadeIn(below), run_time=0.8),
        AnimationGroup(FadeIn(source), FadeIn(neuron), run_time=0.8),
        tween(on, 0.0, 1.0, 0.6),
        Wait(1.0)))


def building_end(scene) -> None:
    """The game goes; the neuron comes to the middle, its light saying 'in the air', a question mark beside it."""
    screen, neuron, source, lamp, answer = (scene.kept[k] for k in ("screen", "neuron", "source", "lamp", "answer"))
    reading = source[1]
    answer.clear_updaters()
    leave(scene, screen, scene.kept.pop("line"), scene.kept.pop("feet"), scene.kept.pop("sides"), reading, answer)
    shift = UP * 0.3 + RIGHT * 0.9 - neuron[0].get_center()
    socket = neuron[4]
    word = serif("in the air", style.WARM[1], 0.42).next_to(socket.get_center() + shift, DOWN, buff=0.5)
    unknown = serif("?", style.WARM[2], 0.7).next_to(socket.get_center() + shift, RIGHT, buff=0.45)
    scene.say("building-end", Succession(
        AnimationGroup(neuron.animate.shift(shift), source[0].animate.shift(shift),
                       lamp.animate.move_to(socket.get_center() + shift).set_opacity(1), FadeIn(word, shift=shift),
                       run_time=1.2),
        Wait(0.6),
        FadeIn(unknown, shift=LEFT * 0.1, run_time=0.8),
        Wait(1.5)))

BEATS = {"building-intro": building_intro, "thermostat": thermostat, "thermostat-2": thermostat_2,
         "simplest-neuron": simplest_neuron, "simplest-neuron-2": simplest_neuron_2, "question": question,
         "answer-key": answer_key, "answer-key-2": answer_key_2, "never-see": never_see, "why-key": why_key,
         "depth": depth,
         "recording": recording, "recording-2": recording_2, "frames": frames, "frames-2": frames_2,
         "frames-3": frames_3, "choosing": choosing, "choosing-2": choosing_2, "the-neuron": the_neuron,
         "building-end": building_end}
