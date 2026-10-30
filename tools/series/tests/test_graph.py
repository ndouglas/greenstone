import unittest

import numpy as np

from series.graph import NeuronGraph


def play(animation) -> None:
    animation.begin()
    animation.interpolate(1.0)
    animation.finish()


def graph(weight: float = -1, bias: float = 176, x_range=(0, 300), y_range=(-150, 250)) -> NeuronGraph:
    return NeuronGraph(weight, bias, x_range, y_range, "depth", "total", "in the air", "on the ground")


def rgb(segment) -> np.ndarray:
    return np.array([c for c in segment.get_stroke_color().to_rgb()])


class GraphTest(unittest.TestCase):
    def test_the_setting_is_where_the_line_crosses(self):
        g = graph()
        self.assertAlmostEqual(g.setting(), 176)
        self.assertAlmostEqual(g.total(g.setting()), 0)
        mark, axis = g.mark.get_center(), g.point(0, 0)[1]
        self.assertAlmostEqual(mark[1], axis, places=5)
        self.assertAlmostEqual(mark[0], g.point(176, 0)[0], places=5)

    def test_the_line_is_warm_above_and_cool_below(self):
        g = graph()
        axis = g.point(0, 0)[1]
        above = [rgb(s) for s in g.segments if s.get_center()[1] > axis + 0.05 and s.get_stroke_opacity() > 0]
        below = [rgb(s) for s in g.segments if s.get_center()[1] < axis - 0.05 and s.get_stroke_opacity() > 0]
        self.assertTrue(above and below)
        self.assertTrue(all(c[0] > c[2] for c in above))
        self.assertTrue(all(c[2] > c[0] for c in below))
        near = min(g.segments, key=lambda s: abs(s.get_center()[1] - axis))
        self.assertLess(rgb(near).max(), 0.3)  # dark at the crossing

    def test_tripling_never_moves_the_setting(self):
        g = graph()
        mark = g.mark.get_center().copy()
        zone = g.yes_zone.get_center().copy()
        play(g.scale_weights(3))
        self.assertAlmostEqual(g.setting(), 176)
        self.assertEqual(g.weight, -3)
        self.assertEqual(g.bias, 528)
        np.testing.assert_allclose(g.mark.get_center(), mark)
        np.testing.assert_allclose(g.yes_zone.get_center(), zone)

    def test_the_line_steepens_threefold(self):
        g = graph()
        def slope():
            live = [s for s in g.segments if s.get_stroke_opacity() > 0]
            s = live[len(live) // 4]
            d = s.get_end() - s.get_start()
            return d[1] / d[0]
        before = slope()
        play(g.scale_weights(3))
        self.assertAlmostEqual(slope() / before, 3, places=3)

    def test_a_crossing_off_the_page_does_not_crash(self):
        g = graph(-1, 500)
        self.assertGreater(g.setting(), 300)
        self.assertFalse(g.mark.get_stroke_opacity() > 0 or g.mark.get_fill_opacity() > 0)
        play(g.scale_weights(3))
        live = [s for s in g.segments if s.get_stroke_opacity() > 0]
        self.assertTrue(all(rgb(s)[0] > rgb(s)[2] for s in live))

    def test_the_sides_are_faintly_warm_and_cool(self):
        g = graph()
        self.assertLess(g.yes_zone.get_center()[0], g.no_zone.get_center()[0])
        self.assertLess(g.yes_zone.get_fill_opacity(), 0.15)
        self.assertLess(g.no_zone.get_fill_opacity(), 0.15)


if __name__ == "__main__":
    unittest.main()
