"""The animations' data agrees with the numbers the script says."""

import json
import unittest

import numpy as np

from series.lab import rules
from series.lab.exp001 import percent
from series.lab.exp002 import OUT
from series.lab.film002 import FILM, load_film


@unittest.skipUnless(FILM.exists() and OUT.exists(), "run series.lab.exp002 and series.lab.film002 first")
class FilmAgreesWithNumbers(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.film = load_film()
        cls.numbers = json.loads(OUT.read_text())

    def said(self, name: str, value: float) -> None:
        self.assertEqual(percent(value)[0], self.numbers[name], name)

    def test_the_points_on_the_map(self):
        f = self.film
        self.assertEqual(len(f["spots"]), self.numbers["spots"])
        both = (f["spot_moved"] > 0) & (f["spot_still"] > 0)
        self.assertEqual(int(both.sum()), self.numbers["spots_with_both"])
        self.assertEqual(int((f["spot_moved"] + f["spot_still"]).sum()), self.numbers["train_moments"])

    def test_the_run_shown_is_the_lowest_order(self):
        f = self.film
        self.said("perceptron_lowest", f["run_scores"][-1])
        np.testing.assert_array_equal(f["run_ends"], f["order_ends"][0])
        self.assertEqual([percent(s)[0] for s in f["run_scores"][-10:]], self.numbers["perceptron_one_run"])

    def test_every_logged_nudge_is_a_mistake_and_its_fix(self):
        f = self.film
        x, y, step = f["x"], f["y"], 2.0 ** -7
        before = np.r_[[np.zeros(3)], f["run_after"][:-1]]
        i = f["run_moment"]
        sign = np.where(y[i] == 1, 1.0, -1.0)
        np.testing.assert_allclose(f["run_after"] - before, step * sign[:, None] * np.c_[x[i], np.ones(len(i))])
        totals = (np.c_[x[i], np.ones(len(i))] * before).sum(1)
        np.testing.assert_array_equal((totals > 0).astype(int) != y[i], True)

    def test_the_orders_scores_and_the_pocket(self):
        f = self.film
        finals = f["order_scores"][:, -1]
        self.said("perceptron_highest", finals.max())
        self.said("perceptron_second_lowest", np.sort(finals)[1])
        self.said("pocket_lowest", f["pocket_scores"].min())
        self.said("pocket_highest", f["pocket_scores"].max())
        self.said("perceptron_dropped_lowest", f["dropped_scores"].min())
        self.said("perceptron_dropped_highest", f["dropped_scores"].max())

    def test_the_control_stops_when_the_script_says(self):
        ends = self.film["control_ends"]
        moving = [p + 1 for p in range(1, len(ends)) if not np.array_equal(ends[p], ends[p - 1])]
        self.assertEqual(max(moving), self.numbers["control_last_moving_pass"])

    def test_the_batch_rules(self):
        f = self.film
        rest = f["batch_square"][-1]
        self.said("batch_square", rules.accuracy(rules.Line(rest[:2], rest[2]), f["xt"], f["yt"]))
        self.assertEqual(round(f["too_big_misses"][-1] / f["too_big_misses"][0]), self.numbers["too_big_growth"])
        scores = f["batch_perceptron_scores"][-100:]
        self.said("batch_perceptron_lowest", scores.min())
        self.said("batch_perceptron_highest", scores.max())
        self.said("hinge", rules.accuracy(rules.Line(f["hinge"][:2], f["hinge"][2]), f["xt"], f["yt"]))

    def test_the_u_bottoms_out_at_the_resting_weights(self):
        f = self.film
        for name in ("position", "speed", "bias"):
            self.assertEqual(int(np.argmin(f[f"u_{name}"])), len(f["u_turns"]) // 2, name)


if __name__ == "__main__":
    unittest.main()
