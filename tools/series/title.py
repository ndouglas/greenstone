"""A section's title screen: its number and title over a dimmed wall of
its own pictures, drifting slowly one way (the chapters design, option A)."""

import textwrap

from manim import BLACK, DOWN, LEFT, Group, Mobject, Rectangle, RoundedRectangle, Text, VGroup, config

from series import style
from series.script import Section

# Tile widths, as fractions of the frame width, row by row: uneven, and
# each row wider than the frame so the wall can drift.
ROWS = ((0.22, 0.30, 0.18, 0.26, 0.20), (0.16, 0.34, 0.24, 0.22, 0.18), (0.26, 0.20, 0.28, 0.22, 0.24))
GAP = 0.08
DIM = 0.82


def board_tiles(section: Section) -> list[Mobject]:
    """One tile per beat with a board, standing in for its picture until it exists."""
    tiles = []
    for line in section.lines:
        if line.board:
            words = textwrap.shorten(line.board, 40, placeholder="…")
            card = RoundedRectangle(corner_radius=0.08, width=2.0, height=1.2, stroke_width=0,
                                    fill_color=style.WARM[0], fill_opacity=0.18)
            label = Text(textwrap.fill(words, 18), font=style.FONT, color=style.TEXT).scale(0.25)
            tiles.append(VGroup(card, label.move_to(card)))
    return tiles


def wall(tiles: list[Mobject]) -> Group:
    """The tiles laid in three rows of uneven widths, filling more than the frame."""
    width, height = config.frame_width, config.frame_height
    row_h = (height - 2 * GAP) / 3
    pool = tiles or [Rectangle(width=1, height=1, stroke_width=0, fill_color=style.WARM[0], fill_opacity=0.12)]
    rows, k = Group(), 0
    for fractions in ROWS:
        row = Group()
        for f in fractions:
            # A slot of the row's height and the tile's width; the picture is
            # scaled evenly to fit inside it, never stretched.
            slot = Rectangle(width=f * width * 1.25, height=row_h, stroke_width=0,
                             fill_color=style.BACKGROUND, fill_opacity=1)
            picture = pool[k % len(pool)].copy()
            k += 1
            picture.scale(min(slot.width / picture.width, slot.height / picture.height) * 0.92)
            row.add(Group(slot, picture.move_to(slot)))
        rows.add(row.arrange(buff=GAP))
    rows.arrange(DOWN, buff=GAP)
    return rows.align_to([-width / 2, 0, 0], LEFT)


def title_card(section: Section, tiles: list[Mobject]) -> Group:
    """The wall (first), its dimming, then the number and title."""
    backdrop = wall(tiles)
    shade = Rectangle(width=config.frame_width * 2.5, height=config.frame_height * 1.2, stroke_width=0,
                      fill_color=BLACK, fill_opacity=DIM)
    words = VGroup()
    if section.number is not None:
        words.add(Text(f"Chapter {section.number}", font=style.FONT, color=style.WARM[0]).scale(0.45))
    words.add(Text(section.title, font=style.FONT, color=style.TEXT).scale(1.0))
    words.arrange(DOWN, buff=0.3)
    # A soft dark field behind the words, so no tile competes with them.
    backing = RoundedRectangle(corner_radius=0.3, width=words.width + 1.6, height=words.height + 1.0,
                               stroke_width=0, fill_color=BLACK, fill_opacity=0.6).move_to(words)
    return Group(backdrop, shade, backing, words)
