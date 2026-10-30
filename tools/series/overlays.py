"""Marks drawn on the game screen: the feet and setting lines, the gap between them, and the speed arrow.
Each one follows the RAM behind the frame on screen, so it is right from the frame it appears."""

from typing import Callable

import numpy as np
from manim import DOWN, LEFT, RIGHT, UP, Arrow, Line, Rectangle, VGroup

from series import style
from series.ember import numeral, serif, swap
from series.screen import Screen

HAIRLINE = 1.2
BAND_LIGHT = 0.32  # fill opacity of a band at full light
LABEL_INSET = 0.3  # how far in from the screen's left edge a line's label starts
ARROW_STROKE = 3
ARROW_ROWS_PER_SPEED = 4  # NES pixel rows of arrow per unit of speed
SCREEN_X, DEPTH, SMALL = 0x03AD, 0x00CE, 0x0754  # SMB1: Mario's column on screen, his depth, and 1 if small
# From (screen x, depth) to his middle, in NES pixels, measured on the sprite: his 15 columns start at the
# screen x; small Mario's rows are depth + 18 to + 32, big Mario's depth + 1 to + 32.
MIDDLE_X, MIDDLE_Y = 7.5, {True: 25, False: 17}


def signed(byte: int) -> int:
    """A byte read as two's complement."""
    return byte - 256 if byte >= 128 else byte


class _Line(VGroup):
    """A hairline across the screen with a serif label above its left end."""

    def __init__(self, screen: Screen, colour: str):
        self.screen, self.colour = screen, colour
        self.line = Line(LEFT, RIGHT, stroke_width=HAIRLINE, color=colour)
        self.label, self.shown = VGroup(), None
        super().__init__(self.line, self.label)

    def y(self) -> float:
        raise NotImplementedError

    def parts(self) -> tuple[str, str | None]:
        """The label (set in italic) and the number after it, if any (set in roman)."""
        raise NotImplementedError

    def refresh(self, *_) -> None:
        y, left, right = self.y(), self.screen.col_x(0), self.screen.col_x(256)
        self.line.put_start_and_end_on([left, y, 0], [right, y, 0])
        parts = self.parts()
        if parts != self.shown:  # building text is slow, so only when it changes
            name, value = parts
            texts = [serif(name, self.colour)] + ([numeral(value, self.colour)] if value else [])
            swap(self.label, *texts).arrange(RIGHT, buff=0.12)
            self.shown = parts
        self.label.next_to([left + LABEL_INSET, y, 0], UP + RIGHT, buff=0.06)


class FeetLine(_Line):
    def __init__(self, screen: Screen, byte: int, offset: int, label: str):
        self.byte, self.offset, self.caption = byte, offset, label
        super().__init__(screen, style.WARM[0])
        self.refresh()
        self.add_updater(self.refresh)

    def value(self) -> int:
        """The depth byte, as of the frame on screen."""
        return int(self.screen.ram()[self.byte])

    def y(self) -> float:
        return self.screen.row_y(self.value() + self.offset)

    def parts(self) -> tuple[str, str | None]:
        return self.caption, style.number(self.value())


class SettingLine(_Line):
    def __init__(self, screen: Screen, row: int, label: str, offset: int):
        self.row, self.offset, self.caption = row, offset, label
        super().__init__(screen, style.WARM[2])
        self.refresh()

    def y(self) -> float:
        return self.screen.row_y(self.row + self.offset)

    def parts(self) -> tuple[str, str | None]:
        return self.caption, None


def feet_line(screen: Screen, byte: int = 0x00CE, offset: int = 33, label: str = "depth") -> FeetLine:
    """A hairline across the screen at the bottom edge of the sprite's lowest row (the depth byte plus
    `offset` rows from the top), with its live reading. `.y()` is its height, `.value()` the byte."""
    return FeetLine(screen, byte, offset, label)


def setting_line(screen: Screen, row: int, label: str, offset: int = 33) -> SettingLine:
    """A fixed gold hairline at depth `row`, drawn where a feet line at that depth would be."""
    return SettingLine(screen, row, label, offset)


class GapBand(Rectangle):
    def __init__(self, screen: Screen, upper: _Line, lower: _Line, weight: float, scale: float):
        self.screen, self.upper, self.lower, self.weight, self.light_scale = screen, upper, lower, weight, scale
        super().__init__(width=1, height=1, stroke_width=0, fill_opacity=0)
        self.refresh()
        self.add_updater(self.refresh)

    def refresh(self, *_) -> None:
        y0, y1 = self.upper.y(), self.lower.y()
        left, right = self.screen.col_x(0), self.screen.col_x(256)
        rows = (y0 - y1) / self.screen.dot  # positive while the upper line is above the lower
        contribution = -self.weight * rows  # weight × (upper's depth − lower's), as the neuron adds it
        light = style.light(contribution, self.light_scale) if rows else 0.0
        colour = style.WARM[2] if contribution > 0 else style.COOL[0]
        self.become(Rectangle(width=right - left, height=max(abs(y0 - y1), 1e-3), stroke_width=0, fill_color=colour,
                              fill_opacity=BAND_LIGHT * light).move_to([(left + right) / 2, (y0 + y1) / 2, 0]))


def gap_band(screen: Screen, upper: _Line, lower: _Line, weight: float = -1,
             scale: float = style.LIGHT_SCALE) -> GapBand:
    """The band between two lines, lit by what it adds to the total, weight × (upper's depth − lower's): warm
    when that is positive (a negative weight with `upper` above `lower`, or a positive one with it below),
    cool when negative, absent when the lines meet. A bigger weight burns brighter at the same height."""
    return GapBand(screen, upper, lower, weight, scale)


def mario_middle(screen: Screen) -> np.ndarray:
    """The middle of Mario's sprite in the picture on screen (SMB1)."""
    small = bool(screen.byte(SMALL))
    return np.array([screen.col_x(screen.byte(SCREEN_X) + MIDDLE_X),
                     screen.row_y(screen.byte(DEPTH) + MIDDLE_Y[small]), 0])


class SpeedArrow(VGroup):
    def __init__(self, screen: Screen, speed_byte: int, at: Callable[[Screen], np.ndarray]):
        self.screen, self.speed_byte, self.at = screen, speed_byte, at
        super().__init__()
        self.refresh()
        self.add_updater(self.refresh)

    def refresh(self, *_) -> None:
        speed = signed(self.screen.byte(self.speed_byte))
        swap(self)
        if not speed:
            return
        middle = self.at(self.screen)
        end = middle + DOWN * speed * ARROW_ROWS_PER_SPEED * self.screen.dot
        colour = style.WARM[0] if speed > 0 else style.COOL[0]
        self.add(Arrow(middle, end, buff=0, stroke_width=ARROW_STROKE, color=colour, max_tip_length_to_length_ratio=0.5))


def speed_arrow(screen: Screen, speed_byte: int, at: Callable[[Screen], np.ndarray] = mario_middle) -> SpeedArrow:
    """An arrow from `at` (Mario's middle by default), as long as his speed (a signed byte, positive
    falling): warm and down while he falls, cool and up while he rises, absent at zero."""
    return SpeedArrow(screen, speed_byte, at)
