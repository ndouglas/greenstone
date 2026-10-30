import unittest

from series import style
from series.formula import Formula


def hex_of(mobject) -> str:
    return mobject.get_color().to_hex().upper()


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
