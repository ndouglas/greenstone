"""The sheet of borders, for chapters 4 and 5: exp002's two maps of every border (mistakes, a staircase; the
total squared miss at each border's best size, a bowl), and how a spot on the sheet is a ruler on Mario's map."""

import numpy as np
from manim import ImageMobject, UpdateFromAlphaFunc

from series.lab.exp002 import MAPS
from series.sheet import Camera, Sheet

with np.load(MAPS) as _maps:
    ANGLES, OFFSETS = _maps["angles"], _maps["offsets"]
    MISTAKES, SQUARED, MIDDLE = _maps["mistakes"], _maps["squared"], _maps["middle"]

SIZE = (7.4, 4.6)  # the sheet's picture, in Manim units
TURN = 2 * np.pi


def border(angle: float, offset: float) -> np.ndarray:
    """The (w, b) of the ruler facing `angle`, slid `offset` from the middle of Mario's map."""
    w = np.array([np.cos(angle), np.sin(angle)])
    return np.r_[w, offset - w @ MIDDLE]


def spot(v) -> tuple[float, float]:
    """Where a border (w, b) sits on the sheet, as shares across (its angle) and up (its offset)."""
    w, b = np.asarray(v[:2], float), float(v[2])
    size = np.linalg.norm(w)
    angle = np.arctan2(w[1], w[0])
    angle = ANGLES[0] + (angle - ANGLES[0]) % TURN
    offset = (b + w @ MIDDLE) / size
    return (angle - ANGLES[0]) / TURN, (offset - OFFSETS[0]) / (OFFSETS[-1] - OFFSETS[0])


def at_share(across: float, up: float) -> tuple[float, float]:
    """The angle and offset at shares across and up the sheet."""
    return ANGLES[0] + (across % 1.0) * TURN, OFFSETS[0] + up * (OFFSETS[-1] - OFFSETS[0])


def lowest(height: np.ndarray) -> tuple[float, float]:
    i, j = np.unravel_index(np.argmin(height), height.shape)
    return (i + 0.5) / len(ANGLES), (j + 0.5) / len(OFFSETS)


class SheetPicture(ImageMobject):
    """A sheet drawn as an image, redrawn when its lift, flatness or glow change."""

    def __init__(self, height: np.ndarray, smooth: bool, camera: Camera | None = None, size=SIZE, lift: float = 1.0,
                 flat: float = 0.0, **sheet_kwargs):
        self.sheet = Sheet(height, **sheet_kwargs)
        self.camera = camera or Camera(pitch=0.9)
        self.size_, self.smooth, self.lift, self.flat, self.glow = size, smooth, lift, flat, None
        super().__init__(self.picture())
        self.set(width=size[0])

    def picture(self) -> np.ndarray:
        return self.sheet.render(self.camera, self.lift, self.size_, flat=self.flat, glow=self.glow, smooth=self.smooth)

    def redraw(self) -> None:
        centre, width = self.get_center(), self.width
        self.pixel_array = self.picture()
        self.set(width=width).move_to(centre)

    def point(self, across: float, up: float, height: float | None = None) -> np.ndarray:
        """Where a spot on the sheet shows on the page."""
        return self.sheet.at(across, up, self.lift, self.camera, height) * (self.width / self.size_[0]) + self.get_center()

    def rise(self, start: float, end: float, flat_from: float | None = None, flat_to: float | None = None):
        def update(m, a: float) -> None:
            m.lift = start + (end - start) * a
            if flat_from is not None:
                m.flat = flat_from + (flat_to - flat_from) * a
            m.redraw()
        return UpdateFromAlphaFunc(self, update)
