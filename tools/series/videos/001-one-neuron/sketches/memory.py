"""001's memory grid: the 2,048 bytes of work RAM as a sea of numbers beside the game that moves them, live for
six seconds; then three squares light up and say what they are."""

from manim import DOWN, UP, config

from series.ember import backdrop
from series.exact import ExactScene
from series.memory import MemoryGrid
from series.recording import Recording
from series.screen import Screen

START = 1247
POSITION, LIVES, JUMPING = 0x0086, 0x075A, 0x001D  # docs/series/games/001.md; lives is SMB1's life counter


class Sketch(ExactScene):
    def construct(self):
        self.add(backdrop())
        u = config.pixel_height / 1080
        screen = Screen(Recording("smb1"), START, scale=2, left_px=round(40 * u))
        grid = MemoryGrid(screen, width=9.0)
        grid.move_to([config.frame_width / 2 - 0.4 - grid.image.width / 2, 0.1, 0])
        self.play(screen.arrive(), run_time=1)
        self.add(grid)
        self.wait(6)
        self.play(grid.light(POSITION, "position", 0), run_time=1)
        self.play(grid.light(JUMPING, "jumping", 1), run_time=1)
        self.play(grid.light(LIVES, "lives", 2, DOWN), run_time=1)
        self.wait(3)
