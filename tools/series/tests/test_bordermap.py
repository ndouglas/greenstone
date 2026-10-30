import unittest

import numpy as np

from manim import ManimColor

from series import style
from series.bordermap import FADE, BorderMap


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

    def test_a_flat_neuron_draws_nothing_and_divides_by_nothing(self):
        with np.errstate(all="raise"):  # a division by a zero weight would raise here
            m = make(weights=(0.0, 0.0), bias=1.0)
            play(m.lean((0.0, 0.0)))
            play(m.slide_bias(2.0))
            for piece in m.pieces:
                self.assertEqual(piece.get_stroke_opacity(), 0)
                self.assertTrue(np.isfinite(piece.points).all())
            for band in m.bands:
                self.assertTrue(np.isfinite(band.points).all())

    def test_a_border_with_one_weight_zero_runs_straight(self):
        with np.errstate(all="raise"):
            m = make(weights=(0.03, 0.0), bias=-3.0)
            a, b = m.border_points()
        self.assertAlmostEqual(a[0], b[0])  # vertical, at depth 100
        self.assertAlmostEqual(a[0], 100.0)
        self.assertGreater(abs(a[1] - b[1]), 1)

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
        for piece, hue in ((m.pieces[0], m.x_hue), (m.pieces[-1], m.y_hue)):  # each end is its own input's warm hue
            got, want = np.array(piece.get_stroke_color().to_rgb()), np.array(ManimColor(hue).to_rgb())
            self.assertLess(np.abs(got - want).max(), 0.08)
            self.assertGreater(np.abs(got - np.array(ManimColor(style.COOL[0]).to_rgb())).max(), 0.2)

    def test_the_side_labels_sit_outside_the_plot(self):
        m = make()
        top = m.page(m.hi)[1]
        for label in (m.yes_label, m.no_label):
            self.assertGreater(label.get_bottom()[1], top)

    def test_the_yes_side_is_tinted_warm_and_the_no_side_cool(self):
        m = make()
        play(m.lean((-0.05, 0.1)))  # and again once the border has turned
        for band in m.bands[:FADE] + m.bands[FADE:]:
            if band.get_fill_opacity() == 0:
                continue
            centre = band.get_center()
            p = m.lo + (centre - m.at + np.array([m.size[0], m.size[1], 0]) / 2)[:2] / m.size * (m.hi - m.lo)
            r, _, b = band.get_fill_color().to_rgb()
            if m.bands.index(band) < FADE:
                self.assertGreater(m.total(p), 0)
                self.assertGreater(r, b)
            else:
                self.assertLess(m.total(p), 0)
                self.assertGreater(b, r)

    def test_the_labels_sit_at_their_own_corners_of_the_plot(self):
        m = make()
        self.assertAlmostEqual(m.yes_label.get_right()[0], m.page(m.hi)[0], places=5)
        self.assertAlmostEqual(m.no_label.get_left()[0], m.page(m.lo)[0], places=5)

    def test_a_lean_pivots_about_the_border_as_it_begins(self):
        m = make()
        turn = m.lean((0.1, 0.1))
        play(m.slide_bias(-8.0))  # after the lean was made
        a, b = m.border_points()
        middle = (a + b) / 2
        turn.begin()
        turn.interpolate(1.0)
        turn.finish()
        self.assertAlmostEqual(m.total(middle), 0, places=6)  # still on the border it was turning about

    def test_maps_do_not_share_a_default_position(self):
        a, b = make(), make()
        a.at += 1
        np.testing.assert_allclose(b.at, 0)
