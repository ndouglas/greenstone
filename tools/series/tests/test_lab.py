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

    def test_a_one_sided_label_is_refused_wherever_it_comes_from(self):
        from series.lab.data import check_rate
        with self.assertRaises(ValueError):
            check_rate(np.array([0] * 199 + [1]))
        check_rate(np.array([0, 1] * 100))

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


class Landing(unittest.TestCase):
    def test_landing_looks_ahead_within_the_air_only(self):
        # In the air on frames 0-5, landing on frame 6; ground after.
        air = np.array([1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0], dtype=bool)
        keep, lands = exp001.landing(air, restarts=[], within=3)
        self.assertEqual(keep.tolist(), [0, 1, 2, 3, 4, 5])
        self.assertEqual(lands.tolist(), [0, 0, 0, 1, 1, 1])

    def test_a_restart_is_not_a_landing(self):
        air = np.array([1, 1, 1, 1, 0, 0, 0, 0], dtype=bool)
        keep, lands = exp001.landing(air, restarts=[2], within=3)
        self.assertNotIn(0, keep.tolist())
        self.assertNotIn(1, keep.tolist())
        self.assertNotIn(2, keep.tolist())


class Keys(unittest.TestCase):
    def test_knockback_counts_as_in_the_air_in_castlevania(self):
        ram = np.zeros((3, 2048), dtype=np.uint8)
        ram[:, 0x046C] = [0x00, 0x01, 0x05]
        self.assertEqual(exp001.label("castlevania")(ram).tolist(), [False, True, True])

    def test_any_nonzero_flag_is_in_the_air_in_smb3(self):
        ram = np.zeros((3, 2048), dtype=np.uint8)
        ram[:, 0x00D8] = [0, 1, 6]
        self.assertEqual(exp001.label("smb3")(ram).tolist(), [False, True, True])


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

    def test_a_setting_counts_as_every_sessions_only_if_all_agree(self):
        self.assertEqual(exp001.agreed_cut([176.0, 176.0, 176.0]), 176)
        self.assertIsNone(exp001.agreed_cut([176.0, 170.0, 176.0]))

    def test_a_false_alarm_is_a_bounce_a_pit_or_late(self):
        air = np.array([1, 1, 1, 1, 0, 1], dtype=bool)
        normal = np.ones(6, dtype=bool)
        depth = np.array([170, 172, 174, 170, 176, 150])
        rising = np.array([4, 4, -4, -4, 0, 0])
        self.assertEqual(exp001.false_alarm_kind(air, rising, depth, normal, [], 0), "bounce")
        falling = np.array([4, 4, 4, 4, 0, 0])
        self.assertEqual(exp001.false_alarm_kind(air, falling, depth, normal, [], 0), "late")
        pit = np.array([180, 200, 230, 250, 255, 176])
        self.assertEqual(exp001.false_alarm_kind(np.ones(6, dtype=bool), falling, pit, normal, [5], 0), "pit")
        # Dying pops Mario upward; that isn't a bounce.
        dying = np.array([True, False, False, False, False, True])
        self.assertEqual(exp001.false_alarm_kind(np.ones(6, dtype=bool), rising, depth, dying, [5], 0), "died")

    def test_a_false_alarm_never_looks_into_the_next_life(self):
        # He neither lands nor falls past the screen before the restart at
        # frame 3; the next life's pit fall isn't this moment's.
        air = np.ones(6, dtype=bool)
        falling = np.array([4, 4, 4, 0, 4, 4])
        depth = np.array([170, 172, 174, 176, 230, 250])
        self.assertEqual(exp001.false_alarm_kind(air, falling, depth, np.ones(6, dtype=bool), [3], 0), "died")

    def test_worked_examples_follow_the_found_numbers(self):
        self.assertEqual(exp001.worked_examples(cut=176, k=5, threshold=152), {
            "example_air_total": 26, "land_example_speed_yes": 20, "land_example_yes": 8,
            "land_example_speed_no": 10, "land_example_no": 22,
            # The rising example on the scale: 10 and 152 in the no pan, against 140.
            "land_balance_no_pan": 162,
            "example_air_depth": 150, "land_example_depth": 140, "land_example_falling": 4, "land_example_rising": 2})

    def test_worked_examples_refuse_numbers_that_flip_their_answers(self):
        # 140 + 5 x 4 - 170 isn't above zero, so "lands soon" would be false.
        with self.assertRaises(ValueError):
            exp001.worked_examples(cut=176, k=5, threshold=170)

    def test_a_recording_must_match_its_answer_key(self):
        rec = load(fake_recording(200))
        rec.meta.update(sha1="ea343f4e445a9050d4b4fbac2c77d0693b1d0922", label_rate=0.25)
        exp001.check_recording("smb1", rec)
        for bad in ({"airborne_byte": 0xD8}, {"sha1": "0" * 40}, {"label_rate": 0.5}, {"frames": 199}):
            wrong = load(fake_recording(200))
            wrong.meta.update({"sha1": "ea343f4e445a9050d4b4fbac2c77d0693b1d0922", "label_rate": 0.25, **bad})
            with self.assertRaises(ValueError, msg=str(bad)):
                exp001.check_recording("smb1", wrong)

    def test_time_between_deaths_comes_from_the_recordings(self):
        self.assertEqual(exp001.seconds_between_deaths(frames=[36000], restarts=[30]), 20)
        self.assertEqual(exp001.recording_minutes(frames=[36000]), 10)

    def test_the_whole_number_border_keeps_its_direction(self):
        self.assertEqual(exp001.whole_number_border(np.array([0.0335, 0.1683]), -5.084), (5, 152))
        with self.assertRaises(ValueError):
            exp001.whole_number_border(np.array([-0.0335, 0.1683]), -5.084)

    def test_the_scripts_plain_words_are_checked(self):
        good = dict(cut=176, standing=176, smb3_cut=128, smb3_standing=128, train=0.938, test=0.940,
                    depth_alone=0.748, not_yet=0.745, best_line=0.941,
                    raised_depth=114, floor_depth=155, falling_depth=0.84, falling_not_yet=0.575)
        exp001.check_claims(**good)
        # "Adding his falling speed doesn't help a straight line here."
        for bad in ({"cut": 170}, {"smb3_cut": 120}, {"test": 0.90}, {"depth_alone": 0.80}, {"best_line": 0.96},
                    # "Landings on blocks and pipes sit further left"; "it can't tell rising from falling".
                    {"raised_depth": 160}, {"falling_depth": 0.60}):
            with self.assertRaises(ValueError, msg=str(bad)):
                exp001.check_claims(**{**good, **bad})

    def test_only_frames_of_play_count_in_super_mario_bros(self):
        ram = np.zeros((3, 2048), dtype=np.uint8)
        # Play; dying; the freeze while the game shrinks a hurt Mario.
        ram[:, 0x0E] = [0x08, 0x0B, 0x0A]
        self.assertEqual(exp001.in_play("smb1", ram).tolist(), [True, False, False])
        self.assertEqual(exp001.in_play("castlevania", ram).tolist(), [True, True, True])

    def test_halves_round_up(self):
        self.assertEqual(exp001.percent(0.125), ("13%", "thirteen percent"))

    def test_numbers_have_spoken_forms(self):
        self.assertEqual(exp001.spoken_number(176), "one hundred and seventy-six")
        self.assertEqual(exp001.spoken_number(1300), "one thousand three hundred")
        self.assertEqual(exp001.spoken_number(2048), "two thousand and forty-eight")
        self.assertEqual(exp001.percent(0.94), ("94%", "ninety-four percent"))

    def test_a_hand_line_is_scored_on_held_out_frames(self):
        x = np.array([[0.0, 0.0], [10.0, 0.0], [0.0, 10.0], [10.0, 10.0]])
        y = np.array([0, 1, 0, 1])
        self.assertEqual(exp001.accuracy(w=np.array([1.0, 0.0]), b=-5.0, x=x, y=y), 1.0)


if __name__ == "__main__":
    unittest.main()
