"""The introduction: its beats, each a function that says its own line given the scene."""

import re

import numpy as np
from manim import (DOWN, LEFT, ORIGIN, RIGHT, UP, AnimationGroup, Circle, Create, FadeIn, FadeOut, Line, Rectangle,
                   Restore, RoundedRectangle, Succession, UpdateFromAlphaFunc, ValueTracker, VGroup, Wait, config,
                   linear, smooth)

from series import style
from series.build import OUT
from series.credits import cited
from series.ember import numeral, serif, swap
from series.homebrew.chase import make as chase
from series.memory import MemoryGrid
from series.pacing import STILLNESS
from series.recording import Film, Recording
from series.screen import Screen
from series.web import Web

from beats.chapter_1 import AIR, CUT, DEPTH, START, grid_away, tick, tween

ARRIVE = 0.8  # a screen's arrival; its footage holds its first frame until it's done
GAMES = (("smb1", START), ("smb3", 600), ("castlevania", 900))  # the about beat's games, each from a moment of play
POSITION, LIVES = 0x0086, 0x075A  # SMB1: the player's x on screen; lives (Data Crystal's RAM map)


def held(scene, line_id: str) -> float:
    """How long a line holds the screen: its voice and the stillness after it."""
    line = next(l for l in scene.video.lines if l.id == line_id)
    return scene.voices[line_id].seconds + (line.pause if line.pause is not None else STILLNESS)


def nes(at: np.ndarray, width: float = 3.2) -> VGroup:
    """The console in hairlines: its body, the darker front stripe, the cartridge door, two buttons, two ports."""
    k = width / 3.2
    body = RoundedRectangle(corner_radius=0.06 * k, width=3.2 * k, height=1.15 * k, stroke_color=style.ASH,
                            stroke_width=1.4)
    stripe = Line(LEFT * 1.6 * k, RIGHT * 1.6 * k, stroke_color=style.ASH, stroke_width=1.0).shift(DOWN * 0.18 * k)
    door = Rectangle(width=1.7 * k, height=0.42 * k, stroke_color=style.ASH, stroke_width=1.2).shift(
        UP * 0.2 * k + RIGHT * 0.15 * k)
    buttons = VGroup(*[Rectangle(width=0.22 * k, height=0.09 * k, stroke_color=style.WARM[0], stroke_width=1.2).shift(
        LEFT * (1.15 - 0.32 * i) * k + DOWN * 0.38 * k) for i in range(2)])
    ports = VGroup(*[Rectangle(width=0.18 * k, height=0.2 * k, stroke_color=style.ASH, stroke_width=1.0).shift(
        RIGHT * (0.3 + 0.35 * i) * k + DOWN * 0.38 * k) for i in range(2)])
    return VGroup(body, stripe, door, buttons, ports).move_to(at)


def about(scene) -> None:
    """The console, and its games on screen in turn."""
    scene.kept = {}
    console = nes(UP * 0.2)
    voice = scene.voices["about"].seconds
    screens = [Screen(Recording(game), start, centre=RIGHT * 2.4) for game, start in GAMES]
    for s in screens:
        s.fade(0)
        s.game.suspend_updating()  # held on its first frame until it arrives
    turn = (voice - 1.6) / 3  # each game's time on screen, after the console's
    steps = [Create(console, run_time=1.2),
             console.animate(run_time=0.4).scale(0.6).move_to(LEFT * 4.6 + UP * 0.2)]
    for i, s in enumerate(screens):
        scene.add(s)
        steps.append(AnimationGroup(s.arrive(run_time=0.5), *([screens[i - 1].depart(run_time=0.5)] if i else []),
                                    run_time=0.5))
        steps.append(Wait(turn - 0.5))
    scene.say("about", Succession(*steps))
    scene.kept.update(console=console, screen=screens[-1])
    scene.remove(*screens[:-1])


def chase_screen(scene) -> Screen:
    """The chase, filmed for this read of its three lines."""
    phases = (held(scene, "who") - ARRIVE, held(scene, "who-2"), held(scene, "who-3"))
    root = chase.film(phases, OUT / "homebrew" / "chase")
    screen = Screen(Film("chase", root), chase.FIRST_DRAWN, centre=UP * 0.45)
    screen.fade(0)
    screen.game.suspend_updating()  # held on its first frame until it arrives
    return screen


def who(scene) -> None:
    """The games go; the chase arrives, and 8-bit Nate walks in. His name beneath."""
    screen = chase_screen(scene)
    scene.play(scene.kept["screen"].depart(0.6), FadeOut(scene.kept.pop("console")), run_time=0.6)
    scene.remove(scene.kept.pop("screen"))
    name = serif("Nathan Douglas", style.TEXT, 0.5, italic=False).move_to(DOWN * 3.05)
    scene.add(screen)
    scene.kept.update(screen=screen, name=name)
    voice = scene.voices["who"].seconds
    scene.say("who", Succession(screen.arrive(run_time=ARRIVE), FadeIn(name, run_time=0.6),
                                Wait(max(0.1, voice - ARRIVE - 0.6))))


def who_2(scene) -> None:
    role = serif("software engineer", style.ASH, 0.38).next_to(scene.kept["name"], DOWN, buff=0.12)
    scene.kept["role"] = role
    scene.say("who-2", Succession(Wait(0.3), FadeIn(role, run_time=0.6), Wait(0.5)))


def who_3(scene) -> None:
    """The chase itself: the line plays over it as it is."""
    scene.say("who-3")


def short_title(title: str) -> str:
    """A source's title without its notes: before any closing parenthesis, and its first sentence only."""
    return re.sub(r"\s*\([^)]*\)\s*$", "", title).split(". ")[0]


def corrections(scene) -> None:
    """The chase goes. A card asks for corrections; the titles of the sources drift behind it."""
    screen = scene.kept.pop("screen")
    scene.play(screen.depart(0.6), FadeOut(scene.kept.pop("name")), FadeOut(scene.kept.pop("role")), run_time=0.6)
    scene.remove(screen)
    titles = [short_title(c.title) for c in cited(scene.video)]
    rows = VGroup()
    for i, title in enumerate(titles):
        row = serif(title, style.ASH, 0.3).set_opacity(0.35)
        row.move_to([(-1) ** i * 1.2 + (i % 3) * 0.7, 3.3 - i * (6.6 / max(1, len(titles) - 1)), 0])
        rows.add(row)
    voice = scene.voices["corrections"].seconds
    drift = 1.4  # units each row travels, alternate rows opposite ways
    words = VGroup(serif("Corrections welcome", style.TEXT, 0.62, italic=False),
                   serif("see the link in the description", style.ASH, 0.38)).arrange(DOWN, buff=0.25)
    card = VGroup(RoundedRectangle(corner_radius=0.15, width=words.width + 1.2, height=words.height + 0.8,
                                   stroke_color=style.WARM[0], stroke_width=1.2, fill_color=style.BACKGROUND,
                                   fill_opacity=0.92), words)
    scene.kept["corrections"] = VGroup(rows, card)
    scene.say("corrections", AnimationGroup(
        FadeIn(rows, run_time=1.0),
        *[row.animate(rate_func=linear).shift(RIGHT * drift * (-1) ** i) for i, row in enumerate(rows)],
        Succession(Wait(0.6), FadeIn(card, run_time=0.8), Wait(max(0.1, voice - 1.4)))))


def grow(web: Web, end: float) -> UpdateFromAlphaFunc:
    """The web growing from what it shows to `end` dots, by the same factor each moment."""
    start = max(1.0, web.shown.get_value())
    return UpdateFromAlphaFunc(web, lambda m, a: m.shown.set_value(start * (end / start) ** a), rate_func=smooth)


def nobody_knows(scene) -> None:
    """The card goes. One dot, then a network growing from it to thousands."""
    scene.play(FadeOut(scene.kept.pop("corrections")), run_time=0.6)
    web = Web()
    web.shown.set_value(1)
    web.level.set_value(0)
    scene.add(web)
    scene.kept["web"] = web
    scene.say("nobody-knows", Succession(tween(web.level, 0, 1, 0.8), Wait(0.6), grow(web, web.count), Wait(0.4)))


def nobody_knows_2(scene) -> None:
    """The camera pushes into the tangle."""
    web = scene.kept["web"]
    web.focus = web.middle + np.array([0.9, 0.5])
    scene.say("nobody-knows-2", tween(web.zoom, 1.0, 2.6, 1.0))


def magnifier(radius: float = 0.75) -> VGroup:
    glass = Circle(radius=radius, stroke_color=style.TEXT, stroke_width=2.0)
    handle = Line(ORIGIN, (DOWN + RIGHT) * radius * 0.9, stroke_color=style.TEXT, stroke_width=5).shift(
        (DOWN + RIGHT) * radius * 0.71)
    return VGroup(glass, handle)


def nobody_knows_3(scene) -> None:
    """Back out: some dots lit, the network's answer beside them, and a glass looking over the lit dots."""
    web = scene.kept["web"]
    rng = np.random.default_rng(3)
    pattern = np.where(rng.random(web.count) < 0.12, rng.choice([-1.0, 1.0], web.count) * rng.uniform(0.5, 1, web.count),
                       0.0)
    heat = ValueTracker(0.0)
    web.add_updater(lambda m: m.lit.__setitem__(slice(None), pattern * heat.get_value()))
    answer = VGroup(serif("its answer", style.ASH, 0.36), serif("in the air", style.WARM[1], 0.5, italic=False)).arrange(
        DOWN, buff=0.15)
    box = RoundedRectangle(corner_radius=0.12, width=answer.width + 0.7, height=answer.height + 0.5,
                           stroke_color=style.WARM[1], stroke_width=1.2, fill_color=style.BACKGROUND, fill_opacity=0.9)
    shown = VGroup(box, answer).move_to(RIGHT * 5.0 + UP * 2.6)
    glass = magnifier().move_to(LEFT * 3.0 + DOWN * 0.8)
    scene.kept.update(answer=shown, glass=glass, heat=heat)
    scene.say("nobody-knows-3", Succession(
        AnimationGroup(tween(web.zoom, 2.6, 1.0, 1.2), tween(heat, 0, 1, 1.2)),
        FadeIn(shown, run_time=0.6),
        FadeIn(glass, run_time=0.5),
        glass.animate(run_time=2.4).move_to(RIGHT * 1.6 + UP * 0.6),
        Wait(0.4)))


def memory(scene) -> None:
    """The network goes; the game arrives with its memory beside it, every number changing as Mario runs."""
    web = scene.kept["web"]
    scene.play(tween(web.level, 1, 0, 0.6), FadeOut(scene.kept.pop("answer")), FadeOut(scene.kept.pop("glass")),
               run_time=0.6)
    web.lit[:] = 0
    web.clear_updaters()
    web.add_updater(lambda m: m.draw())
    scene.remove(web)
    u = config.pixel_height / 1080
    screen = Screen(Recording("smb1"), START, left_px=round(40 * u))
    screen.fade(0)
    screen.game.suspend_updating()  # held on its first frame until it arrives
    grid = MemoryGrid(screen, width=6.9)
    grid.move_to([config.frame_width / 2 - 0.35 - grid.image.width / 2, 1.9, 0])
    grid.set_level(0.0)
    count = VGroup(numeral("2,048", style.TEXT, 0.5), serif("numbers, each from 0 to 255", style.ASH, 0.4)).arrange(
        RIGHT, buff=0.15).next_to(grid.image, DOWN, buff=0.35)
    scene.add(screen, grid)
    scene.kept.update(screen=screen, grid=grid, count=count)
    scene.say("memory", Succession(screen.arrive(run_time=1.0), grid.arrive(run_time=1.2), Wait(1.0),
                                   FadeIn(count, run_time=0.8), Wait(1.5)))


def answer_keys(scene) -> None:
    """Three squares light with what they mean."""
    grid = scene.kept["grid"]
    scene.say("answer-keys", Succession(
        FadeOut(scene.kept.pop("count"), run_time=0.5),
        grid.light(POSITION, "position", 0, UP), Wait(0.3),
        grid.light(LIVES, "lives", 2, DOWN), Wait(0.3),
        grid.light(AIR, "jumping", 1, DOWN), Wait(1.0)))


def answer_keys_2(scene) -> None:
    """A neuron's answer beside the jumping square's, and a tick whenever they agree."""
    screen = scene.kept["screen"]
    rows, left, answers = (-1.75, -2.45), 1.2, 3.75  # the neuron's row, then the game's; where labels and answers start
    label = serif("a neuron says", style.ASH, 0.38)
    key_label = serif("the game says", style.ASH, 0.38)
    for words, y in zip((label, key_label), rows):
        words.move_to([left + words.width / 2, y, 0])
    said, kept_at = VGroup(), VGroup()
    mark = tick(np.array([6.5, sum(rows) / 2, 0]), 0.34)
    level = ValueTracker(0.0)

    def word(air: bool) -> str:
        return "in the air" if air else "on the ground"

    shown = {}

    def refresh(group: VGroup) -> None:
        neuron_air = CUT - screen.byte(DEPTH) > 0  # the neuron of chapter 1: weight -1, bias the setting
        game_air = screen.byte(AIR) != 0
        if shown.get("neuron") != neuron_air:
            shown["neuron"] = neuron_air
            new = serif(word(neuron_air), style.WARM[1] if neuron_air else style.COOL[1], 0.42)
            swap(said, new.move_to([answers + new.width / 2, rows[0], 0]))
        if shown.get("game") != game_air:
            shown["game"] = game_air
            new = serif(word(game_air), style.WARM[1] if game_air else style.COOL[1], 0.42)
            swap(kept_at, new.move_to([answers + new.width / 2, rows[1], 0]))
        a = level.get_value()
        said.set_opacity(a)
        kept_at.set_opacity(a)
        mark.set_stroke(opacity=a if neuron_air == game_air else 0)

    live = VGroup(said, kept_at, mark)
    live.add_updater(refresh)
    statics = VGroup(label, key_label)
    scene.add(live)
    scene.kept["neuron"] = VGroup(statics, live)
    scene.kept["neuron_level"] = level
    scene.say("answer-keys-2", Succession(
        AnimationGroup(FadeIn(label), FadeIn(key_label), tween(level, 0, 1, 0.8), run_time=0.8),
        Wait(2.0)))


def waves(web: Web, heat: ValueTracker, t: ValueTracker) -> None:
    """Broad patterns, warm and cool, sweeping across the whole web."""
    p = web.spots
    k = t.get_value()
    pattern = np.sin(p[:, 0] * 0.9 - k * 2.2) * np.cos(p[:, 1] * 0.7 + k * 1.3) + 0.6 * np.sin(
        (p[:, 0] + p[:, 1]) * 0.5 + k * 1.7)
    web.lit[:] = np.clip(pattern, -1, 1) * heat.get_value()


def doubters(scene) -> None:
    """The game goes; the network returns, and broad patterns sweep across it."""
    screen, grid = scene.kept.pop("screen"), scene.kept.pop("grid")
    level = scene.kept.pop("neuron_level")
    neuron = scene.kept.pop("neuron")
    scene.play(screen.depart(0.6), grid_away(grid), tween(level, 1, 0, 0.6), FadeOut(neuron[0]), run_time=0.6)
    scene.remove(screen, grid, neuron)
    web = scene.kept["web"]
    web.zoom.set_value(1.0)
    heat, t = ValueTracker(0.0), ValueTracker(0.0)
    web.add_updater(lambda m: waves(m, heat, t))
    scene.add(web)
    scene.kept.update(heat=heat, t=t)
    voice = scene.voices["doubters"].seconds
    scene.say("doubters", AnimationGroup(tween(web.level, 0, 1, 1.0), tween(heat, 0, 0.55, 2.0),
                                         tween(t, 0, voice * 0.6, voice, linear)))


def doubters_2(scene) -> None:
    """The patterns fade and the network shrinks back to a single dot."""
    web, heat = scene.kept["web"], scene.kept["heat"]
    t = scene.kept["t"]
    voice = scene.voices["doubters-2"].seconds
    scene.say("doubters-2", AnimationGroup(
        tween(heat, 0.55, 0.0, voice * 0.5),
        tween(t, t.get_value(), t.get_value() + voice * 0.3, voice * 0.5, linear),
        Succession(Wait(voice * 0.25), grow(web, 1.0).set_run_time(voice * 0.7))))


def start_with_one(scene) -> None:
    """One dot, alone; four cards fan out beside it, one for each step ahead."""
    web = scene.kept["web"]
    steps = [s.title for s in scene.video.sections if s.id in ("chapter-1", "chapter-2", "chapter-3", "chapter-5")]
    cards = VGroup()
    for i, title in enumerate(steps):
        words = serif(title, style.TEXT, 0.36)
        frame = RoundedRectangle(corner_radius=0.1, width=3.2, height=0.75, stroke_color=style.WARM[0], stroke_width=1.2)
        cards.add(VGroup(frame, words.move_to(frame)))
    cards.arrange(DOWN, buff=0.28).move_to(RIGHT * 3.4)
    dot = [*web.where(web.middle[None])[0], 0]
    for c in cards:  # each card fans out of the dot
        c.save_state()
        c.scale(0.1).move_to(dot).set_opacity(0)
    scene.add(cards)
    scene.kept["cards"] = cards
    scene.say("start-with-one", Succession(Wait(1.0), *[Restore(c, run_time=0.8) for c in cards], Wait(1.0)))


BEATS = {"about": about, "who": who, "who-2": who_2, "who-3": who_3, "corrections": corrections,
         "nobody-knows": nobody_knows, "nobody-knows-2": nobody_knows_2, "nobody-knows-3": nobody_knows_3,
         "memory": memory, "answer-keys": answer_keys, "answer-keys-2": answer_keys_2, "doubters": doubters,
         "doubters-2": doubters_2, "start-with-one": start_with_one}
