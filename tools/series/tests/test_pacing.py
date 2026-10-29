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
        sec, _ = section(("a", 1.0, None), title="Is it any good?")
        video = Video(name="v", dir=Path("."), title="t", music=None, lines=sec.lines, citations={}, sections=(sec,))
        problems = pacing.too_long(video, {"chapter-1": 256.0})
        self.assertEqual(len(problems), 1)
        self.assertIn("chapter-1", problems[0])
        self.assertIn("Is it any good?", problems[0])
        self.assertIn("4:16", problems[0])
        self.assertEqual(pacing.too_long(video, {"chapter-1": 60.0}), [])

    def test_lengths_estimates_every_section(self):
        sec, voices = section(("a", 10.0, 0.5), ("b", 20.0, None))
        video = Video(name="v", dir=Path("."), title="t", music=None, lines=sec.lines, citations={}, sections=(sec,))
        self.assertEqual(pacing.lengths(video, voices), {"chapter-1": pacing.section_seconds(sec, voices)})

    def test_clock_never_reads_under_the_length(self):
        # A section a fraction over the limit must not read as exactly the limit.
        self.assertEqual(pacing.clock(240.4), "4:01")
        self.assertEqual(pacing.clock(240.0), "4:00")
        self.assertEqual(pacing.clock(59.2), "1:00")


if __name__ == "__main__":
    unittest.main()
