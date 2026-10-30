"""The memory grid: the game's 2,048 bytes of work RAM as a sea of small numbers, live with the picture. The
numbers are one image; a frame redraws only the cells whose byte changed, from a cache of ten-digit tiles
(2,048 live Text objects would take minutes a clip). A square lights up warm, enlarged and labelled, and
can be the start of a neuron's path."""

from typing import Callable

import numpy as np
from manim import UP, Animation, Group, ImageMobject, Line, Rectangle, UpdateFromAlphaFunc
from PIL import Image, ImageDraw, ImageFont

from series import ember, style
from series.ember import _rgb, glow_box, serif
from series.recording import Recording
from series.screen import Screen

BYTES = 2048
CELL_PX = (36, 20)  # a cell's size in the grid's own image
FONT_PX = 17
DIM = 0.8  # a number's brightness; zero is dimmer still
ZERO = 0.5  # zero's share of it
ZOOM = 3.6  # how much larger a lit square is than its cell
REACH = 0.25
GLOW = 0.5


def font() -> ImageFont.FreeTypeFont:
    """The series' Charter at the grid's size (ember.register_fonts has already refused a missing file)."""
    return ImageFont.truetype(str(ember.ROMAN), FONT_PX)


class Lit(Group):
    """One lit square (a Group, since its glow is an image): the cell enlarged and warm with its live number,
    and its label in italic at the end of a hairline."""

    def __init__(self, grid: "MemoryGrid", address: int, label: str, hue: int, side: np.ndarray):
        super().__init__()
        self.grid, self.address, self.colour = grid, address, style.WARM[hue]
        cell = grid.square(address)
        w, h = cell.width * ZOOM, cell.height * ZOOM
        self.box = Rectangle(width=w, height=h, stroke_color=self.colour, stroke_width=1.2,
                             fill_color=style.BACKGROUND, fill_opacity=0.92).move_to(cell)
        self.glow = glow_box(self.colour, min(w, h), REACH).stretch_to_fit_width(w + 2 * REACH)
        self.glow.stretch_to_fit_height(h + 2 * REACH).move_to(cell)
        self._alpha = self.glow.pixel_array[:, :, 3].astype(float)
        self.value = grid.value(address)
        self.number = serif(str(self.value), style.TEXT, 0.3, italic=False).move_to(cell)
        self.label = serif(label, self.colour, 0.4)
        self.label.next_to(self.box, side, buff=0.45)
        edge = self.box.get_edge_center(side)
        self.leader = Line(edge, self.label.get_edge_center(-side), stroke_color=self.colour, stroke_width=0.8)
        self.add(self.glow, self.box, self.number, self.leader, self.label)
        self.level = 0.0
        self.show(0.0)
        self.add_updater(lambda _: self._follow())

    def show(self, level: float) -> None:
        self.level = level
        self.glow.pixel_array[:, :, 3] = np.round(self._alpha * GLOW * level).astype(np.uint8)
        self.box.set_stroke(opacity=level).set_fill(opacity=0.92 * level)
        for part in (self.number, self.label, self.leader):
            part.set_opacity(level)

    def _follow(self) -> None:
        """Keep the number live: rebuilt only when the byte changes."""
        value = self.grid.value(self.address)
        if value != self.value:
            self.value = value
            self.number.become(serif(str(value), style.TEXT, 0.3, italic=False).move_to(self.box).set_opacity(self.level))


class MemoryGrid(Group):
    """Work RAM as `columns` bytes to a row, row after row, each a small ash number. It shows the RAM behind the
    frame on screen when given a Screen (a Recording alone shows its first frame)."""

    def __init__(self, source: Screen | Recording, columns: int = 64, width: float = 9.0, at: np.ndarray = (0, 0, 0)):
        super().__init__()
        if BYTES % columns:
            raise ValueError(f"{columns} columns don't divide {BYTES} bytes evenly")
        self.columns, self.rows = columns, BYTES // columns
        self.cell_px = CELL_PX
        self._ram: Callable[[], np.ndarray] = source.ram if isinstance(source, Screen) else (lambda: source.ram_at(0))
        self._font = font()
        self._tiles: dict[int, np.ndarray] = {}
        self.shown = np.full(BYTES, -1, dtype=int)
        w, h = columns * CELL_PX[0], self.rows * CELL_PX[1]
        pixels = np.zeros((h, w, 4), np.uint8)
        pixels[:, :, :3] = _rgb(style.ASH).astype(np.uint8)
        self.image = ImageMobject(pixels).set(width=width).move_to(at)
        self.add(self.image)
        self._squares: dict[int, Rectangle] = {}
        self.lit: dict[int, Lit] = {}
        self.refresh()
        self.add_updater(lambda _: self.refresh())

    def tile(self, value: int) -> np.ndarray:
        """A byte's number as alpha, set in the middle of a cell."""
        if value not in self._tiles:
            image = Image.new("L", self.cell_px, 0)
            ImageDraw.Draw(image).text((self.cell_px[0] / 2, self.cell_px[1] / 2), str(value), fill=255,
                                       font=self._font, anchor="mm")
            level = DIM * (ZERO if value == 0 else 1.0)
            self._tiles[value] = np.round(np.asarray(image) * level).astype(np.uint8)
        return self._tiles[value]

    def refresh(self) -> None:
        """Redraw the cells whose byte has changed since the last frame."""
        ram = np.asarray(self._ram(), dtype=int)
        cw, ch = self.cell_px
        for address in np.flatnonzero(ram != self.shown):
            r, c = divmod(int(address), self.columns)
            self.image.pixel_array[r * ch:(r + 1) * ch, c * cw:(c + 1) * cw, 3] = self.tile(int(ram[address]))
        self.shown = ram

    def value(self, address: int) -> int:
        """The number a cell shows now."""
        return int(self.shown[address])

    def square(self, address: int) -> Rectangle:
        """The cell itself, invisible, always where the grid's cell is (it moves with the grid)."""
        if not 0 <= address < BYTES:
            raise ValueError(f"address {address:#06x} is outside work RAM")
        if address not in self._squares:
            r, c = divmod(address, self.columns)
            cw, ch = self.image.width / self.columns, self.image.height / self.rows
            x = self.image.get_left()[0] + (c + 0.5) * cw
            y = self.image.get_top()[1] - (r + 0.5) * ch
            cell = Rectangle(width=cw, height=ch, stroke_width=0, fill_opacity=0).move_to([x, y, 0])
            self._squares[address] = cell
            self.add(cell)
        return self._squares[address]

    def point(self, address: int) -> Callable[[], np.ndarray]:
        """Where a square is now, as an Input's source."""
        return lambda: self.square(address).get_center()

    def light(self, address: int, label: str, hue: int = 0, side: np.ndarray = UP) -> Animation:
        """The square warms, enlarges, and shows its label above it (or on `side`)."""
        lit = Lit(self, address, label, hue, side)
        self.lit[address] = lit
        self.add(lit)
        return UpdateFromAlphaFunc(lit, lambda m, alpha: m.show(alpha))
