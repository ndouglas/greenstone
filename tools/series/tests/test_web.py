import time
import unittest

import numpy as np
from manim import tempconfig

from series import style
from series.ember import rgb
from series.web import Web

SMALL = {"pixel_width": 320, "pixel_height": 180}


def drawn(web: Web) -> np.ndarray:
    return web.picture()[..., 3] > 20


class TestWeb(unittest.TestCase):
    def test_it_has_thousands_of_dots_each_joined_onward(self):
        with tempconfig(SMALL):
            web = Web()
        self.assertGreater(web.count, 2000)
        self.assertGreaterEqual(len(web.links), (web.count - (web.column == web.column.max()).sum()) * 2)

    def test_one_dot_shown_is_one_dot_in_the_middle(self):
        with tempconfig(SMALL):
            web = Web()
            web.shown.set_value(1)
            ys, xs = np.nonzero(drawn(web))
        self.assertTrue(len(xs) > 0)
        self.assertLess(xs.max() - xs.min(), 20)
        self.assertLess(abs(xs.mean() - 160), 20)

    def test_it_grows_outward(self):
        with tempconfig(SMALL):
            web = Web()
            web.shown.set_value(200)
            some = drawn(web).sum()
            web.shown.set_value(web.count)
            all_ = drawn(web).sum()
        self.assertGreater(all_, some * 3)

    def test_pushing_in_spreads_the_dots(self):
        with tempconfig(SMALL):
            web = Web()
            near = web.where(web.spots)
            web.zoom.set_value(3.0)
            far = web.where(web.spots)
        self.assertGreater(np.abs(far - web.focus).mean(), 2.5 * np.abs(near - web.focus).mean())

    def test_lit_dots_glow_warm_or_cool(self):
        with tempconfig(SMALL):
            web = Web()
            web.lit[:] = 1.0
            warm = web.picture()[..., :3].reshape(-1, 3).astype(float)
            web.lit[:] = -1.0
            cool = web.picture()[..., :3].reshape(-1, 3).astype(float)
        self.assertGreater(warm[:, 0].sum(), warm[:, 2].sum())  # amber: more red than blue
        self.assertGreater(cool[:, 2].sum(), cool[:, 0].sum())

    def test_it_fades_by_its_level(self):
        with tempconfig(SMALL):
            web = Web()
            web.level.set_value(0)
            self.assertFalse(drawn(web).any())

    def test_a_frame_draws_in_well_under_a_tenth_of_a_second_at_1080p(self):
        with tempconfig({"pixel_width": 1920, "pixel_height": 1080}):
            web = Web()
            start = time.perf_counter()
            web.picture()
            self.assertLess(time.perf_counter() - start, 0.1)
