import json
import subprocess
import tempfile
import unittest
from pathlib import Path

import numpy as np
import soundfile
from manim import Dot, FadeIn, tempconfig

from series.scene import NarratedScene, ScriptOrderError
from series.script import load_video
from series.voice import Voice

RATE = 24000
SCRIPT = '''
title = "t"
[[line]]
id = "a"
text = "One."
framing = true
[[line]]
id = "b"
text = "Two."
framing = true
'''


def fixtures(lengths):
    d = Path(tempfile.mkdtemp())
    (d / "001-test").mkdir()
    (d / "001-test" / "script.toml").write_text(SCRIPT)
    voices = {}
    for line_id, seconds in lengths.items():
        path = d / f"{line_id}.wav"
        soundfile.write(path, np.zeros(int(seconds * RATE), dtype=np.float32), RATE)
        voices[line_id] = Voice(line_id, path, seconds, "synth")
    return load_video(d / "001-test"), voices, d


def render(cls, media):
    # 60 fps, as the videos are, so pads land on whole frames; tiny frames for speed.
    with tempconfig({"quality": "low_quality", "frame_rate": 60, "pixel_width": 160, "pixel_height": 90,
                     "media_dir": str(media), "disable_caching": True,
                     "verbosity": "ERROR", "progress_bar": "none"}):
        scene = cls()
        scene.render()
        return Path(scene._get_manager().file_writer.movie_file_path)


def probe(path, entries):
    out = subprocess.run(["ffprobe", "-v", "error", "-show_entries", entries, "-of", "json", str(path)],
                         capture_output=True, text=True, check=True).stdout
    return json.loads(out)


class Narration(unittest.TestCase):
    def test_a_line_lasts_as_long_as_its_voice(self):
        video, voices, d = fixtures({"a": 1.0, "b": 2.0})

        class S(NarratedScene):
            def construct(self):
                self.say("a")
                self.say("b")

        S.video, S.voices = video, voices
        movie = render(S, d / "media")
        info = probe(movie, "format=duration:stream=codec_type")
        self.assertAlmostEqual(float(info["format"]["duration"]), 3.5, delta=0.1)
        self.assertIn("audio", [s["codec_type"] for s in info["streams"]])

    def test_animations_are_stretched_to_the_line(self):
        video, voices, d = fixtures({"a": 2.0, "b": 0.5})
        times = []

        class S(NarratedScene):
            def construct(self):
                self.say("a", FadeIn(Dot()))
                times.append(self.time)
                self.say("b")

        S.video, S.voices = video, voices
        render(S, d / "media")
        self.assertAlmostEqual(times[0], 2.25, delta=0.1)

    def test_an_unsaid_line_fails_the_render(self):
        video, voices, d = fixtures({"a": 0.5, "b": 0.5})

        class S(NarratedScene):
            def construct(self):
                self.say("a")

        S.video, S.voices = video, voices
        with self.assertRaises(ScriptOrderError) as e:
            render(S, d / "media")
        self.assertIn("b", str(e.exception))

    def test_lines_out_of_order_or_twice_fail(self):
        video, voices, d = fixtures({"a": 0.5, "b": 0.5})

        class Backwards(NarratedScene):
            def construct(self):
                self.say("b")
                self.say("a")

        class Twice(NarratedScene):
            def construct(self):
                self.say("a")
                self.say("a")
                self.say("b")

        for cls in (Backwards, Twice):
            cls.video, cls.voices = video, voices
            with self.assertRaises(ScriptOrderError):
                render(cls, d / "media")


if __name__ == "__main__":
    unittest.main()
