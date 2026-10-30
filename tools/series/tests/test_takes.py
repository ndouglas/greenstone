import tempfile
import tomllib
import unittest
from pathlib import Path

import numpy as np
import soundfile

from series.takes import PAD, TakeStore, trim

RATE = 48000


def speech(at=1.0, length=0.5, total=2.5, amplitude=0.3, rate=RATE):
    """Silence with a 200 Hz tone standing in for a spoken line."""
    audio = np.zeros(int(total * rate), dtype=np.float32)
    t = np.arange(int(length * rate)) / rate
    start = int(at * rate)
    audio[start:start + len(t)] = amplitude * np.sin(2 * np.pi * 200 * t)
    return audio


def store():
    return TakeStore(Path(tempfile.mkdtemp()) / "takes")


class Trim(unittest.TestCase):
    def test_trim_finds_the_speech_with_a_pad(self):
        start, end = trim(speech(at=1.0, length=0.5), RATE)
        self.assertAlmostEqual(start, 1.0 - PAD, delta=0.03)
        self.assertAlmostEqual(end, 1.5 + PAD, delta=0.03)

    def test_trim_clamps_to_the_file(self):
        start, _ = trim(speech(at=0.05), RATE)
        self.assertEqual(start, 0.0)

    def test_room_noise_with_bumps_is_not_speech(self):
        # Like a real "nobody speaking" take: room noise around -40 dB RMS,
        # with a few louder bumps, but nothing held at speaking level.
        rng = np.random.default_rng(1)
        audio = (rng.standard_normal(RATE * 2) * 10 ** (-40 / 20)).astype(np.float32)
        for at in (0.3, 0.9, 1.4):
            audio[int(at * RATE):int(at * RATE) + 960] *= 3.5  # 20 ms bumps near -31 dB
        self.assertIsNone(trim(audio, RATE))

    def test_key_clicks_at_the_ends_are_not_speech(self):
        audio = speech(at=1.0, length=0.5, total=2.5)
        click = np.hanning(240).astype(np.float32) * 0.5
        audio[int(0.05 * RATE):int(0.05 * RATE) + 240] += click  # the start key, released
        audio[-int(0.08 * RATE):-int(0.08 * RATE) + 240] += click  # the stop key, pressed
        start, end = trim(audio, RATE)
        self.assertAlmostEqual(start, 1.0 - PAD, delta=0.03)
        self.assertAlmostEqual(end, 1.5 + PAD, delta=0.03)

    def test_trim_of_silence_is_none(self):
        self.assertIsNone(trim(np.zeros(RATE, dtype=np.float32), RATE))
        hiss = (np.random.default_rng(0).standard_normal(RATE) * 10 ** (-60 / 20)).astype(np.float32)
        self.assertIsNone(trim(hiss, RATE))


class Store(unittest.TestCase):
    def test_takes_are_numbered_and_kept(self):
        s = store()
        for _ in range(3):
            s.add("a", speech(), RATE, "Hello.")
        self.assertEqual([t.number for t in s.takes("a")], [1, 2, 3])
        for n in (1, 2, 3):
            path = s.dir / "raw" / "a" / f"{n}.wav"
            self.assertTrue(path.exists())
            self.assertEqual(soundfile.info(path).subtype, "PCM_24")
        self.assertEqual(s.chosen("a").number, 3)

    def test_choose_an_earlier_take(self):
        s = store()
        s.add("a", speech(), RATE, "Hello.")
        s.add("a", speech(), RATE, "Hello.")
        s.choose("a", 1)
        self.assertEqual(s.chosen("a").number, 1)
        toml = tomllib.loads((s.dir / "takes.toml").read_text())
        self.assertEqual(toml["a"], {"text": "Hello.", "take": 1})

    def test_staleness_follows_the_chosen_take(self):
        s = store()
        s.add("a", speech(), RATE, "Old.")
        s.add("a", speech(), RATE, "New.")
        self.assertEqual(s.chosen("a").text, "New.")
        s.choose("a", 1)
        self.assertEqual(s.chosen("a").text, "Old.")

    def test_a_silent_take_is_kept_but_not_chosen(self):
        s = store()
        s.add("a", speech(), RATE, "Hello.")
        s.add("a", np.zeros(RATE, dtype=np.float32), RATE, "Hello.")
        takes = s.takes("a")
        self.assertEqual(len(takes), 2)
        self.assertEqual(s.chosen("a").number, 1)
        self.assertTrue(s.silent(takes[1]))
        with self.assertRaises(ValueError):
            s.choose("a", 2)
        with self.assertRaises(ValueError):
            s.choose("a", 9)

    def test_the_trim_is_stored_not_applied(self):
        s = store()
        take = s.add("a", speech(at=1.0, length=0.5), RATE, "Hello.")
        self.assertAlmostEqual(take.start, 1.0 - PAD, delta=0.03)
        self.assertAlmostEqual(take.seconds, 0.5 + 2 * PAD, delta=0.06)
        self.assertAlmostEqual(soundfile.info(take.path).duration, 2.5, delta=0.001)

    def test_text_with_quotes_survives(self):
        s = store()
        s.add("a", speech(), RATE, 'He said "100" \\ then left.')
        self.assertEqual(TakeStore(s.dir).chosen("a").text, 'He said "100" \\ then left.')

    def test_the_old_form_still_reads(self):
        s = store()
        s.dir.mkdir(parents=True)
        soundfile.write(s.dir / "a.wav", speech(total=2.0), RATE)
        (s.dir / "takes.toml").write_text('a = "Hello."\n')
        take = s.chosen("a")
        self.assertEqual((take.number, take.text, take.start), (0, "Hello.", 0.0))
        self.assertAlmostEqual(take.end, 2.0, delta=0.001)
        self.assertIsNone(s.chosen("b"))

    def test_room_tone_is_stored(self):
        s = store()
        self.assertIsNone(s.room())
        s.set_room(np.zeros(RATE * 10, dtype=np.float32), RATE)
        self.assertEqual(s.room(), s.dir / "room.wav")


if __name__ == "__main__":
    unittest.main()
