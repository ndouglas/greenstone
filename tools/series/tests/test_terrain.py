import unittest

import numpy as np

from series.ember import rgb
from series.terrain import HIGH, LOW, Ground, View, ramp, render, world


class Terrain(unittest.TestCase):
    def test_the_ramp_runs_from_gold_to_plum_ash(self):
        np.testing.assert_allclose(ramp(0.0), rgb(LOW), atol=1.0)
        np.testing.assert_allclose(ramp(1.0), rgb(HIGH), atol=1.0)

    def test_a_ground_is_drawn_where_the_view_says(self):
        g = Ground(np.full((8, 6), 0.5), top=1.0)
        view = View(yaw=0.4, pitch=0.9, distance=11.0, target=(0.0, 0.0, 0.0))
        size, pixels = (8.0, 4.5), (320, 180)
        image = render([g], view, pixels, floor=False)
        for across, up in ((0.5, 0.5), (0.2, 0.8), (0.85, 0.15)):
            x, y, _ = view.project(g.world(across, up), size)
            col = int(round(pixels[0] / 2 + x / size[0] * pixels[0]))
            row = int(round(pixels[1] / 2 - y / size[1] * pixels[1]))
            self.assertEqual(image[row, col, 3], 255, (across, up))
        self.assertEqual(image[0, 0, 3], 0)  # nothing drawn in the corner: transparent

    def test_heights_are_true_and_stretch_only_above_the_base(self):
        g = Ground(np.full((2, 2), 0.5), top=1.0)
        self.assertAlmostEqual(g.world(0.5, 0.5)[2], world(0.5, 0.5, 0.5, 1.0)[2])
        close = Ground(np.full((2, 2), 0.5), top=1.0, base=0.4, stretch=3.0)
        self.assertAlmostEqual(close.world(0.5, 0.5)[2], world(0.5, 0.5, 0.4 + 0.1 * 3, 1.0)[2])

    def test_a_view_part_way_keeps_its_distance_by_ratio(self):
        a, b = View(distance=10.0), View(distance=0.1)
        self.assertAlmostEqual(a.between(b, 0.5).distance, 1.0)
