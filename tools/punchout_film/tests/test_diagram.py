import unittest

import numpy as np

from punchout_film import diagram
from reservoir_film import draw

RECT = (40, 80, 1200, 420)
ALL_OURS = {"see": "ours", "remember": "ours", "decide": "ours", "learn": "dagger", "yardstick": "ours"}


def top_edge(img, rect):
    x, y, w, h = rect
    return [tuple(p) for p in np.asarray(img)[y, x:x + w]]


class Diagram(unittest.TestCase):
    def test_pieces_by_chapter(self):
        self.assertEqual(diagram.pieces_by_chapter(1), {"see": "mainstream", "remember": "mainstream",
                                                        "decide": "mainstream", "learn": "mainstream",
                                                        "yardstick": "hidden"})
        self.assertEqual(diagram.pieces_by_chapter(4)["learn"], "ours")
        self.assertEqual(diagram.pieces_by_chapter(4)["see"], "mainstream")
        five = diagram.pieces_by_chapter(5)
        self.assertEqual([five[p] for p in ("see", "remember", "decide")], ["ours"] * 3)
        self.assertEqual(diagram.pieces_by_chapter(7)["learn"], "ours")
        self.assertEqual(diagram.pieces_by_chapter(8)["learn"], "dagger")
        self.assertEqual(diagram.pieces_by_chapter(10)["yardstick"], "hidden")
        self.assertEqual(diagram.pieces_by_chapter(11), ALL_OURS)

    def test_learned_boxes_are_gold_and_frozen_boxes_blue(self):
        img = draw.blank()
        diagram.draw_diagram(img, RECT, ALL_OURS)
        rects = diagram.box_rects(RECT, ALL_OURS)
        self.assertIn(diagram.LEARNED, top_edge(img, rects["decide"]))
        self.assertNotIn(diagram.FROZEN, top_edge(img, rects["decide"]))
        self.assertIn(diagram.FROZEN, top_edge(img, rects["see"]))
        self.assertNotIn(diagram.LEARNED, top_edge(img, rects["see"]))

    def test_hidden_pieces_have_no_box(self):
        self.assertNotIn("yardstick", diagram.box_rects(RECT, diagram.pieces_by_chapter(1)))

    def test_swap_renders_at_every_progress(self):
        states = diagram.pieces_by_chapter(4)
        frames = []
        for p in (0.0, 0.5, 1.0):
            img = draw.blank()
            diagram.draw_diagram(img, RECT, states, swap=("see", p))
            frames.append(np.asarray(img))
        self.assertFalse(np.array_equal(frames[0], frames[2]))
        self.assertFalse(np.array_equal(frames[0], frames[1]))

    def test_a_swap_ends_on_our_box(self):
        states = diagram.pieces_by_chapter(4)
        img = draw.blank()
        diagram.draw_diagram(img, RECT, states, swap=("see", 1.0))
        self.assertIn(diagram.FROZEN, top_edge(img, diagram.box_rects(RECT, states)["see"]))

    def test_a_swap_can_go_to_dagger(self):
        states = diagram.pieces_by_chapter(7)
        before, after = draw.blank(), draw.blank()
        diagram.draw_diagram(before, RECT, states)
        diagram.draw_diagram(after, RECT, states, swap=("learn", 1.0, "dagger"))
        self.assertFalse(np.array_equal(np.asarray(before), np.asarray(after)))

    def test_focus_dims_the_other_pieces(self):
        plain, focused = draw.blank(), draw.blank()
        diagram.draw_diagram(plain, RECT, ALL_OURS)
        diagram.draw_diagram(focused, RECT, ALL_OURS, focus="remember")
        rects = diagram.box_rects(RECT, ALL_OURS)
        self.assertEqual(top_edge(plain, rects["remember"]), top_edge(focused, rects["remember"]))
        self.assertNotEqual(top_edge(plain, rects["decide"]), top_edge(focused, rects["decide"]))

    def test_labels_are_the_specs(self):
        self.assertEqual(diagram.OURS["see"].sub, "5,712 pixels to 256 numbers, rolled once")
        self.assertEqual(diagram.OURS["decide"], diagram.Box("one linear readout", "one ridge solve", True))
        self.assertEqual(diagram.MAINSTREAM["see"], diagram.Box("conv net", "learned end to end", True))
