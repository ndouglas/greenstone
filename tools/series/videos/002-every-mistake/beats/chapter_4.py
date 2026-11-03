"""Chapter 4, "Close, but not quite": where the square trick rests against the fewest-mistakes border, and the
sheet of every border, raised by its mistakes into a staircase."""

import numpy as np
from manim import (DOWN, LEFT, RIGHT, UP, AnimationGroup, Arrow, Circle, Create, Dot, FadeIn, FadeOut, Line,
                   Rectangle, UpdateFromAlphaFunc, ValueTracker, VGroup, Wait, linear)

from series import style
from series.ember import in_line, numeral, serif, swap
from series.lab import rules
from series.records import ScoreBars
from series.sheet import Camera

from beats.common import FILM, NUMBERS, Do, Succession, away, fill, new_map, voice
from beats.chapter_1 import counter
from beats.chapter_2 import _ends, percent
from beats.sheets import (LEDGE, LEDGE_MISTAKES, MISTAKES, MOMENTS, TOP, GroundPicture, across_of, at_share, border,
                          INK, exact, height_key, ledge_ground, lowest, pin, spot, staircase_ground, tag, up_of)
from series.terrain import View

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
    m = new_map(at=LEFT * 3.9 + DOWN * 0.3, width=6.0, height=3.8, line=SEARCH)
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


FLAT_VIEW = View(yaw=0.0, pitch=1.2, distance=13.0, target=(0.0, 0.1, 0.0))
STAIRS_VIEW = View(yaw=0.55, pitch=0.62, distance=12.5, target=(0.0, 0.2, 0.35))
STAIRS_AT, STAIRS_SIZE = LEFT * 0.9 + DOWN * 0.25, (10.6, 6.6)
KEY_AT = RIGHT * 5.6 + DOWN * 0.1


def flat_sheet() -> GroundPicture:
    s = GroundPicture([staircase_ground()], FLAT_VIEW, size=(7.0, 4.4), lift=0.0, flat=1.0, floor=False)
    return s.move_to(SHEET_AT)


def chapter_key():
    """Heights on the chapter's one scale: the share of the moments a border gets wrong."""
    return height_key(0.0, TOP, [(0.0, "none"), (0.25, "25%"), (0.5, "50%"), (0.75, "75%"), (1.0, "all")],
                      "moments wrong").move_to(KEY_AT)


def close_key(g):
    low = g.colours[0] * MOMENTS
    ticks = [(g.colours[0] + d / MOMENTS, f"{int(round(low + d)):,}") for d in (0, 25, 50, 75, 100, 125)]
    return height_key(g.colours[0], g.colours[1], ticks, "mistakes").move_to(KEY_AT)


def stretched(g) -> VGroup:
    return tag(f"close up: heights stretched ×{g.stretch:g}", style.ASH, 0.26).to_corner(UP + LEFT, buff=0.4)


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
    """The flat sheet rises into ground while the camera comes round: true heights, on the chapter's scale."""
    k = scene.kept
    s = k["sheet"]
    key = chapter_key()
    talk = voice(scene, "staircase")
    s.floor = True
    scene.say("staircase", Succession(away(scene, s, run_time=0.4),
                                      s.fly(STAIRS_VIEW, lift=(0.0, 1.0), flat=(1.0, 0.0), place=(STAIRS_AT, STAIRS_SIZE),
                                            run_time=max(talk - 1.2, 2.0)),
                                      FadeIn(key, run_time=0.6)))
    scene.kept = {"stairs": s, "key": key}


def _patch():
    fine = ledge_ground(np.radians(4), 0.12, cells=400, stretch=1.0)
    local = fine.colours
    fine.colours = (0.0, TOP)  # on the chapter's colours until it's stretched
    hole = (fine.across[0], fine.across[1], fine.up[0], fine.up[1])
    return staircase_ground(hole=hole), fine, local


def _on_ledge(s, ground: int = 1):
    a, b = float(across_of(LEDGE[0])), float(up_of(LEDGE[1]))
    dot = pin(style.WARM[2]).add_updater(lambda d: d.move_to(s.point(a, b, ground=ground)))
    words = tag(f"the lowest ledge: {LEDGE_MISTAKES} mistakes", style.WARM[2])
    words.add_updater(lambda w: w.next_to(dot, UP + RIGHT, buff=0.08))
    return dot.update(), words.update()


def close_view(fine, yaw: float = 0.35) -> View:
    span = (fine.across[1] - fine.across[0]) * 8.0
    return View(yaw=yaw, pitch=0.68, distance=span * 2.9, target=tuple(fine.world(float(across_of(LEDGE[0])), float(up_of(LEDGE[1])))))


def full_frame() -> tuple[float, float]:
    from manim import config
    return config.frame_width, config.frame_height


def staircase_2(scene) -> None:
    """Down toward the lowest ledge; there, the sheet's samples give way to the ground computed exactly; then
    the heights are stretched (said on screen) so its ledges and drops show, and the camera comes slowly round."""
    k = scene.kept
    s, key = k["stairs"], k["key"]
    _, fine, local = _patch()
    near = close_view(fine)
    mid = near.moved(distance=near.distance * 4.0, pitch=0.8)
    close = GroundPicture([fine], mid, size=full_frame(), floor=False)
    talk = voice(scene, "staircase-2")
    note = stretched(type("g", (), {"stretch": 2})())
    exactly = tag("here, every border counted exactly", style.ASH, 0.26).to_corner(UP + LEFT, buff=0.4)
    new_key = close_key(type("g", (), {"colours": local})())
    dot, words = _on_ledge(close, ground=0)
    scene.say("staircase-2", Succession(s.fly(mid, place=(np.zeros(3), full_frame()), run_time=2.6),
                                        AnimationGroup(FadeIn(close, run_time=0.8), FadeOut(s, run_time=0.8), FadeIn(exactly, run_time=0.8)),
                                        close.fly(near, run_time=1.2),
                                        AnimationGroup(close.fly(stretch=(1.0, 2.0), colours=((0.0, TOP), local), run_time=1.4),
                                                       FadeOut(key, run_time=0.5), FadeIn(new_key, run_time=1.0),
                                                       FadeOut(exactly, run_time=0.4), FadeIn(note, run_time=0.8)),
                                        FadeIn(VGroup(dot, words), run_time=0.4),
                                        close.fly(near.moved(yaw=near.yaw + 0.7), run_time=max(talk - 6.4, 1.5), rate_func=linear)))
    dot.clear_updaters()
    words.clear_updaters()
    scene.kept = {"close": close}


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


def share_of(g, i: float, j: float) -> tuple[float, float]:
    na, no = g.value.shape
    return (g.across[0] + (i + 0.5) / na * (g.across[1] - g.across[0]), g.up[0] + (j + 0.5) / no * (g.up[1] - g.up[0]))


def no_slope(scene) -> None:
    """A marker on a real ledge, with feelers shorter than the ledge: every one reads no change. Then it
    steps off, once up and once down, and only then does the count change."""
    _, fine, local = _patch()
    fine.colours, fine.stretch = local, 2.0
    s = GroundPicture([fine], close_view(fine, yaw=0.9), size=full_frame(), floor=False)
    v = fine.value
    # the widest ledge near the lowest one: the cell whose level reaches furthest in every direction
    dirs = [(np.cos(t), np.sin(t)) for t in np.linspace(0, 2 * np.pi, 8, endpoint=False)]
    na, no = v.shape

    def reach(i, j, d, limit=80):
        for r in range(1, limit):
            a, b = int(round(i + d[0] * r)), int(round(j + d[1] * r))
            if not (0 <= a < na and 0 <= b < no) or v[a, b] != v[i, j]:
                return r
        return limit
    best, where = 0, (na // 2, no // 2)
    for i in range(40, na - 40, 6):
        for j in range(40, no - 40, 6):
            if v[i, j] > v.min() + 60 / MOMENTS:
                continue
            r = min(reach(i, j, d) for d in dirs)
            if r > best:
                best, where = r, (i, j)
    i, j = where
    cell = (fine.across[1] - fine.across[0]) * 8.0 / na  # a cell's width, in world units
    s.view = View(yaw=0.9, pitch=0.75, distance=best * cell * 16, target=tuple(fine.world(*share_of(fine, i, j))))
    s.redraw()
    share = lambda a, b: (fine.across[0] + (a + 0.5) / na * (fine.across[1] - fine.across[0]),
                          fine.up[0] + (b + 0.5) / no * (fine.up[1] - fine.up[0]))
    here = s.point(*share(i, j), ground=0)
    marker = pin(style.TEXT, 0.1).move_to(here)
    length = 0.6 * best
    feelers = VGroup(*[Arrow(here, s.point(*share(i + d[0] * length, j + d[1] * length), ground=0), buff=0.14, stroke_width=4,
                             color=INK, max_tip_length_to_length_ratio=0.22, max_stroke_width_to_length_ratio=8) for d in dirs])
    zeros = VGroup(*[tag("0", style.TEXT, 0.24).move_to(f.get_end() + 0.26 * (f.get_end() - here) / max(np.linalg.norm(f.get_end() - here), 1e-6))
                     for f in feelers])
    level = tag("no change, every way", style.TEXT).next_to(marker, UP, buff=1.0)
    count = int(round(v[i, j] * MOMENTS))
    reading = tag(f"here: {count:,} mistakes", style.TEXT).to_corner(UP + RIGHT, buff=0.4)

    def walk(d):
        """Off the ledge in direction d: to the first cell that isn't level with it, and its change."""
        r = reach(i, j, d) + 3  # a little way past the edge, onto the next ledge
        a, b = int(round(i + d[0] * r)), int(round(j + d[1] * r))
        return (a, b), int(round((v[a, b] - v[i, j]) * MOMENTS))
    offs = [walk(d) for d in dirs]
    up_ = next(((c, n) for c, n in offs if n > 0), None)
    down = next(((c, n) for c, n in offs if n < 0), None)
    steps = []
    for found in (up_, down):
        if found is None:
            continue
        (a, b), change = found
        to = s.point(*share(a, b), ground=0)
        sign = "+" if change > 0 else "\N{MINUS SIGN}"
        label = tag(f"{sign}{abs(change)} mistakes", style.WARM[1] if change > 0 else style.WARM[2]).next_to(to, RIGHT, buff=0.15)
        glide = lambda p, q, t: UpdateFromAlphaFunc(marker, lambda m, a: m.move_to(p + (q - p) * a), run_time=t)
        steps += [glide(here, to, 0.7), FadeIn(label, run_time=0.3), Wait(0.9), FadeOut(label, run_time=0.3), glide(to, here, 0.5)]
    key = close_key(fine)
    scene.say("no-slope", Succession(*fill(scene, "no-slope", away(scene), FadeIn(s, run_time=0.6), FadeIn(key, run_time=0.4),
                                            FadeIn(stretched(fine), run_time=0.4), FadeIn(VGroup(marker, reading), run_time=0.3),
                                            Create(feelers, run_time=0.8), FadeIn(zeros, run_time=0.4), FadeIn(level, run_time=0.4),
                                            Wait(1.2), FadeOut(VGroup(feelers, zeros, level), run_time=0.4), *steps)))
    scene.kept = {"close": s, "key": key}


def staircase_outro(scene) -> None:
    """Back out over the whole staircase, coming round as it goes: the lowest ledge, and where the square
    trick rests, off it."""
    _, fine, local = _patch()
    fine.colours, fine.stretch = local, 2.0
    near = close_view(fine, yaw=0.9)
    mid = near.moved(distance=near.distance * 9.0, pitch=0.8)
    close = GroundPicture([fine], near, size=full_frame(), floor=False)
    s = GroundPicture([staircase_ground()], mid, size=full_frame())
    far = View(yaw=-0.35, pitch=0.85, distance=13.0, target=(0.0, 0.25, 0.3))
    c, d = spot(REST)
    rest_mistakes = int(round(exact([at_share(c, d)[0]], [at_share(c, d)[1]])[0, 0] * MOMENTS))
    dot, words = _on_ledge(s, ground=0)
    rest = pin(style.WARM[1]).add_updater(lambda m: m.move_to(s.point(c, d, ground=0)))
    rest_words = tag(f"the square trick, at rest: {rest_mistakes:,} mistakes", style.WARM[1])
    rest_words.add_updater(lambda w: w.next_to(rest, DOWN + RIGHT, buff=0.08))
    key = chapter_key()
    near_key = close_key(fine)
    talk = voice(scene, "staircase-outro")
    scene.say("staircase-outro", Succession(away(scene), FadeIn(close, run_time=0.5), FadeIn(near_key, run_time=0.3),
                                            AnimationGroup(close.fly(mid, stretch=(2.0, 1.0), colours=(local, (0.0, TOP)), run_time=1.6),
                                                           FadeOut(near_key, run_time=0.6), FadeIn(key, run_time=1.0)),
                                            AnimationGroup(FadeIn(s, run_time=0.8), FadeOut(close, run_time=0.8)),
                                            s.fly(far, run_time=max(talk - 9.5, 3.0)),
                                            FadeIn(VGroup(dot.update(), words.update()), run_time=0.5), Wait(1.5),
                                            FadeIn(VGroup(rest.update(), rest_words.update()), run_time=0.5),
                                            s.fly(far.moved(yaw=far.yaw - 0.25), run_time=3.0, rate_func=linear)))
    for m in (dot, words, rest, rest_words):
        m.clear_updaters()


BEATS = {"short": short, "short-2": short_2, "why-short": why_short, "how-far": how_far, "square-outro": square_outro,
         "map-intro": map_intro, "map-axes": map_axes, "sheet": sheet_beat, "staircase": staircase,
         "staircase-2": staircase_2, "big-drops": big_drops, "no-slope": no_slope, "staircase-outro": staircase_outro}
