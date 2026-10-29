import json
import tempfile
import unittest
from pathlib import Path

import numpy as np

from series.lab import exp001
from series.lab.data import dataset, load
from series.lab.perceptron import Perceptron


def fake_recording(frames=200, airborne_every=4):
    d = Path(tempfile.mkdtemp())
    ram = np.zeros((frames, 2048), dtype=np.uint8)
    ram[:, 0x1D] = [1 if f % airborne_every == 0 else 0 for f in range(frames)]
    ram[:, 0xCE] = np.arange(frames) % 256
    ram.tofile(d / "ram.u8")
    np.zeros(frames, dtype=np.uint8).tofile(d / "buttons.u8")
    (d / "meta.json").write_text(json.dumps({"game": "Super Mario Bros.", "frames": frames, "airborne_byte": 0x1D}))
    return d


class Data(unittest.TestCase):
    def test_recordings_load_with_their_shape(self):
        rec = load(fake_recording(200))
        self.assertEqual(rec.ram.shape, (200, 2048))
        self.assertEqual(rec.buttons.shape, (200,))
        self.assertEqual(rec.meta["game"], "Super Mario Bros.")

    def test_a_one_sided_dataset_is_refused(self):
        rec = load(fake_recording(200, airborne_every=1000))  # airborne on 1 frame of 200
        with self.assertRaises(ValueError):
            dataset([rec], inputs=[0xCE], label=lambda ram: ram[:, 0x1D] != 0)


class Learning(unittest.TestCase):
    def test_the_perceptron_learns_a_separable_line(self):
        rng = np.random.default_rng(0)
        x = rng.uniform(-10, 10, size=(400, 2))
        margin = x[:, 1] - (2 * x[:, 0] + 1)
        keep = np.abs(margin) > 0.5
        x, y = x[keep], (margin[keep] > 0).astype(int)
        p = Perceptron(2)
        mistakes = p.fit(x, y, epochs=50, rate=0.1, seed=0)
        self.assertEqual(mistakes[-1], 0)
        slope = -p.w[0] / p.w[1]
        self.assertAlmostEqual(slope, 2.0, delta=0.2)

    def test_it_never_settles_on_inseparable_data(self):
        x = np.array([[0, 0], [0, 1], [1, 0], [1, 1]] * 25, dtype=float)
        y = np.array([0, 1, 1, 0] * 25)
        mistakes = Perceptron(2).fit(x, y, epochs=200, rate=0.1, seed=0)
        self.assertTrue(all(m > 0 for m in mistakes))


class Experiment(unittest.TestCase):
    def test_the_answer_key_is_not_an_input(self):
        with self.assertRaises(ValueError):
            exp001.check_inputs(game="smb1", inputs=[0x1D, 0xCE])

    def test_the_best_line_finds_a_tilted_boundary(self):
        rng = np.random.default_rng(1)
        x = rng.uniform(0, 100, size=(2000, 2))
        y = (x[:, 0] + x[:, 1] > 100).astype(int)
        w, b, acc = exp001.best_line(x, y)
        self.assertGreater(acc, 0.97)
        self.assertGreater(exp001.accuracy(w, b, x, y), 0.97)

    def test_numbers_have_spoken_forms(self):
        self.assertEqual(exp001.spoken_number(176), "one hundred and seventy-six")
        self.assertEqual(exp001.percent(0.94), ("94%", "ninety-four percent"))

    def test_a_hand_line_is_scored_on_held_out_frames(self):
        x = np.array([[0.0, 0.0], [10.0, 0.0], [0.0, 10.0], [10.0, 10.0]])
        y = np.array([0, 1, 0, 1])
        self.assertEqual(exp001.accuracy(w=np.array([1.0, 0.0]), b=-5.0, x=x, y=y), 1.0)


if __name__ == "__main__":
    unittest.main()
