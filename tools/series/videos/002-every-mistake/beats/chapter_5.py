"""Chapter 5, "A bowl": the same sheet raised by the total squared miss (each border at its best size), its
shelf and its bowl, the two bottoms, and the perceptron added up, rocking."""

import numpy as np
from manim import (DOWN, LEFT, RIGHT, UP, AnimationGroup, Dot, FadeIn, FadeOut, Line, UpdateFromAlphaFunc, ValueTracker,
                   VGroup, VMobject, Wait, linear, smooth)

from series import style
from series.chart import Chart
from series.ember import numeral, serif
from series.knobs import Knobs
from series.terrain import View, world

from beats.common import FILM, NUMBERS, Do, Succession, away, fill, new_map, voice
from beats.chapter_1 import counter
from beats.chapter_2 import _ends, percent
from beats.chapter_4 import KEY_AT, OUTRO_END, chapter_key, full_frame
from beats.sheets import (INK, LEDGE, SQUARED, TOP, GroundPicture, across_of, bowl_ground, height_key, pin, spot,
                          staircase_ground, tag, up_of)

X, Y = FILM["x"], FILM["y"]
T = np.where(Y == 1, 1.0, -1.0)
XB = np.c_[X, np.ones(len(X))]
REST = FILM["batch_square"][-1]


def misses_by_size(v, sizes) -> np.ndarray:
    """The total squared miss of one border at each size: its weights and bias all times the size."""
    f = XB @ np.asarray(v, float)
    return np.array([((T - s * f) ** 2).mean() for s in sizes])


def bowl_picture(view: View = None, size=None) -> GroundPicture:
    return GroundPicture([bowl_ground()], view or OUTRO_END, size=size or full_frame())


def bowl_key():
    """The bowl on chapter 4's scale: a total squared miss of 1 (the shelf) stands as high as every moment wrong."""
    return height_key(0.0, TOP, [(0.0, "0"), (0.25, "0.25"), (0.5, "0.5"), (0.75, "0.75"), (1.0, "1: the shelf")],
                      "total squared miss").move_to(KEY_AT)


def bowl(scene) -> None:
    """The staircase, from where chapter 4 left it, takes the total squared miss for its height: the camera stays."""
    k = getattr(scene, "kept", None) or {}
    stairs = k.get("stairs") or GroundPicture([staircase_ground()], OUTRO_END, size=full_frame())
    old_key = k.get("key") or chapter_key()
    b = bowl_picture()
    key = bowl_key()
    head = tag("the same sheet; height: the total squared miss, at each border's best size", style.ASH).to_edge(UP, buff=0.35)
    first = [FadeIn(stairs), FadeIn(old_key)] if "stairs" not in k else []
    scene.say("bowl", Succession(*fill(scene, "bowl", away(scene, stairs, old_key), *first,
                                       AnimationGroup(FadeIn(b), FadeOut(stairs), FadeOut(old_key), FadeIn(key), run_time=1.6),
                                       FadeIn(head, run_time=0.4), b.fly(OUTRO_END.moved(yaw=OUTRO_END.yaw - 0.2), run_time=3.0,
                                                                         rate_func=linear))))
    scene.kept = {"bowl": b, "key": key}


def size_chart(top: float, at=RIGHT * 2.4 + DOWN * 0.3) -> Chart:
    return Chart((0, 2), (0, top), "size of the weights and bias", "total squared miss", width=5.6, height=4.0, at=at,
                 x_ticks=[0, 1, 2], y_ticks=[0, 1, 2] if top > 2 else [0, 1], y_format=lambda v: f"{v:g}")


def best_size(scene) -> None:
    """One border, its weights and bias scaled together: it doesn't move, but its squared miss does, lowest at one
    size."""
    sizes = np.linspace(0, 2, 101)
    u = misses_by_size(REST, sizes)
    c = size_chart(float(u.max()) * 1.05)
    curve = c.series(sizes, u, style.WARM[0])
    best = int(np.argmin(u))
    m = new_map(at=LEFT * 3.6 + UP * 0.4, width=5.0, height=3.0, line=REST)
    knobs = Knobs(LEFT * 3.6 + DOWN * 2.5, values=REST, scales=(0.6, 1.2, 2.0), gap=1.15).scale(0.85)
    ball = Dot(c.point(sizes[20], u[20]), radius=0.08, color=style.TEXT)
    mark = Dot(c.point(sizes[best], u[best]), radius=0.07, color=style.WARM[2])
    mark_text = serif("the least", style.WARM[2], 0.26).next_to(mark, DOWN, buff=0.12)

    def slide(path):
        def go(_, a: float) -> None:
            i = int(path[0] + (path[1] - path[0]) * a)
            ball.move_to(c.point(sizes[i], u[i]))
            knobs.set_values(REST * sizes[i])
        return UpdateFromAlphaFunc(ball, go)
    scene.say("best-size", Succession(*fill(scene, "best-size", away(scene), FadeIn(m, run_time=0.4), FadeIn(knobs, run_time=0.4),
                                             FadeIn(c, run_time=0.4), FadeIn(curve, run_time=0.6), FadeIn(ball, run_time=0.2),
                                             slide((20, 100)).set_run_time(2.0), slide((100, best)).set_run_time(1.6),
                                             FadeIn(VGroup(mark, mark_text), run_time=0.4))))


def shelf(scene) -> None:
    """A border facing the wrong way: any size above zero makes its misses bigger, so its best size is nothing."""
    back = -REST
    sizes = np.linspace(0, 2, 101)
    u = misses_by_size(back, sizes)
    c = size_chart(float(u.max()) * 1.05)
    curve = c.series(sizes, u, style.COOL[1])
    m = new_map(at=LEFT * 3.6 + UP * 0.4, width=5.0, height=3.0, line=back)
    ball = Dot(c.point(sizes[60], u[60]), radius=0.08, color=style.TEXT)
    note = serif("facing the wrong way", style.COOL[1], 0.28).next_to(m, DOWN, buff=0.15)

    def go(_, a: float) -> None:
        i = int(60 * (1 - a))
        ball.move_to(c.point(sizes[i], u[i]))
    end = VGroup(numeral("1", style.TEXT, 0.3).next_to(c.point(0, 1.0), LEFT, buff=0.3),
                 serif("at nothing: every moment misses by exactly one", style.TEXT, 0.26).next_to(c, UP, buff=0.2))
    scene.say("shelf", Succession(*fill(scene, "shelf", away(scene), FadeIn(m, run_time=0.4), FadeIn(note, run_time=0.3),
                                         FadeIn(c, run_time=0.4), FadeIn(curve, run_time=0.5), FadeIn(ball, run_time=0.2),
                                         UpdateFromAlphaFunc(ball, go, run_time=2.4), FadeIn(end, run_time=0.4))))


SHELF_VIEW = View(yaw=-0.75, pitch=0.38, distance=12.5, target=(-0.6, 1.6, 1.1))


def shelf_2(scene) -> None:
    """A low camera over the bowl; a slice sweeps up the sheet, its profile beside it: where the ruler mostly faces
    the wrong way, the ground is a flat shelf at exactly 1."""
    b = bowl_picture(SHELF_VIEW)
    g = b.grounds[0]
    na, no = g.value.shape
    across = g.across[0] + (np.arange(na) + 0.5) / na * (g.across[1] - g.across[0])
    up_at = lambda j: g.up[0] + (j + 0.5) / no * (g.up[1] - g.up[0])
    c = Chart((0, 360), (0, 1.15), "the ruler's turn, across the sheet", "total squared miss", width=4.6, height=1.3,
              at=LEFT * 4.3 + UP * 2.6, x_ticks=[0, 180, 360], y_ticks=[0, 1], y_format=lambda v: f"{v:g}")
    level = c.level(1.0, "the shelf: 1", style.TEXT)
    profile = VMobject(stroke_color=style.WARM[0], stroke_width=2.0)
    cut = VGroup(VMobject(stroke_color=INK, stroke_width=6), VMobject(stroke_color=style.TEXT, stroke_width=2.2))
    row = ValueTracker(no * 0.1)

    def follow(_) -> None:
        j = int(min(max(row.get_value(), 0), no - 1))
        v = g.value[:, j]
        pts = b.point(across, np.full(na, up_at(j)), v + 0.004)
        for line in cut:
            line.set_points_as_corners(pts)
        profile.set_points_as_corners([c.point(x, y) for x, y in zip((np.arange(na) + 0.5) / na * 360, v)])
    profile.add_updater(follow)
    follow(None)
    flat = SQUARED[::2, ::2] > 0.999
    i, j = np.argwhere(flat)[len(np.argwhere(flat)) // 2]
    shelf_pin = tag("the shelf: flat, at exactly 1", style.TEXT)
    shelf_pin.move_to(b.point(across[i], up_at(j), 1.0) + UP * 0.45)
    key = bowl_key()
    sweep = UpdateFromAlphaFunc(row, lambda t, a: t.set_value(no * (0.1 + 0.8 * a)), run_time=max(voice(scene, "shelf-2") - 3.5, 4.0), rate_func=smooth)
    scene.say("shelf-2", Succession(*fill(scene, "shelf-2", away(scene), FadeIn(b, key, run_time=1.0), FadeIn(c, level, run_time=0.4),
                                           FadeIn(cut, profile, run_time=0.3), sweep, FadeIn(shelf_pin, run_time=0.4))))
    profile.clear_updaters()
    scene.kept = {"bowl": b}


def over_bowl() -> View:
    """The camera over the bowl itself, looking down into it."""
    c, d = spot(REST)
    x, y, _ = world(c, d, 0.0, TOP)
    return OUTRO_END.moved(pitch=0.95, distance=9.0, target=(float(x) + 0.6, float(y), 0.5))


def bowl_2(scene) -> None:
    b = bowl_picture()
    c, d = spot(REST)
    low = pin(style.WARM[1]).move_to(b.point(c, d))
    words = tag("the bottom: where the square trick comes to rest", style.WARM[1]).next_to(low, UP, buff=0.3)
    scene.say("bowl-2", Succession(*fill(scene, "bowl-2", away(scene), FadeIn(b, bowl_key(), run_time=0.8),
                                          b.fly(over_bowl(), run_time=2.0), Do(lambda: low.move_to(b.point(c, d))),
                                          Do(lambda: words.next_to(low, UP, buff=0.3)), FadeIn(low, run_time=0.4), FadeIn(words, run_time=0.4))))


def no_shelf(scene) -> None:
    """From zero (the shelf's height, 1), a small turn of the knobs the right way, and the squared miss falls."""
    sizes = np.linspace(0, 1, 101)
    u = misses_by_size(REST, sizes)
    c = Chart((0, 1), (0, 1.05), "how far the knobs turn from zero, toward where it rests", "total squared miss",
              width=6.4, height=4.2, at=DOWN * 0.3, x_ticks=[0, 1], y_ticks=[0, 1], y_format=lambda v: f"{v:g}")
    curve, grow = c.grow(sizes, u, style.WARM[0])
    zero = VGroup(Dot(c.point(0, u[0]), radius=0.07, color=style.TEXT),
                  serif("all knobs at zero: the shelf's height", style.TEXT, 0.26).next_to(c.point(0, u[0]), UP + RIGHT, buff=0.1))
    scene.say("no-shelf", Succession(*fill(scene, "no-shelf", away(scene), FadeIn(c, run_time=0.5), FadeIn(zero, run_time=0.5),
                                            Wait(1.2), grow.set_run_time(2.4))))


PAIR_VIEW = View(yaw=-0.45, pitch=0.95, distance=14.0, target=(0.0, 0.2, 0.3))
PAIR_SIZE = (6.9, 4.6)


def pair() -> tuple:
    """The staircase and the bowl side by side, the lowest ledge gold and where the square trick rests coral."""
    stairs = GroundPicture([staircase_ground()], PAIR_VIEW, size=PAIR_SIZE).move_to(LEFT * 3.55 + DOWN * 0.3)
    bowl_ = GroundPicture([bowl_ground()], PAIR_VIEW, size=PAIR_SIZE).move_to(RIGHT * 3.55 + DOWN * 0.3)
    a, b = float(across_of(LEDGE[0])), float(up_of(LEDGE[1]))
    c, d = spot(REST)
    marks = VGroup(pin(style.WARM[2]).move_to(stairs.point(a, b)), pin(style.WARM[1]).move_to(stairs.point(c, d)),
                   pin(style.WARM[1]).move_to(bowl_.point(c, d)), pin(style.WARM[2]).move_to(bowl_.point(a, b)))
    return stairs, bowl_, marks


def two_bottoms(scene) -> None:
    stairs, bowl_, marks = pair()
    heads = VGroup(serif("mistakes", style.ASH, 0.3).next_to(stairs, UP, buff=0.0), serif("squared miss", style.ASH, 0.3).next_to(bowl_, UP, buff=0.0))
    words = VGroup(tag(f"the lowest ledge: right {NUMBERS['stairs_bottom']}", style.WARM[2]),
                   tag(f"the bottom of the bowl: right {NUMBERS['bowl_bottom']}", style.WARM[1])).arrange(RIGHT, buff=1.0).to_edge(DOWN, buff=0.4)
    scene.say("two-bottoms", Succession(*fill(scene, "two-bottoms", away(scene), FadeIn(stairs, bowl_, run_time=0.8),
                                               FadeIn(heads, run_time=0.3), FadeIn(marks, run_time=0.5), Wait(0.8),
                                               FadeIn(words, run_time=0.6))))
    scene.kept = {"stairs": stairs, "bowl": bowl_, "marks": marks, "heads": heads, "words": words}


def dim(m, to: float, run_time: float = 1.6) -> UpdateFromAlphaFunc:
    return UpdateFromAlphaFunc(m, lambda d, a: d.set_opacity(1 - (1 - to) * a), run_time=run_time)


def why_bowls(scene) -> None:
    k = scene.kept
    fog = serif("many numbers: no fast way to search", style.ASH, 0.3).next_to(k["stairs"], DOWN, buff=0.1)
    slope = serif("a slope to follow", style.WARM[1], 0.3).next_to(k["bowl"], DOWN, buff=0.1)
    scene.say("why-bowls", Succession(*fill(scene, "why-bowls", FadeOut(k["words"], run_time=0.3),
                                             AnimationGroup(dim(k["stairs"], 0.15), dim(k["marks"][0:2], 0.15)),
                                             FadeIn(fog, run_time=0.4), FadeIn(slope, run_time=0.4))))


def batch_chart(at=RIGHT * 3.0 + DOWN * 0.3) -> Chart:
    return Chart((1, 300), (60, 100), "moves", "score on the exam", width=5.4, height=3.8, at=at,
                 x_ticks=[1, 100, 200, 300], y_ticks=[60, 80, 100], y_format=lambda v: f"{v:.0f}%")


def perceptron_rest(scene) -> None:
    m = new_map(at=LEFT * 3.6 + DOWN * 0.3, width=6.0, height=3.7)
    path = FILM["batch_perceptron"]
    scores = FILM["batch_perceptron_scores"] * 100
    c = batch_chart()
    line, grow = c.grow(np.arange(1, 301), scores, style.WARM[0])
    band = serif(f"last hundred moves: {NUMBERS['batch_perceptron_lowest']} to {NUMBERS['batch_perceptron_highest']}",
                 style.TEXT, 0.28).next_to(c, UP, buff=0.25)
    talk = voice(scene, "perceptron-rest")

    def go(_, a: float) -> None:
        m.set_line(path[max(int(a * 300), 1)])
    scene.say("perceptron-rest", Succession(away(scene), FadeIn(m, run_time=0.4), FadeIn(c, run_time=0.4),
                                            AnimationGroup(grow, UpdateFromAlphaFunc(m.border, go), run_time=max(talk - 1.4, 1.0), rate_func=linear),
                                            FadeIn(band, run_time=0.5)))


def perceptron_rest_2(scene) -> None:
    """The last moves: the border flipping between two, each move almost undoing the one before."""
    m = new_map(at=LEFT * 1.6 + DOWN * 0.3, width=7.4, height=4.3)
    path = FILM["batch_perceptron"][-40:]
    scores = FILM["batch_perceptron_scores"][-40:] * 100
    cuts = NUMBERS["batch_perceptron_rocking_cuts"]
    ghosts = VGroup(*[Line(*_ends(m, v), stroke_color=style.ASH, stroke_width=1.0, stroke_opacity=0.6) for v in path[-2:]])
    score = ValueTracker(scores[0])
    shown = counter("score, %", score, RIGHT * 4.9 + UP * 1.6)
    note = serif(f"standing still, one says yes from about {cuts[0]} blocks, the other {cuts[1]}", style.TEXT, 0.28).next_to(m, UP, buff=0.2)
    talk = voice(scene, "perceptron-rest-2")

    def go(_, a: float) -> None:
        i = min(int(a * len(path)), len(path) - 1)
        m.set_line(path[i])
        score.set_value(scores[i])
    scene.say("perceptron-rest-2", Succession(away(scene), FadeIn(m, run_time=0.4), FadeIn(ghosts, run_time=0.4), FadeIn(shown, run_time=0.3),
                                              FadeIn(note, run_time=0.4),
                                              UpdateFromAlphaFunc(m.border, go, run_time=max(talk - 1.5, 1.0), rate_func=linear)))


def perceptron_rest_3(scene) -> None:
    p = np.linalg.norm(np.diff(FILM["batch_perceptron"], axis=0), axis=1)
    s = np.linalg.norm(np.diff(FILM["batch_square"], axis=0), axis=1)[:300]
    lp, ls = np.log10(p), np.log10(np.maximum(s, 1e-12))
    lo = float(min(lp.min(), ls.min())) - 0.3
    c = Chart((1, 300), (lo, 0.5), "moves", "size of each move (each line down: ten times smaller)", width=6.4, height=4.2,
              at=DOWN * 0.3, x_ticks=[1, 100, 200, 300], y_ticks=list(range(int(np.ceil(lo)), 1)),
              y_format=lambda v: f"10^{v:.0f}" if v else "1")
    a, grow_a = c.grow(np.arange(1, 301), lp, style.WARM[0])
    b, grow_b = c.grow(np.arange(1, 301), ls, style.COOL[2])
    names = VGroup(serif("the perceptron, added up", style.WARM[0], 0.28), serif("the square trick, added up", style.COOL[2], 0.28)
                   ).arrange(RIGHT, buff=0.8).next_to(c, UP, buff=0.25)
    scene.say("perceptron-rest-3", Succession(*fill(scene, "perceptron-rest-3", away(scene), FadeIn(c, run_time=0.4), FadeIn(names, run_time=0.4),
                                                     AnimationGroup(grow_a, grow_b, run_time=3.0))))


def map_outro(scene) -> None:
    stairs, bowl_, marks = pair()
    scene.say("map-outro", Succession(*fill(scene, "map-outro", away(scene), FadeIn(stairs, bowl_, run_time=0.8),
                                             FadeIn(marks, run_time=0.5))))


BEATS = {"bowl": bowl, "best-size": best_size, "shelf": shelf, "shelf-2": shelf_2, "bowl-2": bowl_2, "no-shelf": no_shelf,
         "two-bottoms": two_bottoms, "why-bowls": why_bowls, "perceptron-rest": perceptron_rest,
         "perceptron-rest-2": perceptron_rest_2, "perceptron-rest-3": perceptron_rest_3, "map-outro": map_outro}
