import unittest
from pathlib import Path

import numpy as np

from series.build import OUT
from series.recording import LAG, Recording, measure_lag, smb1_sprite_bottom

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
    def test_ram_is_read_behind_the_picture(self):
        frames, ram = synthetic()
        rec = Recording.__new__(Recording)
        rec.game, rec.frames, rec.ram = "smb1", frames, ram
        self.assertTrue(np.array_equal(rec.ram_at(10), ram[9]))
        self.assertTrue(np.array_equal(rec.ram_at(0), ram[0]))
        self.assertEqual(measure_lag(frames, ram, bottom, BYTE, OFFSET), (1, 1.0))

    def test_a_recording_with_short_ram_is_refused(self):
        import tempfile
        from reservoir_film.model import NES_H, NES_W
        import json
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            take = root / "smb1" / "footage"
            take.mkdir(parents=True)
            (take / "take.json").write_text(json.dumps({"streams": {"game": 5}}))
            np.zeros((5, NES_H, NES_W, 3), dtype=np.uint8).tofile(take / "game.rgb")
            (root / "smb1" / "1").mkdir()
            np.zeros((3, 2048), dtype=np.uint8).tofile(root / "smb1" / "1" / "ram.u8")
            with self.assertRaisesRegex(ValueError, "5.*3"):
                Recording("smb1", root)

    def test_a_recording_with_missing_ram_is_refused(self):
        import tempfile
        import json
        from reservoir_film.model import NES_H, NES_W
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
