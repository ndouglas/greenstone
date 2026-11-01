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

    def test_a_pass_visits_every_moment_once_in_its_own_order(self):
        x, y = mostly(200)
        a = rules.train("perceptron", x, y, rate=0.1, passes=3, seed=1)
        b = rules.train("perceptron", x, y, rate=0.1, passes=3, seed=1)
        c = rules.train("perceptron", x, y, rate=0.1, passes=3, seed=2)
        self.assertEqual(len(a), 3)
        np.testing.assert_array_equal(a[-1].w, b[-1].w)
        self.assertFalse(np.allclose(a[-1].w, c[-1].w))


class Batch(unittest.TestCase):
    def test_the_batch_square_trick_reaches_least_squares(self):
        x, y = mostly()
        line = rules.batch_square(x, y, rate=0.1, steps=3000)
        exact = rules.least_squares(x, y)
        np.testing.assert_allclose(line.w, exact.w, atol=1e-6)
        self.assertAlmostEqual(line.b, exact.b, places=6)

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


if __name__ == "__main__":
    unittest.main()
