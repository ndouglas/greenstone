"""The conclusions: the perceptron that never came to rest, the bowl the square trick settles in, the pocket's
search, the staircase, and what's next."""

import numpy as np
from manim import DOWN, LEFT, RIGHT, UP, Create, Dot, FadeIn, Group, Line, VGroup, Wait

from series import style
from series.ember import serif
from series.pointmap import PointMap
from series.records import ScoreBars
from series.sheet import Camera

from beats.common import FILM, NUMBERS, Succession, away, fill, new_map
from beats.chapter_2 import _ends, percent
from beats.sheets import MISTAKES, SQUARED, SheetPicture, lowest, spot

REST = FILM["batch_square"][-1]


def fan(m, rows, colour=style.WARM[2], width=0.9, opacity=0.7) -> VGroup:
    return VGroup(*[Line(*_ends(m, v), stroke_color=colour, stroke_width=width, stroke_opacity=opacity) for v in rows])


def tldr(scene) -> None:
    m = new_map(at=LEFT * 2.6 + DOWN * 0.3, width=7.0, height=4.2, line=FILM["order_ends"][0, -1])
    borders = fan(m, FILM["order_ends"][1:, -1])
    finals = np.sort(FILM["order_scores"][:, -1] * 100)
    dots = VGroup(*[Dot(np.array([4.6, -2.2 + (f - 60) / 40 * 4.2, 0]), radius=0.06, color=style.WARM[0]) for f in finals])
    axis = Line(np.array([4.6, -2.2, 0]), np.array([4.6, 2.0, 0]), stroke_color=style.ASH, stroke_width=1.0)
    labels = VGroup(serif("60%", style.ASH, 0.24).next_to(axis.get_start(), LEFT), serif("100%", style.ASH, 0.24).next_to(axis.get_end(), LEFT),
                    serif("ten orders' final scores", style.ASH, 0.26).next_to(axis, UP, buff=0.25))
    scene.say("tldr", Succession(*fill(scene, "tldr", away(scene), FadeIn(m, run_time=0.5), Create(borders, run_time=1.4),
                                        FadeIn(VGroup(axis, labels), run_time=0.4), FadeIn(dots, run_time=0.8))))


def tldr_2(scene) -> None:
    s = SheetPicture(SQUARED, smooth=True, size=(9.6, 5.8), camera=Camera(pitch=0.9)).move_to(DOWN * 0.2)
    a, b = lowest(SQUARED)
    low = Dot(s.point(a, b), radius=0.08, color=style.COOL[2])
    words = serif(f"the square trick settles here: {NUMBERS['batch_square']}", style.COOL[2], 0.3).next_to(low, UP, buff=0.4)
    scene.say("tldr-2", Succession(*fill(scene, "tldr-2", away(scene), FadeIn(s, run_time=0.8), FadeIn(low, run_time=0.3),
                                          FadeIn(words, run_time=0.4))))


def tldr_3(scene) -> None:
    bars = ScoreBars(percent("search"), "fewest mistakes", at=np.array([-2.4, -2.0, 0]))
    passes = VGroup(*[VGroup(*[Line(UP * 0.0, UP * 0.18, stroke_color=style.ASH, stroke_width=1.0) for _ in range(30)]).arrange(RIGHT, buff=0.06)
                      for _ in range(10)]).arrange(DOWN, buff=0.12).move_to(RIGHT * 3.6 + UP * 0.2)
    note = serif("behind the memory: every pass scored on every moment learned from", style.ASH, 0.26).next_to(passes, DOWN, buff=0.3)
    scene.say("tldr-3", Succession(*fill(scene, "tldr-3", away(scene), FadeIn(bars, run_time=0.4),
                                          bars.bar(float(np.mean(FILM["pocket_scores"]) * 100), "with a memory").set_run_time(1.0),
                                          Create(passes, lag_ratio=0.02, run_time=2.4), FadeIn(note, run_time=0.4))))


def tldr_4(scene) -> None:
    stairs = SheetPicture(MISTAKES, smooth=False, size=(6.6, 4.2), camera=Camera(pitch=0.95)).move_to(LEFT * 3.5 + DOWN * 0.3)
    bowl = SheetPicture(SQUARED, smooth=True, size=(6.6, 4.2), camera=Camera(pitch=0.95)).move_to(RIGHT * 3.5 + DOWN * 0.3)
    a, b = lowest(MISTAKES)
    c, d = lowest(SQUARED)
    marks = VGroup(Dot(stairs.point(a, b), radius=0.07, color=style.WARM[2]), Dot(bowl.point(c, d), radius=0.07, color=style.COOL[2]))
    words = VGroup(serif(f"lowest ledge: {NUMBERS['stairs_bottom']}", style.WARM[2], 0.3).next_to(stairs, DOWN, buff=0.3),
                   serif(f"bottom of the bowl: {NUMBERS['bowl_bottom']}", style.COOL[2], 0.3).next_to(bowl, DOWN, buff=0.3))
    scene.say("tldr-4", Succession(*fill(scene, "tldr-4", away(scene), FadeIn(Group(stairs, bowl), run_time=0.8),
                                          FadeIn(marks, run_time=0.4), FadeIn(words, run_time=0.4))))


def inside(scene) -> None:
    m = new_map(at=DOWN * 0.3, width=7.6, height=4.4, line=REST)
    many = np.r_[FILM["order_ends"][:, -1], FILM["dropped"]]
    borders = fan(m, many, opacity=0.55)
    m.border.set_stroke(color=style.COOL[2], width=2.4)
    words = VGroup(serif("the perceptron: twenty endings", style.WARM[2], 0.28), serif("the square trick: one", style.COOL[2], 0.28)
                   ).arrange(RIGHT, buff=0.8).next_to(m, UP, buff=0.2)
    scene.say("inside", Succession(*fill(scene, "inside", away(scene), FadeIn(m, run_time=0.5), Create(borders, run_time=2.0),
                                          FadeIn(words, run_time=0.5))))


def next_beat(scene) -> None:
    rng = np.random.default_rng(3)
    a = rng.normal([1.4, 0.8], [0.35, 0.25], (30, 2))
    b = rng.normal([3.6, 1.8], [0.35, 0.25], (30, 2))
    x = np.clip(np.r_[a, b], 0.05, 4.95)
    y = np.r_[np.zeros(30, int), np.ones(30, int)]
    m = PointMap(x, y, 1 - y, at=DOWN * 0.3, width=7.0, height=4.2, x_label="one number", y_label="another", mark=False,
                 radii=(0.07, 0.07), line=np.array([1.0, 1.0, -3.8]))
    words = VGroup(serif("next time: when the line lies", style.WARM[2], 0.4),
                   serif("(made-up dots: a split too perfect to trust)", style.ASH, 0.26)).arrange(DOWN, buff=0.12).next_to(m, UP, buff=0.25)
    scene.say("next", Succession(*fill(scene, "next", away(scene), FadeIn(m, run_time=0.8), FadeIn(words, run_time=0.6))))


def disclaimer(scene) -> None:
    card = VGroup(serif("I'm not an expert, just someone learning this in public.", style.TEXT, 0.4),
                  serif("If you spot a mistake, please tell me.", style.TEXT, 0.4)).arrange(DOWN, buff=0.3)
    scene.say("disclaimer", Succession(*fill(scene, "disclaimer", away(scene), FadeIn(card, run_time=0.8))))


BEATS = {"tldr": tldr, "tldr-2": tldr_2, "tldr-3": tldr_3, "tldr-4": tldr_4, "inside": inside, "next": next_beat}
REST_BEATS = {"disclaimer": disclaimer}
