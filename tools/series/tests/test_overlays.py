import tempfile
import unittest
from pathlib import Path

from manim import config, tempconfig

from series import style
from series.overlays import feet_line, gap_band, setting_line, speed_arrow
from series.screen import Screen
from series.tests.test_screen import FULL_HD, synthetic

DEPTH, SPEED, X = 0x00CE, 0x009F, 0x0086


class TestOverlays(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.recording = synthetic(Path(self.tmp.name))

    def tearDown(self):
        self.tmp.cleanup()

    def screen(self, depth: int = 100, speed: int = 0) -> Screen:
        self.recording.ram[:, DEPTH] = depth
        self.recording.ram[:, SPEED] = speed % 256
        self.recording.ram[:, X] = 80
        return Screen(self.recording, 10)

    def test_the_feet_line_sits_under_the_sprite(self):
        with tempconfig(FULL_HD):
            screen = self.screen(depth=100)
            feet = feet_line(screen)
            self.assertAlmostEqual(feet.y(), screen.row_y(133))
            self.assertEqual(feet.value(), 100)
            self.recording.ram[9, DEPTH] = 7  # frame 10 shows frame 9's RAM, one back
            self.assertEqual(feet.value(), 7)
            self.assertAlmostEqual(feet.y(), screen.row_y(40))
            self.assertAlmostEqual(setting_line(screen, 176, "setting").y(), screen.row_y(209))

    def test_the_gap_is_the_total(self):
        with tempconfig(FULL_HD):
            for depth, warm, rows in [(150, True, 26), (176, None, 0), (200, False, 24)]:
                screen = self.screen(depth=depth)
                band = gap_band(screen, feet_line(screen), setting_line(screen, 176, "setting"))
                if warm is None:
                    self.assertEqual(band.get_fill_opacity(), 0)
                    continue
                self.assertAlmostEqual(band.height, rows * screen.dot)
                self.assertEqual(band.get_fill_color().to_hex().upper(),
                                 (style.WARM[2] if warm else style.COOL[0]).upper())
                self.assertGreater(band.get_fill_opacity(), 0)

    def test_a_bigger_weight_burns_brighter(self):
        with tempconfig(FULL_HD):
            screen = self.screen(depth=150)
            feet, setting = feet_line(screen), setting_line(screen, 176, "setting")
            one, three = gap_band(screen, feet, setting, weight=-1), gap_band(screen, feet, setting, weight=-3)
            self.assertGreater(three.get_fill_opacity(), one.get_fill_opacity())
            self.assertAlmostEqual(three.height, one.height)

    def test_the_speed_arrow_shows_direction(self):
        with tempconfig(FULL_HD):
            falling = speed_arrow(self.screen(speed=3), X, DEPTH, SPEED)
            self.assertEqual(len(falling.submobjects), 1)
            arrow = falling.submobjects[0]
            self.assertLess(arrow.get_end()[1], arrow.get_start()[1])
            self.assertEqual(arrow.get_color().to_hex().upper(), style.WARM[0].upper())
            rising = speed_arrow(self.screen(speed=-3), X, DEPTH, SPEED).submobjects[0]
            self.assertGreater(rising.get_end()[1], rising.get_start()[1])
            self.assertEqual(rising.get_color().to_hex().upper(), style.COOL[0].upper())
            self.assertEqual(len(speed_arrow(self.screen(speed=0), X, DEPTH, SPEED).submobjects), 0)
