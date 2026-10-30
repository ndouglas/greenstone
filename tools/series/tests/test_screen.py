import json
import tempfile
import unittest
from pathlib import Path

import numpy as np
from manim import Mobject, Scene, config, tempconfig
from manim.constants import RESAMPLING_ALGORITHMS

from reservoir_film.model import NES_H, NES_W
from series.recording import Recording
from series.screen import Screen, rounded_mask, screen_glow, screen_overlay

NEAREST = RESAMPLING_ALGORITHMS["nearest"]
FULL_HD = {"pixel_width": 1920, "pixel_height": 1080, "frame_rate": 60}


def synthetic(root: Path, n: int = 120) -> Recording:
    """An smb1 recording of `n` frames: frame f is a checkerboard with f in its top-left pixel, and its RAM
    holds f in byte 0."""
    take = root / "smb1" / "footage"
    take.mkdir(parents=True)
    (take / "take.json").write_text(json.dumps({"streams": {"game": n}}))
    frames = np.zeros((n, NES_H, NES_W, 3), dtype=np.uint8)
    y, x = np.mgrid[0:NES_H, 0:NES_W]
    frames[:] = (((x + y) % 2) * 255)[None, :, :, None]
    frames[:, 0, 0, :] = np.arange(n)[:, None]
    frames.tofile(take / "game.rgb")
    (root / "smb1" / "1").mkdir()
    ram = np.zeros((n, 2048), dtype=np.uint8)
    ram[:, 0] = np.arange(n)
    ram.tofile(root / "smb1" / "1" / "ram.u8")
    return Recording("smb1", root)


def on_whole_pixel(x: float) -> bool:
    return abs(x - round(x)) < 1e-6


class TestScreen(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.recording = synthetic(Path(self.tmp.name))

    def tearDown(self):
        self.tmp.cleanup()

    def test_screens_snap_to_whole_pixels(self):
        with tempconfig(FULL_HD):
            px = config.frame_height / config.pixel_height
            screen = Screen(self.recording, 10)
            self.assertEqual(screen.scale, 3)
            game = screen.game
            self.assertAlmostEqual(game.width / px, 768)
            self.assertAlmostEqual(game.height / px, 720)
            left = (game.get_left()[0] + config.frame_width / 2) / px
            top = (config.frame_height / 2 - game.get_top()[1]) / px
            self.assertTrue(on_whole_pixel(left), left)
            self.assertTrue(on_whole_pixel(top), top)
            self.assertAlmostEqual(screen.dot, 3 * px)
            self.assertAlmostEqual(screen.row_y(0), game.get_top()[1])
            self.assertAlmostEqual(screen.row_y(240), game.get_bottom()[1])
            self.assertAlmostEqual(screen.col_x(0), game.get_left()[0])
            self.assertAlmostEqual(screen.col_x(256), game.get_right()[0])

            snapped = Screen(self.recording, 10, left_px=36.4)
            self.assertAlmostEqual((snapped.game.get_left()[0] + config.frame_width / 2) / px, 36)
            # The glow sits on whole pixels too, centred on the screen.
            glow_left = (snapped.glow.get_left()[0] + config.frame_width / 2) / px
            self.assertTrue(on_whole_pixel(glow_left), glow_left)

    def test_scanlines_only_where_they_fit(self):
        behind = np.array([3, 3, 3])
        dark = screen_overlay(768, 720, 3, behind)[:, 384, 3]
        self.assertTrue((dark[2:-198:3] > dark[0:-200:3]).all())
        self.assertTrue((dark[198:-201:3] == dark[199:-201:3]).all())
        flat = screen_overlay(256, 240, 1, behind)[100:140, 128, 3]
        self.assertTrue((flat == flat[0]).all())
        off = screen_overlay(768, 720, 3, behind, scanlines=False)[300:400, 384, 3]
        self.assertTrue((off == off[0]).all())
        with tempconfig(FULL_HD):
            self.assertFalse(Screen(self.recording, 0, scale=3, scanlines=False).scanlines)
            self.assertTrue(Screen(self.recording, 0, scale=3).scanlines)
            self.assertFalse(Screen(self.recording, 0, scale=1).scanlines)
            with self.assertRaises(ValueError):
                Screen(self.recording, 0, scale=1, scanlines=True)

    def test_small_screens_are_smoothed(self):
        with tempconfig(FULL_HD):
            small = Screen(self.recording, 0, scale=1 / 3)
            self.assertNotEqual(small.game.resampling_algorithm, NEAREST)
            # A checkerboard averaged down is grey, not black and white.
            middle = small.game.pixel_array[10:-10, 10:-10, 0]
            self.assertTrue(((middle > 64) & (middle < 192)).all())
            for scale in (1, 3):
                self.assertEqual(Screen(self.recording, 0, scale=scale).game.resampling_algorithm, NEAREST)

    def test_the_glow_has_no_edge_and_follows_the_corners(self):
        image, margin = screen_glow(np.array([200.0, 100.0, 50.0]), 768, 720, 3)
        self.assertEqual(margin, 4 * 14 * 3)
        self.assertEqual(image.shape[:2], (720 + 2 * margin, 768 + 2 * margin))
        alpha = image[:, :, 3].astype(int)
        for border in (alpha[0], alpha[-1], alpha[:, 0], alpha[:, -1]):
            self.assertTrue((border == 0).all())
        corner = alpha[margin, margin]
        edge = alpha[margin, margin + 384]
        self.assertLess(corner, edge)
        self.assertGreater(corner, 0)
        mask = rounded_mask(768, 720, 26)
        self.assertEqual(mask[0, 0], 0)
        self.assertEqual(mask[360, 384], 1)

    def test_the_cut_away_corners_are_the_glow_there(self):
        with tempconfig(FULL_HD):
            screen = Screen(self.recording, 0)
            glow = screen.glow.pixel_array.astype(float)
            margin = (glow.shape[0] - 720) // 2
            overlay = screen.overlay.pixel_array.astype(float)
            for y, x in ((0, 0), (2, 1), (719, 767)):
                a = glow[margin + y, margin + x, 3] / 255
                expected = np.array([3.0, 3.0, 3.0]) * (1 - a) + glow[margin + y, margin + x, :3] * a
                self.assertTrue(np.allclose(overlay[y, x, :3], expected, atol=1), (y, x, overlay[y, x], expected))
                self.assertEqual(overlay[y, x, 3], 255)

    def test_arrival_holds_the_first_frame(self):
        recording, seen = self.recording, []

        class Arrival(Scene):
            def construct(self):
                screen = Screen(recording, 5)
                watcher = Mobject()
                watcher.add_updater(lambda m: seen.append(screen.frame()))
                self.add(watcher)
                self.play(screen.arrive(run_time=0.8))
                self.wait(0.5)

        with tempfile.TemporaryDirectory() as media:
            with tempconfig({"pixel_width": 320, "pixel_height": 180, "frame_rate": 60, "media_dir": media,
                             "disable_caching": True, "verbosity": "ERROR", "progress_bar": "none"}):
                Arrival().render()
        arriving, after = seen[:48], seen[48:]
        self.assertEqual(set(arriving), {5})
        self.assertGreater(after[-1], 5 + 20)
        self.assertEqual(after, sorted(after))

    def test_the_ram_matches_the_picture(self):
        with tempconfig(FULL_HD):
            screen = Screen(self.recording, 40)
            screen.game.update(0.25)
            self.assertEqual(screen.frame(), 55)
            self.assertTrue(np.array_equal(screen.ram(), self.recording.ram_at(55)))
            self.assertEqual(screen.ram()[0], 54)


if __name__ == "__main__":
    unittest.main()
