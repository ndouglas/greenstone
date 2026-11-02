"""The sheet of borders as ground. Every spot on the sheet is one border on Mario's map: across, the way the
ruler faces (a full turn, so the right edge joins the left); up the sheet, how far it's slid. Raised by a
height (mistakes, or the total squared miss), it becomes terrain: flat ledges with sudden drops, or a smooth
bowl. Drawn by painting each cell's top and its walls from the back forward, so ledges stay flat and drops
stay sharp, into one image.

The camera looks across the sheet from its near edge (the lowest offsets), tilted down; `lift` scales the
heights, from a flat sheet (0) to full ground (1)."""

import math

import numpy as np
from manim import ImageMobject, config
from PIL import Image, ImageDraw

from series import style
from series.ember import rgb

WIDE, DEEP, TALL = 8.0, 4.4, 1.3  # the sheet's size in Manim units: across, into the picture, and its tallest
CELLS = (180, 120)  # cells across and deep that the sheet is drawn with
LOW, HIGH = rgb(style.WARM[2]), rgb("#3A2A22")  # the lowest ground glows; the highest is dark ash
FLAT = rgb("#5A4636")  # a flat sheet's colour, before it rises
# The light: from the left, in front and above; a face turned away from it keeps SHADOW of its colour.
SUN = np.array([-0.45, -0.55, 0.70]) / np.linalg.norm([-0.45, -0.55, 0.70])
SHADOW = 0.35


def coarse(height: np.ndarray, cells: tuple[int, int]) -> np.ndarray:
    """Sample a (angles, offsets) map at cell centres: a cell takes the height at its middle, so a flat ledge
    stays one height."""
    a, o = height.shape
    ia = ((np.arange(cells[0]) + 0.5) * a / cells[0]).astype(int)
    io = ((np.arange(cells[1]) + 0.5) * o / cells[1]).astype(int)
    return height[np.ix_(ia, io)]


class Camera:
    """A perspective camera looking along +y (into the sheet), above its near edge, tilted down by `pitch`."""

    def __init__(self, pitch: float = 0.95, distance: float = 9.0, focal: float = 8.6, rise: float = 0.0):
        self.pitch, self.distance, self.focal, self.rise = pitch, distance, focal, rise

    def project(self, x: np.ndarray, y: np.ndarray, z: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
        """World (x across, y into the picture, z up; the sheet's centre at the origin) to the picture, in
        Manim units about the image's centre."""
        c, s = math.cos(self.pitch), math.sin(self.pitch)
        # the camera sits back along -y and up, looking at the sheet's centre
        cy, cz = -self.distance * c, self.distance * s
        dy, dz = y - cy, z - cz
        depth = dy * c - dz * s  # along the view
        up = dy * s + dz * c
        return self.focal * x / depth, self.focal * up / depth + self.rise


class Sheet:
    """A height map over the sheet, `height` (angles, offsets) in [0, 1] after `low`/`high` scaling, drawn by
    `render`. `colour_by` (same shape) sets each cell's light; default the height itself."""

    def __init__(self, height: np.ndarray, low: float | None = None, high: float | None = None,
                 cells: tuple[int, int] = CELLS, window: tuple[slice, slice] | None = None):
        """`window` crops the map (rows of angles, columns of offsets) before it's drawn, to look closer."""
        height = np.asarray(height, float)
        if window is not None:
            height = height[window]
        h = coarse(height, cells)
        low = h.min() if low is None else low
        high = h.max() if high is None else high
        self.h = np.clip((h - low) / (high - low), 0, 1)
        self.cells = cells

    def corner(self, i: float, j: float, lift: float, h: float) -> tuple[float, float, float]:
        """World coordinates of the point i cells across and j cells in, at height h (0 to 1)."""
        na, no = self.cells
        return (i / na - 0.5) * WIDE, (j / no - 0.5) * DEEP, h * lift * TALL

    def at(self, across: float, up: float, lift: float, camera: Camera, height: float | None = None) -> np.ndarray:
        """Where a spot on the sheet (across and up as shares, 0 to 1) shows in the picture, in Manim units
        about the image's centre; on its ground unless a height is given."""
        na, no = self.cells
        if height is None:
            height = self.h[min(int(across * na), na - 1), min(int(up * no), no - 1)]
        x, y, z = (across - 0.5) * WIDE, (up - 0.5) * DEEP, height * lift * TALL
        px, py = camera.project(np.array(x), np.array(y), np.array(z))
        return np.array([float(px), float(py), 0.0])

    def render(self, camera: Camera, lift: float, size: tuple[float, float], flat: float = 0.0,
               glow: np.ndarray | None = None, smooth: bool = False) -> np.ndarray:
        """The picture, `size` Manim units, as RGBA. `smooth` draws the ground as one continuous surface (for
        the bowl); otherwise each cell is a flat ledge with walls (for the staircase). `flat` (0 to 1) blends
        every cell toward one plain colour, for the sheet before it means anything; `glow` (cells) adds warm
        light to chosen cells."""
        px = config.pixel_width / config.frame_width
        W, H = int(round(size[0] * px)), int(round(size[1] * px))
        image = Image.new("RGBA", (W, H), (0, 0, 0, 0))
        draw = ImageDraw.Draw(image)
        na, no = self.cells
        h = self.h
        xs = (np.arange(na + 1) / na - 0.5) * WIDE
        ys = (np.arange(no + 1) / no - 0.5) * DEEP

        def to_px(x, y, z):
            u, v = camera.project(x, y, z)
            return float(u * px + W / 2), float(H / 2 - v * px)

        def paint(corners, colour, normal) -> None:
            shade = SHADOW + (1 - SHADOW) * max(float(np.dot(normal, SUN)), 0.0)
            draw.polygon([to_px(*c) for c in corners], fill=tuple(int(c) for c in np.clip(colour * shade, 0, 255)) + (255,))

        glowing = (1 - h)[..., None] ** 3  # only the low ground is lit; the rest falls dark quickly
        light = glowing * LOW + (1 - glowing) * HIGH
        light = light * (1 - flat) + FLAT * flat
        if glow is not None:
            light = np.where(glow[..., None], light * 0.4 + rgb(style.WARM[0]) * 0.6, light)
        z = h * lift * TALL
        up, toward = np.array([0.0, 0.0, 1.0]), np.array([0.0, -1.0, 0.0])
        # back to front: far rows first; within a row, from the edges in toward the middle (the camera's x)
        order = sorted(range(na), key=lambda i: -abs((i + 0.5) / na - 0.5))
        if smooth:
            padded = np.pad(z, 1, mode="edge")
            v = (padded[:-1, :-1] + padded[1:, :-1] + padded[:-1, 1:] + padded[1:, 1:]) / 4  # corner heights
            for j in range(no - 1, -1, -1):
                for i in order:
                    c = [(xs[i], ys[j], v[i, j]), (xs[i + 1], ys[j], v[i + 1, j]),
                         (xs[i + 1], ys[j + 1], v[i + 1, j + 1]), (xs[i], ys[j + 1], v[i, j + 1])]
                    n = np.cross(np.subtract(c[2], c[0]), np.subtract(c[3], c[1]))
                    paint(c, light[i, j], n / np.linalg.norm(n))
                    if j == 0:  # the near edge's skirt, down to zero
                        paint([(xs[i], ys[0], 0), (xs[i + 1], ys[0], 0), c[1], c[0]], light[i, j], toward)
            return np.asarray(image)
        for j in range(no - 1, -1, -1):
            y0, y1 = ys[j], ys[j + 1]
            for i in order:
                x0, x1, zz = xs[i], xs[i + 1], z[i, j]
                colour = light[i, j]
                # the wall facing us, down to the ground in front of it (or to zero at the near edge)
                front = z[i, j - 1] if j > 0 else 0.0
                if zz > front + 1e-9:
                    paint([(x0, y0, front), (x1, y0, front), (x1, y0, zz), (x0, y0, zz)], colour, toward)
                # the side wall facing the middle
                if (i + 0.5) / na < 0.5:
                    side, nb, facing = x1, (z[i + 1, j] if i + 1 < na else 0.0), np.array([1.0, 0, 0])
                else:
                    side, nb, facing = x0, (z[i - 1, j] if i > 0 else 0.0), np.array([-1.0, 0, 0])
                if zz > nb + 1e-9:
                    paint([(side, y0, nb), (side, y1, nb), (side, y1, zz), (side, y0, zz)], colour, facing)
                paint([(x0, y0, zz), (x1, y0, zz), (x1, y1, zz), (x0, y1, zz)], colour, up)
        return np.asarray(image)


def sheet_image(sheet: Sheet, camera: Camera, lift: float, size: tuple[float, float], **kwargs) -> ImageMobject:
    return ImageMobject(sheet.render(camera, lift, size, **kwargs)).set(width=size[0])
