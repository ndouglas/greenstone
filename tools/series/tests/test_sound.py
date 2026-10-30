import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import numpy as np
import soundfile

from series import sound
from series.sound import loudness, process, process_room
from series.takes import TakeStore
from series.tests.test_takes import speech


def take(amplitude=0.3, rate=48000, **kw):
    store = TakeStore(Path(tempfile.mkdtemp()) / "takes")
    return store.add("a", speech(amplitude=amplitude, rate=rate, **kw), rate, "Hello.")


def peak(path):
    return float(np.abs(soundfile.read(path)[0]).max())


class Chain(unittest.TestCase):
    def test_takes_come_out_at_the_same_loudness(self):
        cache = Path(tempfile.mkdtemp())
        for amplitude in (0.05, 0.5):
            out, gain = process(take(amplitude, length=1.5), cache)
            self.assertAlmostEqual(loudness(out), sound.TARGET_LUFS, delta=1.0)
        self.assertNotEqual(gain, 0.0)

    def test_the_chain_outputs_48k_from_any_rate(self):
        out, _ = process(take(rate=44100), Path(tempfile.mkdtemp()))
        self.assertEqual(soundfile.info(out).samplerate, 48000)
        self.assertEqual(soundfile.info(out).channels, 1)

    def test_the_trim_is_applied(self):
        t = take(at=1.0, length=0.5)
        out, _ = process(t, Path(tempfile.mkdtemp()))
        self.assertAlmostEqual(soundfile.info(out).duration, t.seconds, delta=0.05)

    def test_the_limiter_holds(self):
        out, _ = process(take(0.99), Path(tempfile.mkdtemp()))
        self.assertLessEqual(peak(out), 0.9)

    def test_processing_is_cached(self):
        t, cache = take(), Path(tempfile.mkdtemp())
        first = process(t, cache)
        with mock.patch.object(subprocess, "run", side_effect=AssertionError("ran ffmpeg again")):
            self.assertEqual(process(t, cache), first)
        with mock.patch.object(sound, "CHAIN_VERSION", sound.CHAIN_VERSION + 1):
            self.assertNotEqual(process(t, cache)[0], first[0])

    def test_room_tone_takes_the_given_gain(self):
        rng = np.random.default_rng(0)
        room = Path(tempfile.mkdtemp()) / "room.wav"
        soundfile.write(room, (rng.standard_normal(48000 * 10) * 10 ** (-50 / 20)).astype(np.float32), 48000)
        cache = Path(tempfile.mkdtemp())
        quiet, loud = process_room(room, 0.0, cache), process_room(room, 6.0, cache)
        self.assertAlmostEqual(loudness(loud) - loudness(quiet), 6.0, delta=1.5)
        self.assertEqual(soundfile.info(loud).samplerate, 48000)


if __name__ == "__main__":
    unittest.main()
