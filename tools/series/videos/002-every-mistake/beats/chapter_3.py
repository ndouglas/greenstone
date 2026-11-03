"""Chapter 3, "How wrong was it?": targets, the miss, the total squared miss, its U and its valley, the pulls
that cancel at the bottom, and a step too big."""

import numpy as np
from manim import (DOWN, LEFT, RIGHT, UP, AnimationGroup, Arrow, Circle, Create, Dot, FadeIn, FadeOut, ImageMobject,
                   Line, Square, UpdateFromAlphaFunc, ValueTracker, VGroup, Wait, linear, smooth)

from series import style
from series.chart import Chart
from series.ember import in_line, numeral, serif, swap
from series.knobs import Knobs, Total
from series.sheet import Camera, Sheet

from beats.common import FILM, NUMBERS, Do, Succession, away, fill, new_map, voice
from beats.chapter_1 import BEFORE, SCALES, counter, example_dot, example_map

RATE = 0.1  # the batch square trick's step (exp002's BATCH_RATE)
X, Y = FILM["x"], FILM["y"]
T = np.where(Y == 1, 1.0, -1.0)
XB = np.c_[X, np.ones(len(X))]
REST = FILM["batch_square"][-1]


def square_intro(scene) -> None:
    """Two moments the border gets wrong, one just past it and one far past: the perceptron nudges each by its
    own numbers, however far past it is."""
    m = example_map()
    v = np.array([0.5, 0.65, -1.85])
    m.set_line(v)
    near, far = np.array([3.0, 0.6]), np.array([4.8, 2.3])  # both said yes, and the screen stayed still
    a, b = example_dot(m, near, False), example_dot(m, far, False)
    totals = [float(v[:2] @ p + v[2]) for p in (near, far)]
    words = VGroup(serif(f"just past: total {style.number(totals[0], 2)}", style.TEXT, 0.28).next_to(a, DOWN, buff=0.12),
                   serif(f"far past: total {style.number(totals[1], 2)}", style.TEXT, 0.28).next_to(b, LEFT, buff=0.12))
    same = serif("the perceptron: the same kind of nudge for both", style.TEXT, 0.32).to_edge(UP, buff=0.4)
    scene.say("square-intro", Succession(*fill(scene, "square-intro", away(scene), FadeIn(m, run_time=0.6),
                                                FadeIn(VGroup(a, b), run_time=0.6), FadeIn(words, run_time=0.6),
                                                Wait(1.0), FadeIn(same, run_time=0.6))))


def target(scene) -> None:
    yes = Total(LEFT * 2.2 + DOWN * 0.2, scale=1.0, half=2.0, targets=(1.0,), label="total, screen moved soon")
    no = Total(RIGHT * 2.2 + DOWN * 0.2, scale=1.0, half=2.0, targets=(-1.0,), label="total, it didn't")
    zero = serif("above zero: yes; below: no", style.ASH, 0.3).to_edge(UP, buff=0.5)
    scene.say("target", Succession(*fill(scene, "target", away(scene), FadeIn(yes, run_time=0.8), FadeIn(no, run_time=0.8),
                                          FadeIn(zero, run_time=0.6))))
    scene.kept = {"yes": yes, "no": no, "zero": zero}


def miss_arrow(total: Total, value: float, aim: float, colour: str) -> Arrow:
    """From the total to its target, beside the bar: the miss, and the way the pull goes."""
    at = total.centre()
    x = at[0] + 0.55
    return Arrow([x, at[1] + total.y(value), 0], [x, at[1] + total.y(aim), 0], buff=0, stroke_width=2.4,
                 color=colour, max_tip_length_to_length_ratio=0.25)


def target_rule(scene) -> None:
    k = scene.kept
    yes = k["yes"]
    rule = VGroup(serif("after every moment, right or wrong, each knob moves by", style.TEXT, 0.32),
                  serif("the step × its own number × the miss", style.WARM[2], 0.36),
                  serif("total below its target: pulled up; above: down", style.TEXT, 0.32)).arrange(DOWN, buff=0.14)
    rule.to_edge(UP, buff=0.3)
    talk = voice(scene, "target-rule")
    scene.say("target-rule", Succession(
        FadeOut(VGroup(k["no"], k["zero"]), run_time=0.5), yes.animate(run_time=0.6).shift(LEFT * 2.0),
        FadeIn(rule[0], run_time=0.5), FadeIn(rule[1], run_time=0.5),
        UpdateFromAlphaFunc(yes, lambda t, a: t.set_value(0.2 * a), run_time=0.8), FadeIn(rule[2], run_time=0.5),
        Wait(max(talk - 3.4, 0.1))))
    k.update(rule=rule)


def target_example(scene) -> None:
    k = scene.kept
    yes = k["yes"]
    arrow = miss_arrow(yes, 0.2, 1.0, style.WARM[1])
    words = VGroup(serif("target +1, total 0.2: the neuron said yes, rightly", style.TEXT, 0.3),
                   serif("the perceptron: no change", style.ASH, 0.3),
                   serif("below its target by 0.8: pulled up", style.WARM[1], 0.3),
                   serif("each knob up by 0.8 × the step × its number", style.WARM[1], 0.3)).arrange(DOWN, aligned_edge=LEFT, buff=0.14)
    words.move_to(RIGHT * 2.6 + DOWN * 0.4)
    scene.say("target-example", Succession(*fill(scene, "target-example", FadeOut(k["rule"][0], run_time=0.3),
                                                  FadeIn(words[0], run_time=0.5), Wait(0.8), FadeIn(words[1], run_time=0.5),
                                                  Wait(0.8), Create(arrow, run_time=0.6), FadeIn(words[2], run_time=0.5),
                                                  FadeIn(words[3], run_time=0.5))))
    k.update(arrow=arrow, words=words)


def target_over(scene) -> None:
    k = scene.kept
    over = Total(LEFT * 1.4 + DOWN * 0.2, scale=1.0, half=2.0, targets=(1.0,), label="another moment")
    arrow = miss_arrow(over, 3.0, 1.0, style.COOL[1])
    words = VGroup(serif("total 3: right, by plenty", style.TEXT, 0.3),
                   serif("above its target by 2: pulled down", style.COOL[1], 0.3),
                   serif("corrected for being too right", style.TEXT, 0.3)).arrange(DOWN, aligned_edge=LEFT, buff=0.14)
    words.move_to(RIGHT * 2.8 + UP * 0.6)
    # the two pulls to scale: 0.8 up and 2 down, for numbers the same size
    unit = 0.75
    base = RIGHT * 2.0 + DOWN * 2.2
    pulls = VGroup(Arrow(base, base + UP * 0.8 * unit, buff=0, color=style.WARM[1], stroke_width=3),
                   Arrow(base + RIGHT * 1.2, base + RIGHT * 1.2 + DOWN * 2.0 * unit + UP * 0.0, buff=0, color=style.COOL[1], stroke_width=3))
    pulls[1].shift(UP * 2.0 * unit)
    labels = VGroup(numeral("0.8", style.WARM[1], 0.3).next_to(pulls[0], LEFT, buff=0.1),
                    numeral("2", style.COOL[1], 0.3).next_to(pulls[1], RIGHT, buff=0.1))
    scale_note = serif("the two pulls, to scale", style.ASH, 0.26).next_to(VGroup(pulls, labels), RIGHT, buff=0.3)
    scene.say("target-over", Succession(*fill(scene, "target-over",
        FadeOut(VGroup(k["words"], k["rule"][1:]), run_time=0.4),
        k["yes"].animate(run_time=0.5).shift(LEFT * 0.6), FadeIn(over, run_time=0.4),
        UpdateFromAlphaFunc(over, lambda t, a: t.set_value(3.0 * a), run_time=0.8), FadeIn(words[0], run_time=0.4),
        Create(arrow, run_time=0.5), FadeIn(words[1], run_time=0.4), Wait(0.6),
        Create(pulls, run_time=0.8), FadeIn(labels, run_time=0.3), FadeIn(scale_note, run_time=0.3),
        Wait(0.8), FadeIn(words[2], run_time=0.5))))


SAMPLE = np.random.default_rng(12).choice(len(Y), 24, replace=False)  # moments whose squares are drawn


def square_why(scene) -> None:
    """Twenty-four moments' misses as squares, side by side, shrinking as the added-up nudges move the border; beside them
    the total squared miss, over every moment, falling."""
    path = FILM["batch_square"][:400]
    misses = FILM["batch_square_misses"][:400]
    m = new_map(at=LEFT * 3.5 + DOWN * 1.2, width=5.6, height=3.4, line=path[1])
    squares = VGroup(*[Square(0.1, stroke_color=style.WARM[2], stroke_width=1.0, fill_opacity=0.15, fill_color=style.WARM[2])
                       for _ in SAMPLE])
    row = LEFT * 1.0 + UP * 2.8
    c = Chart((0, len(path) - 1), (0, float(misses[0]) * 1.05), "moves", "total squared miss", width=4.6, height=2.6,
              at=RIGHT * 3.8 + DOWN * 1.4, x_ticks=[0, 200, 400], y_ticks=[0, 1], y_format=lambda v: f"{v:g}")
    curve, grow = c.grow(np.arange(len(path)), misses, style.WARM[0])

    def lay(step: int) -> None:
        v = path[step]
        miss = T[SAMPLE] - XB[SAMPLE] @ v
        x = row[0]
        for sq, e in zip(squares, miss):
            side = max(abs(e) * 0.24, 0.01)
            sq.become(Square(side, stroke_color=style.WARM[2] if e > 0 else style.COOL[1], stroke_width=1.0,
                             fill_opacity=0.18, fill_color=style.WARM[2] if e > 0 else style.COOL[1]))
            sq.move_to([x + side / 2, row[1] - side / 2, 0])
            x += side + 0.04
        m.set_line(v if step else path[1])
    lay(1)
    note = serif("each square: one moment's miss, times itself", style.ASH, 0.26).next_to(row, UP, buff=0.18).align_to(row, LEFT)
    talk = voice(scene, "square-why")
    scene.say("square-why", Succession(away(scene), FadeIn(m, run_time=0.5), FadeIn(squares, run_time=0.6),
                                       FadeIn(note, run_time=0.3), FadeIn(c, run_time=0.4),
                                       AnimationGroup(grow, UpdateFromAlphaFunc(squares, lambda _, a: lay(max(int(a * (len(path) - 1)), 1))),
                                                      run_time=max(talk - 1.9, 1.0), rate_func=lambda t: t ** 0.5)))


def square_name(scene) -> None:
    card = VGroup(serif("the Widrow–Hoff rule", style.WARM[2], 0.5),
                  serif("least mean squares", style.TEXT, 0.38),
                  serif("Bernard Widrow and Ted Hoff, 1960", style.ASH, 0.34),
                  serif("one textbook: “the square trick”", style.ASH, 0.3)).arrange(DOWN, buff=0.28)
    scene.say("square-name", Succession(*fill(scene, "square-name", away(scene), FadeIn(card[0:2], run_time=0.8),
                                               FadeIn(card[2], run_time=0.6), Wait(1.2), FadeIn(card[3], run_time=0.6))))


def u_chart(at=RIGHT * 2.0 + DOWN * 0.3) -> Chart:
    turns, u = FILM["u_turns"], FILM["u_position"]
    return Chart((REST[0] + turns[0], REST[0] + turns[-1]), (0, float(u.max()) * 1.02), "position knob",
                 "total squared miss", width=6.4, height=4.4, at=at,
                 x_ticks=[round(REST[0] - 1, 1), round(REST[0], 1), round(REST[0] + 1, 1)], y_ticks=[0, 5, 10])


def downhill(scene) -> None:
    turns, u = FILM["u_turns"], FILM["u_position"]
    c = u_chart()
    knobs = Knobs(LEFT * 4.6 + UP * 1.0, values=REST, scales=(0.6, 1.2, 2.0), gap=1.25)
    held = serif("the others held still", style.ASH, 0.26).next_to(knobs, DOWN, buff=0.15)
    xs = REST[0] + turns
    curve, grow = c.grow(xs, u, style.WARM[0])
    ball = Dot(c.point(xs[0], u[0]), radius=0.08, color=style.TEXT)

    def go(_, a: float) -> None:
        i = min(int(a * (len(xs) - 1)), len(xs) - 1)
        ball.move_to(c.point(xs[i], u[i]))
        knobs.dials[0].set_value(xs[i])
    talk = voice(scene, "downhill")
    scene.say("downhill", Succession(away(scene), FadeIn(c, run_time=0.5), FadeIn(knobs, run_time=0.5), FadeIn(held, run_time=0.3),
                                     FadeIn(ball, run_time=0.2),
                                     AnimationGroup(grow, UpdateFromAlphaFunc(ball, go), run_time=max(talk - 1.6, 1.0), rate_func=linear)))
    scene.kept = {"chart": c, "curve": curve}


def valley_image(size=(9.0, 5.6), pitch=0.85) -> tuple:
    sheet = Sheet(FILM["valley"])
    camera = Camera(pitch=pitch)
    return sheet, camera, ImageMobject(sheet.render(camera, 1.0, size, smooth=True)).set(width=size[0])


def downhill_2(scene) -> None:
    """Both weights turned at once, the bias held: a valley. An arrow from one spot, pointing where the
    added-up nudge points: down it."""
    sheet, camera, image = valley_image()
    image.move_to(DOWN * 0.4)
    grid = FILM["valley_turns"]
    n = len(grid)
    i, j = 12, 46  # a spot up the valley's side
    w = np.array([REST[0] + grid[i], REST[1] + grid[j], REST[2]])
    pull = XB.T @ (T - XB @ w) / len(T)  # the added-up nudge, per step
    d = pull[:2] / np.linalg.norm(pull[:2])
    a = (i + 0.5) / n, (j + 0.5) / n
    b = np.clip(np.array(a) + d * np.array([1, 1]) * 0.12, 0, 1)
    start = sheet.at(*a, 1.0, camera) + image.get_center()
    end = sheet.at(*b, 1.0, camera) + image.get_center()
    arrow = Arrow(start, end, buff=0, color=style.WARM[1], stroke_width=3, max_tip_length_to_length_ratio=0.3)
    axes = VGroup(serif("position knob →", style.ASH, 0.28).next_to(image, DOWN, buff=0.05),
                  serif("speed knob, into the picture", style.ASH, 0.28).next_to(image, LEFT, buff=-1.2).shift(UP * 1.8))
    words = serif("the added-up nudge: downhill", style.WARM[1], 0.3).next_to(arrow, UP, buff=0.12)
    scene.say("downhill-2", Succession(*fill(scene, "downhill-2", away(scene), FadeIn(image, run_time=1.2), FadeIn(axes, run_time=0.4),
                                              Wait(1.6), Create(arrow, run_time=0.8), FadeIn(words, run_time=0.4))))


def balance(scene) -> None:
    """At the bottom, every moment still pulls each knob, some up and some down; added up, nothing."""
    miss = T - XB @ REST
    rows = VGroup()
    for k, name in enumerate(("position knob", "speed knob", "bias")):
        pulls = miss * XB[:, k] / len(T)
        up, down = pulls[pulls > 0].sum(), pulls[pulls < 0].sum()
        top = max(up, -down)
        bar_up = Line(ORIGIN_ROW(k), ORIGIN_ROW(k) + RIGHT * 2.4 * up / top, stroke_color=style.WARM[1], stroke_width=8)
        bar_down = Line(ORIGIN_ROW(k), ORIGIN_ROW(k) + LEFT * 2.4 * -down / top, stroke_color=style.COOL[1], stroke_width=8)
        label = serif(name, style.ASH, 0.3).next_to(ORIGIN_ROW(k) + LEFT * 2.6, LEFT, buff=0.2)
        ups = numeral(f"+{up:.3f}", style.WARM[1], 0.28).next_to(bar_up, RIGHT, buff=0.12)
        downs = numeral(style.number(down, 3), style.COOL[1], 0.28).next_to(bar_down, LEFT, buff=0.12).shift(DOWN * 0.3)
        net = numeral(f"= {style.number(up + down, 3)}", style.TEXT, 0.32).move_to(ORIGIN_ROW(k) + RIGHT * 4.4)
        rows.add(VGroup(label, bar_down, bar_up, ups, downs, net))
    mid = Line(UP * 2.2, DOWN * 1.6, stroke_color=style.ASH, stroke_width=1.0)
    head = serif("at the bottom: every moment's pull, added up", style.TEXT, 0.32).to_edge(UP, buff=0.4)
    scene.say("balance", Succession(*fill(scene, "balance", away(scene), FadeIn(head, run_time=0.4), Create(mid, run_time=0.3),
                                           *[AnimationGroup(FadeIn(r[0]), Create(r[1]), Create(r[2]), FadeIn(r[3]), FadeIn(r[4]), run_time=0.8) for r in rows],
                                           Wait(1.0), FadeIn(VGroup(*[r[5] for r in rows]), run_time=0.6))))


def ORIGIN_ROW(k: int) -> np.ndarray:
    return np.array([0.0, 1.3 - 1.3 * k, 0.0])


def swing_chart() -> Chart:
    return Chart((21, 30), (60, 100), "pass", "score on the exam", width=6.0, height=3.8, at=DOWN * 0.3,
                 x_ticks=[21, 25, 30], y_ticks=[60, 80, 100], y_format=lambda v: f"{v:.0f}%")


def settles_2(scene) -> None:
    c = swing_chart()
    sq = FILM["square_scores"][0][-10:] * 100
    pc = FILM["order_scores"][1][-10:] * 100  # a typical order (the one shown fell further)
    xs = np.arange(21, 31)
    a, grow_a = c.grow(xs, sq, style.WARM[2])
    b, grow_b = c.grow(xs, pc, style.COOL[1])
    names = VGroup(serif(f"square trick, one moment at a time: about {NUMBERS['square_swing_points']} in a hundred", style.WARM[2], 0.28),
                   serif(f"the perceptron: {NUMBERS['perceptron_swing_points']}", style.COOL[1], 0.28)).arrange(DOWN, aligned_edge=LEFT, buff=0.1)
    names.next_to(c, UP, buff=0.3)
    talk = voice(scene, "settles-2")
    scene.say("settles-2", Succession(away(scene), FadeIn(c, run_time=0.5), Wait(max(talk * 0.35, 0.1)),
                                      grow_a.set_run_time(1.6), FadeIn(names[0], run_time=0.4), grow_b.set_run_time(1.6),
                                      FadeIn(names[1], run_time=0.4)))


def settles_3(scene) -> None:
    path = FILM["batch_square"]
    m = new_map(at=DOWN * 0.3, width=7.6, height=4.4, line=path[1])
    count = ValueTracker(1)
    moves = counter("moves", count, UP * 3.1 + RIGHT * 3.0)
    idx = np.unique(np.r_[np.arange(1, 50), np.geomspace(50, len(path) - 1, 80).astype(int)])

    def go(_, a: float) -> None:
        i = idx[min(int(a * len(idx)), len(idx) - 1)]
        m.set_line(path[i])
        count.set_value(i)
    talk = voice(scene, "settles-3")
    scene.say("settles-3", Succession(away(scene), FadeIn(m, run_time=0.5), FadeIn(moves, run_time=0.3),
                                      UpdateFromAlphaFunc(m.border, go, run_time=max(talk - 0.8, 1.0), rate_func=linear)))
    scene.kept = {"map": m}


def jumps(rate: float, start: float, n: int) -> np.ndarray:
    """The position knob alone, the others held at the bottom, moved by the added-up nudge with this step."""
    w = [start]
    for _ in range(n):
        v = REST.copy()
        v[0] = w[-1]
        w.append(w[-1] + rate * float(XB[:, 0] @ (T - XB @ v)) / len(T))
    return np.array(w)


def u_value(w0: float) -> float:
    v = REST.copy()
    v[0] = w0
    return float(((T - XB @ v) ** 2).mean())


def too_big(scene) -> None:
    """A ball on the U, the position knob alone moved by twice the step: each jump crosses the bottom and lands
    higher up the other side."""
    c = u_chart(at=DOWN * 0.3)
    turns, u = FILM["u_turns"], FILM["u_position"]
    curve = c.series(REST[0] + turns, u, style.WARM[0])
    path = jumps(2 * RATE, REST[0] - 0.12, 8)
    path = path[np.abs(path - REST[0]) <= 1.0]
    ball = Dot(c.point(path[0], u_value(path[0])), radius=0.08, color=style.TEXT)
    note = serif("one knob alone, twice the step that settled", style.ASH, 0.28).next_to(c, UP, buff=0.25)
    hops = []
    for p, q in zip(path[:-1], path[1:]):
        a, b = c.point(p, u_value(p)), c.point(q, u_value(q))
        hops.append(UpdateFromAlphaFunc(ball, lambda d, t, a=a, b=b: d.move_to(a + (b - a) * t + UP * 1.2 * np.sin(np.pi * t)),
                                        run_time=0.7))
    talk = voice(scene, "too-big")
    scene.say("too-big", Succession(away(scene), FadeIn(c, run_time=0.4), Create(curve, run_time=0.6), FadeIn(note, run_time=0.3),
                                    FadeIn(ball, run_time=0.2), Wait(max(talk - 1.5 - 0.7 * len(hops), 0.1)), *hops))


def too_big_2(scene) -> None:
    misses = FILM["too_big_misses"]
    c = Chart((0, len(misses) - 1), (0, float(misses.max()) * 1.05), "moves, with twice the step", "total squared miss",
              width=6.4, height=4.2, at=DOWN * 0.3, x_ticks=list(range(0, len(misses), 2)),
              y_ticks=[0, round(float(misses.max()) / 2), round(float(misses.max()))])
    line, grow = c.grow(np.arange(len(misses)), misses, style.WARM[1])
    words = serif(f"{NUMBERS['too_big_growth']} times where it started", style.WARM[1], 0.34).next_to(c, UP, buff=0.25)
    scene.say("too-big-2", Succession(*fill(scene, "too-big-2", away(scene), FadeIn(c, run_time=0.5), grow.set_run_time(3.0),
                                             FadeIn(words, run_time=0.5))))


def rest_outro(scene) -> None:
    m = new_map(at=DOWN * 0.3, width=7.6, height=4.4, line=REST)
    still = serif("the square trick, added up: at rest", style.TEXT, 0.3).next_to(m, UP, buff=0.3)
    scene.say("rest-outro", Succession(*fill(scene, "rest-outro", away(scene), FadeIn(m, run_time=0.8), FadeIn(still, run_time=0.5))))


BEATS = {"square-intro": square_intro, "target": target, "target-rule": target_rule, "target-example": target_example,
         "target-over": target_over, "square-why": square_why, "square-name": square_name, "downhill": downhill,
         "downhill-2": downhill_2, "balance": balance, "settles-2": settles_2, "settles-3": settles_3,
         "too-big": too_big, "too-big-2": too_big_2, "rest-outro": rest_outro}
