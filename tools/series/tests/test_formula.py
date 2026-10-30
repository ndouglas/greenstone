import unittest

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
        a.interpolate(0.5)
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
