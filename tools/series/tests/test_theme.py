import tempfile
import unittest
from pathlib import Path

import numpy as np
import soundfile

from series import chiptune, theme
from series.opening import OPENING_SECONDS


class ChiptuneTest(unittest.TestCase):
    def test_pitches_are_the_chips(self):
        # A4 on a pulse: period register 253, so 1789773 / (16 × 254), a little sharp of 440.
        self.assertAlmostEqual(chiptune.pulse_hz(440.0), 1789773 / (16 * 254), places=6)
        self.assertAlmostEqual(chiptune.triangle_hz(110.0), 1789773 / (32 * 508), places=6)
        self.assertAlmostEqual(chiptune.hz("A4"), 440.0)
        self.assertAlmostEqual(chiptune.hz("A2"), 110.0)
        self.assertAlmostEqual(chiptune.hz("C5"), 523.2511, places=3)

    def test_a_pulse_is_two_levels_at_its_volume(self):
        level = chiptune.pulse([(0.0, 0.1, "A4", 9)], 0.1, duty=25, decay=None)
        self.assertEqual(set(np.unique(level)), {0.0, 9.0})
        self.assertAlmostEqual((level > 0).mean(), 0.25, delta=0.02)

    def test_the_triangle_steps_through_sixteen_levels(self):
        level = chiptune.triangle([(0.0, 0.2, "A2")], 0.2)
        self.assertEqual(set(np.unique(level)), set(float(v) for v in range(16)))

    def test_silence_mixes_to_silence(self):
        quiet = np.zeros(1000)
        self.assertEqual(np.abs(chiptune.mix(quiet, quiet, quiet)).max(), 0.0)
        loud = chiptune.mix(np.full(10, 15.0), np.full(10, 15.0), np.full(10, 15.0))
        self.assertLess(loud.max(), 1.0)  # the chip's own mixer never exceeds full scale

    def test_the_output_filter_takes_out_the_offset(self):
        out = chiptune.output(np.full(chiptune.RATE, 0.5))
        self.assertLess(np.abs(out[-1000:]).max(), 1e-3)


class ThemeTest(unittest.TestCase):
    def test_it_lasts_the_opening_and_ends_in_silence(self):
        x = theme.render()
        self.assertAlmostEqual(len(x) / chiptune.RATE, OPENING_SECONDS, delta=0.15)
        self.assertLess(np.abs(x[-int(0.02 * chiptune.RATE):]).max(), 1e-3)
        self.assertLessEqual(np.abs(x).max(), 0.71)

    def test_it_builds_then_fades(self):
        x = theme.render()
        r = chiptune.RATE

        def loudness(a, b):
            return float(np.sqrt((x[int(a * r):int(b * r)] ** 2).mean()))
        self.assertGreater(loudness(3.0, 4.0), loudness(0.0, 1.0))  # additive: it grows
        self.assertGreater(loudness(3.0, 4.0), 4 * loudness(OPENING_SECONDS - 1.0, OPENING_SECONDS - 0.2))

    def test_it_is_written_once_and_the_same_every_time(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = theme.path(Path(tmp))
            self.assertEqual(path.suffix, ".wav")
            first = path.stat().st_mtime_ns
            self.assertEqual(theme.path(Path(tmp)).stat().st_mtime_ns, first)  # cached
            written, rate = soundfile.read(path)
            self.assertEqual(rate, chiptune.RATE)
            self.assertTrue(np.allclose(written, theme.render(), atol=1e-6))


if __name__ == "__main__":
    unittest.main()
