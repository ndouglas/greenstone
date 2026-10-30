import tempfile
import unittest
from pathlib import Path

import numpy as np
from manim import LEFT, ORIGIN, RIGHT, tempconfig

from series import style
from series.neuron import Input, Neuron
from series.overlays import feet_line
from series.screen import Screen
from series.tests.test_screen import FULL_HD, synthetic

CENTRE = RIGHT * 2.3


def constant(value: float):
    return lambda: value


def at(x: float, y: float):
    return lambda: np.array([x, y, 0.0])


def hexes(mobject) -> list[str]:
    return [c.to_hex().upper() for c in mobject.get_stroke_colors()]


def neuron(inputs: list[Input], bias: float, bias_source=None) -> Neuron:
    return Neuron(inputs, bias, bias_source, CENTRE, "in the air", "on the ground")


def crosses(p: np.ndarray, q: np.ndarray, ignore: float = 0.06) -> bool:
    """Whether two polylines cross anywhere farther than `ignore` from either's ends."""
    def away(points):
        far = (np.linalg.norm(points - points[0], axis=1) > ignore) & (np.linalg.norm(points - points[-1], axis=1) > ignore)
        return points[far]
    p, q = away(p[:, :2]), away(q[:, :2])
    for a, b in zip(p, p[1:]):
        for c, d in zip(q, q[1:]):
            def side(o, x, y):
                return np.sign((x[0] - o[0]) * (y[1] - o[1]) - (x[1] - o[1]) * (y[0] - o[0]))
            if side(a, b, c) != side(a, b, d) and side(c, d, a) != side(c, d, b):
                return True
    return False


def polyline(path) -> np.ndarray:
    return np.array([path.point_from_proportion(t) for t in np.linspace(0, 1, 60)])


class TestNeuron(unittest.TestCase):
    def test_the_total_is_the_weighted_sum(self):
        n = neuron([Input(constant(3), 2, None, "a"), Input(constant(5), -1, None, "b")], 4)
        self.assertEqual(n.total(), 5)
        self.assertTrue(n.answer())

    def test_zero_is_dark_and_no(self):
        n = neuron([Input(constant(176), -1, at(-1, -1), "depth")], 176, at(-1, -2))
        self.assertEqual(n.total(), 0)
        self.assertFalse(n.answer())
        self.assertEqual(n.warm_level.get_value(), 0)
        self.assertEqual(n.cool_level.get_value(), 0)
        self.assertEqual(n.dot.get_fill_opacity(), 0)
        self.assertIn(n.socket.get_stroke_color().to_hex().upper(), [c.upper() for c in style.COOL])
        self.assertEqual(n.verdict.original_text, "on the ground")
        self.assertIn(n.verdict.get_color().to_hex().upper(), [c.upper() for c in style.COOL])

    def test_the_answer_is_crisp(self):
        for total in (0.01, 500):
            n = neuron([Input(constant(total), 1, None, "x")], 0)
            self.assertTrue(n.answer())
            self.assertEqual(n.dot.get_fill_opacity(), 1)
            self.assertEqual(n.verdict.original_text, "in the air")
            self.assertEqual(n.verdict.get_color().to_hex().upper(), style.TEXT.upper())

    def test_the_total_is_roman_with_a_true_minus(self):
        n = neuron([Input(constant(200), -1, at(-1, -1), "depth")], 176, at(-1, -2))
        self.assertEqual(n.total_text.text, "−24")
        self.assertEqual(n.total_text.slant, "NORMAL")
        self.assertGreater(n.cool_level.get_value(), 0)
        self.assertEqual(n.warm_level.get_value(), 0)

    def test_the_glow_eases_toward_the_total(self):
        value = [0.0]
        n = neuron([Input(lambda: value[0], 1, None, "x")], 0)
        value[0] = 60
        n.update(1 / 60)
        partway = n.warm_level.get_value()
        self.assertGreater(partway, 0)
        self.assertLess(partway, style.light(60))
        n.update(1)
        self.assertAlmostEqual(n.warm_level.get_value(), style.light(60))

    def test_paths_take_their_contributions_temperature(self):
        n = neuron([Input(constant(176), -1, at(-1, 0), "depth")], 176, at(-1, -2))
        path = hexes(n.paths[0])
        self.assertEqual(path[0], style.WARM[0].upper())
        self.assertIn(path[-1], [c.upper() for c in style.COOL])
        bias = hexes(n.bias_path)
        self.assertIn(bias[-1], [c.upper() for c in style.WARM])
        warm = neuron([Input(constant(176), 1, at(-1, 0), "depth")], -176, at(-1, -2))
        self.assertIn(hexes(warm.paths[0])[-1], [c.upper() for c in style.WARM])
        self.assertIn(hexes(warm.bias_path)[-1], [c.upper() for c in style.COOL])

    def test_a_path_starts_at_its_source_and_ends_on_the_ring(self):
        n = neuron([Input(constant(1), -1, at(-1, 0.5), "depth")], 176, at(-1, -2))
        for path, start in [(n.paths[0], (-1, 0.5)), (n.bias_path, (-1, -2))]:
            np.testing.assert_allclose(path.get_start()[:2], start, atol=1e-9)
            self.assertAlmostEqual(np.linalg.norm(path.get_end() - CENTRE), n.ring.radius, places=6)

    def test_a_bias_with_no_place_has_no_path(self):
        n = neuron([Input(constant(1), 1, at(-1, 0), "depth")], -152)
        self.assertIsNone(n.bias_path)
        self.assertEqual("".join(t.text for t in n.bias_label.submobjects), "−152")
        self.assertEqual(neuron([Input(constant(1), 1, None, "x")], 4).bias_label.submobjects[1].text, "4")

    def test_inputs_can_come_from_anywhere(self):
        with tempfile.TemporaryDirectory() as tmp, tempconfig(FULL_HD):
            recording = synthetic(Path(tmp))
            recording.ram[:, 0xCE] = 100
            screen = Screen(recording, 10, centre=LEFT * 3)
            feet = feet_line(screen)
            ruler = lambda: np.array([screen.col_x(256), feet.y(), 0])
            square = at(-1, 2.5)  # a memory square: a fixed point
            n = neuron([Input(feet.value, -1, ruler, "depth"), Input(constant(7), 1, square, "byte"),
                        Input(constant(3), 2, None, "speed")], 0)
            self.assertEqual(n.total(), -100 + 7 + 6)
            self.assertEqual(len(n.paths), 3)
            np.testing.assert_allclose(n.paths[0].get_start(), ruler(), atol=1e-9)
            np.testing.assert_allclose(n.paths[1].get_start(), square(), atol=1e-9)
            self.assertIsNotNone(n.bare[2])
            self.assertIsNone(n.bare[0])

    def test_paths_follow_their_sources(self):
        y = [0.5]
        n = neuron([Input(constant(1), -1, lambda: np.array([-1, y[0], 0]), "depth")], 176, at(-1, -2))
        y[0] = -0.7
        n.update(1 / 60)
        self.assertAlmostEqual(n.paths[0].get_start()[1], -0.7)

    def test_paths_never_cross(self):
        y = [0.0]
        n = neuron([Input(constant(1), -1, lambda: np.array([-1, y[0], 0]), "depth")], 176, at(-1, -1.5))
        for depth in np.linspace(1.5, -3, 46):
            y[0] = depth
            n.update(1 / 60)
            self.assertFalse(crosses(polyline(n.paths[0]), polyline(n.bias_path)), f"at y {depth:.2f}")

    def test_three_paths_never_cross(self):
        ys = [0.0, 0.0]
        n = neuron([Input(constant(1), -1, lambda: np.array([-1, ys[0], 0]), "depth"),
                    Input(constant(1), 5, lambda: np.array([-1.5, ys[1], 0]), "speed")], 176, at(-1, -1.5))
        for a in np.linspace(1.5, -3, 10):
            for b in np.linspace(1.5, -3, 10):
                ys[:] = a, b
                n.update(1 / 60)
                lines = [polyline(p) for p in n.paths + [n.bias_path]]
                for i in range(3):
                    for j in range(i + 1, 3):
                        self.assertFalse(crosses(lines[i], lines[j]), f"paths {i},{j} at {a:.2f}, {b:.2f}")

    def test_replaced_texts_are_hidden(self):
        value = [10.0]
        n = neuron([Input(lambda: value[0], 1, None, "x")], 0)
        old = [n.total_text, n.verdict] + list(n.bare[0].submobjects)
        value[0] = -10
        n.update(0)
        self.assertEqual(n.total_text.original_text, "−10")
        for text in old:
            self.assertEqual(max(text.get_fill_opacities()), 0)

    def test_arrival_comes_in_order_and_ends_drawn(self):
        n = neuron([Input(constant(10), 1, at(-1, 0), "x")], 0, at(-1, -2))
        steps = n.arrive()
        self.assertEqual(len(steps), 4)
        self.assertEqual(n.dot.get_fill_opacity(), 0)  # hidden until the output arrives
        for step in steps:
            step.begin()
            step.interpolate(1)
            step.finish()
        n.update(0)
        self.assertEqual(n.dot.get_fill_opacity(), 1)
        self.assertAlmostEqual(np.linalg.norm(n.paths[0].get_end() - CENTRE), n.ring.radius, places=6)


if __name__ == "__main__":
    unittest.main()
