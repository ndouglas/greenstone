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

    def test_a_floor_can_lie_under_one_patch_alone(self):
        view = View(yaw=0.0, pitch=1.5, distance=11.0, target=(0.0, 0.0, 0.0))
        size, pixels = (8.0, 4.5), (320, 180)
        image = render([], view, pixels, floor=(0.4, 0.6, 0.4, 0.6, 2.0, 2.0))

        def alpha(across, up):
            x, y, _ = view.project(world(across, up, 0.0, 1.0), size)
            return image[int(round(pixels[1] / 2 - y / size[1] * pixels[1])), int(round(pixels[0] / 2 + x / size[0] * pixels[0])), 3]
        self.assertEqual(alpha(0.5, 0.5), 255)
        self.assertEqual(alpha(0.2, 0.5), 0)

    def test_a_smooth_patch_is_closed_down_to_its_base_on_every_side(self):
        g = Ground(np.full((6, 6), 0.5), top=1.0, smooth=True, across=(0.3, 0.7))
        view = View(yaw=1.5708, pitch=0.05, distance=11.0, target=(0.0, 0.0, 0.2))  # side on, from its left
        size, pixels = (8.0, 4.5), (320, 180)
        image = render([g], view, pixels, floor=False)
        x, y, _ = view.project(g.world(0.3, 0.5, 0.15), size)  # half way down its left side
        self.assertEqual(image[int(round(pixels[1] / 2 - y / size[1] * pixels[1])), int(round(pixels[0] / 2 + x / size[0] * pixels[0])), 3], 255)
