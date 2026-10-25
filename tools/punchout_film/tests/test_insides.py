import unittest
from types import SimpleNamespace

from punchout_film import insides


class Insides(unittest.TestCase):
    def take(self):
        steps = [{"step": k, "move": "Idle"} for k in range(3)]
        return SimpleNamespace(meta={"steps": steps, "moves": ["Idle", "DodgeLeft"]})

    def test_step_at_maps_four_frames_to_a_step(self):
        self.assertEqual(insides.step_at(self.take(), 7)["step"], 1)

    def test_step_at_clamps_past_the_end(self):
        self.assertEqual(insides.step_at(self.take(), 400)["step"], 2)

    def test_moves(self):
        self.assertEqual(insides.moves(self.take()), ["Idle", "DodgeLeft"])
