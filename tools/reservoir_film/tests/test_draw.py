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
        tiles = [[Image.new("RGB", (16, 16), (255, 255, 255))] * 4]
        a, b = draw.blank(), draw.blank()
        draw.rain(a, tiles, 1.5, seed=7)
        draw.rain(b, tiles, 1.5, seed=7)
        self.assertTrue(np.array_equal(np.asarray(a), np.asarray(b)))

    def test_glyph_tiles_wear_smb_palettes_not_matrix_green(self):
        # Eleven screens; each ones digit uses the game's four shades.
        shades = [(0, 0, 0), (0, 102, 120), (76, 154, 236), (236, 238, 236)]
        screens = np.zeros((11, 240, 256, 3), dtype=np.uint8)
        for k in range(4):
            screens[10, 0:8, 2 * k:2 * k + 2] = shades[k]
        sets = draw.glyph_tiles(screens, (0, 0, 8, 8))
        self.assertEqual(len(sets), len(draw.SMB_PALETTES))
        for palette, tiles in zip(draw.SMB_PALETTES, sets):
            used = set(map(tuple, np.asarray(tiles[0]).reshape(-1, 3)))
            self.assertLessEqual(used, {draw.BG, *palette})
            self.assertNotIn(draw.MATRIX, used)

    def test_rain_gives_each_column_one_palette(self):
        red = [Image.new("RGB", (16, 16), (255, 0, 0))]
        blue = [Image.new("RGB", (16, 16), (0, 0, 255))]
        img = draw.blank()
        draw.rain(img, [red, blue], 1.5, seed=7)
        a = np.asarray(img)
        for c in range(W // 16):
            col = a[:, c * 16:(c + 1) * 16].reshape(-1, 3)
            reds, blues = (col[:, 0] > 100).any(), (col[:, 2] > 100).any()
            self.assertFalse(reds and blues, f"column {c} mixes palettes")

    def test_bar_chart_draws_only_the_shown_rows(self):
        rows = [{"label": f"r{i}", "v": 0.1 * (i + 1)} for i in range(5)]
        a, b = draw.blank(), draw.blank()
        draw.bar_chart(a, (100, 100, 800, 400), rows, "v", "label", shown=2)
        draw.bar_chart(b, (100, 100, 800, 400), rows, "v", "label", shown=5)
        self.assertFalse(np.array_equal(np.asarray(a), np.asarray(b)))
