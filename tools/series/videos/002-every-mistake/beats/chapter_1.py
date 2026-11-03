"""Chapter 1, "Every mistake moves the line": the perceptron's rule, nudge by nudge, on examples and then on
Mario's moments; the step that doesn't matter; and the guarantee, checked."""

import numpy as np
from manim import (DOWN, LEFT, RIGHT, UP, AnimationGroup, Circle, Create, Dot, FadeIn, FadeOut, Group, Line,
                   Rectangle, RoundedRectangle, UpdateFromAlphaFunc, ValueTracker, VGroup, Wait, linear)

from series import style
from series.ember import in_line, numeral, serif, swap
from series.knobs import Knobs, Total
from series.lab import rules
from series.pointmap import PointMap

from beats.common import FILM, NUMBERS, Do, Succession, away, fill, new_map, voice
STEP = 0.05  # the examples' step: big enough to see
EXAMPLE = np.array([4.0, 1.0])  # rule-wrong's moment: four blocks in, one pixel a frame
BEFORE = np.array([0.3, 0.6, -1.9])  # weights that get it wrong: total −0.1, so no
FLIP = np.array([3.0, 2.0])  # rule-flip's moment: said yes, the screen stayed still
SCALES = (1.0, 1.0, 2.0)
KNOBS_AT = LEFT * 4.3 + UP * 1.5
MAP_AT = RIGHT * 2.0 + DOWN * 0.3


def counter(text: str, value: ValueTracker, at: np.ndarray, size: float = 0.36) -> VGroup:
    """An italic name and a whole number that follows a tracker."""
    group = VGroup()

    def draw(g) -> None:
        n = int(round(value.get_value()))
        if getattr(g, "shown", None) != n:
            swap(g, *in_line(serif(text, style.ASH, size * 0.9), numeral(str(n), style.TEXT, size), buff=0.16).submobjects)
            g.move_to(at)
            g.shown = n
    draw(group)
    group.add_updater(draw)
    return group


def example_map() -> PointMap:
    """Mario's map dimmed, as the ground the examples stand on, its border set by the examples' weights."""
    m = new_map(at=MAP_AT, width=6.8, height=4.2, line=BEFORE)
    m.level[:] = 0.22
    m.redraw_dots(force=True)
    return m


def example_dot(m: PointMap, p: np.ndarray, moved: bool) -> VGroup:
    """An example moment: a hollow ring in its outcome's colour, labelled as an example."""
    colour = style.WARM[0] if moved else style.COOL[1]
    ring = Circle(radius=0.11, stroke_color=colour, stroke_width=2.2).move_to(m.page(p))
    core = Dot(m.page(p), radius=0.05, color=colour)
    words = serif("an example", style.TEXT, 0.26).next_to(ring, UP + RIGHT, buff=0.04)
    return VGroup(ring, core, words)


def rule_intro(scene) -> None:
    knobs = Knobs(KNOBS_AT, values=BEFORE, scales=SCALES)
    total = Total(LEFT * 4.3 + DOWN * 1.6, scale=1.0, label="total")
    sum_text = VGroup(serif("position × its knob  +  speed × its knob  +  bias", style.TEXT, 0.34),
                      serif("above zero: yes", style.WARM[1], 0.34)).arrange(DOWN, buff=0.16).move_to(UP * 3.2)
    m = example_map()
    total.set_value(float(BEFORE[:2] @ EXAMPLE + BEFORE[2]))
    scene.say("rule-intro", Succession(*fill(scene, "rule-intro",
        away(scene), FadeIn(sum_text[0], run_time=0.8), FadeIn(knobs, run_time=0.8), FadeIn(m, run_time=0.8),
        Wait(1.2), FadeIn(total, run_time=0.6), FadeIn(sum_text[1], run_time=0.6))))
    scene.kept = {"knobs": knobs, "total": total, "map": m, "sum": sum_text}


def one_at_a_time(scene) -> None:
    """A real moment the examples' border already gets right: lit, its answer beside it, nothing moves."""
    k = scene.kept
    m = k["map"]
    x, y = FILM["x"], FILM["y"]
    says = (x @ BEFORE[:2] + BEFORE[2] > 0).astype(int)
    pick = int(np.flatnonzero((says == 1) & (y == 1) & (x[:, 0] > 4.4))[0])
    spot = FILM["spot_of"][pick]
    verdict = VGroup(serif("the neuron: yes", style.WARM[1], 0.32), serif("the recording: the screen moved", style.WARM[1], 0.32),
                     serif("right: nothing changes", style.TEXT, 0.32)).arrange(DOWN, aligned_edge=LEFT, buff=0.12)
    verdict.next_to(m, DOWN, buff=0.1).align_to(m, RIGHT)

    def light() -> None:
        m.ring[:] = False
        m.ring[spot] = True
        m.level[spot] = 1.0
    scene.say("one-at-a-time", Succession(*fill(scene, "one-at-a-time",
        Do(light), Wait(0.8), FadeIn(verdict[0], run_time=0.5), FadeIn(verdict[1], run_time=0.5), Wait(0.8),
        FadeIn(verdict[2], run_time=0.5),
        UpdateFromAlphaFunc(k["total"], lambda t, a: t.set_value(float(BEFORE[:2] @ x[pick] + BEFORE[2])), run_time=0.5))))
    k.update(verdict=verdict, spot=spot)


def nudge_labels(knobs: Knobs, p: np.ndarray, sign: str) -> VGroup:
    """Beside each knob, what the nudge adds: the step times its own number."""
    parts = VGroup()
    for dial, n in zip(knobs.dials, list(p) + [1.0]):
        parts.add(serif(f"{sign} step × {n:g}", style.WARM[1] if sign == "+" else style.COOL[1], 0.26)
                  .next_to(dial.ring, UP, buff=0.14))
    return parts


def rule_wrong(scene) -> None:
    k = scene.kept
    m, knobs, total = k["map"], k["knobs"], k["total"]
    m.ring[:] = False
    dot = example_dot(m, EXAMPLE, moved=True)
    reading = serif("4 blocks, 1 pixel a frame: the neuron said no; the screen moved", style.TEXT, 0.3)
    reading.next_to(m, DOWN, buff=0.1).align_to(m, RIGHT)
    adds = nudge_labels(knobs, EXAMPLE, "+")
    after = BEFORE + STEP * np.r_[EXAMPLE, 0.0]  # the weights only: the bias is the next beat's
    scene.say("rule-wrong", Succession(*fill(scene, "rule-wrong",
        FadeOut(k["verdict"], run_time=0.4), Do(lambda: m.level.__setitem__(k["spot"], 0.22)),
        UpdateFromAlphaFunc(total, lambda t, a: t.set_value(float(BEFORE[:2] @ EXAMPLE + BEFORE[2])), run_time=0.3),
        FadeIn(dot, run_time=0.6), FadeIn(reading, run_time=0.6), Wait(2.6),
        FadeIn(adds[0], run_time=0.4), FadeIn(adds[1], run_time=0.4),
        AnimationGroup(knobs.turn_to(after, total, EXAMPLE), m.turn_to(after), run_time=2.0))))
    k.update(dot=dot, reading=reading, adds=adds, now=after)


def rule_bias(scene) -> None:
    k = scene.kept
    after = k["now"] + np.array([0, 0, STEP])
    scene.say("rule-bias", Succession(*fill(scene, "rule-bias",
        FadeIn(k["adds"][2], run_time=0.4),
        AnimationGroup(k["knobs"].turn_to(after, k["total"], EXAMPLE), k["map"].turn_to(after), run_time=1.2))))
    k["now"] = after


def rule_flip(scene) -> None:
    k = scene.kept
    m, knobs, total, now = k["map"], k["knobs"], k["total"], k["now"]
    dot = example_dot(m, FLIP, moved=False)
    reading = serif("3 blocks, 2 pixels a frame: the neuron said yes; the screen stayed still", style.TEXT, 0.3)
    reading.next_to(m, DOWN, buff=0.1).align_to(m, RIGHT)
    takes = nudge_labels(knobs, FLIP, "−")
    after = now - STEP * np.r_[FLIP, 1.0]
    scene.say("rule-flip", Succession(*fill(scene, "rule-flip",
        FadeOut(VGroup(k["dot"], k["reading"], k["adds"]), run_time=0.4),
        UpdateFromAlphaFunc(total, lambda t, a: t.set_value(float(now[:2] @ FLIP + now[2])), run_time=0.3),
        FadeIn(dot, run_time=0.5), FadeIn(reading, run_time=0.5), Wait(1.0), FadeIn(takes, run_time=0.4),
        AnimationGroup(knobs.turn_to(after, total, FLIP), m.turn_to(after), run_time=1.8))))
    k.update(dot=dot, reading=reading, adds=takes, now=after, flip_before=now)


def rule_why(scene) -> None:
    """Both examples again, slowly: each moment's total moving toward its own side, and the border toward it."""
    k = scene.kept
    m, knobs, total = k["map"], k["knobs"], k["total"]
    first_after = BEFORE + STEP * np.r_[EXAMPLE, 1.0]
    flip_after = k["now"]
    wrong = example_dot(m, EXAMPLE, moved=True)
    note = serif("toward its side of zero", style.TEXT, 0.3).next_to(total, RIGHT, buff=0.5)
    talk = voice(scene, "rule-why")
    half = (talk - 1.4) / 2
    scene.say("rule-why", Succession(
        FadeOut(VGroup(k["reading"], k["adds"]), run_time=0.4), FadeIn(note, run_time=0.4),
        # the moment that moved: from below zero, up
        AnimationGroup(FadeOut(k["dot"]), FadeIn(wrong), knobs.turn_to(BEFORE), m.turn_to(BEFORE),
                       UpdateFromAlphaFunc(total, lambda t, a: t.set_value(float(BEFORE[:2] @ EXAMPLE + BEFORE[2]))),
                       run_time=0.6),
        AnimationGroup(knobs.turn_to(first_after, total, EXAMPLE), m.turn_to(first_after), run_time=half - 0.6),
        # the moment that didn't: from above zero, down
        AnimationGroup(FadeOut(wrong), FadeIn(k["dot"]), knobs.turn_to(k["flip_before"]), m.turn_to(k["flip_before"]),
                       UpdateFromAlphaFunc(total, lambda t, a: t.set_value(float(k["flip_before"][:2] @ FLIP + k["flip_before"][2]))),
                       run_time=0.6),
        AnimationGroup(knobs.turn_to(flip_after, total, FLIP), m.turn_to(flip_after), run_time=half - 0.6)))
    k["note"] = note


def rule_name(scene) -> None:
    card = VGroup(serif("the perceptron's learning rule", style.WARM[2], 0.5),
                  serif("wrong: each knob turns by the step × its own number", style.TEXT, 0.36),
                  serif("up if the screen moved, down if it didn't", style.TEXT, 0.36),
                  in_line(serif("Frank Rosenblatt:", style.ASH, 0.34), serif("the perceptron, 1958; rules like this, by 1962", style.ASH, 0.34)),
                  ).arrange(DOWN, buff=0.3).move_to(UP * 0.3)
    scene.say("rule-name", Succession(*fill(scene, "rule-name", away(scene), FadeIn(card[0], run_time=0.6),
                                            FadeIn(card[1:3], run_time=0.8), Wait(0.6), FadeIn(card[3], run_time=0.8))))
    scene.kept = {"card": card}


def mark_one() -> VGroup:
    """The Mark I's knob racks, drawn: a cabinet of rows of round knobs, each with its pointer and a small motor
    box behind it. A drawing, not a likeness of any photograph."""
    cabinet = RoundedRectangle(corner_radius=0.06, width=4.4, height=3.4, stroke_color=style.ASH, stroke_width=1.4)
    knobs = VGroup()
    rng = np.random.default_rng(1)
    for r in range(4):
        for c in range(6):
            at = np.array([-1.75 + c * 0.7, 1.05 - r * 0.72, 0])
            motor = Rectangle(width=0.4, height=0.18, stroke_color="#5A4636", stroke_width=1.0).move_to(at + DOWN * 0.27)
            knob = Circle(radius=0.2, stroke_color=style.ASH, stroke_width=1.2).move_to(at)
            a = rng.uniform(-2.2, 2.2)
            pointer = Line(at, at + 0.17 * np.array([np.sin(a), np.cos(a), 0]), stroke_color=style.WARM[0], stroke_width=1.4)
            knobs.add(VGroup(motor, knob, pointer))
    return VGroup(cabinet, knobs)


def mark_one_beat(scene) -> None:
    rack = mark_one().move_to(LEFT * 2.6 + DOWN * 0.1)
    words = VGroup(serif("Mark I Perceptron", style.WARM[2], 0.4),
                   serif("its weights: knobs turned by motors", style.TEXT, 0.32),
                   serif("(a drawing)", style.ASH, 0.26)).arrange(DOWN, aligned_edge=LEFT, buff=0.14).move_to(RIGHT * 3.2 + UP * 0.6)
    knobs = Knobs(RIGHT * 3.2 + DOWN * 1.6, values=(0.4, 0.7, -1.9), scales=SCALES)
    pointers = [g[2] for g in rack[1]]
    centres = [g[1].get_center() for g in rack[1]]
    rng = np.random.default_rng(4)
    targets = rng.uniform(-2.2, 2.2, len(pointers))
    starts = [np.arctan2(p.get_end()[0] - c[0], p.get_end()[1] - c[1]) for p, c in zip(pointers, centres)]

    def turn(_, a: float) -> None:
        for p, c, s, t in zip(pointers, centres, starts, targets):
            ang = s + (t - s) * a
            p.put_start_and_end_on(c, c + 0.17 * np.array([np.sin(ang), np.cos(ang), 0]))
    scene.say("mark-one", Succession(*fill(scene, "mark-one", away(scene), Create(rack, run_time=1.6),
                                           FadeIn(words, run_time=0.6), FadeIn(knobs, run_time=0.5),
                                           UpdateFromAlphaFunc(rack, turn, run_time=2.0))))


def passes(scene) -> None:
    """The run shown, pass after pass: each nudge's moment lit as it moves the border, a counter to thirty."""
    m = new_map(at=DOWN * 0.3, width=7.6, height=4.4)
    count = ValueTracker(1)
    shown = counter("pass", count, UP * 3.1 + RIGHT * 3.0)
    after, moment, when = FILM["run_after"], FILM["run_moment"], FILM["run_pass"]
    owner = FILM["spot_of"]
    talk = voice(scene, "passes")
    play = talk - 1.0
    first = np.flatnonzero(when == 0)[:24]  # the first nudges, one at a time
    rest = np.flatnonzero(when > 0)[::60]  # then the rest, faster and faster, sampled
    order = np.r_[first, rest]
    times = np.r_[np.full(len(first), 0.4 * play / len(first)), np.full(len(rest), 0.6 * play / len(rest))]
    clock = np.r_[0, np.cumsum(times)] / times.sum()

    def step(_, a: float) -> None:
        i = min(np.searchsorted(clock, a, side="right") - 1, len(order) - 1)
        n = order[i]
        m.set_line(after[n])
        m.ring[:] = False
        m.ring[owner[moment[n]]] = True
        count.set_value(when[n] + 1)
    scene.say("passes", Succession(away(scene), FadeIn(m, run_time=0.5), FadeIn(shown, run_time=0.3),
                                   UpdateFromAlphaFunc(m.border, step, run_time=play, rate_func=linear),
                                   Do(lambda: (m.ring.__setitem__(slice(None), False), count.set_value(30), m.set_line(after[-1])))))
    scene.kept = {"map": m, "count": shown}


def step_size_2(scene) -> None:
    m = scene.kept["map"]
    v = m.v.copy()
    knobs = Knobs(LEFT * 0.0 + UP * 3.0, values=v, scales=(0.3, 0.6, 2.0))
    sign = serif("× 2", style.WARM[2], 0.4).next_to(knobs, RIGHT, buff=0.4)
    talk = voice(scene, "step-size-2")
    scene.say("step-size-2", Succession(*fill(scene, "step-size-2",
        FadeOut(scene.kept["count"], run_time=0.3), m.animate(run_time=0.6).shift(DOWN * 0.35).scale(0.9),
        FadeIn(knobs, run_time=0.5), Wait(max(talk - 5.2, 0.1)), FadeIn(sign, run_time=0.3),
        AnimationGroup(knobs.turn_to(2 * v), m.turn_to(2 * v), run_time=1.2), Wait(0.6),
        Do(lambda: swap(sign, serif("× ½", style.WARM[2], 0.4).next_to(knobs, RIGHT, buff=0.4))),
        AnimationGroup(knobs.turn_to(v), m.turn_to(v), run_time=1.2))))
    scene.kept.update(knobs=knobs)


def step_size(scene) -> None:
    """The run shown and the same run with eight times the step, side by side, pass by pass."""
    small, big = FILM["run_ends"], FILM["run_bigger_ends"]
    left = new_map(at=LEFT * 3.5 + DOWN * 0.7, width=5.6, height=3.4, line=small[0])
    right = new_map(at=RIGHT * 3.5 + DOWN * 0.7, width=5.6, height=3.4, line=big[0])
    kl = Knobs(LEFT * 3.5 + UP * 2.4, values=small[0], scales=(0.3, 0.6, 2.0), gap=1.2)
    kr = Knobs(RIGHT * 3.5 + UP * 2.4, values=big[0], scales=(2.4, 4.8, 16.0), gap=1.2)
    for k in (kl, kr):
        k.scale(0.8)
    names = VGroup(serif("the step", style.TEXT, 0.32).next_to(kl, UP, buff=0.12),
                   serif(f"{int(FILM['bigger'])} times the step", style.TEXT, 0.32).next_to(kr, UP, buff=0.12))
    p = ValueTracker(1)
    count = counter("pass", p, DOWN * 3.3)

    def go(_, a: float) -> None:
        i = min(int(a * len(small)), len(small) - 1)
        left.set_line(small[i])
        right.set_line(big[i])
        kl.set_values(small[i])
        kr.set_values(big[i])
        p.set_value(i + 1)
    talk = voice(scene, "step-size")
    scene.say("step-size", Succession(away(scene), FadeIn(left, run_time=0.6), FadeIn(right, run_time=0.6),
                                      FadeIn(VGroup(kl, kr, names), run_time=0.6), FadeIn(count, run_time=0.3),
                                      Wait(max(talk * 0.3, 0.1)),
                                      UpdateFromAlphaFunc(left.border, go, run_time=max(talk * 0.7 - 1.5, 1.0), rate_func=linear)))


def toy() -> tuple[np.ndarray, np.ndarray, list]:
    """Two groups a straight border can split, and the rule's border after each of its nudges until it stops."""
    rng = np.random.default_rng(8)
    a = rng.normal([1.5, 0.7], [0.45, 0.3], (18, 2))
    b = rng.normal([3.6, 1.8], [0.45, 0.3], (18, 2))
    x, y = np.r_[a, b], np.r_[np.zeros(18, int), np.ones(18, int)]
    line, path = rules.Line(np.zeros(2), 0.0), []
    for _ in range(200):
        moved = False
        for i in np.random.default_rng(0).permutation(len(y)):
            new = rules.step("perceptron", line, x[i], int(y[i]), 0.1)
            if new is not line:
                path.append(np.r_[new.w, new.b])
                moved = True
            line = new
        if not moved:
            break
    return x, y, path


def guarantee(scene) -> None:
    x, y, path = toy()
    m = PointMap(x, y, 1 - y, at=DOWN * 0.3, width=7.0, height=4.2, x_label="one number", y_label="another", mark=False,
                 radii=(0.07, 0.07))
    note = serif("an example: made-up dots that a straight border can split", style.ASH, 0.28).next_to(m, UP, buff=0.3)
    stopped = serif("no mistakes left: it stops", style.TEXT, 0.32).next_to(m, DOWN, buff=0.35)
    talk = voice(scene, "guarantee")
    scene.say("guarantee", Succession(away(scene), FadeIn(m, run_time=0.6), FadeIn(note, run_time=0.4),
                                      m.jump(np.array(path)).set_run_time(max(talk - 2.2, 1.0)),
                                      FadeIn(stopped, run_time=0.5)))


def control_map() -> PointMap:
    split = FILM["control_split"]
    x, y = FILM["x"][split], FILM["y"][split]
    spots, owner = np.unique(x, axis=0, return_inverse=True)
    owner = owner.ravel()
    moved = np.bincount(owner, weights=y, minlength=len(spots)).astype(int)
    still = np.bincount(owner, minlength=len(spots)) - moved
    return PointMap(spots, moved, still, at=DOWN * 0.3, width=7.6, height=4.4)


def guarantee_check(scene) -> None:
    full = new_map(at=DOWN * 0.3, width=7.6, height=4.4)
    m = control_map()
    ends = FILM["control_ends"]
    last = int(NUMBERS["control_last_moving_pass"])
    p = ValueTracker(1)
    count = counter("pass", p, UP * 3.1 + RIGHT * 3.0)
    stopped = serif(f"still from pass {last}: every dot on its side", style.TEXT, 0.32).next_to(m, DOWN, buff=0.45)
    removed = serif("without the moments the best border gets wrong", style.ASH, 0.3).next_to(m, UP, buff=0.35).shift(LEFT * 1.5)

    def go(_, a: float) -> None:
        i = min(int(a * (last + 20)), len(ends) - 1)  # up to a little past the last pass that moved
        m.set_line(ends[i])
        p.set_value(i + 1)
    talk = voice(scene, "guarantee-check")
    scene.say("guarantee-check", Succession(
        away(scene), FadeIn(full, run_time=0.5), Wait(1.2),
        AnimationGroup(FadeOut(full), FadeIn(m), FadeIn(removed), run_time=1.4), FadeIn(count, run_time=0.3),
        UpdateFromAlphaFunc(m.border, go, run_time=max(talk - 4.4, 1.0), rate_func=linear), FadeIn(stopped, run_time=0.5)))
    scene.kept = {"map": m, "count": count, "stopped": stopped, "removed": removed}


def rule_outro(scene) -> None:
    k = scene.kept
    full = new_map(at=DOWN * 0.3, width=7.6, height=4.4, line=k["map"].v)
    scene.say("rule-outro", Succession(*fill(scene, "rule-outro", Wait(2.0),
        AnimationGroup(FadeOut(k["count"]), FadeOut(k["stopped"]), FadeOut(k["removed"]), run_time=0.4),
        AnimationGroup(FadeOut(k["map"]), FadeIn(full), run_time=2.0))))
    scene.kept = {"map": full}


BEATS = {"rule-intro": rule_intro, "one-at-a-time": one_at_a_time, "rule-wrong": rule_wrong, "rule-bias": rule_bias,
         "rule-flip": rule_flip, "rule-why": rule_why, "rule-name": rule_name, "mark-one": mark_one_beat,
         "passes": passes, "step-size-2": step_size_2, "step-size": step_size, "guarantee": guarantee,
         "guarantee-check": guarantee_check, "rule-outro": rule_outro}
