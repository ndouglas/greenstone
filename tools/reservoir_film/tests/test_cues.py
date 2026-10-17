import unittest

from reservoir_film.music.core import compile_cue
from reservoir_film.music.cues import CHAPTER_CUES, CUES, STINGS


class Cues(unittest.TestCase):
    def test_every_cue_and_sting_compiles(self):
        for name, cue in {**CUES, **STINGS}.items():
            with self.subTest(name):
                frames, writes = compile_cue(cue, loops=1)
                self.assertGreater(frames, 0)
                self.assertTrue(any(a in (0x4002, 0x4006, 0x400A) for _, a, _ in writes), "plays a pitch")

    def test_every_chapter_has_a_cue(self):
        self.assertEqual(sorted(CHAPTER_CUES), list(range(10)))
        for name in CHAPTER_CUES.values():
            self.assertIn(name, CUES)

    def test_credits_name_the_two_borrowed_tunes(self):
        self.assertEqual(CUES["korobeiniki"].credit, "traditional")
        self.assertIn("Petzold", CUES["minuet"].credit)
        originals = [n for n, c in CUES.items() if c.credit == "original"]
        self.assertEqual(sorted(originals), ["coin_run", "fooled", "lab", "reprise", "underground"])
