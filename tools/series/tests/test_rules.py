import unittest

import numpy as np

from series.lab import rules


def separable(n=400, seed=0):
    """Dots on either side of y = 2x + 1, kept clear of the line."""
    rng = np.random.default_rng(seed)
    x = rng.uniform(-1, 1, size=(n, 2))
    margin = x[:, 1] - (2 * x[:, 0] + 0.1)
    keep = np.abs(margin) > 0.1
    return x[keep], (margin[keep] > 0).astype(int)


def mostly(n=2000, seed=0):
    """Mostly on one side: a tenth of the labels flipped."""
    x, y = separable(n, seed)
    flip = np.random.default_rng(seed + 1).random(len(y)) < 0.1
    return x, np.where(flip, 1 - y, y)


class PerMoment(unittest.TestCase):
    def test_the_rules_that_wait_for_mistakes_find_a_clean_border(self):
        x, y = separable()
        for rule in ("perceptron", "simple"):
            line = rules.train(rule, x, y, rate=0.1, passes=50, seed=0)[-1]
            self.assertEqual(rules.accuracy(line, x, y), 1.0, rule)

    def test_the_perceptron_border_ignores_its_step_size(self):
        x, y = mostly()
        small = rules.train("perceptron", x, y, rate=0.01, passes=5, seed=3)
        large = rules.train("perceptron", x, y, rate=1.0, passes=5, seed=3)
        for a, b in zip(small, large):
            np.testing.assert_allclose(a.w * 100, b.w)
            np.testing.assert_array_equal(rules.says(a, x), rules.says(b, x))

    def test_only_the_square_trick_moves_on_a_right_answer(self):
        x = np.array([[1.0, 0.5]])
        y = np.array([1])
        start = rules.Line(np.array([0.2, 0.0]), 0.0)  # says yes (0.2), short of the target (1)
        for rule, moves in (("perceptron", False), ("simple", False), ("square", True)):
            after = rules.step(rule, start, x[0], y[0], rate=0.1)
            self.assertEqual(not np.allclose(after.w, start.w), moves, rule)

    def test_every_perceptron_nudge_points_partly_back_toward_zero(self):
        x, y = mostly(300)
        rng = np.random.default_rng(4)
        for _ in range(20):
            line = rules.Line(rng.normal(size=2), float(rng.normal()))
            before = np.r_[line.w, line.b]
            for xi, yi in zip(x, y):
                after = rules.step("perceptron", line, xi, yi, rate=0.1)
                nudge = np.r_[after.w, after.b] - before
                if nudge.any():
                    self.assertLess(nudge @ before, 0)

    def test_the_weights_around_each_nudge_differ_by_that_nudge(self):
        x, y = mostly(300)
        before, after = rules.weights_around_nudges(x, y, rate=0.1, passes=4, seed=2, count_from=3)
        counts = rules.nudges(x, y, rate=0.1, passes=4, seed=2, count_from=3)
        self.assertEqual(len(before), counts.sum())
        self.assertEqual(before.shape[1], 3)
        steps = np.linalg.norm(after - before, axis=1)
        sizes = np.linalg.norm(np.c_[x, np.ones(len(x))], axis=1) * 0.1
        for s in steps:
            self.assertTrue(np.isclose(sizes, s).any())

    def test_a_pass_visits_every_moment_once_in_its_own_order(self):
        x, y = mostly(200)
        a = rules.train("perceptron", x, y, rate=0.1, passes=3, seed=1)
        b = rules.train("perceptron", x, y, rate=0.1, passes=3, seed=1)
        c = rules.train("perceptron", x, y, rate=0.1, passes=3, seed=2)
        self.assertEqual(len(a), 3)
        np.testing.assert_array_equal(a[-1].w, b[-1].w)
        self.assertFalse(np.allclose(a[-1].w, c[-1].w))

    def test_leaving_moments_out_keeps_everyone_elses_order(self):
        x, y = mostly(200)
        none = np.zeros(len(y), bool)
        np.testing.assert_array_equal(rules.train("perceptron", x, y, 0.1, 3, 5, skip=none)[-1].w,
                                      rules.train("perceptron", x, y, 0.1, 3, 5)[-1].w)
        # Skipping a moment is the same as visiting it when it can't move the line.
        skip = np.zeros(len(y), bool)
        skip[7] = True
        by_hand, line = rules.Line(np.zeros(2), 0.0), None
        rng = np.random.default_rng(5)
        for _ in range(3):
            for i in rng.permutation(len(y)):
                if i != 7:
                    by_hand = rules.step("perceptron", by_hand, x[i], int(y[i]), 0.1)
        np.testing.assert_array_equal(rules.train("perceptron", x, y, 0.1, 3, 5, skip=skip)[-1].w, by_hand.w)

    def test_nudges_can_be_counted_from_a_later_pass(self):
        x, y = mostly()
        every = rules.nudges(x, y, rate=0.1, passes=6, seed=0)
        late = rules.nudges(x, y, rate=0.1, passes=6, seed=0, count_from=4)
        self.assertLess(late.sum(), every.sum())
        self.assertTrue((late <= every).all())

    def test_sizes_are_measured_in_steps_before_each_nudge(self):
        x = np.array([[1.0, 0.0]])
        y = np.array([1])
        # From zero the first visit is a mistake (the total isn't above
        # zero): one nudge, taken at size zero; then it's right.
        self.assertEqual(rules.sizes_before_nudges(x, y, rate=0.5, passes=2, seed=0).tolist(), [0.0])


class Batch(unittest.TestCase):
    def test_no_pull_past_the_target(self):
        x = np.array([[3.0, 0.0], [-3.0, 0.0]])
        y = np.array([1, 0])
        # Both moments are already past their targets with w = (1, 0): nothing moves.
        line = rules.batch_squared_hinge(x, y, rate=0.1, steps=5, start=rules.Line(np.array([1.0, 0.0]), 0.0))
        np.testing.assert_array_equal(line.w, [1.0, 0.0])
        self.assertEqual(line.b, 0.0)

    def test_the_batch_square_trick_reaches_least_squares(self):
        x, y = mostly()
        line = rules.batch_square(x, y, rate=0.1, steps=3000)
        exact = rules.least_squares(x, y)
        np.testing.assert_allclose(line.w, exact.w, atol=1e-6)
        self.assertAlmostEqual(line.b, exact.b, places=6)

    def test_batch_steps_are_returned_one_per_step(self):
        x, y = mostly()
        sizes = rules.batch_square_step_sizes(x, y, rate=0.1, steps=50)
        self.assertEqual(len(sizes), 50)
        self.assertTrue(all(b < a for a, b in zip(sizes, sizes[1:])))

    def test_a_small_step_shrinks_the_miss_and_a_big_one_grows_it(self):
        x, y = mostly()
        small = rules.batch_square_misses(x, y, rate=0.1, steps=30)
        big = rules.batch_square_misses(x, y, rate=10.0, steps=30)
        # The miss before the first step, then after every step.
        self.assertEqual(len(small), 31)
        self.assertTrue(all(b < a for a, b in zip(small, small[1:])))
        self.assertTrue(all(b > a for a, b in zip(big, big[1:])))
        self.assertEqual(small[0], 1.0)  # from zero, every total misses by one

    def test_square_borders_ignore_how_the_answers_are_coded(self):
        x, y = mostly()
        a = rules.least_squares(x, y, codes=(-1.0, 1.0))
        b = rules.least_squares(x, y, codes=(0.0, 1.0))
        np.testing.assert_array_equal(rules.says(a, x), rules.says(b, x))

    def test_the_logistic_trick_settles(self):
        x, y = mostly()
        a = rules.batch_logistic(x, y, rate=0.5, steps=4000)
        b = rules.batch_logistic(x, y, rate=0.5, steps=8000)
        self.assertLess(np.abs(a.w - b.w).max(), 0.05)


class Measures(unittest.TestCase):
    def test_swing_is_the_spread_of_scores_over_the_last_passes(self):
        self.assertAlmostEqual(rules.swing([0.5, 0.9, 0.7, 0.8, 0.75], last=3), 0.1)

    def test_turn_is_the_widest_angle_between_borders(self):
        lines = [rules.Line(np.array([1.0, 0.0]), 0.0), rules.Line(np.array([0.0, 2.0]), 0.0),
                 rules.Line(np.array([-1.0, 0.0]), 0.0)]
        self.assertAlmostEqual(rules.turn(lines), 180.0)

    def test_disagreement_counts_moments_two_borders_answer_differently(self):
        x = np.array([[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]])
        a = rules.Line(np.array([1.0, 0.0]), 0.0)
        b = rules.Line(np.array([0.0, 1.0]), 0.0)
        self.assertEqual(rules.disagreement(a, b, x), 0.5)


class Remembering(unittest.TestCase):
    def test_the_batch_perceptron_returns_its_line_after_every_step(self):
        x, y = mostly()
        lines = rules.batch_perceptron(x, y, rate=0.1, steps=7)
        self.assertEqual(len(lines), 7)

    def test_the_pocket_keeps_the_line_with_fewest_mistakes(self):
        x, y = separable()
        bad = rules.Line(np.array([1.0, 0.0]), 0.0)
        good = rules.train("perceptron", x, y, rate=0.1, passes=50, seed=0)[-1]
        self.assertIs(rules.pocket([bad, good, bad], x, y), good)

    def test_the_averaged_perceptron_is_the_mean_of_every_line_it_passed_through(self):
        x = np.array([[1.0, 0.0], [0.0, 1.0]])
        y = np.array([1, 0])
        # Visits (in this seed's order) move the line, and the average is
        # taken after every visit; checked against doing it by hand.
        order = np.random.default_rng(0).permutation(2)
        line, total = rules.Line(np.zeros(2), 0.0), np.zeros(3)
        for i in order:
            line = rules.step("perceptron", line, x[i], int(y[i]), 1.0)
            total += np.r_[line.w, line.b]
        avg = rules.averaged_perceptron(x, y, rate=1.0, passes=1, seed=0)
        np.testing.assert_allclose(np.r_[avg.w, avg.b], total / 2)

    def test_nudges_are_counted_per_moment(self):
        x, y = separable()
        counts = rules.nudges(x, y, rate=0.1, passes=50, seed=0)
        self.assertEqual(len(counts), len(y))
        self.assertGreater(counts.sum(), 0)
        x, y = mostly()
        self.assertGreater(rules.nudges(x, y, rate=0.1, passes=5, seed=0).max(), 1)


class Landscapes(unittest.TestCase):
    def test_a_border_is_scored_by_its_mistakes(self):
        x = np.array([[1.0, 0.0], [-1.0, 0.0], [2.0, 0.0]])
        y = np.array([1, 0, 0])
        mistakes, _ = rules.landscape(x, y, angles=np.array([0.0]), offsets=np.array([0.0]))
        self.assertAlmostEqual(mistakes[0, 0], 1 / 3)

    def test_the_bowls_bottom_is_the_least_squares_border(self):
        x, y = mostly()
        angles = np.radians(np.arange(0, 360, 0.5))
        offsets = np.linspace(-2, 2, 401)
        _, squared = rules.landscape(x, y, angles, offsets)
        i, j = np.unravel_index(np.argmin(squared), squared.shape)
        exact = rules.least_squares(x, y)
        self.assertLess(rules.turn([exact, rules.Line(np.array([np.cos(angles[i]), np.sin(angles[i])]), 0.0)]), 1.0)
        self.assertAlmostEqual(offsets[j], exact.b / np.linalg.norm(exact.w), delta=0.02)

    def test_right_by_too_much_counts_right_answers_past_their_target(self):
        x = np.array([[3.0], [0.5], [-3.0], [-0.5]])
        y = np.array([1, 1, 1, 0])
        line = rules.Line(np.array([1.0]), 0.0)
        self.assertEqual(rules.right_by_too_much(line, x, y).tolist(), [True, False, False, False])


class Experiment(unittest.TestCase):
    def test_numbers_are_refused_if_a_claim_fails(self):
        from series.lab.exp002 import check_claims
        check_claims(True, True, True)
        for failing in ((False, True, True), (True, False, True), (True, True, False)):
            with self.assertRaises(AssertionError):
                check_claims(*failing)
        with self.assertRaises(AssertionError):
            check_claims(True, True, True, control_settles=False)
        with self.assertRaises(AssertionError):
            check_claims(True, True, True, too_big_grows=False)
        with self.assertRaises(AssertionError):
            check_claims(True, True, True, one_run_ends_lowest=False)
        with self.assertRaises(AssertionError):
            check_claims(True, True, True, pulls_lean_back=False)
        with self.assertRaises(AssertionError):
            check_claims(True, True, True, sheet_edges_sloped=False)
        with self.assertRaises(AssertionError):
            check_claims(True, True, True, hidden_fraction=False)
        with self.assertRaises(AssertionError):
            check_claims(True, True, True, batch_steps_differ=False)
        with self.assertRaises(AssertionError):
            check_claims(True, True, True, search_words_fit=False)


    def test_low_points_are_counted_where_the_map_slopes(self):
        from series.lab.exp002 import local_lows
        a, o = np.meshgrid(np.linspace(-1, 1, 20), np.linspace(-1, 1, 15), indexing="ij")
        bowl = a ** 2 + o ** 2
        none = np.zeros(bowl.shape, bool)
        self.assertEqual(local_lows(bowl, none), 1)
        two = np.minimum((a - 0.5) ** 2, (a + 0.5) ** 2 + 0.1) + o ** 2
        self.assertEqual(local_lows(two, none), 2)
        flat = a > 0.4
        self.assertEqual(local_lows(two, flat), 1)

    def test_scroll_moments_keep_only_the_moments_the_question_is_about(self):
        from series.lab.data import Recording
        from series.lab.exp002 import SOON, scroll_moments
        n = 200
        ram = np.zeros((n, 2048), np.uint8)
        ram[:, 0x000E] = 8  # play
        ram[:, 0x0755] = 40  # left of the line
        ram[:, 0x0057] = 16  # one pixel a frame, to the right
        ram[100, 0x0775] = 1  # the screen moves once, at frame 100
        ram[10, 0x0755] = 90  # past the line
        ram[20, 0x0057] = 0xF0  # walking left
        ram[150:155, 0x000E] = 6  # not play
        rec = Recording(ram=ram, buttons=np.zeros(n, np.uint8), meta={"restart_frames": [60]})
        keep, soon = scroll_moments(rec)
        kept = set(keep.tolist())
        self.assertNotIn(10, kept)
        self.assertNotIn(20, kept)
        self.assertFalse(kept & set(range(60 - SOON, 60)), "a window crossed a restart")
        self.assertFalse(kept & set(range(150 - SOON, 155)), "a window left play")
        self.assertTrue(all(i < n - SOON for i in kept))
        by_frame = dict(zip(keep.tolist(), soon.tolist()))
        self.assertTrue(by_frame[100 - SOON] and by_frame[99])
        self.assertFalse(by_frame[100] or by_frame[100 - SOON - 1])


if __name__ == "__main__":
    unittest.main()
