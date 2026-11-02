"""The series' opening, in place of a video's introduction title: a thread of light enters a glass cartridge,
catches three neurons inside, and leaves split into the spectrum (warm above, cool below); the series' title
settles beneath, then, after a pause, the episode's; then all of it fades to black. The theme plays under it."""

import numpy as np
from manim import (DOWN, LEFT, RIGHT, UP, Circle, Create, FadeIn, Line, Polygon, Rectangle, RoundedRectangle, Scene, ValueTracker,
                   VGroup, config, linear, rate_functions)

from series import style
from series.ember import fade_image, glow, serif

SERIES = "Through the Glass Cartridge"
SUBTITLE = "Journeys in Interpretability"

CART = np.array([-3.4, 0.9, 0])  # the cartridge's centre
W, H = 2.5, 2.8
BEAM_Y = CART[1] + 0.15
NODES = ((0.25, 0.35), (0.5, -0.25), (0.75, 0.3))  # where the neurons sit: across the cartridge, off the beam
NODE_HUES = (style.WARM[0], style.WARM[1], style.WARM[2])
RAYS = (*style.WARM[:4], *style.COOL)  # warm above the beam, cool below
SPREAD = 14  # degrees either side

# Seconds, in order.
DARK, DRAW, BEAM_IN, STEP, CATCH, FAN, TITLE, SUB, PAUSE, EPISODE, HOLD, TO_BLACK, BLACK = (
    0.3, 1.0, 0.55, 0.22, 0.12, 0.7, 0.7, 0.5, 0.8, 1.0, 1.4, 1.4, 0.3)
OPENING_SECONDS = (DARK + DRAW + BEAM_IN + (len(NODES) + 1) * STEP + len(NODES) * CATCH + FAN + TITLE + SUB + PAUSE
                   + EPISODE + HOLD + TO_BLACK + BLACK)


def episode_label(title: str) -> str:
    """'001: One neuron draws a line' as the opening sets it: '001 · One neuron draws a line'."""
    number, sep, name = title.partition(": ")
    return f"{number} · {name}" if sep else title


def cartridge() -> VGroup:
    """A Game Pak's front, in glass, as in its photographs: the outline narrowing above its bottom edge (the end
    that goes in), the column of grip ridges down its left with a small tab at their foot, the label's recess
    with its round mark, and the arrow pointing in. Measured as shares of a cartridge photographed without its
    label (Evan-Amos, Wikimedia Commons, public domain)."""
    def at(fx: float, fy: float) -> np.ndarray:
        """A point fx across the front from its left, fy down from its top."""
        return CART + np.array([(fx - 0.5) * W, (0.5 - fy) * H, 0.0])

    def hair(width: float, opacity: float) -> dict:
        return {"stroke_color": style.TEXT, "stroke_width": width, "stroke_opacity": opacity}
    outline = Polygon(*[at(x, y) for x, y in ((0.035, 0.0), (0.965, 0.0), (0.965, 0.79), (0.91, 0.79), (0.91, 1.0),
                                              (0.09, 1.0), (0.09, 0.79), (0.035, 0.79))],
                      **hair(1.3, 0.75), fill_color=style.TEXT, fill_opacity=0.035)
    column = Polygon(at(0.18, 0.02), at(0.38, 0.02), at(0.38, 0.87), at(0.18, 0.87), **hair(0.9, 0.4), fill_opacity=0)
    ridges = VGroup(*[Line(at(0.18, y), at(0.38, y), **hair(0.8, 0.35 if y > 0.15 else 0.5))
                      for y in np.r_[np.linspace(0.045, 0.135, 4), np.arange(0.17, 0.86, 0.036)]])
    tab = RoundedRectangle(corner_radius=0.03, width=0.19 * W, height=0.09 * H, **hair(0.9, 0.4), fill_opacity=0).move_to(at(0.275, 0.925))
    label = RoundedRectangle(corner_radius=0.03, width=0.44 * W, height=0.64 * H, **hair(1.0, 0.45), fill_opacity=0).move_to(at(0.62, 0.35))
    mark = Circle(radius=0.012 * W, **hair(0.8, 0.4)).move_to(at(0.49, 0.57))
    arrow = Polygon(at(0.46, 0.73), at(0.54, 0.73), at(0.5, 0.79), **hair(1.0, 0.45), fill_opacity=0)
    glint = Line(at(0.44, 0.08), at(0.62, 0.3), stroke_color=style.TEXT, stroke_width=2.5, stroke_opacity=0.12)
    return VGroup(outline, column, ridges, tab, label, mark, arrow, glint)


def play_opening(scene: Scene, title: str, theme: str | None = None) -> None:
    """Play the opening on `scene` for the episode called `title`, with the theme (a WAV path) under it, and
    leave the screen as the scene found it."""
    if theme is not None:
        scene.add_sound(theme)
    found = list(scene.mobjects)
    left, right = CART[0] - W / 2, CART[0] + W / 2
    nodes = [np.array([left + W * f, BEAM_Y + dy, 0]) for f, dy in NODES]
    leave = np.array([right, BEAM_Y, 0])
    path = [np.array([-config.frame_width / 2 - 0.3, BEAM_Y, 0]), np.array([left, BEAM_Y, 0]), *nodes, leave]

    scene.wait(DARK)
    cart = cartridge()
    scene.play(Create(cart, lag_ratio=0.05), run_time=DRAW)
    for k, (a, b) in enumerate(zip(path, path[1:])):
        segment = Line(a, b, stroke_color=style.TEXT, stroke_width=1.4)
        scene.play(Create(segment), run_time=BEAM_IN if k == 0 else STEP, rate_func=linear)
        if 1 <= k <= len(nodes):  # the light reaches a neuron, which catches it
            level = ValueTracker(0)
            light = glow(NODE_HUES[k - 1], 0.55).move_to(nodes[k - 1])
            fade_image(light, level)
            dot = RoundedRectangle(corner_radius=0.06, width=0.12, height=0.12, stroke_width=0,
                                   fill_color=style.WARM[2], fill_opacity=1).move_to(nodes[k - 1])
            scene.add(light, dot)
            scene.play(level.animate.set_value(1), FadeIn(dot), run_time=CATCH)
    rays = VGroup(*[Line(leave, leave + 9 * np.array([np.cos(a), np.sin(a), 0]), stroke_color=hue, stroke_width=2.2,
                         stroke_opacity=0.85)
                    for hue, a in zip(RAYS, np.radians(np.linspace(SPREAD, -SPREAD, len(RAYS))))])
    scene.play(*[Create(r) for r in rays], run_time=FAN, rate_func=rate_functions.ease_out_cubic)

    name = serif(SERIES, style.TEXT, 1.05, italic=False).move_to(DOWN * 1.75)
    sub = serif(SUBTITLE, style.ASH, 0.6).next_to(name, DOWN, buff=0.25)
    episode = serif(episode_label(title), style.WARM[0], 0.5).next_to(sub, DOWN, buff=0.55)
    scene.play(FadeIn(name, shift=UP * 0.1), run_time=TITLE)
    scene.play(FadeIn(sub), run_time=SUB)
    scene.wait(PAUSE)
    scene.play(FadeIn(episode), run_time=EPISODE)
    scene.wait(HOLD)
    # To black: a cover over everything, the glows included (an image's opacity can't simply be set).
    cover = Rectangle(width=config.frame_width + 1, height=config.frame_height + 1, stroke_width=0,
                      fill_color="#000000", fill_opacity=0)
    scene.add(cover)
    scene.play(cover.animate.set_fill(opacity=1), run_time=TO_BLACK, rate_func=linear)
    scene.wait(BLACK)
    # Everything it added, not only what it made: a play adds each animated part on its own (the rays one by one).
    scene.remove(*[m for m in scene.mobjects if not any(m is f for f in found)])
