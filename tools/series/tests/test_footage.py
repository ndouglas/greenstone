import json
import tempfile
import unittest
from pathlib import Path

import numpy as np

from series.footage import GameFootage

H, W = 240, 256


def fake_take(root, name="clip"):
    d = root / name
    d.mkdir(parents=True)
    frames = np.zeros((2, H, W, 3), dtype=np.uint8)
    frames[1] = 200
    (d / "game.rgb").write_bytes(frames.tobytes())
    (d / "take.json").write_text(json.dumps({"streams": {"game": 2}}))


class Footage(unittest.TestCase):
    def test_footage_shows_the_frame_for_the_time(self):
        root = Path(tempfile.mkdtemp())
        fake_take(root)
        footage = GameFootage("clip", "game", root=root)
        self.assertEqual(int(footage.pixel_array[0, 0, 0]), 0)
        footage.update(1 / 60)
        self.assertEqual(int(footage.pixel_array[0, 0, 0]), 200)

    def test_missing_take_names_its_own_films_record_hint(self):
        with self.assertRaises(FileNotFoundError) as e:
            GameFootage("p9-missing", "punchout", root=Path(tempfile.mkdtemp()),
                        hint="cargo run --example punchout -- film-takes --only {name}")
        self.assertIn("--example punchout -- film-takes --only p9-missing", str(e.exception))

    def test_missing_take_names_the_record_hint(self):
        with self.assertRaises(FileNotFoundError) as e:
            GameFootage("nope", "game", root=Path(tempfile.mkdtemp()))
        self.assertIn("record it with", str(e.exception))


if __name__ == "__main__":
    unittest.main()
