import unittest

from manim import Rectangle

from series import style
from series.records import ScoreBars, Tapes


def play(animation) -> None:
    animation.begin()
    animation.interpolate(1.0)
    animation.finish()


class ScoreBarsTest(unittest.TestCase):
    def test_bars_are_proportional(self):
        bars = ScoreBars(54)
        play(bars.add(94, "the neuron"))
        play(bars.add(27, "half", faint=True))
        base, result, small = (b.box.height for b in bars.bars)
        self.assertAlmostEqual(result / base, 94 / 54, places=4)
        self.assertAlmostEqual(small / base, 27 / 54, places=4)
        bottoms = {round(b.box.get_bottom()[1], 6) for b in bars.bars}
        self.assertEqual(len(bottoms), 1)  # they stand on one line

    def test_the_baseline_is_ash(self):
        bars = ScoreBars(54)
        play(bars.add(94, "the neuron"))
        self.assertEqual(bars.bars[0].colour, style.ASH)
        self.assertEqual(bars.bars[1].colour, style.WARM[0])
        self.assertEqual(bars.bars[0].box.get_stroke_colors()[0].to_hex().upper(), style.ASH)

    def test_a_faint_bar_is_faint(self):
        bars = ScoreBars(54)
        play(bars.add(94, "the neuron"))
        play(bars.add(94, "to beat", faint=True))
        self.assertLess(bars.bars[2].box.get_stroke_opacity(), bars.bars[1].box.get_stroke_opacity())

    def test_a_bar_grows_from_nothing(self):
        bars = ScoreBars(54)
        a = bars.add(94, "the neuron")
        a.begin()
        a.interpolate(0.0)
        self.assertLess(bars.bars[1].box.height, 0.01)
        a.interpolate(1.0)
        a.finish()


class TapesTest(unittest.TestCase):
    def test_labelled_strips(self):
        tapes = Tapes(3, ["1", "2", "3"])
        self.assertEqual([t.label.text for t in tapes.tapes], ["1", "2", "3"])
        ys = [t.strip.get_center()[1] for t in tapes.tapes]
        self.assertEqual(ys, sorted(ys, reverse=True))

    def test_the_third_slides_apart(self):
        tapes = Tapes(3, ["1", "2", "3"])
        before = [t.strip.get_center().copy() for t in tapes.tapes]
        play(tapes.apart(2))
        after = [t.strip.get_center() for t in tapes.tapes]
        self.assertTrue((after[0] == before[0]).all() and (after[1] == before[1]).all())
        self.assertGreater(abs(after[2] - before[2]).max(), 0.3)
        self.assertGreater(after[2][0], before[2][0])
        self.assertTrue(abs(tapes.tapes[2].label.get_center()[0] - after[2][0]) < tapes.tapes[2].strip.width)

    def test_dying_stretches_grey_out(self):
        tapes = Tapes(3, ["1", "2", "3"])
        play(tapes.grey(1, [(0.2, 0.3), (0.7, 0.8)]))
        greys = tapes.tapes[1].greys
        self.assertEqual(len(greys), 2)
        strip = tapes.tapes[1].strip
        for g, (a, b) in zip(greys, [(0.2, 0.3), (0.7, 0.8)]):
            self.assertAlmostEqual(g.get_left()[0], strip.get_left()[0] + a * strip.width, places=5)
            self.assertAlmostEqual(g.width, (b - a) * strip.width, places=5)
            self.assertEqual(g.get_fill_color().to_hex().upper(), style.ASH)
