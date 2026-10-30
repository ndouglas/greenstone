"""Ember's ground and light: the backdrop, the glow, the serif type, and live (self-redrawing) objects."""

from pathlib import Path
from typing import Callable

import manimpango
import numpy as np
from manim import ITALIC, RIGHT, ImageMobject, Mobject, Text, ValueTracker, config

from series import style

# The type travels with the repo (XCharter, the free Bitstream Charter extended), so a render on any machine
# sets the same letters; a missing file is an error, never a quiet substitute.
ROMAN = Path(__file__).parent / "fonts" / "XCharter-Roman.otf"
ITALIC_FILE = Path(__file__).parent / "fonts" / "XCharter-Italic.otf"


def register_fonts() -> None:
    for path in (ROMAN, ITALIC_FILE):
        if not path.exists():
            raise FileNotFoundError(f"the series' type is missing: {path}")
        if not manimpango.register_font(str(path)):
            raise RuntimeError(f"Pango wouldn't register {path}")


register_fonts()


def backdrop(focus: np.ndarray = RIGHT * 3) -> ImageMobject:
    """The full-frame ground at the frame's pixel size, near-black, warming toward the focus point, dithered.
    Costs about 150 ms at 1080p, once per scene."""
    w, h = config.pixel_width, config.pixel_height
    y, x = np.mgrid[0:h, 0:w]
    fx = (focus[0] / config.frame_width + 0.5) * w
    fy = (0.5 - focus[1] / config.frame_height) * h
    d = np.sqrt(((x - fx) / (0.9 * w)) ** 2 + ((y - fy) / (0.9 * h)) ** 2)
    t = np.clip(d / 0.75, 0, 1)[..., None] ** 0.8
    warm, dark = rgb(style.EMBER), rgb(style.BACKGROUND)
    rng = np.random.default_rng(0)
    noise = rng.random((h, w, 3)) + rng.random((h, w, 3)) - 1  # triangular dither: 8 bits alone would band
    pixels = np.clip((warm * (1 - t) + dark * t + noise).round(), 0, 255).astype(np.uint8)
    image = ImageMobject(np.dstack([pixels, np.full((h, w), 255, np.uint8)]))
    return image.set(width=config.frame_width, height=config.frame_height)


def glow(colour: str, radius: float) -> ImageMobject:
    """A soft radial light, one gradient image whose alpha is exactly 0 at its border."""
    n = 257  # odd, so a pixel sits at the centre
    y, x = np.mgrid[0:n, 0:n]
    r = np.sqrt((x - n // 2) ** 2 + (y - n // 2) ** 2) / (n // 2)
    alpha = np.clip(np.exp(-(r / 0.42) ** 2) - np.exp(-(1 / 0.42) ** 2), 0, 1)
    alpha[r >= 1] = 0
    colours = np.broadcast_to(rgb(colour), (n, n, 3))
    return ImageMobject(np.dstack([colours, (alpha * 255).round()]).astype(np.uint8)).set(height=2 * radius)


def glow_box(colour: str, side: float, pad: float) -> ImageMobject:
    """A soft light round a square of the given side: one gradient image, falling off smoothly with the distance
    from the square and exactly 0 at its own border, pad beyond the square."""
    n = 257
    y, x = np.mgrid[0:n, 0:n]
    half = side / (side + 2 * pad) * (n // 2)  # the square's half-side in pixels
    d = np.hypot(np.maximum(np.abs(x - n // 2) - half, 0), np.maximum(np.abs(y - n // 2) - half, 0)) / (n // 2 - half)
    alpha = np.clip(np.exp(-(d / 0.45) ** 2) - np.exp(-(1 / 0.45) ** 2), 0, 1)
    alpha[(np.abs(x - n // 2) == n // 2) | (np.abs(y - n // 2) == n // 2)] = 0
    colours = np.broadcast_to(rgb(colour), (n, n, 3))
    return ImageMobject(np.dstack([colours, (alpha * 255).round()]).astype(np.uint8)).set(width=side + 2 * pad)


def serif(text: str, colour: str, size: float = 0.42, italic: bool = True) -> Text:
    """Text in the series' serif; italic for quantities, upright for answers."""
    return Text(text, font=style.FONT, slant=ITALIC if italic else "NORMAL", color=colour).scale(size)


def numeral(text: str, colour: str, size: float = 0.42) -> Text:
    """A number in the series' serif, upright: the answer, not the quantity."""
    return serif(text, colour, size, italic=False)


def live(mobject: Mobject, place: Callable, shown: ValueTracker) -> Mobject:
    """Redraw the mobject every frame through place, faded by the tracker. For Text and other vector
    mobjects only (an image fades through fade_image). Building a Text is slow, so place should rebuild
    one only when what it shows has changed."""
    def update(m: Mobject) -> None:
        place(m)
        m.set_opacity(shown.get_value())
    update(mobject)
    mobject.add_updater(update)
    return mobject


def fade_image(image: ImageMobject, level: ValueTracker) -> None:
    """Fade an image by scaling its own alpha, keeping its gradient (set_opacity would flatten it)."""
    image.fade_alpha = image.pixel_array[:, :, 3].astype(float)
    image.fade_level = level  # on the image, so a copy fades by its own copy of the tracker
    _faded(image)
    image.add_updater(_faded)


def _faded(m: ImageMobject) -> None:
    m.pixel_array[:, :, 3] = np.round(m.fade_alpha * m.fade_level.get_value()).astype(np.uint8)


def swap(group: Mobject, *children: Mobject) -> Mobject:
    """Replace a group's children with new ones, hiding the old first: during a play Manim keeps drawing
    every mobject the play began with, so a child merely dropped would linger on screen until it ends."""
    for old in group.submobjects:
        if not any(old is new for new in children):  # one kept is not hidden
            old.set_opacity(0)
    group.submobjects = []
    return group.add(*children)


def rgb(colour: str) -> np.ndarray:
    return np.array([int(colour[i:i + 2], 16) for i in (1, 3, 5)], float)
