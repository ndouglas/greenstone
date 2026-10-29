import json
import subprocess
import tempfile
import unittest
from pathlib import Path

import numpy as np
import soundfile
from manim import Dot, FadeIn, Text, tempconfig

from series.scene import CREDITS_SECONDS, NarratedScene, ScriptOrderError, StoryboardScene
from series.title import title_card
from series.script import Section, load_video
from series.voice import Voice

RATE = 24000
SCRIPT = '''
title = "t"
[[line]]
id = "a"
text = "One."
framing = true
[[line]]
id = "b"
text = "Two."
framing = true
'''


def fixtures(lengths, script=SCRIPT):
    d = Path(tempfile.mkdtemp())
    (d / "001-test").mkdir()
    (d / "001-test" / "script.toml").write_text(script)
    voices = {}
    for line_id, seconds in lengths.items():
        path = d / f"{line_id}.wav"
        soundfile.write(path, np.zeros(int(seconds * RATE), dtype=np.float32), RATE)
        voices[line_id] = Voice(line_id, path, seconds, "synth")
    return load_video(d / "001-test"), voices, d


def render(cls, media):
    # 60 fps, as the videos are, so pads land on whole frames; tiny frames for speed.
    with tempconfig({"quality": "low_quality", "frame_rate": 60, "pixel_width": 160, "pixel_height": 90,
                     "media_dir": str(media), "disable_caching": True,
                     "verbosity": "ERROR", "progress_bar": "none"}):
        scene = cls()
        scene.render()
        return Path(scene._get_manager().file_writer.movie_file_path)


def probe(path, entries):
    out = subprocess.run(["ffprobe", "-v", "error", "-show_entries", entries, "-of", "json", str(path)],
                         capture_output=True, text=True, check=True).stdout
    return json.loads(out)


class Narration(unittest.TestCase):
    def test_a_line_lasts_as_long_as_its_voice(self):
        video, voices, d = fixtures({"a": 1.0, "b": 2.0})

        class S(NarratedScene):
            def construct(self):
                self.say("a")
                self.say("b")

        S.video, S.voices = video, voices
        movie = render(S, d / "media")
        info = probe(movie, "format=duration:stream=codec_type")
        # Each line is followed by 2 s of stillness.
        self.assertAlmostEqual(float(info["format"]["duration"]), 7.0, delta=0.1)
        self.assertIn("audio", [s["codec_type"] for s in info["streams"]])

    def test_animations_are_stretched_to_the_line(self):
        video, voices, d = fixtures({"a": 2.0, "b": 0.5})
        times = []

        class S(NarratedScene):
            def construct(self):
                self.say("a", FadeIn(Dot()))
                times.append(self.time)
                self.say("b")

        S.video, S.voices = video, voices
        render(S, d / "media")
        self.assertAlmostEqual(times[0], 4.0, delta=0.1)

    def test_an_unsaid_line_fails_the_render(self):
        video, voices, d = fixtures({"a": 0.5, "b": 0.5})

        class S(NarratedScene):
            def construct(self):
                self.say("a")

        S.video, S.voices = video, voices
        with self.assertRaises(ScriptOrderError) as e:
            render(S, d / "media")
        self.assertIn("b", str(e.exception))

    def test_lines_out_of_order_or_twice_fail(self):
        video, voices, d = fixtures({"a": 0.5, "b": 0.5})

        class Backwards(NarratedScene):
            def construct(self):
                self.say("b")
                self.say("a")

        class Twice(NarratedScene):
            def construct(self):
                self.say("a")
                self.say("a")
                self.say("b")

        for cls in (Backwards, Twice):
            cls.video, cls.voices = video, voices
            with self.assertRaises(ScriptOrderError):
                render(cls, d / "media")


class Placeholders(unittest.TestCase):
    def test_a_zero_pad_is_allowed(self):
        video, voices, d = fixtures({"a": 1.0, "b": 0.5})

        class S(NarratedScene):
            def construct(self):
                self.say("a", pad=0)
                self.say("b", pad=0)

        S.video, S.voices = video, voices
        self.assertAlmostEqual(float(probe(render(S, d / "media"), "format=duration")["format"]["duration"]),
                               1.5, delta=0.1)

    def test_placeholders_say_their_lines(self):
        video, voices, d = fixtures({"a": 1.0, "b": 0.5})

        class S(NarratedScene):
            def construct(self):
                self.placeholder("a")
                self.placeholder("b")

        S.video, S.voices = video, voices
        self.assertAlmostEqual(float(probe(render(S, d / "media"), "format=duration")["format"]["duration"]),
                               5.5, delta=0.1)

    def test_a_placeholder_for_an_unknown_line_names_it(self):
        video, voices, d = fixtures({"a": 0.5, "b": 0.5})

        class S(NarratedScene):
            def construct(self):
                self.placeholder("zzz")

        S.video, S.voices = video, voices
        with self.assertRaises(ScriptOrderError) as e:
            render(S, d / "media")
        self.assertIn("zzz", str(e.exception))


class Credits(unittest.TestCase):
    def test_credits_are_required_when_asked(self):
        video, voices, d = fixtures({"a": 0.5, "b": 0.5})

        class S(NarratedScene):
            require_credits = True

            def construct(self):
                self.say("a")
                self.say("b")

        S.video, S.voices = video, voices
        with self.assertRaises(ScriptOrderError) as e:
            render(S, d / "media")
        self.assertIn("credits", str(e.exception))

    def test_credits_come_after_every_line(self):
        video, voices, d = fixtures({"a": 0.5, "b": 0.5})

        class S(NarratedScene):
            def construct(self):
                self.say("a")
                self.roll_credits()
                self.say("b")

        S.video, S.voices = video, voices
        with self.assertRaises(ScriptOrderError) as e:
            render(S, d / "media")
        self.assertIn("b", str(e.exception))



CHAPTERS = '''
title = "t"
[[section]]
kind = "chapter"
title = "One"
[[section.line]]
id = "i1"
role = "intro"
text = "In."
framing = true
pause = 0.5
[[section.line]]
id = "c1"
role = "conclusion"
text = "Out."
framing = true
[[section]]
kind = "chapter"
title = "Two"
[[section.line]]
id = "i2"
role = "intro"
text = "In again."
framing = true
[[section.line]]
id = "c2"
role = "conclusion"
text = "Out again."
framing = true
'''


class Chapters(unittest.TestCase):
    def test_a_beat_can_shorten_its_stillness(self):
        video, voices, d = fixtures({"i1": 1.0, "c1": 1.0, "i2": 1.0, "c2": 1.0}, CHAPTERS)
        times = []

        class S(NarratedScene):
            def construct(self):
                self.show_title("chapter-1")
                self.say("i1")
                times.append(self.time)
                self.say("c1")
                self.show_title("chapter-2")
                self.say("i2")
                self.say("c2")

        S.video, S.voices = video, voices
        render(S, d / "media")
        self.assertAlmostEqual(times[0], 4.0 + 1.0 + 0.5, delta=0.1)

    def test_a_title_screen_is_silent_and_holds(self):
        video, voices, d = fixtures({"i1": 1.0, "c1": 1.0, "i2": 1.0, "c2": 1.0}, CHAPTERS)
        times = []

        class S(NarratedScene):
            def construct(self):
                self.show_title("chapter-1")
                # Nothing is said during it: no sound was added yet.
                times.append((self.time, len(self.said)))
                for line_id in ("i1", "c1"):
                    self.say(line_id)
                self.show_title("chapter-2")
                for line_id in ("i2", "c2"):
                    self.say(line_id)

        S.video, S.voices = video, voices
        movie = render(S, d / "media")
        self.assertAlmostEqual(times[0][0], 4.0, delta=0.05)
        self.assertEqual(times[0][1], 0)
        # Total: two titles, four 1 s voices, stillness 0.5 + 2 + 2 + 2.
        self.assertAlmostEqual(float(probe(movie, "format=duration")["format"]["duration"]), 8.0 + 4.0 + 6.5, delta=0.1)

    def test_a_title_screen_starts_from_a_clear_screen(self):
        video, voices, d = fixtures({"i1": 0.5, "c1": 0.5, "i2": 0.5, "c2": 0.5}, CHAPTERS)
        left = []

        class S(NarratedScene):
            def construct(self):
                self.show_title("chapter-1")
                dot = Dot()
                self.add(dot)
                self.say("i1")
                self.say("c1")
                self.show_title("chapter-2")
                left.append(dot in self.mobjects)
                self.say("i2")
                self.say("c2")

        S.video, S.voices = video, voices
        render(S, d / "media")
        self.assertEqual(left, [False])

    def test_a_sections_first_beat_needs_its_title_screen(self):
        video, voices, d = fixtures({"i1": 0.5, "c1": 0.5, "i2": 0.5, "c2": 0.5}, CHAPTERS)

        class NoTitle(NarratedScene):
            def construct(self):
                self.say("i1")

        class TwoTitles(NarratedScene):
            def construct(self):
                self.show_title("chapter-1")
                self.show_title("chapter-1")

        class Unknown(NarratedScene):
            def construct(self):
                self.show_title("chapter-9")

        for cls in (NoTitle, TwoTitles, Unknown):
            cls.video, cls.voices = video, voices
            with self.assertRaises(ScriptOrderError) as e:
                render(cls, d / "media")
            self.assertIn("chapter-", str(e.exception))

    def test_the_storyboard_shows_every_title(self):
        video, voices, d = fixtures({"i1": 0.5, "c1": 0.5, "i2": 0.5, "c2": 0.5}, CHAPTERS)

        class S(StoryboardScene):
            pass

        S.video, S.voices = video, voices
        expected = 2 * 4.0 + 4 * 0.5 + (0.5 + 2.0 + 2.0 + 2.0) + CREDITS_SECONDS
        self.assertAlmostEqual(float(probe(render(S, d / "media"), "format=duration")["format"]["duration"]),
                               expected, delta=0.2)

    def test_title_card_numbers_only_chapters(self):
        video = load_video(fixtures({}, CHAPTERS)[2] / "001-test")
        # Manim's Text keeps its words without their spaces.
        texts = lambda card: [m.text for m in card.get_family() if isinstance(m, Text)]
        chapter = title_card(video.sections[0], [])
        self.assertIn("Chapter1", texts(chapter))
        intro = Section(id="introduction", kind="introduction", number=None, title="Introduction", lines=())
        self.assertFalse(any("Chapter" in t for t in texts(title_card(intro, []))))


    def test_the_wall_keeps_each_pictures_shape(self):
        from manim import Rectangle
        from series.title import wall
        tile = Rectangle(width=2.0, height=1.0)
        for row in wall([tile]):
            for slot in row:
                picture = slot[-1]
                self.assertAlmostEqual(picture.width / picture.height, 2.0, delta=0.01)


if __name__ == "__main__":
    unittest.main()
