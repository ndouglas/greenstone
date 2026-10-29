"""Greenstone footage inside a Manim scene, played at the NES's 60 frames a second."""

from pathlib import Path

from manim import ImageMobject
from manim.constants import RESAMPLING_ALGORITHMS
from manim.utils.images import change_to_rgba_array

from reservoir_film.model import TAKES_DIR, Takes

NES_FPS = 60


class GameFootage(ImageMobject):
    """A take's frames: frame `start_frame + round(t * 60)` at time `t` after
    it is added, with crisp, unsmoothed pixels."""

    def __init__(self, take: str, stream: str, start_frame: int = 0, root: Path = TAKES_DIR, **kwargs):
        self.frames = Takes(root)[take].frames(stream)
        self.start_frame = start_frame
        self.elapsed = 0.0
        super().__init__(self.frames[start_frame], **kwargs)
        self.set_resampling_algorithm(RESAMPLING_ALGORITHMS["nearest"])
        self.add_updater(GameFootage.advance)

    def advance(self, dt: float) -> None:
        self.elapsed += dt
        frame = self.frames[self.start_frame + round(self.elapsed * NES_FPS)]
        self.pixel_array = change_to_rgba_array(frame)
