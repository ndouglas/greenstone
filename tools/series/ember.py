"""Ember's ground and light: the backdrop, the glow, the serif type, and live (self-redrawing) objects."""

from typing import Callable

import numpy as np
from manim import ITALIC, RIGHT, ImageMobject, Mobject, Text, ValueTracker, config

from series import style


def backdrop(focus: np.ndarray = RIGHT * 3) -> ImageMobject:
    """The full-frame ground at the frame's pixel size, near-black, warming toward the focus point, dithered."""
    w, h = config.pixel_width, config.pixel_height
    y, x = np.mgrid[0:h, 0:w]
    fx = (focus[0] / config.frame_width + 0.5) * w
    fy = (0.5 - focus[1] / config.frame_height) * h
    d = np.sqrt(((x - fx) / (0.9 * w)) ** 2 + ((y - fy) / (0.9 * h)) ** 2)
    t = np.clip(d / 0.75, 0, 1)[..., None] ** 0.8
    warm, dark = _rgb(style.EMBER), _rgb(style.BACKGROUND)
    rng = np.random.default_rng(0)
    noise = rng.random((h, w, 3)) + rng.random((h, w, 3)) - 1  # triangular dither: 8 bits alone would band
    rgb = np.clip((warm * (1 - t) + dark * t + noise).round(), 0, 255).astype(np.uint8)
    image = ImageMobject(np.dstack([rgb, np.full((h, w), 255, np.uint8)]))
    return image.set(width=config.frame_width, height=config.frame_height)


def glow(colour: str, radius: float) -> ImageMobject:
    """A soft radial light, one gradient image whose alpha is exactly 0 at its border."""
    n = 257  # odd, so a pixel sits at the centre
    y, x = np.mgrid[0:n, 0:n]
    r = np.sqrt((x - n // 2) ** 2 + (y - n // 2) ** 2) / (n // 2)
    alpha = np.clip(np.exp(-(r / 0.42) ** 2) - np.exp(-(1 / 0.42) ** 2), 0, 1)
    alpha[r >= 1] = 0
    rgb = np.broadcast_to(_rgb(colour), (n, n, 3))
    return ImageMobject(np.dstack([rgb, (alpha * 255).round()]).astype(np.uint8)).set(height=2 * radius)


def serif(text: str, colour: str, size: float = 0.42, italic: bool = True) -> Text:
    """Text in the series' serif; italic for quantities, upright for answers."""
    return Text(text, font=style.FONT, slant=ITALIC if italic else "NORMAL", color=colour).scale(size)


def live(mobject: Mobject, place: Callable, shown: ValueTracker) -> Mobject:
    """Redraw the mobject every frame through place, faded by the tracker."""
    def update(m: Mobject) -> None:
        place(m)
        m.set_opacity(shown.get_value())
    update(mobject)
    mobject.add_updater(update)
    return mobject


def fade_image(image: ImageMobject, level: ValueTracker) -> None:
    """Fade an image by scaling its own alpha, keeping its gradient (set_opacity would flatten it)."""
    alpha = image.pixel_array[:, :, 3].astype(float)

    def update(m: ImageMobject) -> None:
        m.pixel_array[:, :, 3] = np.round(alpha * level.get_value()).astype(np.uint8)
    update(image)
    image.add_updater(update)


def swap(group: Mobject, *children: Mobject) -> Mobject:
    """Replace a group's children with new ones, hiding the old first: during a play Manim keeps drawing
    every mobject the play began with, so a child merely dropped would linger on screen until it ends."""
    for old in group.submobjects:
        old.set_opacity(0)
    group.submobjects = []
    return group.add(*children)


def _rgb(colour: str) -> np.ndarray:
    return np.array([int(colour[i:i + 2], 16) for i in (1, 3, 5)], float)
