import tempfile
import unittest

import numpy as np
from pathlib import Path
from unittest import mock

from manim import config, tempconfig

from series import overlays, style
from series.overlays import feet_line, gap_band, mario_middle, setting_line, speed_arrow
from series.footage import NES_FPS
from series.recording import DATA, MARIO_RED, MARIO_SHOE, Recording
from series.screen import Screen
from series.tests.test_screen import FULL_HD, synthetic

DEPTH, SPEED, X, SMALL = 0x00CE, 0x009F, 0x03AD, 0x0754


class TestOverlays(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.recording = synthetic(Path(self.tmp.name))

    def tearDown(self):
        self.tmp.cleanup()

    def screen(self, depth: int = 100, speed: int = 0, small: bool = True) -> Screen:
        self.recording.ram[:, DEPTH] = depth
        self.recording.ram[:, SPEED] = speed % 256
        self.recording.ram[:, X] = 50
        self.recording.ram[10, X] = 80  # the screen x is read on the frame shown, not one back
        self.recording.ram[:, SMALL] = int(small)
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
                top, bottom = sorted([screen.row_y(depth + 33), screen.row_y(209)], reverse=True)
                self.assertAlmostEqual(band.get_top()[1], top)  # it fills exactly the gap between the lines
                self.assertAlmostEqual(band.get_bottom()[1], bottom)
                self.assertEqual(band.get_fill_color().to_hex().upper(),
                                 (style.WARM[2] if warm else style.COOL[0]).upper())
                self.assertGreater(band.get_fill_opacity(), 0)

    def test_the_band_takes_the_sign_of_its_contribution(self):
        with tempconfig(FULL_HD):
            for depth, weight, warm in [(150, -1, True), (150, 1, False), (200, -1, False), (200, 1, True)]:
                screen = self.screen(depth=depth)
                band = gap_band(screen, feet_line(screen), setting_line(screen, 176, "setting"), weight=weight)
                self.assertEqual(band.get_fill_color().to_hex().upper(),
                                 (style.WARM[2] if warm else style.COOL[0]).upper(), (depth, weight))

    def test_a_bigger_weight_burns_brighter(self):
        with tempconfig(FULL_HD):
            screen = self.screen(depth=150)  # 26 rows: weights this small stay well below saturation
            feet, setting = feet_line(screen), setting_line(screen, 176, "setting")
            one = gap_band(screen, feet, setting, weight=-0.1, scale=300)
            three = gap_band(screen, feet, setting, weight=-0.3, scale=300)
            self.assertGreater(three.get_fill_opacity(), one.get_fill_opacity())
            self.assertAlmostEqual(three.height, one.height)

    def test_the_speed_arrow_shows_direction(self):
        with tempconfig(FULL_HD):
            falling = speed_arrow(self.screen(speed=3), SPEED)
            self.assertEqual(len(falling.submobjects), 1)
            arrow = falling.submobjects[0]
            self.assertLess(arrow.get_end()[1], arrow.get_start()[1])
            self.assertEqual(arrow.get_color().to_hex().upper(), style.WARM[0].upper())
            rising = speed_arrow(self.screen(speed=-3), SPEED).submobjects[0]
            self.assertGreater(rising.get_end()[1], rising.get_start()[1])
            self.assertEqual(rising.get_color().to_hex().upper(), style.COOL[0].upper())
            self.assertEqual(len(speed_arrow(self.screen(speed=0), SPEED).submobjects), 0)

    def test_the_label_is_italic_and_the_number_roman(self):
        with tempconfig(FULL_HD):
            name, number = feet_line(self.screen(depth=100)).label.submobjects
            self.assertEqual(name.original_text, "depth")  # not Mobject.name, the class's name
            self.assertEqual(setting_line(self.screen(), 176, "setting").label.submobjects[0].original_text,
                             "setting")
            self.assertEqual(name.slant, "ITALIC")
            self.assertEqual(number.slant, "NORMAL")
            self.assertEqual(number.text, "100")

    def test_text_is_only_rebuilt_when_the_reading_changes(self):
        with tempconfig(FULL_HD):
            screen = self.screen(depth=100)
            feet = feet_line(screen)
            with (mock.patch.object(overlays, "serif", wraps=overlays.serif) as serif,
                  mock.patch.object(overlays, "numeral", wraps=overlays.numeral) as numeral):
                feet.refresh()
                feet.refresh()
                self.assertEqual(serif.call_count + numeral.call_count, 0)
                self.recording.ram[9, DEPTH] = 7
                feet.refresh()
                self.assertEqual((serif.call_count, numeral.call_count), (1, 1))
                self.assertEqual(feet.label.submobjects[1].text, "7")

    def test_replaced_marks_are_hidden(self):
        # During a play Manim keeps drawing the mobjects it began with, so a replaced one must be hidden.
        with tempconfig(FULL_HD):
            screen = self.screen(depth=100, speed=3)
            feet, arrow = feet_line(screen), speed_arrow(screen, SPEED)
            old_label, old_arrow = list(feet.label.submobjects), arrow.submobjects[0]
            self.recording.ram[9, DEPTH] = 7
            feet.refresh()
            arrow.refresh()
            for old in old_label + [old_arrow]:
                self.assertTrue(all(o == 0 for o in old.get_fill_opacities()) and
                                all(o == 0 for o in old.get_stroke_opacities()))

    def test_the_speed_arrow_is_as_long_as_the_speed_and_starts_at_mario(self):
        with tempconfig(FULL_HD):
            screen = self.screen(depth=100, speed=3)
            three = speed_arrow(screen, SPEED).submobjects[0]
            start = three.get_start()
            self.assertAlmostEqual(start[0], screen.col_x(87.5))
            self.assertAlmostEqual(start[1], screen.row_y(125))  # small Mario's middle
            six = speed_arrow(self.screen(depth=100, speed=6), SPEED).submobjects[0]
            self.assertAlmostEqual(six.get_length(), 2 * three.get_length())
            big = speed_arrow(self.screen(depth=100, speed=3, small=False), SPEED).submobjects[0]
            self.assertAlmostEqual(big.get_start()[1], screen.row_y(117))

    @unittest.skipUnless((DATA / "smb1" / "footage").exists(), "smb1 footage not recorded")
    def test_the_arrow_starts_inside_mario_on_the_real_footage(self):
        with tempconfig(FULL_HD):
            screen = Screen(Recording("smb1"), 1247)
            inside = seen = 0
            for f in range(1247, 1943, 3):
                screen.game.elapsed = (f - 1247) / NES_FPS
                picture = np.asarray(screen.recording.frames[screen.frame()])[..., :3]
                mario = np.argwhere((picture == MARIO_RED).all(2) | (picture == MARIO_SHOE).all(2))
                if len(mario) < 10:
                    continue
                seen += 1
                x, y, _ = mario_middle(screen)
                col, row = (x - screen.col_x(0)) / screen.dot, (screen.row_y(0) - y) / screen.dot
                (top, left), (bottom, right) = mario.min(0), mario.max(0) + 1
                inside += left <= col <= right and top <= row <= bottom
            self.assertGreater(seen, 150)
            self.assertGreaterEqual(inside / seen, 0.97)
