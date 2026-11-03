"""Chapter 2, "It never settles": why some moment is always wrong, who keeps the border moving, and what that
does to its score."""

import numpy as np
from manim import (DOWN, LEFT, RIGHT, UP, AnimationGroup, Circle, Rectangle, RoundedRectangle, Create, FadeIn, FadeOut, Line, Mobject, Transform, UpdateFromAlphaFunc,
                   ValueTracker, VGroup, VMobject, Wait, linear)

from series import style
from series.chart import Chart
from series.ember import in_line, numeral, serif, swap
from series.pointmap import PointMap
from series.props import Pad
from series.footage import NES_FPS
from series.lab.exp002 import POSITION, SCROLL, SOON, SPEED
from series.records import ScoreBars

from beats.common import (BOTH, BUTTONS, FILM, NUMBERS, Do, Readout, Succession, arrive_held, away, blocks, fill,
                          hold, new_map, pause, play_frames, recording, resume, screen_at, show, speed, voice)
from beats.chapter_1 import counter

# Two moments at one point of the map (3.31 blocks, 0.81 pixels a frame), from the footage's session: in the
# first the player keeps holding Right and the screen moves 23 frames on; in the second he lets go, Mario slides
# to a stop, and the screen holds still for 58 frames, well past the half second the beat shows.
SAME_YES, SAME_NO = 13473, 13020
SAME_POINT = (53 / 16, 13 / 16)  # in blocks and pixels a frame
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
    # one point holding both answers (unseen's pair's), alone, and a border bending first one way round it, then
    # the other: whichever side it lands on, every moment there gets that side's answer
    focus = (pts[keep, 0] == SAME_POINT[0]) & (pts[keep, 1] == SAME_POINT[1])
    yes, no = int(mv[keep][focus][0]), int(st[keep][focus][0])
    x0p, y0p = SAME_POINT

    def edge(xs):  # the border, straight, through the point
        return y0p - 0.22 * (np.asarray(xs) - x0p)

    def bend(side: float) -> VMobject:
        xs = np.linspace(x0, x1, 120)
        ys = np.clip(edge(xs) + side * 0.26 * np.exp(-((xs - x0p) / 0.22) ** 2), y0, y1)
        curve = VMobject(stroke_color=style.WARM[2], stroke_width=2.2)
        curve.set_points_smoothly([close.page((x, y)) for x, y in zip(xs, ys)])
        return curve
    under, over = bend(-1), bend(1)  # the point above the border (the yes side), then below it
    spot = close.page(SAME_POINT)
    here = serif(f"this point: {yes} moments the screen moved, {no} it didn't", style.TEXT, 0.28)
    here.move_to(close.page((3.45, 0.32)))  # an empty corner, well below the border
    pointer = Line(here.get_top() + UP * 0.06, spot + DOWN * 0.12, stroke_color=style.ASH, stroke_width=1.2)
    sides = VGroup(serif("yes", style.WARM[1], 0.28).move_to(close.page((2.25, float(edge(2.25)) + 0.12))),
                   serif("no", style.COOL[1], 0.28).move_to(close.page((2.25, float(edge(2.25)) - 0.12))))
    verdict_yes = serif(f"all {yes + no} get yes: the {no} that didn't move are wrong", style.WARM[1], 0.28)
    verdict_no = serif(f"all {yes + no} get no: the {yes} that moved are wrong", style.COOL[1], 0.28)
    for v in (verdict_yes, verdict_no):
        v.next_to(here, DOWN, buff=0.14).align_to(here, LEFT)
    talk = voice(scene, "spots")
    text = next(l for l in scene.video.lines if l.id == "spots").text
    turn = talk * text.index("A border") / len(text)  # when the last sentence starts
    scene.say("spots", Succession(
        away(scene), FadeIn(grid, run_time=0.8), FadeIn(close, run_time=0.8),
        Wait(talk * 0.36), FadeIn(tally[0], run_time=0.5), Wait(0.8), FadeIn(tally[1], run_time=0.5),
        Wait(max(turn - talk * 0.36 - 3.4, 0.1)),
        AnimationGroup(close.fade_points(~focus, 0.18), FadeOut(tally), FadeIn(here, pointer), run_time=0.8),
        Create(under, run_time=1.0), FadeIn(sides, run_time=0.4), FadeIn(verdict_yes, run_time=0.4), Wait(1.4),
        AnimationGroup(Transform(under, over), FadeOut(verdict_yes), run_time=1.2), FadeIn(verdict_no, run_time=0.4)))


def unseen(scene) -> None:
    """The same point twice: in one the player holds Right and the screen moves; in the other he lets go."""
    leaving = away(scene)
    _check_pair()
    a = screen_at(SAME_YES + 1, LEFT * 3.4 + UP * 1.2)
    b = screen_at(SAME_NO + 1, RIGHT * 3.4 + UP * 1.2)
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
                        f" {NUMBERS['right_ahead_mixed_no']} when it didn't", style.ASH, 0.26)).next_to(verdicts, DOWN, buff=0.3)
    talk = voice(scene, "unseen")
    watch = SOON / NES_FPS / 0.5  # the half second, at half speed
    scene.say("unseen", Succession(
        leaving, AnimationGroup(arrive_held(a), arrive_held(b), run_time=0.6), FadeIn(same, run_time=0.4),
        AnimationGroup(*[UpdateFromAlphaFunc(p.level, lambda t, al: t.set_value(al)) for p in pads], run_time=0.4),
        Wait(max(talk * 0.3, 0.1)), AnimationGroup(play_frames(a, SOON, watch), play_frames(b, SOON, watch)), hold(a, b),
        FadeIn(verdicts, run_time=0.5), Wait(0.6), FadeIn(held, run_time=0.6)))


NUMBERS_POINT = "3.31 blocks in, 0.81 pixels a frame"


def _check_pair() -> None:
    """The two moments sit at NUMBERS_POINT, and the half second shown proves each verdict with frames to spare:
    one screen scrolls inside it, the other doesn't scroll for a while after it."""
    ram = recording().ram
    a, b = ram[SAME_YES], ram[SAME_NO]
    point = f"{a[POSITION] / 16:.2f} blocks in, {np.int8(a[SPEED]) / 16:.2f} pixels a frame"
    if (a[POSITION], a[SPEED]) != (b[POSITION], b[SPEED]) or point != NUMBERS_POINT:
        raise ValueError(f"unseen's two moments aren't both at {NUMBERS_POINT} (the first is at {point})")
    if not (ram[SAME_YES:SAME_YES + SOON - 3, SCROLL] != 0).any():
        raise ValueError("unseen's first screen doesn't scroll in time to show it")
    if (ram[SAME_NO:SAME_NO + SOON * 3 // 2, SCROLL] != 0).any():
        raise ValueError("unseen's second screen scrolls too soon after the half second shown")


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


def _spots(x: np.ndarray, y: np.ndarray) -> tuple:
    """A map's points: each different (position, speed), its moments of each answer, and which point each
    moment is at."""
    spots, owner = np.unique(x, axis=0, return_inverse=True)
    owner = owner.ravel()
    moved = np.bincount(owner, weights=y, minlength=len(spots)).astype(int)
    return spots, moved, np.bincount(owner, minlength=len(spots)) - moved, owner


def towards(m: Mobject, to: np.ndarray | None = None, scale: float = 1.0, fade: bool = False, run_time: float = 0.5,
            by: np.ndarray | None = None) -> UpdateFromAlphaFunc:
    """Move (and shrink, and fade) a mobject to a spot, or `by` a shift; it sets out from wherever it is when the
    move begins."""
    began: dict = {}

    def update(d, a: float) -> None:
        if not began:
            began.update(at=d.get_center(), copy=d.copy())
        end = began["at"] + by if to is None else to
        d.become(began["copy"]).scale(1 + (scale - 1) * a).move_to(began["at"] + (end - began["at"]) * a)
        if fade:
            d.set_opacity(1 - a)
    return UpdateFromAlphaFunc(m, update, run_time=run_time)


def exam(scene) -> None:
    """The six recordings as tapes, each as long as its moments: 1, 2, 4 and 5 pour into the map it learns
    from; 3 and 6 into an exam map behind glass. After a pass, its border is copied across, the exam's points
    with a wrong moment are ringed, and the score is read off."""
    names = ("1", "2", "4", "5", "3", "6")
    sizes = np.r_[FILM["train_sizes"], FILM["test_sizes"]]
    longest = float(sizes.max())
    tapes = VGroup()
    for k, (name, n) in enumerate(zip(names, sizes)):
        y = 2.3 - k * 0.78 - (0.35 if k >= 4 else 0)
        strip = Rectangle(width=2.0 * n / longest, height=0.32, stroke_color=style.ASH, stroke_width=1.0,
                          fill_color=style.WARM[2] if k >= 4 else style.ASH, fill_opacity=0.25)
        strip.move_to([-6.8 + strip.width / 2, y, 0])
        label = serif(f"recording {name}", style.TEXT, 0.24).next_to(strip, UP, buff=0.06).align_to(strip, LEFT)
        count = numeral(f"{n:,} moments", style.ASH, 0.2).next_to(strip, RIGHT, buff=0.1)
        tapes.add(VGroup(strip, label, count))
    learn = new_map(at=LEFT * 1.0 + DOWN * 0.3, width=3.7, height=2.6, line=np.zeros(3), mark=False)
    spots, moved, still, owner = _spots(FILM["xt"], FILM["yt"])
    test = PointMap(spots, moved, still, at=RIGHT * 4.4 + DOWN * 0.3, width=3.7, height=2.6, line=np.zeros(3), mark=False)
    for m in (learn, test):
        m.level[:] = 0.0
        m.redraw_dots(force=True)
    glass = RoundedRectangle(corner_radius=0.18, width=5.0, height=4.0, stroke_color=style.ASH, stroke_width=1.2,
                             fill_color=style.TEXT, fill_opacity=0.035).move_to(test.get_center() + LEFT * 0.25 + DOWN * 0.1)
    heads = VGroup(serif("what it learns from", style.ASH, 0.3).next_to(learn, UP, buff=0.35),
                   serif("the exam: never learned from", style.WARM[2], 0.3).next_to(glass, UP, buff=0.15))
    _, _, _, learn_owner = _spots(FILM["x"], FILM["y"])
    ends = np.cumsum(np.r_[0, FILM["train_sizes"]])
    test_ends = np.cumsum(np.r_[0, FILM["test_sizes"]])

    def pour(k: int):
        """Tape k goes into its map, and the points its moments are at light."""
        m, (a, z), owners = (learn, (ends[k], ends[k + 1]), learn_owner) if k < 4 else \
            (test, (test_ends[k - 4], test_ends[k - 3]), owner)
        strip = tapes[k][0].copy()

        def light() -> None:
            m.level[np.unique(owners[a:z])] = 1.0
            m.redraw_dots()
        return Succession(towards(strip, m.get_center(), 0.15, True, run_time=0.45), Do(light), Wait(0.05))
    first = FILM["run_ends"][0]
    wrong = np.zeros(len(spots), bool)
    said = (FILM["xt"] @ first[:2] + first[2] > 0).astype(int)
    wrong[np.unique(owner[said != FILM["yt"]])] = True
    score = FILM["run_scores"][0] * 100

    def ring() -> None:
        test.ring[:] = wrong
        test.redraw_dots()
    copy = learn.border.copy()
    result = serif(f"after pass 1: {score:.0f}% right on the exam", style.TEXT, 0.3).next_to(glass, DOWN, buff=0.2)
    scene.say("exam", Succession(*fill(scene, "exam", away(scene), FadeIn(tapes, run_time=0.4), FadeIn(learn, test, glass, heads, run_time=0.4),
                                        *[pour(k) for k in range(6)], Do(lambda: learn.set_line(first)),
                                        Do(lambda: copy.become(learn.border.copy())),
                                        towards(copy, by=test.at - learn.at, run_time=0.6),
                                        Do(lambda: test.set_line(first)), FadeOut(copy, run_time=0.1), Do(ring), Wait(0.3),
                                        FadeIn(result, run_time=0.4))))


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
    scene.say("remember", Succession(away(scene), FadeIn(m, run_time=0.5), AnimationGroup(
        UpdateFromAlphaFunc(Mobject(), swing, rate_func=linear),
        Succession(Wait(1.0), Create(kept, run_time=0.6), FadeIn(kept_text, run_time=0.3), FadeIn(bars, run_time=0.5),
                   bars.bar(float(np.mean(FILM["pocket_scores"]) * 100), "with a memory").set_run_time(1.2),
                   FadeIn(words, run_time=0.5), Wait(max(talk - 5.1, 0.1))),
        run_time=talk - 1.0)))
    scene.kept = {"map": m}


def never_outro(scene) -> None:
    m = scene.kept["map"]
    late = FILM["run_after"][FILM["run_pass"] >= 27][::9]
    scene.say("never-outro", Succession(away(scene, m), m.jump(late).set_run_time(max(voice(scene, "never-outro") - 0.5, 1.0))))


BEATS = {"never-intro": never_intro, "spots": spots, "unseen": unseen, "always-wrong": always_wrong,
         "who-moves-it": who_moves_it, "bounded": bounded, "exam": exam, "one-run": one_run, "order": order,
         "dropped": dropped, "remember": remember, "never-outro": never_outro}
