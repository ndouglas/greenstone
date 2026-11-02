"""What 002's beats share: the film data, the numbers, the map, the footage and its marks, and timing."""

import json
from pathlib import Path
from typing import Callable

import numpy as np
from manim import (DOWN, LEFT, RIGHT, UP, Animation, Arrow, FadeOut, Line, Mobject, UpdateFromAlphaFunc,
                   ValueTracker, VGroup, Wait, linear, smooth)

from series import style
from series.ember import in_line, numeral, serif, swap
from series.lab.film002 import load_film
from series.pacing import STILLNESS
from series.pointmap import PointMap
from series.recording import DATA, Recording
from series.screen import Arrival, Screen

FILM = load_film()
NUMBERS = json.loads((Path(__file__).parents[1] / "numbers.json").read_text())
BUTTONS = DATA / "smb1" / "1" / "buttons.u8"  # the footage is session 1's
POSITION, SPEED, SCROLL = 0x0755, 0x0057, 0x0775  # SMB1, as exp002 reads them
MARK = 80  # the mark, in pixels from the screen's left edge
BLOCK = 16
BOTH = (FILM["spot_moved"] > 0) & (FILM["spot_still"] > 0)  # points where both answers happened

_recording: Recording | None = None


def recording() -> Recording:
    """The smb1 footage take, loaded once (it is large)."""
    global _recording
    if _recording is None:
        _recording = Recording("smb1")
    return _recording


def held(scene, line_id: str) -> float:
    """How long a line holds the screen: its voice and the stillness after it."""
    line = next(l for l in scene.video.lines if l.id == line_id)
    return scene.voices[line_id].seconds + (line.pause if line.pause is not None else STILLNESS)


def voice(scene, line_id: str) -> float:
    return scene.voices[line_id].seconds


def fill(scene, line_id: str, *steps: Animation) -> list[Animation]:
    """The steps, then a wait for whatever is left of the line's voice, so the steps keep their own times."""
    from manim.animation.animation import prepare_animation
    steps = [prepare_animation(s) for s in steps]
    used = sum(s.run_time for s in steps)
    return [*steps, Wait(max(voice(scene, line_id) - used, 1e-3))]


class Do(Animation):
    """Call something once, as a moment in a Succession (resume footage, swap a label)."""

    def __init__(self, action: Callable[[], None], **kwargs):
        self.action = action
        super().__init__(Mobject(), run_time=1 / 60, **kwargs)

    def begin(self) -> None:
        super().begin()
        self.action()


def tween(tracker: ValueTracker, start: float, end: float, run_time: float, rate_func=smooth) -> UpdateFromAlphaFunc:
    return UpdateFromAlphaFunc(tracker, lambda m, a: m.set_value(start + (end - start) * a), run_time=run_time,
                               rate_func=rate_func)


def new_map(at=DOWN * 0.15, width: float = 7.6, height: float = 4.6, line=None, **kwargs) -> PointMap:
    return PointMap(FILM["spots"], FILM["spot_moved"], FILM["spot_still"], at=np.asarray(at, float), width=width,
                    height=height, line=line, **kwargs)


def legend(at: np.ndarray) -> VGroup:
    """Warm: the screen moved soon; cool: it didn't."""
    from manim import Dot
    rows = VGroup()
    for colour, text in ((style.WARM[0], "screen moved soon"), (style.COOL[1], "it didn't")):
        words = serif(text, colour, 0.3)
        rows.add(VGroup(Dot(radius=0.06, color=colour).next_to(words, LEFT, buff=0.14), words))
    return rows.arrange(DOWN, aligned_edge=LEFT, buff=0.14).move_to(at)


def line_of(v) -> np.ndarray:
    return np.asarray(v, float)


def blocks(screen: Screen) -> float:
    return screen.byte(POSITION) / BLOCK


def speed(screen: Screen) -> float:
    b = screen.byte(SPEED)
    return (b - 256 if b > 127 else b) / BLOCK


class MarkOnScreen(VGroup):
    """The mark over the game: a gold line down the screen at MARK pixels, its name above it."""

    def __init__(self, screen: Screen):
        super().__init__()
        x = screen.col_x(MARK)
        self.line = Line([x, screen.row_y(0), 0], [x, screen.row_y(240), 0], stroke_color=style.WARM[2], stroke_width=1.6)
        self.label = serif("the mark", style.WARM[2], 0.3).next_to(self.line, UP, buff=0.1)
        self.add(self.line, self.label)


class BlockRuler(VGroup):
    """Blocks of sixteen pixels counted along the top of the screen, from its left edge to the mark."""

    def __init__(self, screen: Screen, count: int = 5):
        super().__init__()
        y = screen.row_y(0) + 0.12
        for b in range(count + 1):
            x = screen.col_x(b * BLOCK)
            self.add(Line([x, y - 0.06, 0], [x, y + 0.06, 0], stroke_color=style.WARM[2], stroke_width=1.2))
            if b:
                self.add(numeral(str(b), style.WARM[2], 0.24).move_to([screen.col_x((b - 0.5) * BLOCK), y + 0.16, 0]))
        self.add(Line([screen.col_x(0), y, 0], [screen.col_x(count * BLOCK), y, 0], stroke_color=style.WARM[2],
                      stroke_width=1.0))


class Readout(VGroup):
    """A live reading: an italic name and its number, redrawn when what it shows changes."""

    def __init__(self, name: str, value: Callable[[], float], at: np.ndarray, places: int = 2,
                 colour_by_sign: bool = False):
        super().__init__()
        self.name_, self.value, self.at, self.places, self.by_sign = name, value, np.asarray(at, float), places, colour_by_sign
        self.shown = None
        self.level = ValueTracker(0.0)  # it redraws itself, so it fades by its own level, not FadeIn
        self.update_text()
        self.add_updater(lambda m: m.update_text())

    def update_text(self) -> None:
        v = self.value()
        text = style.number(v, self.places)
        if text == self.shown:
            self.set_opacity(self.level.get_value())
            return
        self.shown = text
        colour = style.temperature(v) if self.by_sign else style.WARM[0]
        row = in_line(serif(self.name_, style.ASH, 0.34), numeral(text, colour, 0.4), buff=0.2)
        swap(self, *row.submobjects)
        self.arrange(RIGHT, buff=0.2)
        in_line(*self.submobjects, buff=0.2)
        self.move_to(self.at, aligned_edge=LEFT)
        self.set_opacity(self.level.get_value())


class SpeedArrowRight(VGroup):
    """An arrow from Mario along his speed: right and warm when he moves right, left and cool when left."""

    def __init__(self, screen: Screen, scale: float = 0.6):
        super().__init__()
        self.screen, self.scale_ = screen, scale
        self.level = ValueTracker(0.0)
        self.refresh()
        self.add_updater(lambda m: m.refresh())

    def refresh(self) -> None:
        from series.overlays import mario_middle
        s = speed(self.screen)
        swap(self)
        if abs(s) < 1e-9:
            return
        start = mario_middle(self.screen) + UP * 0.45
        end = start + RIGHT * s * self.scale_
        colour = style.WARM[0] if s > 0 else style.COOL[0]
        self.add(Arrow(start, end, buff=0, stroke_width=2.4, color=colour, max_tip_length_to_length_ratio=0.35))
        self.set_opacity(self.level.get_value())


def show(m: Mobject, run_time: float = 0.5) -> Animation:
    """Fade something in: by its own level if it redraws itself (FadeIn's copy would go stale), else FadeIn."""
    from manim import AnimationGroup, FadeIn
    level = getattr(m, "level", None)
    if isinstance(level, ValueTracker):
        return tween(level, 0.0, 1.0, run_time)
    if isinstance(m, VGroup) and any(isinstance(getattr(c, "level", None), ValueTracker) for c in m.submobjects):
        return AnimationGroup(*[show(c, run_time) for c in m.submobjects], run_time=run_time)
    return FadeIn(m, run_time=run_time)


def screen_at(start: int, centre: np.ndarray, hold: bool = True) -> Screen:
    """The footage from `start`, faded out, held on its first frame until resumed."""
    s = Screen(recording(), start, centre=np.asarray(centre, float))
    s.fade(0)
    if hold:
        s.game.suspend_updating()
    return s


class HeldArrival(Arrival):
    """A screen's arrival that leaves its footage held on its first frame, for resume() to start."""

    def finish(self) -> None:
        Animation.finish(self)


def arrive_held(screen: Screen, run_time: float = 0.6) -> Animation:
    return HeldArrival(screen, run_time=run_time)


def resume(screen: Screen, speed: float = 1.0) -> Do:
    def go() -> None:
        screen.game.speed = speed
        screen.game.resume_updating()
    return Do(go)


def pause(screen: Screen) -> Do:
    return Do(screen.game.suspend_updating)


def away(scene, *keep: Mobject, run_time: float = 0.5) -> Animation:
    """Fade out everything on screen but the ground and what's kept (a screen departs as screens do); each is
    taken off as it goes. The first step of a beat that starts afresh."""
    from manim import AnimationGroup
    gone = [m for m in scene.mobjects if m is not scene.ground and all(m is not k for k in keep)]
    for m in gone:  # leaving, they stop redrawing themselves (FadeOut fades a copy made as it begins)
        if not isinstance(m, Screen):
            for part in m.get_family():
                part.clear_updaters()
    steps = [m.depart(run_time=run_time, remover=True) if isinstance(m, Screen) else FadeOut(m, run_time=run_time)
             for m in gone]
    return AnimationGroup(*steps, run_time=run_time) if steps else Wait(1e-3)
