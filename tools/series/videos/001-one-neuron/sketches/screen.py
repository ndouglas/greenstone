"""The game screen: one 3× screen arriving and playing; then three 1× screens and nine 1/3× screens of the
three games side by side (the 3× ones with scanlines, the small ones averaged down)."""

import numpy as np
from manim import Scene, config

from series.ember import backdrop
from series.recording import Recording
from series.screen import Screen

GAMES = ("smb1", "smb3", "castlevania")


def at(x_px: float, y_px: float) -> np.ndarray:
    """A point given in pixels from the frame's centre (y up)."""
    px = config.frame_height / config.pixel_height
    return np.array([x_px * px, y_px * px, 0])


class Sketch(Scene):
    def construct(self):
        self.add(backdrop())
        recordings = {game: Recording(game) for game in GAMES}
        big = Screen(recordings["smb1"], 1247, left_px=36 * config.pixel_height // 1080)
        self.play(big.arrive())
        self.wait(5)
        self.play(big.depart(run_time=0.5))
        self.remove(big)

        u = config.pixel_height / 1080  # the layout scales with the frame; the screens' sizes are fixed
        ones = [Screen(recordings[g], 600, scale=1, centre=at((i - 1) * 300 * u, 150 * u))
                for i, g in enumerate(GAMES)]
        thirds = [Screen(recordings[g], 300 + 700 * k, scale=1 / 3, centre=at((3 * k + i - 4) * 110 * u, -220 * u))
                  for k in range(3) for i, g in enumerate(GAMES)]
        self.play(*(s.arrive() for s in ones + thirds))
        self.wait(4)
