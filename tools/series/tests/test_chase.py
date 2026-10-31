import shutil
import tempfile
import unittest
from pathlib import Path

import numpy as np

from series.homebrew.chase import make, music


class ChaseTest(unittest.TestCase):
    def test_a_tile_is_two_bit_planes(self):
        tile = np.zeros((8, 8), np.uint8)
        tile[0] = [0, 1, 2, 3, 0, 0, 0, 3]
        data = make.chr_bytes(tile)
        self.assertEqual(len(data), 16)
        self.assertEqual(data[0], 0b01010001)  # low plane: colours 1 and 3
        self.assertEqual(data[8], 0b00110001)  # high plane: colours 2 and 3

    def test_every_pose_stands_on_its_feet_and_shares_a_middle(self):
        _, metas, _ = make.art()
        for name, entries in metas.items():
            grid = make.clip_grid() if name == "clip" else make.nate_grid(make.NATE_FRAMES[name])
            self.assertEqual(min(dy for dy, *_ in entries), -grid.shape[0], name)  # the art's top row
            lefts = [dx for *_, dx in entries]
            self.assertEqual(min(lefts), (make.BOX - grid.shape[1]) // 2, name)  # centred in the box

    def test_the_tiles_fit_and_nate_keeps_his_colours(self):
        data, metas, _ = make.art()
        self.assertEqual(len(data), 8192)
        self.assertEqual({e[2] for n, es in metas.items() if n != "clip" for e in es}, {make.NATE})
        self.assertEqual({e[2] for e in metas["clip"]}, {make.CLIP_PAL})

    def test_the_clips_chase_but_never_pass_him(self):
        for f, (nate, *clips) in enumerate(make.choreography()):
            for x, _, pose in clips:
                if pose is not None and nate[2] is not None:
                    self.assertLess(x, nate[0], f)
        frames = make.choreography()
        self.assertEqual(frames[make.FRAMES - 1][0][2], None)  # he's gone by the end
        self.assertIn("leap", {nate[2] for nate, *_ in frames})

    def test_no_line_ever_needs_more_than_eight_sprites(self):
        _, metas, _ = make.art()
        for f, actors in enumerate(make.choreography()):
            lines = np.zeros(240, int)
            for x, y, pose in actors:
                if pose is None:
                    continue
                for dy, _, _, dx in metas[pose]:
                    if 0 <= x + dx < 256:
                        lines[max(0, y + dy):max(0, y + dy + 8)] += 1
            self.assertLessEqual(lines.max(), 8, f"frame {f}: {lines.max()} sprites on a line, so some flicker")

    def test_the_tune_reads_as_written(self):
        bars = music.bars((music.HERE / "stures.abc").read_text())
        self.assertEqual([sum(n[1] for n in bar) for bar in bars[1:8]], [8.0] * 7)  # whole 4/4 bars, in eighths
        self.assertEqual([n[0] for n in bars[1]][:4], [67, 66, 67, 69])  # G, F sharp (written), G, A
        self.assertEqual(bars[1][4][0], 70)  # B flat, from the key
        self.assertEqual(bars[3][0][2], "D")
        self.assertEqual([n[0] for n in music.excerpt()[-1]], [67, 70, 67, 62])  # home: G, B flat, G, D

    def test_the_music_fits_the_chase_and_uses_two_channels(self):
        writes = music.writes(make.FRAMES)
        self.assertEqual(len(writes), make.FRAMES)
        registers = {r for frame in writes for r, _ in frame}
        self.assertEqual(registers, {0x4000, 0x4002, 0x4003, 0x4008, 0x400A, 0x400B})
        self.assertTrue(writes[0])  # the pickup starts at once
        self.assertEqual(music.pulse_period(69), 253)  # A4 on the chip

    @unittest.skipUnless(shutil.which("ca65") and shutil.which("ld65"), "cc65 not installed")
    def test_it_builds_an_nrom_cartridge(self):
        with tempfile.TemporaryDirectory() as tmp:
            rom = make.build(Path(tmp)).read_bytes()
        self.assertEqual(rom[:4], b"NES\x1a")
        self.assertEqual((rom[4], rom[5], rom[6] >> 4), (2, 1, 0))  # 32 KB program, 8 KB tiles, mapper 0
        self.assertEqual(len(rom), 16 + 32768 + 8192)


if __name__ == "__main__":
    unittest.main()
