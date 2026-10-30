import unittest

from series import style
from series.balance import Balance


def play(animation) -> None:
    animation.begin()
    animation.interpolate(1.0)
    animation.finish()


def land() -> Balance:
    b = Balance("yes", "no")
    play(b.drop(140, "140"))
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
        self.assertLess(b.sum(), 0)
        self.assertGreater(b.pan_centre("yes")[1], b.pan_centre("no")[1])

    def test_the_tilt_saturates(self):
        b = Balance("yes", "no")
        play(b.drop(5000, "5000"))
        big = b.angle()
        play(b.drop(5000, "5000"))
        self.assertLess(abs(b.angle() - big), 0.02)

    def test_the_verdict_takes_its_sides_colour(self):
        b = land()
        self.assertEqual(b.verdict(), "yes")
        self.assertEqual(b.verdict_label.get_color().to_hex().upper(), style.WARM[1])
        play(b.drop(-162, "162"))
        self.assertEqual(b.verdict(), "no")
        self.assertEqual(b.verdict_label.get_color().to_hex().upper(), style.COOL[1])

    def test_a_balanced_scale_says_no(self):
        b = land()
        play(b.drop(-140, "140"))
        self.assertEqual(b.sum(), 0)
        self.assertEqual(b.verdict(), "no")
        self.assertAlmostEqual(b.angle(), 0)
        self.assertEqual(Balance("yes", "no").verdict(), "no")

    def test_the_beam_runs_warm_to_cool(self):
        b = Balance("yes", "no")
        left, right = b.beam[0], b.beam[-1]
        l, r = left.get_stroke_color().to_rgb(), right.get_stroke_color().to_rgb()
        self.assertGreater(l[0], l[2])
        self.assertGreater(r[2], r[0])
