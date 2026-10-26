import unittest

from punchout_film.film import PUNCHOUT
from reservoir_film.model import Context, Takes, load_numbers

HAVE_TAKES = (PUNCHOUT.takes_dir / "boot" / "take.json").exists()


def context():
    return Context(takes=Takes(PUNCHOUT.takes_dir, PUNCHOUT.record_hint), numbers=load_numbers(PUNCHOUT.numbers))


class Music(unittest.TestCase):
    def test_every_chapter_has_a_cue_and_every_sting_exists(self):
        cues, chapter_cues, stings = PUNCHOUT.cues()
        chapters = PUNCHOUT.chapters()
        self.assertEqual([c.number for c in chapters], list(range(1, 12)))
        for c in chapters:
            self.assertIn(c.number, chapter_cues, f"chapter {c.number} has no cue")
            entry = chapter_cues[c.number]
            spans = [(c.shots[0].name, 0, entry)] if isinstance(entry, str) else entry
            for shot, _, name in spans:
                self.assertIn(name, cues)
                self.assertIn(shot, [s.name for s in c.shots], f"chapter {c.number} has no shot {shot}")
            for shot in c.shots:
                if not callable(shot.stings):
                    for _, name in shot.stings:
                        self.assertIn(name, stings)
        self.assertIn("bell", stings)

    @unittest.skipUnless(HAVE_TAKES, "record the takes first")
    def test_every_span_fits(self):
        from reservoir_film.build import cue_plan, unfit_spans
        cues, chapter_cues, _ = PUNCHOUT.cues()
        plan = cue_plan(PUNCHOUT.chapters(), chapter_cues, cues, context())
        self.assertEqual(unfit_spans(plan), [])

    def test_credits_name_exactly_the_cues_used(self):
        from punchout_film.chapters.c11_verdict import CREDITS
        heads = [h for h, _ in CREDITS]
        for who in ["Claude Joseph Rouget de Lisle", "Jacques Offenbach", "Frédéric Chopin", "Ludwig van Beethoven",
                    "Wolfgang Amadeus Mozart", "Richard Wagner", "Korobeiniki", "Ach du lieber Augustin",
                    "Everything else"]:
            self.assertEqual(heads.count(who), 1, who)
        text = repr(CREDITS)
        for gone in ["Minuet", "Petzold", "Fučík", "Strauss", "Teike"]:
            self.assertNotIn(gone, text)
        self.assertEqual(len(CREDITS), 9 + 3)

    def test_the_score_must_fit(self):
        self.assertTrue(PUNCHOUT.strict_score)

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

    def test_the_triumph_is_its_own_tune(self):
        from punchout_film.music import ode, triumph
        self.assertNotEqual(triumph.loop.parts[0].notes, ode.loop.parts[0].notes)
        self.assertEqual(triumph.loop.parts[1].instrument, ode.loop.parts[0].instrument, "a brass countermelody on p2")

    def test_the_fanfare_lands_on_its_tonic(self):
        from punchout_film.music import RING_CHORDS
        self.assertEqual(RING_CHORDS[-1], "C")
