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
