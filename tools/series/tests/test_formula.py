import unittest

import numpy as np

from series import style
from series.formula import Formula


def hex_of(mobject) -> str:
    return mobject.get_color().to_hex().upper()


def shown(mobject) -> float:
    return max(m.get_fill_opacity() for m in mobject.family_members_with_points())


def play(animation) -> None:
    animation.begin()
    animation.interpolate(1.0)
    animation.finish()


class FormulaTest(unittest.TestCase):
    def test_symbols_start_neutral(self):
        f = Formula(["w·x", "+ b"])
        self.assertEqual(f.text(), "w·x + b > 0")
        for key in ("f:w", "f:x", "f:b", "cmp", "dot0.1", "op1"):
            self.assertEqual(hex_of(f.token(key)), style.TEXT.upper(), key)

    def test_filled_terms_take_their_sign(self):
        f = Formula(["w·x", "+ b"])
        play(f.fill("w", -1))
        play(f.fill("b", 176))
        self.assertEqual(f.text(), "−1 · x + 176 > 0")
        self.assertEqual(hex_of(f.token("f:w")), style.COOL[0].upper())
        self.assertEqual(hex_of(f.token("f:b")), style.WARM[0].upper())
        self.assertEqual(hex_of(f.token("f:x")), style.TEXT.upper())
        self.assertEqual(f.token("f:w").slant, "NORMAL")

    def test_the_threshold_moves_into_the_bias(self):
        f = Formula(["w·x", "+ b"], "> 100")
        self.assertEqual(f.text(), "w·x + b > 100")
        start = f.token("thr").get_center()
        play(f.move_into_bias(100))
        self.assertEqual(f.text(), "w·x + b − 100 > 0")
        self.assertLess(f.token("thr").get_center()[0], start[0])
        self.assertLess(f.token("thr").get_center()[0], f.token("cmp").get_center()[0])
        self.assertGreater(f.token("thr").get_center()[0], f.token("f:b").get_center()[0])

    def test_the_animation_travels_and_pieces_do_not_linger(self):
        f = Formula(["w·x", "+ b"], "> 100")
        start = f.token("thr").get_center()[0]
        a = f.move_into_bias(100)
        a.begin()
        a.interpolate(0.0)
        self.assertAlmostEqual(f.token("thr").get_center()[0], start, places=3)
        a.interpolate(0.5)
        mid = f.token("thr").get_center()[0]
        a.interpolate(1.0)
        a.finish()
        self.assertGreater(mid, f.token("thr").get_center()[0])
        self.assertEqual(len(f.submobjects), 9)

    def test_a_term_takes_the_sign_of_its_contribution(self):
        f = Formula(["w·x", "- b"])
        play(f.fill("b", 5))
        self.assertEqual(f.text(), "w·x − 5 > 0")
        self.assertEqual(hex_of(f.token("f:b")), style.COOL[0].upper())
        play(f.fill("b", -5))
        self.assertEqual(f.text(), "w·x + 5 > 0")
        self.assertEqual(hex_of(f.token("f:b")), style.WARM[0].upper())

    def test_a_hyphen_in_a_name_is_not_a_minus(self):
        self.assertEqual(Formula(["x-y"]).text(), "x-y > 0")

    def test_filling_does_not_move_the_formula(self):
        f = Formula(["w·x", "+ b"], "> 100")
        f.scale(2.2).shift([1.0, -0.5, 0])
        before = f.get_center()
        for name, value in [("w", -1), ("b", 176)]:
            a = f.fill(name, value)
            a.begin()
            a.interpolate(0.5)
            a.interpolate(1.0)
            a.finish()
            self.assertAlmostEqual(f.get_center()[0], before[0], places=4)
        self.assertAlmostEqual(f.get_center()[1], before[1], places=1)

    def test_a_negative_bias_shows_a_true_minus(self):
        f = Formula(["w·x", "+ b"])
        play(f.fill("b", -5))
        self.assertEqual(f.text(), "w·x − 5 > 0")
        self.assertEqual(hex_of(f.token("f:b")), style.COOL[0].upper())


class StagingTest(unittest.TestCase):
    def test_a_changing_symbol_crossfades_instead_of_morphing(self):
        f = Formula(["w·x", "+ b"])
        a = f.fill("b", 176)
        a.begin()
        a.interpolate(0.4)
        new = f.token("f:b")
        ghosts = [m for m in f.submobjects if all(m is not t for t in f.tokens.values())]
        self.assertEqual(len(ghosts), 1)
        self.assertEqual(ghosts[0].original_text, "b")
        self.assertLess(shown(ghosts[0]), 1)
        self.assertGreater(shown(ghosts[0]), 0)
        self.assertEqual(new.original_text, "176")
        self.assertLess(shown(new), 1)
        a.interpolate(0.0)
        self.assertEqual(shown(ghosts[0]), 1)
        self.assertEqual(shown(new), 0)
        a.interpolate(1.0)
        a.finish()
        self.assertEqual(len(f.submobjects), len(f.tokens))
        self.assertEqual(shown(new), 1)

    def test_mid_fill_the_new_text_is_the_final_colour(self):
        f = Formula(["w·x", "+ b"])
        a = f.fill("b", -5)
        a.begin()
        a.interpolate(0.8)
        new = f.token("f:b")
        self.assertEqual((new.original_text, hex_of(new)), ("5", style.COOL[0].upper()))
        self.assertEqual(f.text(), "w·x − 5 > 0")
        ghost = [m for m in f.submobjects if all(m is not t for t in f.tokens.values()) and m.original_text == "b"][0]
        self.assertEqual((ghost.original_text, hex_of(ghost)), ("b", style.TEXT.upper()))

    def test_the_old_and_new_look_never_overprint(self):
        f = Formula(["w·x", "+ b"])
        a = f.fill("b", 176)
        a.begin()
        for alpha in np.linspace(0, 1, 61):
            a.interpolate(alpha)
            seen = [m for m in f.submobjects if shown(m) > 0.05 and m.original_text in ("b", "176")]
            self.assertLessEqual(len(seen), 1, f"at {alpha:.2f}")

    def test_nothing_visible_touches_while_the_threshold_moves(self):
        f = Formula(["w·x", "+ b"], "> 100")
        play(f.fill("w", -1))
        play(f.fill("b", 176))
        a = f.move_into_bias(100)
        a.begin()
        for alpha in np.linspace(0, 1, 121):
            a.interpolate(alpha)
            seen = [m for m in f.submobjects if shown(m) > 0.05]
            for i, p in enumerate(seen):
                for q in seen[i + 1:]:
                    apart = (p.get_right()[0] < q.get_left()[0] or q.get_right()[0] < p.get_left()[0] or
                             p.get_top()[1] < q.get_bottom()[1] or q.get_top()[1] < p.get_bottom()[1])
                    self.assertTrue(apart, f"{p.original_text} touches {q.original_text} at {alpha:.2f}")
        a.finish()

    def test_the_move_is_staged_so_nothing_collides(self):
        f = Formula(["w·x", "+ b"], "> 100")
        a = f.move_into_bias(100)
        a.begin()
        a.interpolate(0.0)
        self.assertEqual(shown(f.token("minus")), 0)
        self.assertEqual(shown(f.token("zero")), 0)
        a.interpolate(0.3)  # the threshold is well under way; the comparison has barely left
        self.assertEqual(shown(f.token("minus")), 0)
        self.assertEqual(shown(f.token("zero")), 0)
        a.interpolate(0.6)  # the threshold is home; the minus has begun
        self.assertGreater(shown(f.token("minus")), 0)
        self.assertEqual(shown(f.token("zero")), 0)
        top = f.token("thr").get_center()[1]
        a.interpolate(0.3)
        self.assertGreater(f.token("thr").get_center()[1], top)  # it hops over, not through
        a.interpolate(1.0)
        a.finish()
        self.assertEqual(shown(f.token("zero")), 1)
