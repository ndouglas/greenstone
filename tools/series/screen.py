"""The game screen: the footage at exact pixels, an old TV's corners, scanlines and glow, and the RAM behind
whatever frame is showing."""

import numpy as np
from manim import ORIGIN, Animation, Group, ImageMobject, Rectangle, ValueTracker, config
from manim.constants import RESAMPLING_ALGORITHMS
from manim.utils.images import change_to_rgba_array
from PIL import Image
from scipy.ndimage import gaussian_filter

from series import style
from series.ember import _rgb, fade_image
from series.footage import NES_FPS, GameFootage
from series.recording import Recording

NES_W, NES_H = 256, 240
GLOW = 0.3  # the light at the screen's edge, as alpha
SCANLINE = 0.12
EDGE_DARK = 0.14
NEAREST = RESAMPLING_ALGORITHMS["nearest"]


def scanlines_fit(scale: float) -> bool:
    """Scanlines need a whole number of screen rows per NES line, and at least 3 of them, or they alias."""
    return float(scale).is_integer() and scale >= 3


def corner_radius(scale: float) -> int:
    return int(26 * scale // 3)


def rounded_mask(w: int, h: int, r: int) -> np.ndarray:
    """1 inside a w × h rectangle with corners of radius r, 0 outside, antialiased by a pixel."""
    y, x = np.mgrid[0:h, 0:w]
    cx, cy = np.clip(x + 0.5, r, w - r), np.clip(y + 0.5, r, h - r)
    return np.clip(r - np.sqrt((x + 0.5 - cx) ** 2 + (y + 0.5 - cy) ** 2) + 0.5, 0, 1)


def screen_overlay(w: int, h: int, scale: float, behind: np.ndarray, scanlines: bool | None = None) -> np.ndarray:
    """What makes the footage read as an old TV, pixel for pixel over it: faint scanlines (the last screen
    row of every NES line) where they fit, edges darkening softly toward the corners, and rounded corners
    painted `behind` (the colour of what's behind the screen there: one colour, or an h × w image)."""
    scanlines = scanlines_fit(scale) if scanlines is None else scanlines
    alpha = np.zeros((h, w))
    if scanlines:
        alpha[int(scale) - 1::int(scale), :] = SCANLINE
    y, x = np.mgrid[0:h, 0:w]
    u, v = (x + 0.5) / w * 2 - 1, (y + 0.5) / h * 2 - 1
    edge = np.clip((np.maximum(np.abs(u), np.abs(v)) - 0.55) / 0.45, 0, 1) ** 2 * EDGE_DARK
    alpha = 1 - (1 - alpha) * (1 - edge)
    corner = 1 - rounded_mask(w, h, corner_radius(scale))
    rgb = np.empty((h, w, 3))
    rgb[:] = _rgb(style.BACKGROUND)
    rgb[corner > 0] = np.broadcast_to(behind, (h, w, 3))[corner > 0]
    alpha = 1 - (1 - alpha) * (1 - corner)
    return np.dstack([rgb, alpha * 255]).round().astype(np.uint8)


def screen_glow(colour: np.ndarray, w: int, h: int, scale: float) -> tuple[np.ndarray, int]:
    """The picture's light spilling into the dark around it: the rounded screen itself, blurred, in the
    clip's colour, so the glow follows the corners. Returns the image and its margin in pixels beyond the
    screen on each side; the image's own border is alpha 0."""
    sigma = 14 * scale
    margin = round(4 * sigma)
    mask = np.zeros((h + 2 * margin, w + 2 * margin))
    mask[margin:margin + h, margin:margin + w] = rounded_mask(w, h, corner_radius(scale))
    # Twice the blur, so the light at the screen's edge (where the blur is about half) is GLOW.
    alpha = np.clip(gaussian_filter(mask, sigma) * 2, 0, 1) * GLOW
    alpha[[0, -1], :] = 0
    alpha[:, [0, -1]] = 0
    return np.dstack([np.broadcast_to(colour, alpha.shape + (3,)), alpha * 255]).round().astype(np.uint8), margin


def screen_bloom(colour: np.ndarray, w: int, h: int, scale: float) -> np.ndarray:
    """The picture's light blooming over its own rim, continuous with the glow outside, so no dark border
    forms where the screen meets the dark. It stops at the rounded edge: the cut-away corners show the glow."""
    r = corner_radius(scale)
    y, x = np.mgrid[0:h, 0:w] + 0.5
    off = np.sqrt((x - np.clip(x, r, w - r)) ** 2 + (y - np.clip(y, r, h - r)) ** 2)
    straight = np.minimum.reduce([x, w - x, y, h - y])
    inset = np.clip(np.where(off > 0, r - off, straight), 0, None)
    alpha = np.exp(-(inset / (6 * scale)) ** 2) * GLOW * 0.6 * rounded_mask(w, h, r)
    return np.dstack([np.broadcast_to(colour, (h, w, 3)), alpha * 255]).round().astype(np.uint8)


def clip_colour(recording: Recording, start: int) -> np.ndarray:
    """The average colour of the ten seconds from `start` (sampled once a second)."""
    frames = range(start, min(start + 600, len(recording)), 60)
    return np.mean([recording.frames[f].reshape(-1, 3).mean(0) for f in frames], axis=0)


class Footage(GameFootage):
    """A recording's frames, played as GameFootage plays a take's; averaged down to `size` (w, h) when it is
    smaller than the NES's picture."""

    def __init__(self, recording: Recording, start: int, size: tuple[int, int] | None = None):
        if not 0 <= start < len(recording):
            raise ValueError(f"{recording.game} has {len(recording)} frames; start {start} is outside it")
        self.frames, self.take, self.size = recording.frames, recording.game, size
        self.warned, self.start_frame, self.elapsed = False, start, 0.0
        ImageMobject.__init__(self, self.picture(start))
        self.set_resampling_algorithm(RESAMPLING_ALGORITHMS["bilinear"] if size else NEAREST)
        self.add_updater(Footage.advance)

    def __deepcopy__(self, memo: dict) -> "Footage":
        memo[id(self.frames)] = self.frames  # copies share the frames: a recording is hundreds of megabytes
        return super().__deepcopy__(memo)

    def picture(self, index: int) -> np.ndarray:
        frame = np.asarray(self.frames[index])
        if self.size:
            frame = np.asarray(Image.fromarray(frame).resize(self.size, Image.Resampling.BOX))
        return change_to_rgba_array(frame)

    def index(self) -> int:
        return min(self.start_frame + round(self.elapsed * NES_FPS), len(self.frames) - 1)

    def advance(self, dt: float) -> None:
        GameFootage.advance(self, dt)  # keeps its clock and its warning past the end
        if self.size:
            self.pixel_array = self.picture(self.index())


class Screen(Group):
    """A recording on screen from frame `start`: `scale` screen pixels per NES pixel (3 at 1080p; a whole
    number, or any value below 1), its left edge at `left_px` or centred on `centre`, always on whole pixels.
    The scale is kept as `pixel_scale`."""

    def __init__(self, recording: Recording, start: int, scale: float | None = None,
                 scanlines: bool | None = None, left_px: int | None = None, centre: np.ndarray | None = None):
        self.recording = recording
        scale = max(1, config.pixel_height * 2 // 720) if scale is None else scale
        if scale >= 1 and not float(scale).is_integer():
            raise ValueError(f"a screen can't be drawn at scale {scale}: it must be a whole number of at least 1, "
                             f"or any value below 1 (anything else stretches NES pixels unevenly)")
        self.pixel_scale = int(scale) if scale >= 1 else scale
        if scanlines and not scanlines_fit(self.pixel_scale):
            raise ValueError(f"scanlines don't fit at {self.pixel_scale}×: they need a whole number of at least 3 "
                             f"screen rows per NES line")
        self.scanlines = scanlines_fit(self.pixel_scale) if scanlines is None else scanlines
        w, h = round(NES_W * self.pixel_scale), round(NES_H * self.pixel_scale)
        px = config.frame_height / config.pixel_height
        centre = ORIGIN if centre is None else np.asarray(centre)
        if left_px is None:
            left_px = config.pixel_width / 2 + centre[0] / px - w / 2
        left, top = round(left_px), round(config.pixel_height / 2 - centre[1] / px - h / 2)
        middle = np.array([(left + w / 2) * px - config.frame_width / 2,
                           config.frame_height / 2 - (top + h / 2) * px, 0])

        def placed(image: ImageMobject, height: int) -> ImageMobject:
            image.set_resampling_algorithm(NEAREST)
            return image.set(height=height * px).move_to(middle)

        self.game = Footage(recording, start, (w, h) if self.pixel_scale < 1 else None)
        self.game.set(height=h * px).move_to(middle)
        self.dot = self.game.height / NES_H
        colour = clip_colour(recording, start)
        glow_image, margin = screen_glow(colour, w, h, self.pixel_scale)
        # The cut-away corners are painted as the glow lights the dark there, so they vanish into it.
        lit = glow_image[margin:margin + h, margin:margin + w, 3:] / 255
        behind = _rgb(style.BACKGROUND) * (1 - lit) + colour * lit
        self.glow = placed(ImageMobject(glow_image), h + 2 * margin)
        self.shown = ValueTracker(1.0)
        fade_image(self.glow, self.shown)
        self.overlay = placed(ImageMobject(screen_overlay(w, h, self.pixel_scale, behind, self.scanlines)), h)
        self.bloom = placed(ImageMobject(screen_bloom(colour, w, h, self.pixel_scale)), h)
        self.cover = Rectangle(width=(w + 2) * px, height=(h + 2) * px, stroke_width=0,
                               fill_color=style.BACKGROUND, fill_opacity=1).move_to(middle)
        super().__init__(self.glow, self.game, self.overlay, self.bloom)

    def frame(self) -> int:
        """The frame on screen now."""
        return self.game.index()

    def ram(self) -> np.ndarray:
        """The RAM the picture on screen shows."""
        return self.recording.ram_at(self.frame())

    def byte(self, address: int) -> int:
        """One byte as the picture on screen shows it, at its own measured lag."""
        return self.recording.byte_at(self.frame(), address)

    def row_y(self, row: float) -> float:
        """The top edge of NES pixel row `row`."""
        return self.game.get_top()[1] - row * self.dot

    def col_x(self, col: float) -> float:
        """The left edge of NES pixel column `col`."""
        return self.game.get_left()[0] + col * self.game.width / NES_W

    def arrive(self, run_time: float = 0.8) -> Animation:
        """The game arriving from black: a cover fades off it and the glow fades up, while the footage holds
        its frame until the arrival ends."""
        return Arrival(self, run_time=run_time)

    def depart(self, run_time: float = 0.8) -> Animation:
        """The game leaving into black: a cover fades onto it and the glow fades down; then the footage
        stops."""
        return Departure(self, run_time=run_time)

    def __deepcopy__(self, memo: dict) -> "Screen":
        memo[id(self.recording)] = self.recording  # copies share the recording: it is hundreds of megabytes
        return super().__deepcopy__(memo)


class Arrival(Animation):
    """Screen.arrive's animation. It works on the screen itself, never a copy (Manim's FadeIn animated a
    copy of the footage that played on, and then the real one restarted)."""

    def __init__(self, screen: Screen, **kwargs):
        super().__init__(screen, suspend_mobject_updating=False, introducer=True, **kwargs)

    def create_starting_mobject(self) -> Screen:
        return self.mobject

    def get_all_mobjects_to_update(self) -> list:
        return []

    def begin(self) -> None:
        self.mobject.game.suspend_updating()
        self.mobject.add(self.mobject.cover)
        super().begin()

    def interpolate_mobject(self, alpha: float) -> None:
        a = self.rate_func(alpha)
        self.mobject.cover.set_fill(opacity=1 - a)
        self.mobject.shown.set_value(a)

    def finish(self) -> None:
        super().finish()
        self.mobject.remove(self.mobject.cover)
        self.mobject.game.resume_updating()


class Departure(Animation):
    """Screen.depart's animation: Arrival in reverse, on the screen itself. The cover stays on afterwards."""

    def __init__(self, screen: Screen, **kwargs):
        super().__init__(screen, suspend_mobject_updating=False, **kwargs)

    def create_starting_mobject(self) -> Screen:
        return self.mobject

    def get_all_mobjects_to_update(self) -> list:
        return []

    def begin(self) -> None:
        self.mobject.add(self.mobject.cover)
        super().begin()

    def interpolate_mobject(self, alpha: float) -> None:
        a = self.rate_func(alpha)
        self.mobject.cover.set_fill(opacity=a)
        self.mobject.shown.set_value(1 - a)

    def finish(self) -> None:
        super().finish()
        self.mobject.game.suspend_updating()
