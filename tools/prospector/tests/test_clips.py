import json
import shutil
import tempfile
import unittest
from pathlib import Path

import numpy as np

from prospector import clips
from reservoir_film.model import Take


def fake_take(root, frames=3):
    d = Path(root) / "A68E"
    d.mkdir(parents=True)
    (d / "smb.rgb").write_bytes(np.zeros((frames, 240, 256, 3), np.uint8).tobytes())
    meta = {"rank": 1, "offset": 0xA68E, "hex": "$A68E", "clear": "distance", "bottleneck": 2,
            "kill_rate": 0.81, "rounds": 97, "x_reached": 3080, "narrower_than": ["1-1"],
            "intro_frames": 1, "streams": {"smb": frames}}
    (d / "take.json").write_text(json.dumps(meta))
    return Take(d)


class Clips(unittest.TestCase):
    def test_compose_is_frame_sized(self):
        with tempfile.TemporaryDirectory() as root:
            img = clips.compose(fake_take(root), 1)
            self.assertEqual(img.size, (1280, 720))

    def test_caption_names_the_offset_and_comparison(self):
        with tempfile.TemporaryDirectory() as root:
            lines = clips.caption_lines(fake_take(root).meta)
            self.assertIn("$A68E", lines[0])
            self.assertTrue(any("narrower than 1-1" in l for l in lines))

    @unittest.skipUnless(shutil.which("ffmpeg"), "needs ffmpeg")
    def test_encode_writes_an_mp4(self):
        with tempfile.TemporaryDirectory() as root:
            out = Path(root) / "clip.mp4"
            clips.encode(fake_take(root), out)
            self.assertGreater(out.stat().st_size, 0)


if __name__ == "__main__":
    unittest.main()
