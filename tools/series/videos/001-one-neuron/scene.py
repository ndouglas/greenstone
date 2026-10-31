"""001: One neuron draws a line. Each section plays its built beats; the rest are placeholder cards until built."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from beats import chapter_1  # noqa: E402

from series.scene import NarratedScene  # noqa: E402

BEATS = {"chapter-1": chapter_1.BEATS}


class VideoScene(NarratedScene):
    def construct(self):
        for section in self.video.sections:
            self.play_section(section.id, BEATS.get(section.id, {}))
        self.play_rest({})
        self.roll_credits()
