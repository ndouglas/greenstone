import tempfile
import unittest
from pathlib import Path

import numpy as np

from series.clips import CLIP_SECONDS, FADE, Capture, ClipTile, load_clip, save_clip

H, W = 36, 64


def frame(value):
    return np.full((H * 4, W * 4, 4), value, np.uint8)


class TestCapture(unittest.TestCase):
    def test_keeps_the_busiest_stretch(self):
        capture = Capture(fps=60)
        for _ in range(240):  # 4 s still
            capture.add(frame(10))
        for i in range(120):  # 2 s of change
            capture.add(frame(20 + i))
        capture.add(frame(139), repeat=240)  # 4 s still again, where it ended
        frames = capture.frames()
        self.assertEqual(len(frames), round(CLIP_SECONDS * capture.rate))
        levels = [int(f[0, 0, 0]) for f in frames]
        self.assertGreater(min(levels), 10)  # none of the stillness before the change
        self.assertTrue(any(20 <= level < 139 for level in levels))  # the change itself
        self.assertEqual(frames[0].shape, (H, W, 3))  # a quarter of each side, without alpha

    def test_keeps_every_other_frame_at_sixty(self):
        capture = Capture(fps=60)
        self.assertEqual(capture.rate, 30)
        for i in range(10):
            capture.add(frame(i * 20))
        self.assertEqual([int(f[0, 0, 0]) for f in capture.frames()], [0, 40, 80, 120, 160])

    def test_a_short_beat_keeps_all_it_has(self):
        capture = Capture(fps=60)
        capture.add(frame(10), repeat=4)
        self.assertEqual(len(capture.frames()), 2)

    def test_nothing_drawn_keeps_nothing(self):
        self.assertEqual(Capture(fps=60).frames(), [])


class TestSaveLoad(unittest.TestCase):
    def test_round_trip(self):
        path = Path(tempfile.mkdtemp()) / "a" / "b.mp4"
        frames = [np.full((H, W, 3), v, np.uint8) for v in (40, 120, 200)]
        save_clip(frames, 30, path)
        clip = load_clip(path)
        self.assertEqual(clip.fps, 30)
        self.assertEqual(len(clip.frames), 3)
        self.assertEqual(clip.frames[0].shape[:2], (H, W))
        self.assertAlmostEqual(float(clip.frames[1].mean()), 120, delta=6)

    def test_odd_sizes_are_trimmed_even(self):
        path = Path(tempfile.mkdtemp()) / "odd.mp4"
        save_clip([np.zeros((35, 63, 3), np.uint8)], 30, path)
        self.assertEqual(load_clip(path).frames[0].shape[:2], (34, 62))


class TestClipTile(unittest.TestCase):
    def clip(self, path=None):
        path = path or Path(tempfile.mkdtemp()) / "c.mp4"
        save_clip([np.full((H, W, 3), 200, np.uint8)] * 30, 30, path)
        return load_clip(path)

    def test_fades_through_black_at_the_seam(self):
        tile = ClipTile(self.clip())
        self.assertLess(tile.pixel_array[..., :3].mean(), 20)  # the loop's start
        tile.update(FADE * 2)
        self.assertGreater(tile.pixel_array[..., :3].mean(), 180)
        tile.update(1.0 - FADE * 2 - 0.01)  # just before the loop's end
        self.assertLess(tile.pixel_array[..., :3].mean(), 20)

    def test_a_phase_starts_it_partway(self):
        tile = ClipTile(self.clip(), phase=0.5)
        self.assertGreater(tile.pixel_array[..., :3].mean(), 180)

    def test_copies_share_the_frames_and_keep_playing(self):
        tile = ClipTile(self.clip())
        copy = tile.copy()
        self.assertIs(copy.clip.frames, tile.clip.frames)
        copy.update(0.5)
        self.assertGreater(copy.pixel_array[..., :3].mean(), 180)
        self.assertLess(tile.pixel_array[..., :3].mean(), 20)

    def test_keeps_its_size_as_it_plays(self):
        tile = ClipTile(self.clip()).scale(2)
        width = tile.width
        tile.update(0.5)
        self.assertAlmostEqual(tile.width, width)
