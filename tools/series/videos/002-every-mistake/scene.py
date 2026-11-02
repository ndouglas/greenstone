"""002: Every mistake moves the line. Each section plays its built beats; the rest are placeholder cards until built."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from beats import chapter_1, chapter_2, chapter_3, chapter_4, chapter_5, conclusions, introduction  # noqa: E402

from series.scene import NarratedScene  # noqa: E402

BEATS = {"introduction": introduction.BEATS, "chapter-1": chapter_1.BEATS, "chapter-2": chapter_2.BEATS,
         "chapter-3": chapter_3.BEATS, "chapter-4": chapter_4.BEATS,
         "chapter-5": chapter_5.BEATS, "conclusions": conclusions.BEATS}


class VideoScene(NarratedScene):
    def construct(self):
        for section in self.video.sections:
            self.play_section(section.id, BEATS.get(section.id, {}))
        self.play_rest(conclusions.REST_BEATS)
        self.roll_credits()
