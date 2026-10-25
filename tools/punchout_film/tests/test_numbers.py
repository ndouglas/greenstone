import json
import unittest

from punchout_film.film import PUNCHOUT

# The Brain takes the film records (Task 2's FILM_TAKES), as (phase, kind, round, opponent).
FILM_BRAIN_TAKES = [
    (1, "bank", "r4", "glass-joe"), (1, "echo0_5", "r4", "glass-joe"),
    (2, "stack4", "r4", "von-kaiser"), (3, "stack4", "r4", "glass-joe"), (2, "stack4", "r4", "bald-bull"),
    (3, "cheat", "r4", "von-kaiser"), (3, "stack4", "r4", "von-kaiser"), (2, "stack4", "r0", "bald-bull"),
]
SWEEP = ["stack1", "stack2", "stack4", "stack8", "echo0_2", "echo0_5", "echo0_9"]


def numbers():
    return json.loads(PUNCHOUT.numbers.read_text())


class Numbers(unittest.TestCase):
    def test_numbers_match_the_ideas_doc(self):
        n = numbers()
        self.assertEqual(round(n["phase2"]["stack4"]["r0"]["bald-bull"]["hpm"], 2), 12.77)
        self.assertEqual(round(n["phase2"]["stack4"]["r4"]["bald-bull"]["hpm"], 2), 22.76)
        self.assertEqual(round(n["phase1"]["echo0_5"]["r4"]["von-kaiser"]["hpm"], 2), 17.39)
        self.assertEqual(round(n["sweep"]["stack8"]["von-kaiser"]["hpm"], 2), 20.31)
        self.assertEqual(round(n["idle"]["glass-joe"]["hpm"], 2), 20.98)
        vk = n["teacher"]["phase3"]["von-kaiser"]
        self.assertEqual((vk["won"], vk["steps"], vk["hits"], vk["knockdowns"]), (True, 801, 0, 3))
        self.assertEqual(round(n["teacher_idle_share"]["phase1"], 3), 0.967)

    def test_caption_references_reach_dotted_brains(self):
        from reservoir_film.model import fill
        self.assertEqual(fill("{numbers.phase1.echo0_5.r4.von-kaiser.hpm:.2f}", None, numbers()), "17.39")

    def test_numbers_cover_every_film_take(self):
        n = numbers()
        for phase, kind, rnd, opponent in FILM_BRAIN_TAKES:
            self.assertIn("hpm", n[f"phase{phase}"][kind][rnd][opponent], (phase, kind, rnd, opponent))
        for kind in SWEEP:
            self.assertIn("hpm", n["sweep"][kind]["von-kaiser"], kind)
