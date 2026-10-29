import contextlib
import io
import json
import re
import subprocess
import tempfile
import unittest
from pathlib import Path

import numpy as np
import soundfile

from series.build import CREDITS_SECONDS, build
from series.voice import TAKES

RATE = 24000
SCRIPT = '''
title = "t"
{music}
[[line]]
id = "one"
text = "The first line."
framing = true
[[line]]
id = "two"
text = "The second line."
framing = true
'''
SCENE = '''
from series.scene import NarratedScene


class VideoScene(NarratedScene):
    def construct(self):
        self.say("one")
        self.say("two")
        self.roll_credits()
'''


def tone(path, hz, seconds, amplitude=0.5):
    t = np.arange(int(seconds * RATE)) / RATE
    soundfile.write(path, (amplitude * np.sin(2 * np.pi * hz * t)).astype(np.float32), RATE)


class ToneEngine:
    """Narrates every line as a 440 Hz tone, a second per 20 characters."""

    name = "tone"

    def synthesize(self, text, out):
        out.parent.mkdir(parents=True, exist_ok=True)
        tone(out, 440, len(text) / 20)


def videos(music="", scene=True):
    root = Path(tempfile.mkdtemp())
    d = root / "001-test"
    d.mkdir()
    (d / "script.toml").write_text(SCRIPT.format(music=music))
    if scene:
        (d / "scene.py").write_text(SCENE)
    return root, d


def duration(path):
    out = subprocess.run(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "json", str(path)],
                         capture_output=True, text=True, check=True).stdout
    return float(json.loads(out)["format"]["duration"])


def band_level(path, hz):
    """Mean volume, in dB, of the audio around `hz`."""
    err = subprocess.run(["ffmpeg", "-i", str(path), "-af", f"bandpass=f={hz}:width_type=h:w=100,volumedetect",
                          "-f", "null", "-"], capture_output=True, text=True).stderr
    return float(re.search(r"mean_volume: (-?[\d.]+) dB", err).group(1))


class Build(unittest.TestCase):
    def test_build_reports_stale_takes(self):
        root, d = videos()
        (d / TAKES).mkdir()
        tone(d / TAKES / "one.wav", 440, 1.0)
        (d / TAKES / "takes.toml").write_text('one = "An older first line."\n')
        out = Path(tempfile.mkdtemp())
        report = io.StringIO()
        with contextlib.redirect_stdout(report):
            movie = build("001-test", ToneEngine(), draft=True, videos=root, out_root=out)
        self.assertTrue(movie.exists())
        self.assertIn("one", report.getvalue())
        self.assertIn("stale", report.getvalue().lower())
        self.assertTrue((movie.parent / "description.txt").exists())

    def test_storyboard_needs_no_scene_file(self):
        root, d = videos(music='music = "MUSIC"', scene=False)
        music = d / "music.wav"
        tone(music, 1000, 20.0)
        (d / "script.toml").write_text((d / "script.toml").read_text().replace('"MUSIC"', f'"{music}"'))
        out = Path(tempfile.mkdtemp())
        with contextlib.redirect_stdout(io.StringIO()):
            movie = build("001-test", ToneEngine(), draft=True, storyboard=True, videos=root, out_root=out)
        self.assertEqual(movie.name, "001-test-storyboard.mp4")
        lines = len("The first line.") / 20 + len("The second line.") / 20
        self.assertAlmostEqual(duration(movie), lines + 2 * 0.25 + CREDITS_SECONDS, delta=0.15)

    def test_music_is_mixed_under_the_narration(self):
        root, d = videos(music='music = "MUSIC"')
        music = d / "music.wav"
        tone(music, 1000, 20.0)
        (d / "script.toml").write_text((d / "script.toml").read_text().replace('"MUSIC"', f'"{music}"'))
        out = Path(tempfile.mkdtemp())
        with contextlib.redirect_stdout(io.StringIO()):
            movie = build("001-test", ToneEngine(), draft=True, videos=root, out_root=out)
        narration, bed = band_level(movie, 440), band_level(movie, 1000)
        self.assertLess(bed, narration - 10)
        self.assertGreater(bed, narration - 40)


if __name__ == "__main__":
    unittest.main()
