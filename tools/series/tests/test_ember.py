import unittest

import manimpango
import numpy as np
from manim import ValueTracker

from series import style
from series.ember import fade_image, glow, serif
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
        alpha = glow(WARM[1], 1.0).pixel_array[:, :, 3]
        for border in (alpha[0], alpha[-1], alpha[:, 0], alpha[:, -1]):
            self.assertTrue((border == 0).all())
        self.assertEqual(alpha.max(), alpha[alpha.shape[0] // 2, alpha.shape[1] // 2])
        self.assertGreater(len(np.unique(alpha)), 150)

    def test_serif_is_charter(self):
        self.assertIn("Charter", manimpango.list_fonts(), "install Charter")
        self.assertEqual(serif("x", TEXT).font, "Charter")
        self.assertEqual(style.FONT, "Charter")

    def test_fading_an_image_keeps_its_gradient(self):
        image = glow(WARM[0], 1.0)
        full = image.pixel_array[:, :, 3].astype(int)
        level = ValueTracker(1.0)
        fade_image(image, level)
        level.set_value(0.5)
        image.update(0)
        half = image.pixel_array[:, :, 3].astype(int)
        self.assertLessEqual(np.abs(half - full / 2).max(), 1)
        self.assertEqual(half.argmax(), full.argmax())
        self.assertGreater(len(np.unique(half)), 100)


if __name__ == "__main__":
    unittest.main()
