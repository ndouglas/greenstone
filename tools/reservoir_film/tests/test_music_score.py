import unittest

import numpy as np

from reservoir_film.music.core import NES_FRAME_HZ, Cue, Instrument, Part, Section
from reservoir_film.music.score import SAMPLE_RATE, TEMPO_RANGE, crossfade_join, cue_seconds, fit, trim

I = Instrument()
TOY = Cue(
    name="toy", title="Toy", bpm=120,
    intro=Section([Part("p1", I, "C5/1")]),
    loop=Section([Part("p1", I, "C5/1 D5/1")]),
    ending=Section([Part("p1", I, "C5/1")]),
)


class Fit(unittest.TestCase):
    def test_fit_lands_within_a_frame(self):
        for seconds in (30.0, 61.3, 95.0):
            loops, bpm, needs_trim = fit(TOY, seconds)
            self.assertFalse(needs_trim)
            self.assertLess(abs(cue_seconds(TOY, loops, bpm) - seconds), 1 / NES_FRAME_HZ, seconds)
            self.assertTrue(TEMPO_RANGE[0] * TOY.bpm <= bpm <= TEMPO_RANGE[1] * TOY.bpm)

    def test_too_short_a_chapter_asks_for_a_trim(self):
        loops, bpm, needs_trim = fit(TOY, 2.0)
        self.assertEqual(loops, 0)
        self.assertTrue(needs_trim)


class Trim(unittest.TestCase):
    def test_trim_is_exact_and_fades_what_it_cuts(self):
        x = np.ones(SAMPLE_RATE * 5, dtype=np.float32)
        y = trim(x, 3.0)
        self.assertEqual(len(y), 3 * SAMPLE_RATE)
        self.assertAlmostEqual(float(y[-1]), 0.0, places=3)
        self.assertAlmostEqual(float(y[0]), 1.0)

    def test_trim_pads_short_audio(self):
        y = trim(np.ones(100, dtype=np.float32), 1.0)
        self.assertEqual(len(y), SAMPLE_RATE)
        self.assertEqual(float(y[-1]), 0.0)


class Join(unittest.TestCase):
    def test_join_keeps_the_total_length(self):
        a = np.ones(SAMPLE_RATE * 2, dtype=np.float32)
        b = np.ones(SAMPLE_RATE * 3, dtype=np.float32)
        self.assertEqual(len(crossfade_join([a, b])), 5 * SAMPLE_RATE)
