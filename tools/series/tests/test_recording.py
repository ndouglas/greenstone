import json
import tempfile
import unittest
from pathlib import Path

import numpy as np

from reservoir_film.model import NES_H, NES_W
from series.build import OUT
from series.recording import BYTE_LAG, LAG, MARIO_RED, MARIO_SHOE, Recording, measure_lag, smb1_sprite_bottom
from series.tests.test_screen import synthetic as screen_synthetic

DATA = OUT / "001-one-neuron" / "data"
BYTE, OFFSET = 7, 32


def synthetic(n: int = 40, lag: int = 1):
    """A RAM whose byte moves, and frames whose sprite row trails it by `lag`."""
    ram = np.zeros((n, 2048), dtype=np.uint8)
    ram[:, BYTE] = (np.arange(n) * 3) % 100
    frames = []
    for f in range(n):
        frame = np.zeros((240, 256, 3), dtype=np.uint8)
        frame[ram[max(f - lag, 0), BYTE] + OFFSET, 10, :] = 255
        frames.append(frame)
    return frames, ram


def bottom(frame: np.ndarray) -> int | None:
    rows = np.flatnonzero(frame[:, 10, 0] == 255)
    return int(rows[-1]) if len(rows) else None


class TestRecording(unittest.TestCase):
    def test_a_span_outside_the_data_is_refused(self):
        frames, ram = synthetic()
        for span in (range(0, 20), range(2, 20), range(10, 41), range(10, 1000)):
            with self.assertRaisesRegex(ValueError, "span"):
                measure_lag(frames, ram, bottom, BYTE, OFFSET, span=span)
        self.assertEqual(measure_lag(frames, ram, bottom, BYTE, OFFSET, span=range(3, 40)), (1, 1.0))

    def test_a_directory_is_not_a_ram_file(self):
        with tempfile.TemporaryDirectory() as tmp:
            take = Path(tmp) / "smb1" / "footage"
            take.mkdir(parents=True)
            (take / "take.json").write_text(json.dumps({"streams": {"game": 5}}))
            np.zeros((5, NES_H, NES_W, 3), dtype=np.uint8).tofile(take / "game.rgb")
            (Path(tmp) / "smb1" / "1" / "ram.u8").mkdir(parents=True)
            with self.assertRaisesRegex(FileNotFoundError, "ram.u8"):
                Recording("smb1", Path(tmp))

    def test_games_not_yet_measured_stay_out_of_lag(self):
        self.assertEqual(set(LAG), {"smb1"})  # smb3 and castlevania are refused until measured
        for game in ("smb3", "castlevania"):
            rec = Recording.__new__(Recording)
            rec.game, rec.frames, rec.ram = game, [], np.zeros((1, 2048), dtype=np.uint8)
            with self.assertRaisesRegex(KeyError, f"{game}.*measure_lag"):
                rec.ram_at(0)

    def test_ram_is_read_behind_the_picture(self):
        frames, ram = synthetic()
        rec = Recording.__new__(Recording)
        rec.game, rec.frames, rec.ram = "smb1", frames, ram
        self.assertTrue(np.array_equal(rec.ram_at(10), ram[9]))
        self.assertTrue(np.array_equal(rec.ram_at(0), ram[0]))
        self.assertEqual(measure_lag(frames, ram, bottom, BYTE, OFFSET), (1, 1.0))

    def test_a_recording_with_short_ram_is_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            take = root / "smb1" / "footage"
            take.mkdir(parents=True)
            (take / "take.json").write_text(json.dumps({"streams": {"game": 5}}))
            np.zeros((5, NES_H, NES_W, 3), dtype=np.uint8).tofile(take / "game.rgb")
            (root / "smb1" / "1").mkdir()
            np.zeros((3, 2048), dtype=np.uint8).tofile(root / "smb1" / "1" / "ram.u8")
            with self.assertRaisesRegex(ValueError, r"footage has 5 frames but the RAM only 3"):
                Recording("smb1", root)

    def test_a_recording_with_missing_ram_is_refused(self):
        with tempfile.TemporaryDirectory() as tmp:
            take = Path(tmp) / "smb1" / "footage"
            take.mkdir(parents=True)
            (take / "take.json").write_text(json.dumps({"streams": {"game": 5}}))
            np.zeros((5, NES_H, NES_W, 3), dtype=np.uint8).tofile(take / "game.rgb")
            with self.assertRaisesRegex(FileNotFoundError, r"smb1.*ram\.u8.*5 frames.*series_record"):
                Recording("smb1", Path(tmp))

    def test_an_unmeasured_game_is_refused(self):
        frames, ram = synthetic()
        rec = Recording.__new__(Recording)
        rec.game, rec.frames, rec.ram = "tetris", frames, ram
        with self.assertRaisesRegex(KeyError, "tetris.*measure_lag"):
            rec.ram_at(3)

    def test_a_byte_can_have_its_own_lag(self):
        with tempfile.TemporaryDirectory() as tmp:
            rec = screen_synthetic(Path(tmp))
            rec.ram[:, 0x03AD] = np.arange(len(rec.ram)) % 256
            rec.ram[:, 0x00CE] = np.arange(len(rec.ram)) % 256
            self.assertEqual(rec.byte_at(30, 0x03AD), 30)  # Mario's screen x: read on the frame shown
            self.assertEqual(rec.byte_at(30, 0x00CE), 29)  # his depth: the game's lag, one back

    @unittest.skipUnless((DATA / "smb1" / "footage").exists(), "smb1 footage not recorded")
    def test_smb1s_screen_x_is_read_on_the_frame_shown(self):
        rec = Recording("smb1")
        spread = []
        for lag in range(3):
            error = []
            for f in range(300, 1940):
                if len({int(rec.ram[f - k][0x03AD]) for k in range(3)}) == 1:
                    continue  # standing still says nothing about lag
                picture = np.asarray(rec.frames[f])[..., :3]
                cols = np.argwhere((picture == MARIO_RED).all(2) | (picture == MARIO_SHOE).all(2))[:, 1]
                if len(cols) >= 10:
                    error.append((cols.min() + cols.max()) / 2 - int(rec.ram[f - lag][0x03AD]))
            spread.append(float(np.std(error)))
        self.assertEqual(int(np.argmin(spread)), BYTE_LAG["smb1"][0x03AD])
        self.assertLess(spread[0], 1.5)

    @unittest.skipUnless((DATA / "smb1" / "footage").exists(), "smb1 footage not recorded")
    def test_smb1s_lag_is_measured(self):
        rec = Recording("smb1")
        lag, share = measure_lag(rec.frames, rec.ram, smb1_sprite_bottom, 0x00CE, 32, span=range(1000, 1940))
        self.assertEqual(lag, 1)
        self.assertGreaterEqual(share, 0.95)
        self.assertEqual(lag, LAG["smb1"])
        self.assertEqual(len(rec), len(rec.frames))


if __name__ == "__main__":
    unittest.main()
