import json
import tempfile
import unittest
from pathlib import Path

import numpy as np

from reservoir_film.model import (
    Caption, Chapter, Context, Shot, Takes, caption_spans, check_captions, fill, reading_seconds,
)


def make_take(root, name, meta, streams):
    d = Path(root) / name
    d.mkdir(parents=True)
    counts = {}
    for stream, n in streams.items():
        frames = np.zeros((n, 240, 256, 3), dtype=np.uint8)
        for i in range(n):
            frames[i, 0, 0, 0] = i
        frames.tofile(d / f"{stream}.rgb")
        counts[stream] = n
    (d / "take.json").write_text(json.dumps({**meta, "streams": counts}))


class TakesTest(unittest.TestCase):
    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        make_take(self.dir.name, "walk", {"best": {"x": [10, 20]}, "rate": 0.25, "none": None}, {"smb": 3})
        self.takes = Takes(self.dir.name)

    def tearDown(self):
        self.dir.cleanup()

    def test_frames_clamp_past_the_end(self):
        f = self.takes["walk"].frames("smb")
        self.assertEqual(len(f), 3)
        self.assertEqual(int(f[1][0, 0, 0]), 1)
        self.assertEqual(int(f[99][0, 0, 0]), 2, "holds the last frame")
        self.assertEqual(int(f[-5][0, 0, 0]), 0)

    def test_a_missing_take_says_how_to_record_it(self):
        with self.assertRaisesRegex(FileNotFoundError, "--take nope"):
            self.takes["nope"]

    def test_fill_reads_paths_indexes_and_formats(self):
        numbers = {"test_onsets": 15}
        self.assertEqual(fill("x {walk.best.x.1}, {walk.rate:.0%}, {numbers.test_onsets}", self.takes, numbers),
                         "x 20, 25%, 15")

    def test_fill_names_a_missing_or_null_reference(self):
        with self.assertRaisesRegex(KeyError, "walk.best.y"):
            fill("{walk.best.y}", self.takes, {})
        with self.assertRaisesRegex(KeyError, "walk.none"):
            fill("{walk.none}", self.takes, {})


class CaptionsTest(unittest.TestCase):
    def test_reading_time(self):
        self.assertEqual(reading_seconds(""), 0)
        self.assertEqual(reading_seconds("Three short words"), 3.0)
        self.assertAlmostEqual(reading_seconds(" ".join(["w"] * 30)), 8.0)

    def test_spans_and_the_timing_check(self):
        ctx = Context(takes=None, numbers={})
        shot = Shot(name="s", seconds=10, draw=None, captions=[
            Caption(0, "A caption of exactly six words here."), Caption(2, "Too soon for the first one."),
            Caption(6, lambda c: "Computed text."),
        ])
        spans = caption_spans(shot, ctx)
        self.assertEqual(spans, [(0, 2, "A caption of exactly six words here."), (2, 6, "Too soon for the first one."),
                                 (6, 10, "Computed text.")])
        problems = check_captions(Chapter(1, "T", "", [shot]), ctx)
        self.assertEqual(len(problems), 1)
        self.assertIn("A caption of exactly six words", problems[0])
