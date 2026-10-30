import unittest

import numpy as np

from series import style
from series.bordermap import BorderMap


def play(animation) -> None:
    animation.begin()
    animation.interpolate(1.0)
    animation.finish()


def make(weights=(0.0335, 0.168), bias=-5.08) -> BorderMap:
    rng = np.random.default_rng(1)
    points = np.stack([rng.uniform(0, 240, 60), rng.uniform(-20, 40, 60)], 1)
    outcomes = ["floor" if i % 3 == 0 else "blocks" if i % 3 == 1 else None for i in range(60)]
    return BorderMap(points, outcomes, weights, bias, "depth", "falling speed", "lands soon", "not yet")


class BorderMapTest(unittest.TestCase):
    def test_the_border_is_where_the_total_is_zero(self):
        m = make()
        a, b = m.border_points()
        for p in (a, b):
            self.assertAlmostEqual(float(np.dot(m.weights, p) + m.bias), 0, places=6)
        self.assertGreater(np.linalg.norm(a - b), 1)

    def test_a_flat_neuron_has_no_border_and_does_not_crash(self):
        m = make(weights=(0.0, 0.0), bias=1.0)
        a, b = m.border_points()
        self.assertTrue(np.allclose(a, b))
        play(m.lean((0.0, 0.0)))

    def test_outcomes_light_their_dots(self):
        m = make()
        lit = {o: {d.get_fill_color().to_hex().upper() for d, k in zip(m.dots, m.outcomes) if k == o} for o in ("floor", "blocks")}
        self.assertEqual(lit["floor"], {style.WARM[0]})
        self.assertEqual(lit["blocks"], {style.WARM[3]})
        ash = [d for d, k in zip(m.dots, m.outcomes) if k is None]
        self.assertTrue(all(d.get_fill_color().to_hex().upper() == "#6B5B52" for d in ash))
        self.assertTrue(all(d.get_fill_opacity() < 0.8 for d in ash))

    def test_sliding_the_bias_keeps_the_lean(self):
        m = make()
        a, b = m.border_points()
        slope = (b - a)[1] / (b - a)[0]
        play(m.slide_bias(-12.0))
        self.assertEqual(m.bias, -12.0)
        a2, b2 = m.border_points()
        self.assertAlmostEqual((b2 - a2)[1] / (b2 - a2)[0], slope, places=6)
        self.assertAlmostEqual(float(np.dot(m.weights, a2) + m.bias), 0, places=6)
        self.assertGreater(np.linalg.norm(a2 - a), 1)

    def test_leaning_turns_the_border(self):
        m = make()
        play(m.lean((0.1, 0.1)))
        a, b = m.border_points()
        self.assertAlmostEqual((b - a)[1] / (b - a)[0], -1.0, places=6)
        self.assertAlmostEqual(float(np.dot(m.weights, a) + m.bias), 0, places=6)

    def test_the_border_is_warm_end_to_end(self):
        m = make()
        for hue in (m.x_hue, m.y_hue):
            self.assertIn(hue.upper(), style.WARM)
        for piece in (m.pieces[0], m.pieces[-1]):
            r, _, b = piece.get_stroke_color().to_rgb()
            self.assertGreater(r, b)

    def test_the_side_labels_sit_outside_the_plot(self):
        m = make()
        top = m.page(m.hi)[1]
        for label in (m.yes_label, m.no_label):
            self.assertGreater(label.get_bottom()[1], top)
