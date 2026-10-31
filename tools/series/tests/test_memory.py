import tempfile
import unittest
from pathlib import Path

import numpy as np
from manim import ImageMobject, tempconfig

from series import ember, memory, style
from series.ember import numeral
from series.memory import MemoryGrid
from series.screen import Screen
from series.tests.test_screen import FULL_HD, synthetic


def play(animation) -> None:
    animation.begin()
    animation.interpolate(1.0)
    animation.finish()


class MemoryGridTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.recording = synthetic(Path(self.tmp.name))
        self.recording.ram[:, 0xCE] = (np.arange(len(self.recording)) * 3) % 256
        self.recording.ram[:, 0x75A] = 2

    def tearDown(self):
        self.tmp.cleanup()

    def test_the_grid_draws_its_numbers_in_the_bundled_charter(self):
        self.assertEqual(Path(memory.font().path), ember.ROMAN)

    def test_the_grid_fades_in_by_its_own_drawing(self):
        with tempconfig(FULL_HD):
            grid = MemoryGrid(Screen(self.recording, 10), width=6.0)
            full = int(grid.image.pixel_array[:, :, 3].max())
            self.assertGreater(full, 0)
            grid.set_level(0.0)
            self.assertEqual(int(grid.image.pixel_array[:, :, 3].max()), 0)
            grid.update(1 / 60)  # its own redraw keeps the level
            self.assertEqual(int(grid.image.pixel_array[:, :, 3].max()), 0)
            animation = grid.arrive()
            animation.begin()
            animation.interpolate(0.5)
            self.assertAlmostEqual(int(grid.image.pixel_array[:, :, 3].max()), round(full * 0.5), delta=2)
            animation.interpolate(1.0)
            animation.finish()
            self.assertEqual(int(grid.image.pixel_array[:, :, 3].max()), full)

    def test_the_grid_holds_every_byte(self):
        with tempconfig(FULL_HD):
            screen = Screen(self.recording, 40, scale=1)
            grid = MemoryGrid(screen)
            self.assertEqual(len(grid.shown), 2048)
            self.assertEqual(grid.value(0xCE), int(screen.ram()[0xCE]))
            self.assertEqual(grid.value(0xCE), (39 * 3) % 256)  # the lagged RAM of the frame on screen
            self.assertEqual(grid.value(0x75A), 2)
            screen.game.elapsed = 10 / 60  # ten frames on
            grid.update(0)
            self.assertEqual(grid.value(0xCE), int(screen.ram()[0xCE]))
            self.assertEqual(grid.value(0xCE), (49 * 3) % 256)

    def test_only_changed_cells_are_redrawn(self):
        with tempconfig(FULL_HD):
            screen = Screen(self.recording, 40, scale=1)
            grid = MemoryGrid(screen)
            before = grid.image.pixel_array.copy()
            screen.game.elapsed = 1 / 60
            grid.update(0)
            changed = (grid.image.pixel_array != before).any(axis=2)
            rows, cols = np.nonzero(changed)
            # byte 0 and byte 0xCE changed; nothing else was touched
            for r, c in zip(rows, cols):
                self.assertIn((r // grid.cell_px[1]) * grid.columns + c // grid.cell_px[0], (0, 0xCE))

    def test_it_is_a_grid_of_columns(self):
        with tempconfig(FULL_HD):
            grid = MemoryGrid(self.recording, columns=32)
            self.assertEqual(grid.rows, 64)
            a, b = grid.square(0).get_center(), grid.square(33).get_center()
            step = grid.square(1).get_center()[0] - a[0]
            self.assertGreater(step, 0)  # columns run left to right, evenly
            self.assertAlmostEqual(grid.square(2).get_center()[0] - grid.square(1).get_center()[0], step)
            self.assertAlmostEqual(a[0], grid.square(32).get_center()[0])  # 32 wraps back to column 0
            self.assertAlmostEqual(b[0], a[0] + step)  # 33 is column 1 of row 1
            self.assertLess(b[1], a[1])

    def test_a_lit_square_is_a_source(self):
        with tempconfig(FULL_HD):
            grid = MemoryGrid(Screen(self.recording, 40, scale=1))
            source = grid.point(0x1D)
            self.assertTrue(np.allclose(source(), grid.square(0x1D).get_center()))
            grid.shift([1.0, -0.5, 0])  # it follows the grid
            self.assertTrue(np.allclose(source(), grid.square(0x1D).get_center()))
            self.assertAlmostEqual(source()[0], grid.image.get_left()[0] + (0x1D + 0.5) * grid.image.width / 64, places=5)
            lit = grid.light(0x1D, "jumping")
            play(lit)
            box = grid.lit[0x1D].box  # lit, the path starts at the enlarged box's edge, not under it
            self.assertTrue(np.allclose(source(), box.get_right()))
            self.assertGreater(source()[0], grid.square(0x1D).get_right()[0])
            self.assertIn(style.WARM[0], [c.to_hex().upper() for c in grid.lit[0x1D].box.get_stroke_colors()])
            self.assertEqual(grid.lit[0x1D].label.text, "jumping")

    def test_a_lit_square_draws_the_byte_as_it_changes(self):
        with tempconfig(FULL_HD):
            screen = Screen(self.recording, 40)
            grid = MemoryGrid(screen)
            play(grid.light(0xCE, "depth"))
            lit = grid.lit[0xCE]
            for _ in range(3):
                screen.update(1 / 60)
                grid.update(1 / 60)
                drawn = numeral(str(screen.byte(0xCE)), style.TEXT, 0.3).move_to(lit.box)
                points = lit.number.get_all_points()
                self.assertTrue(points.shape == drawn.get_all_points().shape and np.allclose(points, drawn.get_all_points()),
                                f"the square shows a stale number, not {screen.byte(0xCE)}")
