"""Chapter 4, "Close, but not quite": where the square trick rests against the fewest-mistakes border, and the
sheet of every border, raised by its mistakes into a staircase."""

import numpy as np
from manim import (DOWN, LEFT, RIGHT, UP, AnimationGroup, Arrow, Circle, Create, Dot, FadeIn, FadeOut, Line,
                   Rectangle, Succession, UpdateFromAlphaFunc, ValueTracker, VGroup, Wait, linear)

from series import style
from series.ember import in_line, numeral, serif, swap
from series.lab import rules
from series.records import ScoreBars
from series.sheet import Camera

from beats.common import FILM, NUMBERS, Do, away, fill, new_map, voice
from beats.chapter_1 import counter
from beats.chapter_2 import _ends, percent
from beats.sheets import MISTAKES, SheetPicture, at_share, border, lowest, spot

REST = FILM["batch_square"][-1]
SEARCH = FILM["search"]
X, Y = FILM["x"], FILM["y"]


def line_on(m, v, colour: str, width: float = 1.8) -> Line:
    a, z = _ends(m, v)
    return Line(a, z, stroke_color=colour, stroke_width=width)


def short(scene) -> None:
    m = new_map(at=LEFT * 2.4 + DOWN * 0.3, width=7.2, height=4.3, line=SEARCH)
    sq = line_on(m, REST, style.COOL[2], 2.0)
    pts = FILM["spots"]
    differ = (pts @ REST[:2] + REST[2] > 0) != (pts @ SEARCH[:2] + SEARCH[2] > 0)
    names = VGroup(serif("fewest mistakes", style.WARM[2], 0.28).next_to(m, UP, buff=0.15).align_to(m, LEFT),
                   serif("the square trick, at rest", style.COOL[2], 0.28))
    names[1].next_to(names[0], RIGHT, buff=0.6)
    bars = ScoreBars(percent("search"), "fewest mistakes", at=np.array([4.6, -2.1, 0]))

    def light() -> None:
        m.level[:] = np.where(differ, 1.0, 0.18)
    scene.say("short", Succession(*fill(scene, "short", away(scene), FadeIn(m, run_time=0.5), FadeIn(names[0], run_time=0.3),
                                         Create(sq, run_time=0.8), FadeIn(names[1], run_time=0.3), FadeIn(bars, run_time=0.4),
                                         bars.bar(percent("batch_square"), "square trick").set_run_time(1.0),
                                         Wait(1.4), Do(light))))
    scene.kept = {"map": m, "sq": sq, "names": names, "bars": bars}


def short_2(scene) -> None:
    k = scene.kept
    m = k["map"]
    cuts = VGroup()
    for v, colour, still, walk in ((SEARCH, style.WARM[2], NUMBERS["search_cut_blocks"], NUMBERS["search_cut_walking_blocks"]),
                                   (REST, style.COOL[2], NUMBERS["batch_square_cut_blocks"], NUMBERS["batch_square_cut_walking_blocks"])):
        for x, y in ((still, 0.0), (walk, 1.5)):
            if 0 <= x <= 5:
                cuts.add(Dot(m.page((x, y)), radius=0.06, color=colour))
    walking = Line(m.page((0, 1.5)), m.page((5, 1.5)), stroke_color=style.ASH, stroke_width=0.8, stroke_opacity=0.6)
    notes = VGroup(serif("standing still: about 4 blocks; walking: about 1.5", style.WARM[2], 0.28),
                   serif("standing still: never before the mark; walking: almost from the left edge", style.COOL[2], 0.28)
                   ).arrange(DOWN, aligned_edge=LEFT, buff=0.1).next_to(m, DOWN, buff=0.15).align_to(m, LEFT)
    scene.say("short-2", Succession(*fill(scene, "short-2", FadeOut(k["bars"], run_time=0.3),
                                           Do(lambda: m.level.__setitem__(slice(None), 1.0)), Create(walking, run_time=0.5),
                                           FadeIn(cuts, run_time=0.5), FadeIn(notes[0], run_time=0.5), Wait(1.0),
                                           FadeIn(notes[1], run_time=0.5))))


def why_short(scene) -> None:
    m = new_map(at=LEFT * 2.4 + DOWN * 0.3, width=7.2, height=4.3, line=SEARCH)
    sq = line_on(m, REST, style.COOL[2], 2.0)
    hinge = line_on(m, FILM["hinge"], style.WARM[1], 2.0)
    bars = ScoreBars(percent("search"), "fewest mistakes", slots=3, at=np.array([3.7, -2.1, 0]))
    note = serif("past its target, on its own side: no pull", style.WARM[1], 0.28).next_to(m, UP, buff=0.15)
    scene.say("why-short", Succession(*fill(scene, "why-short", away(scene), FadeIn(m, run_time=0.4), FadeIn(sq, run_time=0.3),
                                             FadeIn(bars, run_time=0.3), bars.bar(percent("batch_square"), "square trick").set_run_time(0.6),
                                             Wait(1.0), FadeIn(note, run_time=0.4), Create(hinge, run_time=1.0),
                                             bars.bar(percent("hinge"), "no pull past it").set_run_time(1.0))))


def how_far(scene) -> None:
    left = new_map(at=LEFT * 3.5 + DOWN * 0.4, width=5.6, height=3.4, line=REST)
    right = new_map(at=RIGHT * 3.5 + DOWN * 0.4, width=5.6, height=3.4, line=SEARCH)
    heads = VGroup(serif("judged by adding up squared misses", style.COOL[2], 0.3).next_to(left, UP, buff=0.25),
                   serif("judged by counting wrong dots", style.WARM[2], 0.3).next_to(right, UP, buff=0.25))
    scores = VGroup(numeral(NUMBERS["batch_square"], style.COOL[2], 0.4).next_to(left, DOWN, buff=0.15),
                    numeral(NUMBERS["search"], style.WARM[2], 0.4).next_to(right, DOWN, buff=0.15))
    scene.say("how-far", Succession(*fill(scene, "how-far", away(scene), FadeIn(left, run_time=0.5), FadeIn(heads[0], run_time=0.4),
                                           FadeIn(right, run_time=0.5), FadeIn(heads[1], run_time=0.4), Wait(1.0),
                                           FadeIn(scores, run_time=0.5))))


def square_outro(scene) -> None:
    m = new_map(at=DOWN * 0.3, width=7.6, height=4.4, line=SEARCH)
    sq = line_on(m, REST, style.COOL[2], 2.0)
    names = VGroup(serif("fewest mistakes", style.WARM[2], 0.28), serif("square trick", style.COOL[2], 0.28)
                   ).arrange(RIGHT, buff=0.6).next_to(m, UP, buff=0.15)
    scene.say("square-outro", Succession(*fill(scene, "square-outro", away(scene), FadeIn(m, run_time=0.5), FadeIn(sq, run_time=0.4),
                                                FadeIn(names, run_time=0.4))))


MAP_AT, SHEET_AT = LEFT * 3.9 + DOWN * 0.5, RIGHT * 2.8 + DOWN * 0.2


def small_map(line=SEARCH):
    return new_map(at=MAP_AT, width=5.4, height=3.3, line=line)


def flat_sheet() -> SheetPicture:
    s = SheetPicture(MISTAKES, smooth=False, lift=0.0, flat=1.0, size=(7.0, 4.4), camera=Camera(pitch=1.0))
    return s.move_to(SHEET_AT)


def map_intro(scene) -> None:
    m = small_map()
    s = flat_sheet()
    label = serif("borders", style.ASH, 0.34).next_to(s, UP, buff=0.1)
    scene.say("map-intro", Succession(*fill(scene, "map-intro", away(scene), FadeIn(m, run_time=0.5),
                                             FadeIn(s, run_time=0.8), FadeIn(label, run_time=0.4))))
    scene.kept = {"map": m, "sheet": s, "label": label}


def map_axes(scene) -> None:
    """A ruler on Mario's map: turned about the middle (half a turn swaps its yes and no sides), then slid."""
    m = scene.kept["map"]
    angle, offset = ValueTracker(np.radians(420)), ValueTracker(0.0)
    middle = Dot(m.page((2.5, 1.25)), radius=0.05, color=style.TEXT)
    m.border.add_updater(lambda _: m.set_line(border(angle.get_value(), offset.get_value())))
    talk = voice(scene, "map-axes")
    t = (talk - 1.0) / 3
    scene.say("map-axes", Succession(FadeIn(middle, run_time=0.4),
                                     UpdateFromAlphaFunc(angle, lambda v, a: v.set_value(np.radians(420 + 180 * a)), run_time=t * 1.6),
                                     UpdateFromAlphaFunc(angle, lambda v, a: v.set_value(np.radians(600 - 180 * a)), run_time=t * 0.6),
                                     UpdateFromAlphaFunc(offset, lambda v, a: v.set_value(0.8 * np.sin(np.pi * a * 2)), run_time=t * 0.8)))
    m.border.clear_updaters()
    scene.kept.update(middle=middle)


def sheet_beat(scene) -> None:
    """A spot moving on the sheet and the ruler on the map moving with it; off the right edge, back on the left."""
    k = scene.kept
    m, s = k["map"], k["sheet"]
    across, up = ValueTracker(0.3), ValueTracker(0.5)
    marker = Dot(radius=0.07, color=style.WARM[0])
    marker.add_updater(lambda d: d.move_to(s.point(across.get_value() % 1.0, up.get_value(), 0.0)))
    m.border.add_updater(lambda _: m.set_line(border(*at_share(across.get_value(), up.get_value()))))
    axes = VGroup(serif("turns the ruler →  (one full turn)", style.ASH, 0.26).next_to(s, DOWN, buff=0.1),
                  serif("slides it →", style.ASH, 0.26).rotate(np.pi / 2).next_to(s, RIGHT, buff=0.08))
    a, b = lowest(MISTAKES)
    best = Dot(s.point(a, b, 0.0), radius=0.07, color=style.WARM[2])
    best_text = serif("the best border we found", style.WARM[2], 0.26).next_to(best, DOWN, buff=0.1)
    talk = voice(scene, "sheet")
    t = talk - 2.0
    scene.say("sheet", Succession(FadeIn(marker, run_time=0.3), FadeIn(axes, run_time=0.4),
                                  UpdateFromAlphaFunc(across, lambda v, al: v.set_value(0.3 + 0.95 * al), run_time=t * 0.5),
                                  UpdateFromAlphaFunc(up, lambda v, al: v.set_value(0.5 + 0.25 * np.sin(2 * np.pi * al)), run_time=t * 0.2),
                                  UpdateFromAlphaFunc(across, lambda v, al: v.set_value(1.25 + (a + 1 - 1.25) * al), run_time=t * 0.15),
                                  UpdateFromAlphaFunc(up, lambda v, al: v.set_value(0.5 + (b - 0.5) * al), run_time=t * 0.15),
                                  FadeIn(best, run_time=0.3), FadeIn(best_text, run_time=0.3)))
    m.border.clear_updaters()
    marker.clear_updaters()
    k.update(marker=marker, axes=axes, best=VGroup(best, best_text))


def staircase(scene) -> None:
    k = scene.kept
    s = k["sheet"]
    height = serif("height: mistakes", style.ASH, 0.3).next_to(s, UP, buff=0.1)
    talk = voice(scene, "staircase")
    scene.say("staircase", Succession(FadeOut(VGroup(k["marker"], k["best"], k["label"]), run_time=0.3),
                                      FadeIn(height, run_time=0.3),
                                      s.rise(0.0, 1.0, 1.0, 0.0).set_run_time(max(talk - 0.6, 1.0))))
    k.update(height=height)


def close_window() -> tuple:
    i, j = np.unravel_index(np.argmin(MISTAKES), MISTAKES.shape)
    return (slice(i - 50, i + 50), slice(j - 50, j + 50)), (i, j)


def staircase_2(scene) -> None:
    """Closer, around the lowest ledge: flat ledges and sudden drops, the lowest ledge lit."""
    window, (i, j) = close_window()
    s = SheetPicture(MISTAKES, smooth=False, size=(10.0, 5.8), camera=Camera(pitch=0.72), window=window, cells=(100, 100))
    h = MISTAKES[window]
    s.glow = np.isclose(h, h.min())[::1, ::1][:100, :100]
    s.redraw()
    s.move_to(DOWN * 0.3)
    note = serif("closer: around the lowest ledge", style.ASH, 0.28).to_edge(UP, buff=0.35)
    low = Dot(s.point(0.505, 0.505), radius=0.07, color=style.WARM[2])
    low_text = serif("the lowest ledge", style.WARM[2], 0.26).next_to(low, UP, buff=0.1)
    scene.say("staircase-2", Succession(*fill(scene, "staircase-2", away(scene), FadeIn(s, run_time=1.0), FadeIn(note, run_time=0.4),
                                               Wait(1.5), FadeIn(VGroup(low, low_text), run_time=0.5))))
    scene.kept = {"close": s}


def big_drops(scene) -> None:
    """A border slid across the crowded point where every life starts: the count of mistakes jumps."""
    m = new_map(at=LEFT * 1.2 + DOWN * 0.3, width=7.4, height=4.3)
    start = np.array([2.5, 0.0])
    owner = np.flatnonzero((FILM["spots"] == start).all(1))[0]
    ring = Circle(radius=0.22, stroke_color=style.TEXT, stroke_width=1.6).move_to(m.page(start))
    ring_text = serif(f"{FILM['spot_moved'][owner] + FILM['spot_still'][owner]} moments: where each life starts",
                      style.TEXT, 0.26).next_to(ring, DOWN + RIGHT, buff=0.05)
    # Steep, and at a slope no two points of the grid share, so near the start it crosses that point alone:
    # the count drops by its 710 moments at once (from 4,938 to 4,228), and only by a few either side.
    w = np.array([1.0, np.pi])
    cut = ValueTracker(2.3)
    count = ValueTracker(0)

    def place(_) -> None:
        v = np.r_[w, -(w[0] * cut.get_value())]
        m.set_line(v)
        count.set_value(int(((X @ v[:2] + v[2] > 0).astype(int) != Y).sum()))
    m.border.add_updater(place)
    place(None)
    shown = counter("mistakes", count, RIGHT * 5.0 + UP * 1.0)  # after the count is set, so it doesn't redraw as it fades in
    scene.say("big-drops", Succession(*fill(scene, "big-drops", away(scene), FadeIn(m, run_time=0.5), FadeIn(shown, run_time=0.3),
                                             Create(ring, run_time=0.5), FadeIn(ring_text, run_time=0.4), Wait(1.0),
                                             UpdateFromAlphaFunc(cut, lambda v, a: v.set_value(2.3 + 0.32 * a), run_time=4.0, rate_func=linear))))
    m.border.clear_updaters()


def no_slope(scene) -> None:
    window, _ = close_window()
    s = SheetPicture(MISTAKES, smooth=False, size=(10.0, 5.8), camera=Camera(pitch=0.72), window=window, cells=(100, 100))
    s.move_to(DOWN * 0.3)
    spot_at = s.point(0.3, 0.62)
    marker = Dot(spot_at, radius=0.08, color=style.TEXT)
    arrows = VGroup(*[Arrow(spot_at, spot_at + 0.55 * np.array([np.cos(a), 0.55 * np.sin(a), 0]), buff=0.08, stroke_width=2,
                            color=style.ASH, max_tip_length_to_length_ratio=0.3) for a in np.linspace(0, 2 * np.pi, 8, endpoint=False)])
    level = serif("level, every way", style.TEXT, 0.28).next_to(marker, UP, buff=0.55)
    scene.say("no-slope", Succession(*fill(scene, "no-slope", away(scene), FadeIn(s, run_time=0.6), FadeIn(marker, run_time=0.3),
                                            Create(arrows, run_time=0.8), FadeIn(level, run_time=0.4))))


def staircase_outro(scene) -> None:
    s = SheetPicture(MISTAKES, smooth=False, size=(10.0, 6.0), camera=Camera(pitch=0.9))
    s.move_to(DOWN * 0.2)
    a, b = lowest(MISTAKES)
    c, d = spot(REST)
    low = Dot(s.point(a, b), radius=0.07, color=style.WARM[2])
    rest = Dot(s.point(c, d), radius=0.07, color=style.COOL[2])
    words = VGroup(serif("lowest ledge", style.WARM[2], 0.26).next_to(low, UP + LEFT, buff=0.05),
                   serif("the square trick", style.COOL[2], 0.26).next_to(rest, DOWN + RIGHT, buff=0.05))
    scene.say("staircase-outro", Succession(*fill(scene, "staircase-outro", away(scene), FadeIn(s, run_time=0.8),
                                                   FadeIn(VGroup(low, words[0]), run_time=0.4), Wait(1.0),
                                                   FadeIn(VGroup(rest, words[1]), run_time=0.4))))


BEATS = {"short": short, "short-2": short_2, "why-short": why_short, "how-far": how_far, "square-outro": square_outro,
         "map-intro": map_intro, "map-axes": map_axes, "sheet": sheet_beat, "staircase": staircase,
         "staircase-2": staircase_2, "big-drops": big_drops, "no-slope": no_slope, "staircase-outro": staircase_outro}
