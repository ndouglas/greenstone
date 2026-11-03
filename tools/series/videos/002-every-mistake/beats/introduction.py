"""The introduction: last time's map, the new question on the game itself, its map, and why the neuron must find
its own border."""

import json

import numpy as np
from manim import (DOWN, LEFT, RIGHT, UP, AnimationGroup, Arc, Circle, Create, Dot, FadeIn, FadeOut, Line,
                   UpdateFromAlphaFunc, ValueTracker, VGroup, VMobject, Wait, linear, smooth)

from series import style
from series.bordermap import BorderMap
from series.ember import in_line, numeral, serif, swap
from series.knobs import Knobs
from series.lab.data import load
from series.lab.exp001 import LAND_WITHIN, in_play, label, landing
from series.props import BUTTON_B, Pad, nes, tick
from series.recording import DATA
from series.records import ScoreBars
from series.script import VIDEOS

from beats.common import (BOTH, BUTTONS, FILM, MARK, NUMBERS, BlockRuler, Do, MarkOnScreen, Readout, arrive_held,
                          show, tween, SpeedArrowRight, Succession, away, blocks, fill, legend, new_map, pause,
                          resume, screen_at, speed, voice)
START = 1  # the footage's first life: Mario walks, then holds B and runs
# A life that walks to the mark without running, from two frames in (the picture trails the RAM by one, and the
# life's first frame still shows the last life's position); the screen starts to move 76 frames on.
WALK = 8864
ASK = WALK + 50  # Mario short of the mark, the screen still; it moves 26 frames later
LEFTWARD = 15650  # Mario turns and walks left, short of the mark
PLAYER = 9680  # the random player, busy
SCREEN_AT = LEFT * 2.6 + UP * 0.2


def about(scene) -> None:
    console = nes(UP * 0.2)
    screen = screen_at(START, RIGHT * 2.6 + UP * 0.2)
    scene.add(screen)
    scene.say("about", Succession(*fill(scene, "about",
        Create(console, run_time=1.4),
        console.animate(run_time=0.6).scale(0.55).move_to(LEFT * 4.4 + UP * 0.2),
        screen.arrive(run_time=0.8), resume(screen))))


def landing_moments(n: int = 400):
    """001's map: a sample of its moments in the air (depth across, falling speed up) and where each landed."""
    r = load(DATA / "smb1" / "1")
    air = label("smb1")(r.ram)
    keep, lands = landing(air, r.meta.get("restart_frames", []), LAND_WITHIN)
    play = in_play("smb1", r.ram)[keep]
    keep, lands = keep[play], lands[play]
    vy = r.ram[keep, 0x009F].astype(float)
    points = np.stack([r.ram[keep, 0x00CE].astype(float), np.where(vy > 127, vy - 256, vy)], 1)
    outcomes = ["floor" if soon else None for soon in lands]
    pick = np.random.default_rng(3).choice(len(points), min(n, len(points)), replace=False)
    return points[pick], [outcomes[i] for i in pick]


def last_map() -> BorderMap:
    w0, w1, b = json.loads((VIDEOS / "001-one-neuron" / "numbers.json").read_text())["smb1_land_weights"]
    points, outcomes = landing_moments()
    m = BorderMap(points, outcomes, (w0, w1), b, "depth", "falling speed", "lands soon", "not yet",
                  width=8.4, height=4.8, at=DOWN * 0.1)
    m.best = (np.array([w0, w1]), b)
    return m


def last_time(scene) -> None:
    m = last_map()
    scene.kept = {"last": m}
    scene.say("last-time", Succession(*fill(scene, "last-time", away(scene), FadeIn(m, run_time=1.6))))


def last_time_2(scene) -> None:
    m = scene.kept["last"]
    (w, b) = m.best
    rng = np.random.default_rng(7)
    tries = []
    for k in range(6):  # borders tried, each leaning and sliding somewhere else, then the best
        tries.append(m.lean(tuple(w * rng.uniform(0.4, 1.8, 2))))
        tries.append(m.slide_bias(b * rng.uniform(0.6, 1.4)))
    t = (voice(scene, "last-time-2") - 2.4) / len(tries)
    scene.say("last-time-2", Succession(*[a.set_run_time(t) for a in tries],
                                        m.lean(tuple(w)).set_run_time(1.2), m.slide_bias(b).set_run_time(1.2)))


def the_line(scene) -> None:
    screen = screen_at(WALK, UP * 0.15)
    scene.add(screen)
    ruler, mark = BlockRuler(screen), MarkOnScreen(screen)
    mark.label.next_to(ruler, UP, buff=0.32).set_x(mark.line.get_x())
    talk = voice(scene, "the-line")
    walk = 76 / 60 + 1.2  # to the mark, then a little of the screen moving
    scene.say("the-line", Succession(
        away(scene, screen),
        arrive_held(screen, run_time=0.8),
        Create(ruler, run_time=2.0),
        Create(mark.line, run_time=0.8), FadeIn(mark.label, run_time=0.4),
        Wait(max(talk - 0.5 - 0.8 - 2.0 - 1.2 - walk, 0.1)),
        resume(screen), Wait(walk)))
    scene.kept = {"screen": screen, "ruler": ruler, "mark": mark}


def timer(at: np.ndarray, frames: ValueTracker) -> VGroup:
    """Half a second as a ring filling frame by frame, its count of thirty in the middle."""
    ring = Circle(radius=0.42, stroke_color=style.ASH, stroke_width=1.2).move_to(at)
    arc = Arc(radius=0.42, arc_center=at, start_angle=np.pi / 2, angle=-1e-4, stroke_color=style.WARM[0], stroke_width=3)
    count = VGroup()

    def draw(m) -> None:
        f = frames.get_value()
        arc.become(Arc(radius=0.42, arc_center=at, start_angle=np.pi / 2, angle=-2 * np.pi * max(f, 1e-3) / 30,
                       stroke_color=style.WARM[0], stroke_width=3))
        text = f"{int(f)}"
        if getattr(count, "shown", None) != text:
            swap(count, numeral(text, style.TEXT, 0.34).move_to(at))
            count.shown = text
    group = VGroup(ring, arc, count)
    group.add_updater(draw)
    draw(group)
    group.add(serif("frames of 30", style.ASH, 0.28).next_to(ring, DOWN, buff=0.16))
    return group


def question(scene) -> None:
    screen = screen_at(ASK, SCREEN_AT)
    scene.add(screen)
    mark = MarkOnScreen(screen)
    ask = VGroup(serif("will the screen start to move", style.TEXT, 0.4),
                 serif("within the next half second?", style.TEXT, 0.4)).arrange(DOWN, buff=0.14)
    ask.move_to(RIGHT * 3.6 + UP * 1.4)
    frames = ValueTracker(0)
    clock = timer(RIGHT * 3.6 + DOWN * 0.6, frames)
    scene.say("question", Succession(*fill(scene, "question",
        away(scene, screen), arrive_held(screen, run_time=0.6), Create(mark, run_time=0.6),
        FadeIn(ask, run_time=0.8), FadeIn(clock, run_time=0.6))))
    scene.kept = {"screen": screen, "mark": mark, "ask": ask, "clock": clock, "frames": frames}


def test_question(scene) -> None:
    k = scene.kept
    screen, frames = k["screen"], k["frames"]
    slow = 0.25  # a quarter of the game's speed, so half a second takes two
    watch = 30 / 60 / slow
    answer = VGroup(tick(np.zeros(3), 0.3), serif("it moved", style.WARM[1], 0.4)).arrange(RIGHT, buff=0.2)
    answer.next_to(k["clock"], DOWN, buff=0.55)
    note = serif("a quarter of the game's speed", style.ASH, 0.26).next_to(screen.game, DOWN, buff=0.2)
    scene.say("test-question", Succession(
        FadeIn(note, run_time=0.4), resume(screen, slow),
        UpdateFromAlphaFunc(frames, lambda m, a: m.set_value(30 * a), run_time=watch, rate_func=linear),
        pause(screen), Create(answer[0], run_time=0.4), FadeIn(answer[1], run_time=0.4),
        Wait(max(voice(scene, "test-question") - watch - 1.25, 0.1))))
    scene.kept["answer"] = answer


def readouts(screen, at: np.ndarray) -> VGroup:
    return VGroup(Readout("blocks from the left edge", lambda: blocks(screen), at),
                  Readout("pixels a frame, right", lambda: speed(screen), at + DOWN * 0.6, colour_by_sign=True))


def numbers(scene) -> None:
    leaving = away(scene)
    screen = screen_at(WALK, SCREEN_AT)
    scene.add(screen)
    mark = MarkOnScreen(screen)
    rows = readouts(screen, RIGHT * 1.2 + UP * 0.6)
    arrow = SpeedArrowRight(screen)
    scene.add(rows, arrow)
    note = serif("half the game's speed", style.ASH, 0.26).next_to(screen.game, DOWN, buff=0.2)
    talk = voice(scene, "numbers")
    scene.say("numbers", Succession(
        leaving, arrive_held(screen, run_time=0.6), Create(mark, run_time=0.5),
        show(rows[0], 0.6), show(rows[1], 0.6), show(arrow, 0.2),
        Wait(max(talk - 0.6 - 0.5 - 1.4 - 0.4 - 74 / 30, 0.1)),
        FadeIn(note, run_time=0.4), resume(screen, 0.5), Wait(74 / 30), pause(screen)))
    scene.kept = {"screen": screen}


def numbers_2(scene) -> None:
    leaving = away(scene)
    screen = screen_at(START, SCREEN_AT)
    scene.add(screen)
    rows = readouts(screen, RIGHT * 1.2 + UP * 1.6)
    pad = Pad(screen, RIGHT * 3.0 + DOWN * 1.2, BUTTONS)
    pad.level.set_value(0)
    arrow = SpeedArrowRight(screen)
    scene.add(rows[1], pad, arrow)
    note = serif("half the game's speed", style.ASH, 0.26).next_to(screen.game, DOWN, buff=0.2)
    run = 110 / 30  # to running speed (frame 98), at half speed
    talk = voice(scene, "numbers-2")
    scene.say("numbers-2", Succession(
        leaving, arrive_held(screen, run_time=0.6),
        show(rows[1], 0.5), tween(pad.level, 0, 1, 0.5), show(arrow, 0.2), FadeIn(note, run_time=0.3),
        Wait(max(talk - 2.1 - run, 0.1)), resume(screen, 0.5), Wait(run), pause(screen)))


def only_these(scene) -> None:
    leaving = away(scene)
    screen = screen_at(LEFTWARD, SCREEN_AT)
    scene.add(screen)
    mark = MarkOnScreen(screen)
    rows = readouts(screen, RIGHT * 1.2 + UP * 0.6)
    arrow = SpeedArrowRight(screen)
    scene.add(rows, arrow)
    out = serif("left out", style.COOL[1], 0.42).move_to(RIGHT * 3.4 + DOWN * 1.2)
    walk = 50 / 30
    talk = voice(scene, "only-these")
    scene.say("only-these", Succession(
        leaving, arrive_held(screen, run_time=0.6), FadeIn(mark, run_time=0.6), show(rows, 0.6), show(arrow, 0.6),
        resume(screen, 0.5), Wait(walk), pause(screen), FadeIn(out, run_time=0.5),
        Wait(max(talk - 1.8 - walk - 0.5, 0.1))))


def the_map(scene) -> None:
    m = new_map(at=DOWN * 0.15 + LEFT * 0.6)
    key = legend(RIGHT * 5.0 + UP * 2.2)
    scene.say("the-map", Succession(*fill(scene, "the-map", away(scene), FadeIn(m, run_time=1.8), FadeIn(key, run_time=0.6))))
    scene.kept = {"map": m, "legend": key}


def player(scene) -> None:
    leaving = away(scene)
    screen = screen_at(PLAYER, UP * 0.6)
    scene.add(screen)
    pad = Pad(screen, DOWN * 3.0, BUTTONS)
    pad.set_x(screen.get_x())  # centred under the screen
    pad.level.set_value(0)
    scene.add(pad)
    scene.say("player", Succession(*fill(scene, "player",
        leaving, screen.arrive(run_time=0.6), tween(pad.level, 0, 1, 0.5), resume(screen))))


def search(scene) -> None:
    m = new_map(at=LEFT * 2.2 + DOWN * 0.15, width=7.0, height=4.4)
    best = FILM["search"]
    bars = ScoreBars(float(NUMBERS["scrolls_soon_rate"].rstrip("%")), "always yes", at=np.array([4.6, -2.0, 0]))
    rng = np.random.default_rng(11)
    tries = []
    for _ in range(14):  # borders turned and slid at random, as a search tries them
        a = rng.uniform(0, 2 * np.pi)
        w = np.array([np.cos(a), np.sin(a)]) * np.linalg.norm(best[:2])
        mid = np.array([rng.uniform(0.5, 4.5), rng.uniform(0.2, 2.3)])
        tries.append(np.r_[w, -w @ mid])
    tries.append(best)
    sweep = voice(scene, "search") - 3.4
    scene.say("search", Succession(
        away(scene), FadeIn(m, run_time=0.8), m.jump(np.array(tries)).set_run_time(sweep), Wait(0.3),
        FadeIn(bars, run_time=0.5), bars.bar(float(NUMBERS["search"].rstrip("%")), "best border").set_run_time(1.3)))
    scene.kept = {"map": m, "bars": bars}


def search_2(scene) -> None:
    m = scene.kept["map"]
    w, b = FILM["search"][:2], FILM["search"][2]
    still = -b / w[0]
    walking = (-b - 1.5 * w[1]) / w[0]
    marks = VGroup()
    for x, y, words in ((still, 0.0, "about 4 blocks, standing still"), (walking, 1.5, "about 1.5, walking")):
        p = m.page((x, y))
        marks.add(Dot(p, radius=0.06, color=style.TEXT),
                  serif(words, style.TEXT, 0.28).next_to(p, UP + RIGHT, buff=0.08))
    walk_line = Line(m.page((0, 1.5)), m.page((5, 1.5)), stroke_color=style.ASH, stroke_width=0.8, stroke_opacity=0.6)
    yes = serif("yes", style.WARM[1], 0.36).move_to(m.page((4.5, 2.3)))
    scene.say("search-2", Succession(*fill(scene, "search-2",
        FadeOut(scene.kept["bars"], run_time=0.5), Create(walk_line, run_time=0.6),
        FadeIn(marks[0:2], run_time=0.6), Wait(1.4), FadeIn(marks[2:4], run_time=0.6), FadeIn(yes, run_time=0.5))))


RUNGS = (2, 3, 5, 7, 10, 14, 16, 40)  # numbers read, at each of which the heap passes a known thing


def rungs(a: float, stops=RUNGS) -> float:
    """Numbers read, part way through: an equal time from each rung to the next, easing between."""
    k = min(a * (len(stops) - 1), len(stops) - 1 - 1e-9)
    i, f = int(k), k - int(k)
    f = f * f * (3 - 2 * f)
    return stops[i] + (stops[i + 1] - stops[i]) * f


def too_many(scene) -> None:
    """The memory's numbers lit one by one, two first (the map's), then in address order; beside it, one grain of
    sand for every border the same search would try, heaped, and the camera pulling back past things we know."""
    from series.memory import MemoryGrid
    from beats.common import recording
    from beats.scale import Heap, borders, count
    leaving = away(scene)
    grid = MemoryGrid(recording(), columns=64, width=5.8, at=LEFT * 3.7 + UP * 0.9)
    image, rows = grid.image, 2048 // 64
    two = [0x0755, 0x0057]
    order = two + [a for a in range(2048) if a not in two][:RUNGS[-1] - 2]
    squares = VGroup(*[Dot([image.get_left()[0] + (a % 64 + 0.5) * image.width / 64,
                            image.get_top()[1] - (a // 64 + 0.5) * image.height / rows, 0], radius=0.05, color=style.WARM[0])
                       for a in order])
    n = ValueTracker(2.0)
    pile = Heap(RIGHT * 3.4 + DOWN * 2.0, width=6.4)
    grain = serif("one grain of sand, a millimetre across, for each border", style.ASH, 0.26).move_to(RIGHT * 3.4 + UP * 3.1)
    words = VGroup()
    share = serif(f"{RUNGS[-1]} numbers: {RUNGS[-1] / 2048:.0%} of the console's memory", style.TEXT, 0.3).move_to(LEFT * 3.7 + DOWN * 3.3)

    def draw(_) -> None:
        k = int(round(n.get_value()))
        for i, d in enumerate(squares):
            d.set_opacity(1.0 if i < k else 0.0)
        pile.show(n.get_value())
        text = (f"{k} numbers read", f"{count(borders(k))} borders to try")
        if getattr(words, "text_", None) != text:
            swap(words, VGroup(serif(text[0], style.WARM[0], 0.36), serif(text[1], style.WARM[2], 0.36))
                 .arrange(DOWN, buff=0.18).move_to(LEFT * 3.7 + DOWN * 2.3))
            words.text_ = text
    words.add_updater(draw)
    draw(words)
    talk = voice(scene, "too-many")
    scene.say("too-many", Succession(
        leaving, FadeIn(grid, run_time=0.6), FadeIn(VGroup(squares, words), run_time=0.4),
        AnimationGroup(FadeIn(pile, run_time=0.6), FadeIn(grain, run_time=0.6)), Wait(1.2),
        UpdateFromAlphaFunc(n, lambda m, a: m.set_value(rungs(a)), run_time=max(talk - 5.0, 3.0), rate_func=linear),
        FadeIn(share, run_time=0.4)))
    words.clear_updaters()
    scene.kept = {"words": words}


def no_fast_way(scene) -> None:
    """The time the same search would take on this computer, on a ribbon of spans we know, as numbers are added:
    past a lifetime at six, past the age of the universe at ten."""
    from beats.scale import AGE, YEAR, seconds, span
    left, right, y = -6.3, 6.3, -0.4
    top = np.log10(AGE * YEAR * 8)

    def x_of(s: float) -> float:
        return left + (right - left) * np.log10(max(s, 1.0)) / top
    axis = Line([left, y, 0], [right, y, 0], stroke_color=style.ASH, stroke_width=1.4)
    marks = VGroup()
    for s, name in ((1, "a second"), (60, "a minute"), (86400, "a day"), (YEAR, "a year"), (80 * YEAR, "a lifetime"),
                    (5000 * YEAR, "all of written history"), (AGE * YEAR, "the age of the universe")):
        t = Line([x_of(s), y - 0.12, 0], [x_of(s), y + 0.12, 0], stroke_color=style.ASH, stroke_width=1.4)
        marks.add(t, serif(name, style.ASH, 0.24).rotate(0.6).next_to(t, DOWN, buff=0.1).shift(RIGHT * 0.25))
    n = ValueTracker(2.0)
    bar = VMobject()
    words = VGroup()

    def draw(_) -> None:
        s = seconds(n.get_value())
        bar.become(Line([x_of(1.0), y, 0], [x_of(s), y, 0], stroke_color=style.WARM[2], stroke_width=9))
        k = int(n.get_value() + 1e-6)
        text = f"{k} numbers: {span(seconds(k))}"
        if getattr(words, "text_", None) != text:
            swap(words, serif(text, style.WARM[2], 0.34).move_to([np.clip(x_of(seconds(k)), -4.2, 4.2), y + 0.75, 0]))
            words.text_ = text
    bar.add_updater(draw)
    draw(bar)
    note = serif("how long the same search would take, on this computer", style.ASH, 0.28).move_to(UP * 2.4)
    talk = voice(scene, "no-fast-way")
    scene.say("no-fast-way", Succession(away(scene), FadeIn(VGroup(axis, marks, note), run_time=0.6), FadeIn(VGroup(bar, words), run_time=0.3),
                                        UpdateFromAlphaFunc(n, lambda m, a: m.set_value(rungs(a, (2, 3, 4, 5, 6, 7, 8, 9, 10))),
                                                            run_time=max(talk - 1.2, 2.5), rate_func=linear)))
    bar.clear_updaters()


def learn(scene) -> None:
    m = new_map(at=RIGHT * 1.9 + DOWN * 0.3, width=6.8, height=4.2)
    knobs = Knobs(LEFT * 4.6 + UP * 1.2, scales=(0.03, 0.03, 0.03))
    first = FILM["run_after"][:12]
    moments = FILM["run_moment"][:12]
    x, owner = FILM["x"], FILM["spot_of"]
    talk = voice(scene, "learn")
    steps = []
    for v, i in zip(first, moments):
        spot = owner[i]

        def ring(spot=spot) -> None:
            m.ring[:] = False
            m.ring[spot] = True
        steps += [Do(ring), Wait(0.35), AnimationGroup(m.turn_to(v), knobs.turn_to(v), run_time=0.45)]
    scene.say("learn", Succession(
        away(scene), FadeIn(m, run_time=0.8), FadeIn(knobs, run_time=0.6),
        Wait(max(talk - 1.4 - 0.81 * len(first) - 0.2, 0.1)), *steps, Do(lambda: m.ring.__setitem__(slice(None), False))))
    knobs.add_updater(lambda k: k.set_values(m.v))  # from here on the knobs read the border's weights
    scene.kept = {"map": m, "knobs": knobs}


def this_video(scene) -> None:
    m, knobs = scene.kept["map"], scene.kept["knobs"]
    late = FILM["run_after"][FILM["run_pass"] >= 25][::7][:120]
    scene.say("this-video", m.jump(late))


BEATS = {"about": about, "last-time": last_time, "last-time-2": last_time_2, "the-line": the_line,
         "question": question, "test-question": test_question, "numbers": numbers, "numbers-2": numbers_2,
         "only-these": only_these, "the-map": the_map, "player": player, "search": search, "search-2": search_2,
         "too-many": too_many, "no-fast-way": no_fast_way, "learn": learn, "this-video": this_video}
