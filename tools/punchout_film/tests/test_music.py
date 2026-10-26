import unittest

from punchout_film.film import PUNCHOUT


class Music(unittest.TestCase):
    def test_every_chapter_has_a_cue_and_every_sting_exists(self):
        cues, chapter_cues, stings = PUNCHOUT.cues()
        chapters = PUNCHOUT.chapters()
        self.assertEqual([c.number for c in chapters], list(range(1, 12)))
        for c in chapters:
            self.assertIn(c.number, chapter_cues, f"chapter {c.number} has no cue")
            self.assertIn(chapter_cues[c.number], cues)
            for shot in c.shots:
                if not callable(shot.stings):
                    for _, name in shot.stings:
                        self.assertIn(name, stings)
        self.assertIn("bell", stings)

    def test_roar_is_a_sting(self):
        from reservoir_film.music.core import compile_cue
        _, _, stings = PUNCHOUT.cues()
        self.assertIn("roar", stings)
        compile_cue(stings["roar"], 1)

    NEW_CUES = ["ring_entrance", "cancan", "funeral", "marseillaise", "marseillaise_fast",
                "marseillaise_elevator", "marseillaise_stall", "ode", "triumph", "augustin", "turca", "valkyries"]

    def test_cues_compile_and_credit_their_composers(self):
        from reservoir_film.music.core import compile_cue
        cues, _, _ = PUNCHOUT.cues()
        for name in self.NEW_CUES:
            cue = cues[name]
            compile_cue(cue, 2)
            self.assertEqual(cue.credit == "original", name == "ring_entrance", name)
