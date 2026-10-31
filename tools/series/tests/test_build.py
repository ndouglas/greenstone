import contextlib
import io
import json
import re
import subprocess
import tempfile
import unittest
from unittest import mock
from pathlib import Path

import numpy as np
import soundfile

from series import pacing
from series.build import NotNarrated, SectionTooLong, build, mix, room_tone
from series.credits import credits_seconds
from series.pacing import STILLNESS, TITLE_HOLD
from series.script import load_video
from series.takes import TakeStore
from series.tests.test_takes import speech
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


def tone(path, hz, seconds, amplitude=0.5, subtype=None):
    t = np.arange(int(seconds * RATE)) / RATE
    soundfile.write(path, (amplitude * np.sin(2 * np.pi * hz * t)).astype(np.float32), RATE, subtype=subtype)


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


def band_level(path, hz, start=0.0, length=None):
    """Mean volume, in dB, of the audio around `hz` (optionally of a window)."""
    trim = f"atrim=start={start}" + (f":duration={length}" if length else "")
    err = subprocess.run(["ffmpeg", "-i", str(path), "-vn",
                          "-af", f"{trim},bandpass=f={hz}:width_type=h:w=100,volumedetect",
                          "-f", "null", "-"], capture_output=True, text=True).stderr
    return float(re.search(r"mean_volume: (-?(?:[\d.]+|inf)) dB", err).group(1))  # silence is -inf


def sectioned(chapter_text):
    def beat(id, text, role=None):
        r = f'role = "{role}"\n' if role else ""
        return f'[[section.line]]\nid = "{id}"\n{r}text = "{text}"\nboard = "a picture"\nframing = true\n'
    return ('title = "t"\n[[section]]\nkind = "introduction"\ntitle = "Introduction"\n' + beat("hi", "Hi.") +
            '[[section]]\nkind = "chapter"\ntitle = "Too long"\n' + beat("in", chapter_text, "intro") +
            beat("out", "Done.", "conclusion") +
            '[[section]]\nkind = "conclusions"\ntitle = "Conclusions"\n' + beat("bye", "Bye."))


SECTIONED_SCENE = '''
from series.scene import NarratedScene


class VideoScene(NarratedScene):
    def construct(self):
        self.show_title("introduction")
        self.say("hi")
        self.show_title("chapter-1")
        self.say("in")
        self.say("out")
        self.show_title("conclusions")
        self.say("bye")
        self.roll_credits()
'''


def record(d, line_id, text):
    TakeStore(d / TAKES).add(line_id, speech(at=0.2, length=0.5, total=1.0), 48000, text)


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

    def test_a_missing_videoscene_fails_before_any_synthesis(self):
        root, d = videos()
        (d / "scene.py").write_text("from series.scene import NarratedScene\n\nclass Other(NarratedScene):\n    pass\n")
        engine = ToneEngine()
        engine.synthesize = lambda text, out: self.fail("synthesized before finding the scene")
        with self.assertRaises(LookupError) as e:
            build("001-test", engine, draft=True, videos=root, out_root=Path(tempfile.mkdtemp()))
        self.assertIn("VideoScene", str(e.exception))

    def test_takes_are_reported_even_if_the_render_fails(self):
        root, d = videos()
        (d / "scene.py").write_text(SCENE.replace('self.say("one")', 'raise RuntimeError("boom")'))
        (d / TAKES).mkdir()
        tone(d / TAKES / "one.wav", 440, 1.0)
        (d / TAKES / "takes.toml").write_text('one = "An older first line."\n')
        report = io.StringIO()
        with contextlib.redirect_stdout(report), self.assertRaises(RuntimeError):
            build("001-test", ToneEngine(), draft=True, videos=root, out_root=Path(tempfile.mkdtemp()))
        self.assertIn("STALE", report.getvalue())

    def test_a_scene_that_never_rolls_credits_fails(self):
        root, d = videos()
        (d / "scene.py").write_text(SCENE.replace("self.roll_credits()", "pass"))
        with contextlib.redirect_stdout(io.StringIO()), self.assertRaises(Exception) as e:
            build("001-test", ToneEngine(), draft=True, videos=root, out_root=Path(tempfile.mkdtemp()))
        self.assertIn("credits", str(e.exception))

    def test_the_music_bed_loops_and_fades_out(self):
        root, d = videos(music='music = "MUSIC"')
        music = d / "music.wav"
        tone(music, 1000, 1.0)  # far shorter than the video
        (d / "script.toml").write_text((d / "script.toml").read_text().replace('"MUSIC"', f'"{music}"'))
        with contextlib.redirect_stdout(io.StringIO()):
            movie = build("001-test", ToneEngine(), draft=True, videos=root, out_root=Path(tempfile.mkdtemp()))
        end = duration(movie)
        self.assertGreater(band_level(movie, 1000, start=3.0, length=2.0), -45)
        self.assertLess(band_level(movie, 1000, start=end - 0.2, length=0.2),
                        band_level(movie, 1000, start=3.0, length=2.0) - 10)

    def test_24_bit_music_and_room_tone_are_mixed(self):
        # ffmpeg 9 sniffs some quiet 24-bit WAVs as MPEG-TS unless told they are WAV.
        root, d = videos()
        with contextlib.redirect_stdout(io.StringIO()):
            movie = build("001-test", ToneEngine(), draft=True, videos=root, out_root=Path(tempfile.mkdtemp()))
        folder = Path(tempfile.mkdtemp())
        music, room = folder / "music.wav", folder / "room.wav"
        for path, hz in ((music, 3000), (room, 3100)):
            tone(path, hz, 1.0, amplitude=0.01, subtype="PCM_24")
        mixed = folder / "mixed.mp4"
        mix(movie, music, mixed, room=room)
        self.assertAlmostEqual(duration(mixed), duration(movie), delta=0.2)

    def test_the_mix_does_not_clip(self):
        root, d = videos(music='music = "MUSIC"')
        music = d / "music.wav"
        tone(music, 1000, 20.0, amplitude=0.99)
        (d / "script.toml").write_text((d / "script.toml").read_text().replace('"MUSIC"', f'"{music}"'))
        loud = ToneEngine()
        loud.synthesize = lambda text, out: (out.parent.mkdir(parents=True, exist_ok=True),
                                             tone(out, 440, len(text) / 20, amplitude=0.99))
        with contextlib.redirect_stdout(io.StringIO()):
            movie = build("001-test", loud, draft=True, videos=root, out_root=Path(tempfile.mkdtemp()))
        err = subprocess.run(["ffmpeg", "-i", str(movie), "-af", "volumedetect", "-f", "null", "-"],
                             capture_output=True, text=True).stderr
        self.assertLess(float(re.search(r"max_volume: (-?[\d.]+) dB", err).group(1)), 0.0)

    def test_missing_music_fails_before_any_work(self):
        root, d = videos(music='music = "/nowhere/music.wav"')
        engine = ToneEngine()
        engine.synthesize = lambda text, out: self.fail("synthesized before checking the music")
        with self.assertRaises(FileNotFoundError) as e:
            build("001-test", engine, draft=True, videos=root, out_root=Path(tempfile.mkdtemp()))
        self.assertIn("/nowhere/music.wav", str(e.exception))

    def test_half_registered_takes_are_printed(self):
        root, d = videos()
        (d / TAKES).mkdir()
        tone(d / TAKES / "one.wav", 440, 1.0)
        report = io.StringIO()
        with contextlib.redirect_stdout(report):
            build("001-test", ToneEngine(), draft=True, videos=root, out_root=Path(tempfile.mkdtemp()))
        self.assertIn("one.wav", report.getvalue())

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
        self.assertAlmostEqual(duration(movie), lines + 2 * STILLNESS + credits_seconds(load_video(d)), delta=0.15)

    def test_release_refuses_beats_in_the_narrators_voice(self):
        root, d = videos()
        record(d, "one", "The first line.")
        out = Path(tempfile.mkdtemp())
        with contextlib.redirect_stdout(io.StringIO()), self.assertRaises(NotNarrated) as e:
            build("001-test", ToneEngine(), draft=True, release=True, videos=root, out_root=out)
        self.assertIn("two", str(e.exception))
        self.assertNotIn("one", str(e.exception).replace("1 of", ""))
        self.assertFalse((out / "001-test" / "media").exists())

    def test_release_refuses_a_stale_take(self):
        root, d = videos()
        record(d, "one", "The first line.")
        record(d, "two", "An older second line.")
        with contextlib.redirect_stdout(io.StringIO()), self.assertRaises(NotNarrated) as e:
            build("001-test", ToneEngine(), draft=True, release=True, videos=root, out_root=Path(tempfile.mkdtemp()))
        self.assertIn("two", str(e.exception))

    def test_release_builds_when_every_beat_is_a_take(self):
        root, d = videos()
        record(d, "one", "The first line.")
        record(d, "two", "The second line.")
        with contextlib.redirect_stdout(io.StringIO()):
            movie = build("001-test", ToneEngine(), draft=True, release=True, videos=root,
                          out_root=Path(tempfile.mkdtemp()))
        self.assertTrue(movie.exists())

    def test_the_build_counts_the_narrators_beats(self):
        root, d = videos()
        record(d, "one", "The first line.")
        report = io.StringIO()
        with contextlib.redirect_stdout(report):
            build("001-test", ToneEngine(), draft=True, videos=root, out_root=Path(tempfile.mkdtemp()))
        self.assertIn("1 of 2 beats narrated by Kokoro", report.getvalue())

    def test_room_tone_plays_only_under_takes(self):
        def room_level(recorded):
            root, d = videos()
            for line_id, text in recorded:
                record(d, line_id, text)
            store = TakeStore(d / TAKES)
            t = np.arange(48000 * 10) / 48000
            store.set_room((0.01 * np.sin(2 * np.pi * 3000 * t)).astype(np.float32), 48000)
            with contextlib.redirect_stdout(io.StringIO()):
                movie = build("001-test", ToneEngine(), draft=True, videos=root, out_root=Path(tempfile.mkdtemp()))
            return movie, band_level(movie, 3000, 1.3, 0.8)  # in the stillness after the first line

        movie, pause = room_level([("one", "The first line."), ("two", "The second line.")])
        self.assertGreater(pause, -75)
        self.assertLess(band_level(movie, 3000, duration(movie) - 3.0, 2.0), pause - 20)  # the credits
        _, mixed = room_level([("one", "The first line.")])
        self.assertLess(mixed, pause - 20)

    def test_no_room_tone_without_beats(self):
        root, d = videos()
        TakeStore(d / TAKES).set_room(np.zeros(48000, dtype=np.float32), 48000)
        self.assertIsNone(room_tone(load_video(d), {}, Path(tempfile.mkdtemp())))

    def test_a_section_over_the_limit_stops_the_build(self):
        root, d = videos(scene=False)
        (d / "script.toml").write_text(sectioned("x" * 5000))  # ToneEngine: 250 s
        out = Path(tempfile.mkdtemp())
        with contextlib.redirect_stdout(io.StringIO()):
            with self.assertRaises(SectionTooLong) as e:
                build("001-test", ToneEngine(), draft=True, storyboard=True, videos=root, out_root=out)
        self.assertIn("chapter-1", str(e.exception))
        self.assertFalse((out / "001-test" / "media").exists())

    def test_a_scene_that_runs_a_section_long_is_caught(self):
        # The estimate counts only voices, stillness and the title; a scene's
        # own waits are measured as it renders.
        root, d = videos(scene=False)
        (d / "script.toml").write_text(sectioned("Short."))
        (d / "scene.py").write_text(SECTIONED_SCENE.replace('self.say("in")', 'self.say("in")\n        self.wait(3)'))
        report = io.StringIO()
        with mock.patch.object(pacing, "SECTION_LIMIT", 10.0), contextlib.redirect_stdout(report):
            with self.assertRaises(SectionTooLong) as e:
                build("001-test", ToneEngine(), draft=True, videos=root, out_root=Path(tempfile.mkdtemp()))
        self.assertIn("chapter-1", str(e.exception))
        self.assertIn("as rendered", str(e.exception))

    def test_music_is_mixed_under_the_narration(self):
        root, d = videos(music='music = "MUSIC"')
        music = d / "music.wav"
        tone(music, 1000, 20.0)
        (d / "script.toml").write_text((d / "script.toml").read_text().replace('"MUSIC"', f'"{music}"'))
        out = Path(tempfile.mkdtemp())
        with contextlib.redirect_stdout(io.StringIO()):
            movie = build("001-test", ToneEngine(), draft=True, videos=root, out_root=out)
        # While the first line is spoken (the stillness after it would dilute the narration's mean).
        speaking = len("The first line.") / 20
        narration, bed = band_level(movie, 440, 0, speaking), band_level(movie, 1000, 0, speaking)
        self.assertLess(bed, narration - 10)
        self.assertGreater(bed, narration - 40)


if __name__ == "__main__":
    unittest.main()


class Only(unittest.TestCase):
    def test_one_section_renders_alone_under_its_own_name(self):
        root, d = videos(scene=False)
        (d / "script.toml").write_text(sectioned("Short."))
        out = Path(tempfile.mkdtemp())
        with contextlib.redirect_stdout(io.StringIO()):
            movie = build("001-test", ToneEngine(), draft=True, storyboard=True, only="chapter-1", videos=root,
                          out_root=out)
        self.assertEqual(movie.name, "001-test-chapter-1-storyboard.mp4")
        # Its title, then "Short." and "Done." (ToneEngine: 1/20 s a character), each with its stillness; no credits.
        self.assertAlmostEqual(duration(movie), TITLE_HOLD + (6 + 5) / 20 + 2 * STILLNESS, delta=0.15)
