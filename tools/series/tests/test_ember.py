import unittest

import manimpango
import numpy as np
from manim import ValueTracker

from series import style
from series.ember import backdrop, fade_image, glow, serif
from series.style import ASH, COOL, TEXT, WARM, light, number, temperature


class TestStyle(unittest.TestCase):
    def test_temperature_is_sign(self):
        self.assertIn(temperature(5), WARM)
        self.assertIn(temperature(-5), COOL)
        self.assertEqual(temperature(0), ASH)
        self.assertEqual(temperature(2, hue=1), WARM[1])

    def test_light_is_smooth_and_saturating(self):
        self.assertEqual(light(0, 60), 0)
        values = [light(v, 60) for v in (1, 10, 30, 100, 1000)]
        self.assertEqual(values, sorted(values))
        self.assertEqual(light(-30, 60), light(30, 60))
        self.assertLess(light(10_000, 60), 1)

    def test_negative_numbers_use_a_true_minus(self):
        self.assertEqual(number(-41), "−41")
        self.assertEqual(number(3), "3")
        self.assertEqual(number(-0.4), "0")


class TestEmber(unittest.TestCase):
    def test_glow_has_no_edge(self):
        alpha = glow(WARM[1], 1.0).pixel_array[:, :, 3].astype(int)
        for border in (alpha[0], alpha[-1], alpha[:, 0], alpha[:, -1]):
            self.assertTrue((border == 0).all())
        c = alpha.shape[0] // 2
        self.assertEqual(alpha.max(), alpha[c, c])

    def test_glow_is_smooth(self):
        alpha = glow(WARM[1], 1.0).pixel_array[:, :, 3].astype(int)
        c = alpha.shape[0] // 2
        rays = [alpha[c, c:], alpha[c:, c][:], alpha[c, :c + 1][::-1], alpha[:c + 1, c][::-1],
                np.diagonal(alpha[c:, c:])]
        for ray in rays:
            self.assertTrue((np.diff(ray) <= 0).all())
        self.assertLessEqual(np.abs(np.diff(alpha, axis=0)).max(), 6)
        self.assertLessEqual(np.abs(np.diff(alpha, axis=1)).max(), 6)

    def test_backdrop_is_dithered_not_banded(self):
        rgb = backdrop().pixel_array[:, :, :3].astype(int)
        w = rgb.shape[1]
        row = rgb[rgb.shape[0] // 2, w * 3 // 10:w * 7 // 10, 0]
        longest = run = 1
        for a, b in zip(row, row[1:]):
            run = run + 1 if a == b else 1
            longest = max(longest, run)
        self.assertLessEqual(longest, 16)
        self.assertGreater((np.diff(row) != 0).mean(), 0.3)  # undithered, most neighbours are equal
        self.assertLessEqual(np.abs(np.diff(row)).max(), 3)

    def test_serif_is_charter(self):
        self.assertIn("Charter", manimpango.list_fonts(), "install Charter")
        self.assertEqual(serif("x", TEXT).font, "Charter")
        self.assertEqual(style.FONT, "Charter")

    def test_fading_an_image_keeps_its_gradient(self):
        image = glow(WARM[0], 1.0)
        full = image.pixel_array[:, :, 3].astype(int)
        c = full.shape[0] // 2
        mid = (c, c + c // 2)
        level = ValueTracker(1.0)
        fade_image(image, level)
        level.set_value(0.5)
        image.update(0)
        half = image.pixel_array[:, :, 3].astype(int)
        self.assertLessEqual(np.abs(half - full / 2).max(), 1)
        self.assertGreater(half[c, c], half[mid] + 5)
        self.assertAlmostEqual(half[mid] / half[c, c], full[mid] / full[c, c], delta=0.02)
        level.set_value(1.0)
        image.update(0)
        self.assertTrue((image.pixel_array[:, :, 3] == full).all())


if __name__ == "__main__":
    unittest.main()
