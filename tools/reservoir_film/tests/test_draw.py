import unittest

import numpy as np
from PIL import Image

from reservoir_film import draw
from reservoir_film.model import H, W


class Widgets(unittest.TestCase):
    def test_cards_are_frame_sized(self):
        for img in (draw.title_card(3, "Pour Mario into Tetris", "a subtitle"), draw.text_card(["one", "two"], kicker="k")):
            self.assertEqual(img.size, (W, H))
            self.assertEqual(img.mode, "RGB")

    def test_caption_refuses_three_lines(self):
        img = draw.blank()
        draw.caption(img, "A short caption.")
        with self.assertRaises(ValueError):
            draw.caption(img, " ".join(["overlong"] * 60))

    def test_board_fills_cells_at_the_threshold(self):
        img = draw.blank()
        cells = bytes([0xEF] * 199 + [0x7B]).hex()
        draw.board(img, cells, (0, 0, 100, 200))
        bottom_right = img.getpixel((95, 195))
        top_left = img.getpixel((5, 5))
        self.assertNotEqual(bottom_right, top_left)

    def test_empty_cells_are_0xef_and_everything_else_is_a_block(self):
        cells = bytes([0xEF] * 198 + [0x7B, 0x7C]).hex()
        self.assertEqual(draw.filled(cells), {198, 199})
        img = draw.blank()
        draw.board(img, cells, (0, 0, 100, 200))
        self.assertEqual(img.getpixel((5, 5)), draw.BG, "an empty cell stays background")

    def test_rain_is_deterministic(self):
        tiles = [Image.new("RGB", (16, 16), (255, 255, 255))] * 4
        a, b = draw.blank(), draw.blank()
        draw.rain(a, tiles, 1.5, seed=7)
        draw.rain(b, tiles, 1.5, seed=7)
        self.assertTrue(np.array_equal(np.asarray(a), np.asarray(b)))

    def test_bar_chart_draws_only_the_shown_rows(self):
        rows = [{"label": f"r{i}", "v": 0.1 * (i + 1)} for i in range(5)]
        a, b = draw.blank(), draw.blank()
        draw.bar_chart(a, (100, 100, 800, 400), rows, "v", "label", shown=2)
        draw.bar_chart(b, (100, 100, 800, 400), rows, "v", "label", shown=5)
        self.assertFalse(np.array_equal(np.asarray(a), np.asarray(b)))
