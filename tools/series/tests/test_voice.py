import tempfile
import unittest
from pathlib import Path

import numpy as np
import soundfile

from series.script import load_video
from importlib.metadata import version

from series.takes import TakeStore
from series.tests.test_takes import speech
from series.voice import TAKES, KokoroEngine, narrator, take_problems, voices

RATE = 24000


def silence(path, seconds):
    path.parent.mkdir(parents=True, exist_ok=True)
    soundfile.write(path, np.zeros(int(seconds * RATE), dtype=np.float32), RATE)


class FakeEngine:
    name = "fake"

    def __init__(self):
        self.calls = []

    def synthesize(self, text, out):
        self.calls.append(text)
        silence(out, len(text) / 20)


SCRIPT = '''
title = "t"
[[line]]
id = "one"
text = "The first line."
framing = true
[[line]]
id = "two"
text = "w·x + b"
spoken = "w times x, plus b"
framing = true
'''


def video(script=SCRIPT):
    d = Path(tempfile.mkdtemp()) / "001-test"
    d.mkdir()
    (d / "script.toml").write_text(script)
    return d


def take(dir, line_id, text, seconds):
    silence(dir / TAKES / f"{line_id}.wav", seconds)
    toml = dir / TAKES / "takes.toml"
    old = toml.read_text() if toml.exists() else ""
    toml.write_text(old + f'{line_id} = "{text}"\n')


class Voices(unittest.TestCase):
    def setUp(self):
        self.out = Path(tempfile.mkdtemp())

    def test_each_line_is_synthesized_once_and_cached(self):
        v, engine = load_video(video()), FakeEngine()
        first, stale = voices(v, engine, self.out)
        voices(v, engine, self.out)
        self.assertEqual(engine.calls, ["The first line.", "w times x, plus b"])
        self.assertEqual(stale, [])
        self.assertAlmostEqual(first["one"].seconds, len("The first line.") / 20, places=3)
        self.assertEqual(first["one"].source, "synth")

    def test_the_cache_is_keyed_by_the_voiced_text(self):
        d, engine = video(), FakeEngine()
        voices(load_video(d), engine, self.out)
        (d / "script.toml").write_text(SCRIPT.replace("w times x, plus b", "w times x plus b"))
        voices(load_video(d), engine, self.out)
        self.assertEqual(engine.calls[2:], ["w times x plus b"])

    def test_a_matching_take_wins(self):
        d = video()
        take(d, "one", "The first line.", 3.0)
        got, stale = voices(load_video(d), FakeEngine(), self.out)
        self.assertEqual(got["one"].source, "take")
        self.assertAlmostEqual(got["one"].seconds, 3.0, places=3)
        self.assertEqual(stale, [])

    def test_a_stale_take_falls_back_and_is_reported(self):
        d = video()
        take(d, "one", "An older first line.", 3.0)
        got, stale = voices(load_video(d), FakeEngine(), self.out)
        self.assertEqual(got["one"].source, "synth")
        self.assertEqual(stale, ["one"])

    def test_a_chosen_take_is_processed_and_used(self):
        d = video()
        TakeStore(d / TAKES).add("one", speech(), 48000, "The first line.")
        got, stale = voices(load_video(d), FakeEngine(), self.out)
        self.assertEqual(got["one"].source, "take")
        self.assertEqual(got["one"].path.parent, self.out / "takes")
        self.assertIsNotNone(got["one"].gain_db)
        self.assertEqual(soundfile.info(got["one"].path).samplerate, 48000)
        self.assertIsNone(got["two"].gain_db)

    def test_a_take_goes_stale_when_the_text_changes(self):
        d = video()
        TakeStore(d / TAKES).add("one", speech(), 48000, "Old.")
        got, stale = voices(load_video(d), FakeEngine(), self.out)
        self.assertEqual(got["one"].source, "synth")
        self.assertEqual(stale, ["one"])

    def test_the_narrator_is_kept_for_recorded_lines(self):
        d, engine = video(), FakeEngine()
        TakeStore(d / TAKES).add("one", speech(), 48000, "The first line.")
        v = load_video(d)
        voices(v, engine, self.out)
        self.assertTrue(narrator(v, engine, self.out)["one"].exists())
        voices(v, engine, self.out)
        self.assertEqual(engine.calls, ["The first line.", "w times x, plus b"])

    def test_raw_takes_for_unknown_ids_are_reported(self):
        d = video()
        TakeStore(d / TAKES).add("ghost", speech(), 48000, "Boo.")
        problems = take_problems(load_video(d))
        self.assertTrue(any("ghost" in p for p in problems), problems)

    def test_half_registered_takes_are_reported(self):
        d = video()
        silence(d / TAKES / "one.wav", 1.0)  # recorded, never listed
        (d / TAKES / "takes.toml").write_text('two = "w·x + b"\nghost = "Gone."\n')  # listed, no wav; unknown id
        problems = take_problems(load_video(d))
        self.assertEqual(len(problems), 3)
        for line_id in ("one", "two", "ghost"):
            self.assertTrue(any(line_id in p for p in problems), line_id)

    def test_a_clean_takes_folder_has_no_problems(self):
        d = video()
        take(d, "one", "The first line.", 1.0)
        self.assertEqual(take_problems(load_video(d)), [])

    def test_a_crash_mid_synthesis_leaves_nothing_cached(self):
        v, engine = load_video(video()), FakeEngine()
        real = engine.synthesize

        def crash(text, out):
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_bytes(b"RIFF half a file")
            raise OSError("disk full")

        engine.synthesize = crash
        with self.assertRaises(Exception):
            voices(v, engine, self.out)
        engine.synthesize = real
        got, _ = voices(v, engine, self.out)
        self.assertEqual(len(engine.calls), 2)
        self.assertAlmostEqual(got["one"].seconds, len("The first line.") / 20, places=3)

    def test_old_synthesis_is_pruned(self):
        d, engine = video(), FakeEngine()
        voices(load_video(d), engine, self.out)
        (d / "script.toml").write_text(SCRIPT.replace("The first line.", "The first line, edited."))
        voices(load_video(d), engine, self.out)
        self.assertEqual(len(list((self.out / "tts").glob("*.wav"))), 2)

    def test_a_failed_synthesis_names_the_line(self):
        engine = FakeEngine()

        def fail(text, out):
            raise ValueError("no phonemes")

        engine.synthesize = fail
        with self.assertRaises(RuntimeError) as e:
            voices(load_video(video()), engine, self.out)
        self.assertIn("one", str(e.exception))

    def test_kokoro_engine_names_its_version(self):
        self.assertIn(version("kokoro"), KokoroEngine("af_heart").name)
        self.assertIn("speed0.9", KokoroEngine().name)

    def test_the_speed_is_part_of_the_cache_key(self):
        self.assertNotEqual(KokoroEngine(speed=0.9).name, KokoroEngine(speed=1.0).name)

    def test_kokoro_says_when_there_is_nothing_to_pronounce(self):
        engine = KokoroEngine("af_heart")
        engine.pipeline = lambda text, voice, speed: iter([])
        with self.assertRaises(ValueError) as e:
            engine.synthesize("...", self.out / "x.wav")
        self.assertIn("nothing to pronounce", str(e.exception))

    def test_swapping_one_take_leaves_other_lines_cached(self):
        d, engine = video(), FakeEngine()
        voices(load_video(d), engine, self.out)
        take(d, "two", "w·x + b", 2.0)
        got, _ = voices(load_video(d), engine, self.out)
        self.assertEqual(len(engine.calls), 2)
        self.assertEqual(got["two"].source, "take")


if __name__ == "__main__":
    unittest.main()
