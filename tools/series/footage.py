"""Greenstone footage inside a Manim scene, played at the NES's 60 frames a second."""

import warnings
from pathlib import Path

from manim import ImageMobject
from manim.constants import RESAMPLING_ALGORITHMS
from manim.utils.images import change_to_rgba_array

from reservoir_film.model import RECORD_HINT, TAKES_DIR, Takes

NES_FPS = 60


class GameFootage(ImageMobject):
    """A take's frames: frame `start_frame + round(t * 60)` at time `t` after
    it is added, with crisp, unsmoothed pixels. `root` and `hint` say where
    the take lives and how to record it if it's missing (each film has its
    own)."""

    def __init__(self, take: str, stream: str, start_frame: int = 0, root: Path = TAKES_DIR,
                 hint: str = RECORD_HINT, **kwargs):
        self.frames = Takes(root, hint)[take].frames(stream)
        if not 0 <= start_frame < len(self.frames):
            raise ValueError(f"take {take} has {len(self.frames)} frames; start_frame {start_frame} is outside it")
        self.take = take
        self.warned = False
        self.start_frame = start_frame
        self.elapsed = 0.0
        super().__init__(self.frames[start_frame], **kwargs)
        self.set_resampling_algorithm(RESAMPLING_ALGORITHMS["nearest"])
        self.add_updater(GameFootage.advance)

    def advance(self, dt: float) -> None:
        self.elapsed += dt
        index = self.start_frame + round(self.elapsed * NES_FPS)
        if index >= len(self.frames) and not self.warned:
            warnings.warn(f"footage ran past the end of take {self.take} and froze on its last frame", RuntimeWarning)
            self.warned = True
        frame = self.frames[index]
        self.pixel_array = change_to_rgba_array(frame)
