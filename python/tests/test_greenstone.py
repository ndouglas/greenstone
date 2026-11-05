"""The Python face of the emulator, checked against Super Mario Bros. (the ROM must be in roms2/)."""

import unittest

import numpy as np

import greenstone as gs

POSITION = 0x0086  # Mario's x within the screen's page
SPEED = 0x0057  # his horizontal speed


class Bindings(unittest.TestCase):
    def setUp(self):
        self.nes = gs.boot("smb1")

    def test_boot_lands_in_play(self):
        self.assertEqual(self.nes.peek(0x000E), 0x08)  # player state 08: normal play
        self.assertEqual(self.nes.ram.shape, (2048,))
        self.assertEqual(self.nes.ram.dtype, np.uint8)

    def test_holding_right_speeds_mario_up(self):
        self.nes.step(gs.RIGHT, frames=30)
        self.assertGreater(self.nes.peek(SPEED), 0)
        self.assertEqual(self.nes.frame, 30)

    def test_the_picture_is_the_screen(self):
        p = self.nes.picture()
        self.assertEqual((p.shape, p.dtype), ((240, 256, 3), np.uint8))
        self.assertGreater(len(np.unique(p.reshape(-1, 3), axis=0)), 3)

    def test_play_records_memory_after_every_frame(self):
        buttons = np.full(20, gs.RIGHT, np.uint8)
        twin = gs.boot("smb1")
        ram = self.nes.play(buttons)
        self.assertEqual(ram.shape, (20, 2048))
        for f in range(20):
            twin.step(gs.RIGHT)
            np.testing.assert_array_equal(ram[f], twin.ram)

    def test_play_can_keep_the_pictures(self):
        ram, pictures = self.nes.play(np.zeros(3, np.uint8), pictures=True)
        self.assertEqual(pictures.shape, (3, 240, 256, 3))
        np.testing.assert_array_equal(pictures[-1], self.nes.picture())

    def test_a_saved_moment_replays_exactly(self):
        self.nes.step(gs.RIGHT, frames=10)
        state = self.nes.save()
        a = self.nes.play(np.full(15, gs.RIGHT | gs.A, np.uint8))
        self.nes.load(state)
        self.assertEqual(self.nes.frame, 10)
        b = self.nes.play(np.full(15, gs.RIGHT | gs.A, np.uint8))
        np.testing.assert_array_equal(a, b)

    def test_poke_writes_memory(self):
        self.nes.poke(0x0300, 0x42)
        self.assertEqual(self.nes.peek(0x0300), 0x42)

    def test_an_unknown_game_is_refused(self):
        with self.assertRaises(ValueError):
            gs.boot("zelda")


if __name__ == "__main__":
    unittest.main()
