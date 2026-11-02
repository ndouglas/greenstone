"""Chapter 2, "It never settles": why some moment is always wrong, who keeps the border moving, and what that
does to its score."""

import numpy as np
from manim import (DOWN, LEFT, RIGHT, UP, AnimationGroup, Circle, Create, FadeIn, FadeOut, Line,
                   Succession, UpdateFromAlphaFunc, ValueTracker, VGroup, VMobject, Wait, linear)

from series import style
from series.chart import Chart
from series.ember import in_line, numeral, serif, swap
from series.pointmap import PointMap
from series.props import Pad
from series.records import ScoreBars

from beats.common import (BOTH, BUTTONS, FILM, NUMBERS, Do, Readout, arrive_held, away, blocks, fill, new_map, pause,
                          resume, screen_at, show, speed, voice)
from beats.chapter_1 import counter

# Two moments at one point of the map (2.94 blocks, 0.75 pixels a frame), from the footage's session: in the
# first the player keeps holding Right and the screen moves; in the second he lets go, and it doesn't.
SAME_YES, SAME_NO = 601, 9105
LAST = 10


def percent(name: str) -> float:
    return float(NUMBERS[name].rstrip("%"))


def never_intro(scene) -> None:
    """A straight border, then a wiggly one: at the points where both answers happened, each is wrong somewhere."""
    m = new_map(at=DOWN * 0.3, width=7.6, height=4.4, line=FILM["search"])  # the title screen cleared the last
    xs = np.linspace(0, 5, 160)
    w, b = FILM["search"][:2], FILM["search"][2]
    base = (-b - w[0] * xs) / w[1]
    wiggle = VMobject(stroke_color=style.WARM[2], stroke_width=1.6)
    wiggle.set_points_smoothly([m.page((x, min(max(y + 0.35 * np.sin(5 * x), 0), 2.6))) for x, y in zip(xs, base)])
    talk = voice(scene, "never-intro")

    def ring(on: bool) -> None:
        m.ring[:] = BOTH if on else False
    scene.say("never-intro", Succession(
        away(scene), FadeIn(m, run_time=0.6), Do(lambda: ring(True)), Wait(talk * 0.35),
        AnimationGroup(FadeOut(m.border), Create(wiggle), run_time=1.6), Wait(max(talk * 0.65 - 2.2, 0.1))))
    scene.kept = {"map": m, "wiggle": wiggle}


def spots(scene) -> None:
    """Close in on part of the map: moments stacked on a grid of points, those holding both answers ringed."""
    x0, x1, y0, y1 = 2.0, 4.0, 0.0, 1.2
    pts, mv, st = FILM["spots"], FILM["spot_moved"], FILM["spot_still"]
    keep = (pts[:, 0] >= x0) & (pts[:, 0] <= x1) & (pts[:, 1] >= y0) & (pts[:, 1] <= y1)
    close = PointMap(pts[keep], mv[keep], st[keep], x_range=(x0, x1), y_range=(y0, y1), width=8.6, height=5.0,
                     at=DOWN * 0.2, mark=False, radii=(0.035, 0.09))
    close.ring[:] = BOTH[keep]
    close.redraw_dots(force=True)
    grid = VGroup()
    for k in range(int((x1 - x0) * 16) + 1):  # a column for each pixel, a row for each sixteenth of a pixel a frame
        x = x0 + k / 16
        grid.add(Line(close.page((x, y0)), close.page((x, y1)), stroke_color="#2A2226", stroke_width=0.6))
    for k in range(int((y1 - y0) * 16) + 1):
        y = y0 + k / 16
        grid.add(Line(close.page((x0, y)), close.page((x1, y)), stroke_color="#2A2226", stroke_width=0.6))
    tally = VGroup(in_line(numeral(f"{NUMBERS['spots']:,}", style.TEXT, 0.36), serif("points on the whole map", style.ASH, 0.3)),
                   in_line(numeral(f"{NUMBERS['spots_with_both']}", style.TEXT, 0.36), serif("of them held both answers", style.ASH, 0.3))
                   ).arrange(DOWN, aligned_edge=LEFT, buff=0.12).to_corner(UP + RIGHT, buff=0.3)
    talk = voice(scene, "spots")
    scene.say("spots", Succession(away(scene), FadeIn(grid, run_time=0.8), FadeIn(close, run_time=0.8),
                                  Wait(talk * 0.5), FadeIn(tally[0], run_time=0.5), Wait(0.8), FadeIn(tally[1], run_time=0.5)))


def unseen(scene) -> None:
    """The same point twice: in one the player holds Right and the screen moves; in the other he lets go."""
    leaving = away(scene)
    a = screen_at(SAME_YES + 1, LEFT * 3.4 + UP * 0.9)
    b = screen_at(SAME_NO + 1, RIGHT * 3.4 + UP * 0.9)
    for s in (a, b):
        s.scale(0.75)
    scene.add(a, b)
    pads = [Pad(s, s.get_bottom() + DOWN * 0.75, BUTTONS).scale(0.6) for s in (a, b)]
    for p, s in zip(pads, (a, b)):
        p.set_x(s.get_x())  # centred under its screen
    for p in pads:
        p.level.set_value(0)
    scene.add(*pads)
    same = serif(f"both: {NUMBERS_POINT}", style.TEXT, 0.3).move_to(UP * 3.6)
    verdicts = VGroup(serif("the screen moved", style.WARM[1], 0.32).next_to(pads[0], DOWN, buff=0.25),
                      serif("it didn't", style.COOL[1], 0.32).next_to(pads[1], DOWN, buff=0.25))
    held = VGroup(serif(f"at points like this, Right held: {NUMBERS['right_ahead_mixed_yes']} of the next half second when it moved,"
                        f" {NUMBERS['right_ahead_mixed_no']} when it didn't", style.ASH, 0.26)).move_to(DOWN * 3.6)
    talk = voice(scene, "unseen")
    watch = 30 / 60 / 0.5
    scene.say("unseen", Succession(
        leaving, AnimationGroup(arrive_held(a), arrive_held(b), run_time=0.6), FadeIn(same, run_time=0.4),
        AnimationGroup(*[UpdateFromAlphaFunc(p.level, lambda t, al: t.set_value(al)) for p in pads], run_time=0.4),
        Wait(max(talk * 0.3, 0.1)), resume(a, 0.5), resume(b, 0.5), Wait(watch), pause(a), pause(b),
        FadeIn(verdicts, run_time=0.5), Wait(0.6), FadeIn(held, run_time=0.6)))


NUMBERS_POINT = "2.94 blocks in, 0.75 pixels a frame"


def always_wrong(scene) -> None:
    """Late in the run: each nudge fixes one moment's side, and the border, moved, gets another wrong."""
    m = new_map(at=DOWN * 0.3, width=7.6, height=4.4)
    late = np.flatnonzero(FILM["run_pass"] == 25)[:10]
    after, moment, owner = FILM["run_after"], FILM["run_moment"], FILM["spot_of"]
    m.set_line(after[late[0] - 1])
    steps = []
    for n in late:
        def ring(n=n) -> None:
            m.ring[:] = False
            m.ring[owner[moment[n]]] = True
        steps += [Do(ring), Wait(0.5), m.turn_to(after[n]).set_run_time(0.45)]
    scene.say("always-wrong", Succession(*fill(scene, "always-wrong", away(scene), FadeIn(m, run_time=0.6),
                                                Wait(1.0), *steps)))
    scene.kept = {"map": m}


def who_moves_it(scene) -> None:
    """Each point sized by how many nudges its moments caused in the last ten passes; troublemakers warm-ringed."""
    counts = np.bincount(FILM["spot_of"], weights=FILM["late_nudges"], minlength=len(FILM["spots"]))
    m = new_map(at=DOWN * 0.3, width=7.6, height=4.4)
    m.level[:] = 0.2
    m.redraw_dots(force=True)
    top = counts.max()
    circles = VGroup(*[Circle(radius=0.03 + 0.32 * np.sqrt(c / top), stroke_color=style.TEXT, stroke_width=1.0,
                              stroke_opacity=0.8).move_to(m.page(p)) for p, c in zip(FILM["spots"], counts) if c > 0])
    words = VGroup(in_line(serif("troublemakers:", style.ASH, 0.3), numeral(NUMBERS["wrong_for_search"], style.TEXT, 0.34),
                           serif("of the moments", style.ASH, 0.3)),
                   in_line(serif("in the last ten passes,", style.ASH, 0.3), numeral(NUMBERS["wrong_share_of_nudges"], style.TEXT, 0.34),
                           serif("of the nudges", style.ASH, 0.3))).arrange(DOWN, aligned_edge=LEFT, buff=0.12)
    words.to_corner(UP + LEFT, buff=0.3)
    note = serif("circles: nudges caused, one run", style.ASH, 0.26).next_to(m, DOWN, buff=0.1).align_to(m, RIGHT)
    scene.say("who-moves-it", Succession(*fill(scene, "who-moves-it", away(scene), FadeIn(m, run_time=0.6),
                                                Create(circles, run_time=2.0), FadeIn(note, run_time=0.4),
                                                FadeIn(words[0], run_time=0.5), Wait(1.0), FadeIn(words[1], run_time=0.5))))


def bounded(scene) -> None:
    """The weights, nudge by nudge, as a path: the speed knob against the position knob, wandering, never still."""
    v = FILM["run_after"]
    reach = float(np.abs(v[:, :2]).max()) * 1.15
    c = Chart((-0.2 * reach, reach), (-0.2 * reach, reach), "position knob", "speed knob", width=5.4, height=5.4,
              at=LEFT * 1.0 + DOWN * 0.2)
    far = float(np.linalg.norm(v[:, :2], axis=1).max())
    origin = c.point(0, 0)
    unit = c.point(1, 0)[0] - origin[0]
    edge = Circle(radius=far * unit, stroke_color=style.ASH, stroke_width=1.0, stroke_opacity=0.7).move_to(origin)
    edge_text = serif("the furthest it went, in thirty passes", style.ASH, 0.26).next_to(edge, RIGHT, buff=0.1).shift(UP * 1.6)
    sample = v[::25]
    path, grow = c.grow(sample[:, 0], sample[:, 1], style.WARM[0], width=1.0)
    talk = voice(scene, "bounded")
    scene.say("bounded", Succession(away(scene), FadeIn(c, run_time=0.6), Create(edge, run_time=0.8),
                                    FadeIn(edge_text, run_time=0.4), grow.set_run_time(max(talk - 1.8, 1.0))))


def exam(scene) -> None:
    learned = VGroup(*[serif(f"recording {n}", style.TEXT, 0.3) for n in (1, 2, 4, 5)]).arrange(DOWN, buff=0.12)
    tested = VGroup(*[serif(f"recording {n}", style.WARM[2], 0.3) for n in (3, 6)]).arrange(DOWN, buff=0.12)
    heads = VGroup(serif("learned from", style.ASH, 0.32), serif("the exam: never learned from", style.ASH, 0.32))
    learned.move_to(LEFT * 3.0)
    tested.move_to(RIGHT * 3.0)
    heads[0].next_to(learned, UP, buff=0.3)
    heads[1].next_to(tested, UP, buff=0.3)
    scene.say("exam", Succession(*fill(scene, "exam", away(scene), FadeIn(VGroup(heads[0], learned), run_time=0.6),
                                        FadeIn(VGroup(heads[1], tested), run_time=0.6))))


def score_chart(at=RIGHT * 3.3 + DOWN * 0.2) -> Chart:
    return Chart((1, 30), (50, 100), "pass", "score on the exam", width=5.0, height=4.0, at=at,
                 x_ticks=[1, 10, 20, 30], y_ticks=[50, 75, 100], y_format=lambda v: f"{v:.0f}%")


def one_run(scene) -> None:
    m = new_map(at=LEFT * 3.4 + DOWN * 0.3, width=6.2, height=3.9)
    c = score_chart()
    ends, scores = FILM["run_ends"], FILM["run_scores"] * 100
    best = c.level(percent("search"), "best border", style.ASH)
    line, grow = c.grow(np.arange(1, 31), scores, style.WARM[0])
    count = ValueTracker(1)

    def follow(_, a: float) -> None:
        i = min(int(a * 30), 29)
        m.set_line(ends[i])
        count.set_value(i + 1)
    listed = serif("last ten: " + ", ".join(NUMBERS["perceptron_one_run"]), style.TEXT, 0.26).next_to(c, DOWN, buff=0.3)
    talk = voice(scene, "one-run")
    run = max(talk - 2.0, 1.0)
    scene.say("one-run", Succession(away(scene), FadeIn(m, run_time=0.5), FadeIn(c, run_time=0.4), FadeIn(best, run_time=0.3),
                                    AnimationGroup(grow, UpdateFromAlphaFunc(m.border, follow), run_time=run, rate_func=linear),
                                    FadeIn(listed, run_time=0.5)))
    scene.kept = {"map": m, "chart": c, "line": line, "listed": listed}


def order(scene) -> None:
    """Ten orders: their thirty passes' scores, and their final borders on the map."""
    k = scene.kept
    m, c = k["map"], k["chart"]
    scores = FILM["order_scores"] * 100
    lines = [c.series(np.arange(1, 31), s, style.WARM[0] if i == 0 else style.WARM[2], width=1.6 if i == 0 else 0.8)
             for i, s in enumerate(scores)]
    finals = FILM["order_ends"][:, -1]
    borders = VGroup()
    for v in finals[1:]:
        a, z = _ends(m, v)
        borders.add(Line(a, z, stroke_color=style.WARM[2], stroke_width=0.9, stroke_opacity=0.7))
    words = serif(f"nine ended between {NUMBERS['perceptron_second_lowest']} and {NUMBERS['perceptron_highest']};"
                  f" the one we followed, {NUMBERS['perceptron_lowest']}", style.TEXT, 0.28).to_edge(DOWN, buff=0.3)
    scene.say("order", Succession(*fill(scene, "order", FadeOut(k["listed"], run_time=0.3),
                                         Create(VGroup(*lines[1:]), run_time=2.4), Create(borders, run_time=1.6),
                                         FadeIn(words, run_time=0.6))))
    k.update(borders=borders)


def _ends(m: PointMap, v) -> tuple:
    old = m.v.copy()
    m.v = np.asarray(v, float)
    a, z = m.border_ends()
    m.v = old
    return m.page(a), m.page(z)


def dropped(scene) -> None:
    m = new_map(at=DOWN * 0.4, width=7.6, height=4.4, line=FILM["run_ends"][-1])
    borders = VGroup()
    for v in FILM["dropped"]:
        a, z = _ends(m, v)
        borders.add(Line(a, z, stroke_color=style.WARM[2], stroke_width=0.9, stroke_opacity=0.8))
    words = serif(f"{NUMBERS['dropped_moments']} of more than ten thousand moments left out, ten times:"
                  f" final scores {NUMBERS['perceptron_dropped_lowest']} to {NUMBERS['perceptron_dropped_highest']}",
                  style.TEXT, 0.28).to_edge(UP, buff=0.35)
    scene.say("dropped", Succession(*fill(scene, "dropped", away(scene), FadeIn(m, run_time=0.6), Wait(1.0),
                                           FadeIn(words, run_time=0.5), Create(borders, lag_ratio=0.3, run_time=3.0))))


def remember(scene) -> None:
    """The run's border swinging, with a still copy kept where it scored best on what it learned from; the
    pocket's score over the ten orders."""
    m = new_map(at=LEFT * 3.0 + DOWN * 0.3, width=6.6, height=4.1)
    ends = FILM["run_ends"]
    a, z = _ends(m, FILM["pocket"][0])
    kept = Line(a, z, stroke_color=style.WARM[0], stroke_width=2.2)
    kept_text = serif("kept", style.WARM[0], 0.28).next_to(kept.get_end(), UP, buff=0.08)
    bars = ScoreBars(percent("search"), "best border", at=np.array([3.8, -2.2, 0]), slots=2)
    pocket = f"{NUMBERS['pocket_lowest']}" if NUMBERS["pocket_lowest"] == NUMBERS["pocket_highest"] else \
        f"{NUMBERS['pocket_lowest']}–{NUMBERS['pocket_highest']}"
    words = serif(f"with a memory, over ten orders: {pocket}", style.TEXT, 0.3).to_edge(UP, buff=0.4).shift(RIGHT * 2.8)
    talk = voice(scene, "remember")

    def swing(_, al: float) -> None:
        m.set_line(ends[min(int(al * 60) % 30, 29)])
    scene.say("remember", AnimationGroup(away(scene), Succession(
        FadeIn(m, run_time=0.5), Wait(1.0), Create(kept, run_time=0.6), FadeIn(kept_text, run_time=0.3),
        FadeIn(bars, run_time=0.5), bars.bar(float(np.mean(FILM["pocket_scores"]) * 100), "with a memory").set_run_time(1.2),
        FadeIn(words, run_time=0.5), Wait(max(talk - 4.6, 0.1))),
        UpdateFromAlphaFunc(m.border, swing, run_time=talk, rate_func=linear)))
    scene.kept = {"map": m}


def never_outro(scene) -> None:
    m = scene.kept["map"]
    late = FILM["run_after"][FILM["run_pass"] >= 27][::9]
    scene.say("never-outro", Succession(away(scene, m), m.jump(late).set_run_time(max(voice(scene, "never-outro") - 0.5, 1.0))))


BEATS = {"never-intro": never_intro, "spots": spots, "unseen": unseen, "always-wrong": always_wrong,
         "who-moves-it": who_moves_it, "bounded": bounded, "exam": exam, "one-run": one_run, "order": order,
         "dropped": dropped, "remember": remember, "never-outro": never_outro}
