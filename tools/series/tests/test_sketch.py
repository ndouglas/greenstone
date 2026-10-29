import json
import subprocess
import tempfile
import unittest
from pathlib import Path

from PIL import Image

from series.sketch import STILL_WIDTH, sketch, still_times

SKETCH = '''
from manim import Dot, FadeIn, Scene


class Sketch(Scene):
    def construct(self):
        self.play(FadeIn(Dot()), run_time=1.0)
'''


def sketch_file(body=SKETCH):
    d = Path(tempfile.mkdtemp()) / "001-test" / "sketches"
    d.mkdir(parents=True)
    path = d / "dot.py"
    path.write_text(body)
    return path


def probe(path):
    out = subprocess.run(["ffprobe", "-v", "error", "-show_entries", "format=duration:stream=codec_type",
                          "-of", "json", str(path)], capture_output=True, text=True, check=True).stdout
    return json.loads(out)


class Sketches(unittest.TestCase):
    def test_a_sketch_renders_alone(self):
        movie, _ = sketch(sketch_file(), out_root=Path(tempfile.mkdtemp()))
        info = probe(movie)
        self.assertAlmostEqual(float(info["format"]["duration"]), 1.0, delta=0.1)
        self.assertEqual([s["codec_type"] for s in info["streams"]], ["video"])
        self.assertEqual(movie.name, "dot.mp4")
        self.assertEqual(movie.parent.name, "sketches")
        self.assertEqual(movie.parent.parent.name, "001-test")

    def test_the_contact_sheet_has_one_still_per_slot(self):
        movie, sheet = sketch(sketch_file(), stills=4, out_root=Path(tempfile.mkdtemp()))
        image = Image.open(sheet)
        still_height = round(STILL_WIDTH * 9 / 16)
        self.assertEqual(image.size, (2 * STILL_WIDTH, 2 * still_height))
        self.assertEqual(still_times(1.0, 4), [0.125, 0.375, 0.625, 0.875])

    def test_a_file_without_sketch_is_an_error(self):
        path = sketch_file("from manim import Scene\n\nclass Other(Scene):\n    pass\n")
        with self.assertRaises(LookupError) as e:
            sketch(path, out_root=Path(tempfile.mkdtemp()))
        self.assertIn("dot.py", str(e.exception))


if __name__ == "__main__":
    unittest.main()
