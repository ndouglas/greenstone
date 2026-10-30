import unittest

import numpy as np
from manim import ImageMobject

from series import style
from series.balance import SOFT, TILT, Balance


def play(animation) -> None:
    animation.begin()
    animation.interpolate(1.0)
    animation.finish()


def settle(b: Balance, seconds: float = 3.0) -> None:
    """Let the scale run on after a drop, as the scene's updaters do between frames."""
    for _ in range(round(seconds * 60)):
        b.update(1 / 60)


def land() -> Balance:
    b = Balance("yes", "no")
    play(b.drop(140, "140"))
    settle(b)
    return b


class BalanceTest(unittest.TestCase):
    def test_blocks_fall_by_sign(self):
        b = Balance("yes", "no")
        play(b.drop(140, "140"))
        play(b.drop(-10, "10", 1))
        play(b.drop(-152, "152", 3))
        self.assertEqual([k.value for k in b.pans["yes"]], [140])
        self.assertEqual([k.value for k in b.pans["no"]], [-10, -152])
        yes, no = b.pan_centre("yes"), b.pan_centre("no")
        for k in b.pans["yes"]:
            self.assertAlmostEqual(k.get_center()[0], yes[0], places=5)
        for k in b.pans["no"]:
            self.assertAlmostEqual(k.get_center()[0], no[0], places=5)
            self.assertGreater(k.get_center()[1], no[1])  # resting on the pan
        self.assertEqual(b.pans["no"][0].colour, style.COOL[1])  # its own hue within the temperature
        self.assertEqual(b.pans["no"][1].colour, style.COOL[3])
        self.assertEqual(b.pans["yes"][0].colour, style.WARM[0])
        big, small = b.pans["no"][1], b.pans["no"][0]
        self.assertGreater(big.side, small.side)  # sized by |value|

    def test_a_block_falls_into_its_pan(self):
        b = Balance("yes", "no")
        a = b.drop(140, "140")
        a.begin()
        a.interpolate(0.0)
        high = b.pans["yes"][0].get_center()[1]
        a.interpolate(1.0)
        a.finish()
        self.assertGreater(high, b.pans["yes"][0].get_center()[1] + 0.5)

    def test_the_beam_tips_to_the_heavier_pan(self):
        b = land()
        self.assertGreater(b.sum(), 0)
        self.assertLess(b.pan_centre("yes")[1], b.pan_centre("no")[1])
        play(b.drop(-10, "10"))
        play(b.drop(-152, "152"))
        settle(b)
        self.assertLess(b.sum(), 0)
        self.assertGreater(b.pan_centre("yes")[1], b.pan_centre("no")[1])

    def test_the_tilt_saturates(self):
        b = Balance("yes", "no")
        play(b.drop(5000, "5000"))
        settle(b)
        big = b.angle()
        play(b.drop(5000, "5000"))
        settle(b)
        self.assertLess(abs(b.angle() - big), 0.02)

    def test_the_verdict_takes_its_sides_colour(self):
        b = land()
        self.assertEqual(b.verdict(), "yes")
        self.assertEqual(b.verdict_label.get_color().to_hex().upper(), style.WARM[1])
        play(b.drop(-162, "162"))
        settle(b)
        self.assertEqual(b.verdict(), "no")
        self.assertEqual(b.verdict_label.get_color().to_hex().upper(), style.COOL[1])

    def test_a_balanced_scale_says_no(self):
        b = land()
        play(b.drop(-140, "140"))
        settle(b)
        self.assertEqual(b.sum(), 0)
        self.assertEqual(b.verdict(), "no")
        self.assertAlmostEqual(b.angle(), 0, places=3)
        self.assertEqual(Balance("yes", "no").verdict(), "no")

    def test_the_beam_runs_warm_to_cool(self):
        b = Balance("yes", "no")
        left, right = b.beam[0], b.beam[-1]
        l, r = left.get_stroke_color().to_rgb(), right.get_stroke_color().to_rgb()
        self.assertGreater(l[0], l[2])
        self.assertGreater(r[2], r[0])

    def test_the_beam_holds_until_the_block_lands(self):
        b = Balance("yes", "no")
        a = b.drop(140, "140")
        a.begin()
        for alpha in (0.0, 0.4, 0.8, 0.99):
            a.interpolate(alpha)
            settle(b, 0.2)
            self.assertEqual(b.angle(), 0.0)
            self.assertEqual(b.sum(), 0)
        a.interpolate(1.0)
        a.finish()
        self.assertEqual(b.sum(), 140)

    def test_the_beam_swings_when_a_block_lands(self):
        b = Balance("yes", "no")
        a = b.drop(140, "140")
        a.begin()
        a.interpolate(1.0)
        a.finish()
        target = TILT * float(np.tanh(140 / SOFT))
        angles = []
        for _ in range(90):  # 1.5 s
            b.update(1 / 60)
            angles.append(b.angle())
        self.assertGreater(angles[5], 0)  # moving toward it soon after landing
        self.assertLess(angles[0], target)
        self.assertGreater(max(angles), target * 1.05)  # overshoots
        self.assertLess(abs(angles[71] - target), 0.01)  # settled by 1.2 s
        self.assertLess(abs(angles[-1] - target), 0.01)
        for _ in range(60):
            b.update(1 / 60)
        self.assertAlmostEqual(b.angle(), target, places=3)

    def test_the_pans_follow_the_swinging_beam(self):
        b = Balance("yes", "no")
        play(b.drop(140, "140"))
        for _ in range(12):
            b.update(1 / 60)
        k = b.pans["yes"][0]
        self.assertAlmostEqual(k.get_center()[0], b.pan_centre("yes")[0], places=5)
        self.assertLess(b.pan_centre("yes")[1], b.end("yes")[1])

    def test_a_block_has_no_glow_image(self):
        b = land()
        k = b.pans["yes"][0]
        k.show(1.0)
        self.assertFalse(any(isinstance(m, ImageMobject) for m in k.get_family()))
