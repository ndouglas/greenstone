import unittest
from pathlib import Path

from series import pacing
from series.script import Line, Section, Video
from series.voice import Voice


def section(*beats, title="One"):
    lines = tuple(Line(id=i, text=i, spoken=None, cite=(), framing=True, pause=p, section="chapter-1") for i, _, p in beats)
    voices = {i: Voice(i, Path(f"{i}.wav"), seconds, "synth") for i, seconds, _ in beats}
    return Section(id="chapter-1", kind="chapter", number=1, title=title, lines=lines), voices


class Pacing(unittest.TestCase):
    def test_pacing_defaults(self):
        self.assertEqual(pacing.STILLNESS, 2.0)
        self.assertEqual(pacing.TITLE_HOLD, 4.0)
        self.assertEqual(pacing.SECTION_LIMIT, 240.0)
        self.assertEqual(pacing.NARRATION_SPEED, 0.9)


    def test_section_seconds_adds_voices_pauses_and_the_title(self):
        sec, voices = section(("a", 10.0, 0.5), ("b", 20.0, None))
        self.assertAlmostEqual(pacing.section_seconds(sec, voices), 4.0 + 10.5 + 22.0)

    def test_too_long_names_the_section(self):
        sec, voices = section(("a", 250.0, None), title="Is it any good?")
        video = Video(name="v", dir=Path("."), title="t", music=None, lines=sec.lines, citations={}, sections=(sec,))
        problems = pacing.too_long(video, voices)
        self.assertEqual(len(problems), 1)
        self.assertIn("chapter-1", problems[0])
        self.assertIn("Is it any good?", problems[0])
        self.assertIn("4:16", problems[0])
        short, short_voices = section(("a", 60.0, None))
        self.assertEqual(pacing.too_long(Video(name="v", dir=Path("."), title="t", music=None, lines=short.lines,
                                               citations={}, sections=(short,)), short_voices), [])


if __name__ == "__main__":
    unittest.main()
