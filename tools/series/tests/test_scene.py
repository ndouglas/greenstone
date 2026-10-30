import json
import subprocess
import tempfile
import unittest
from pathlib import Path

import numpy as np
import soundfile
from manim import Dot, FadeIn, ImageMobject, Text, VGroup, tempconfig

from series import style
from series.credits import PAGE_SECONDS, credits_seconds
from series.scene import NarratedScene, ScriptOrderError, StoryboardScene
from series.title import board_tiles, title_card
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


def fixtures(lengths, script=SCRIPT, citations=None):
    d = Path(tempfile.mkdtemp())
    (d / "001-test").mkdir()
    (d / "001-test" / "script.toml").write_text(script)
    if citations:
        (d / "001-test" / "citations.toml").write_text(citations)
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

    def test_the_credits_turn_their_pages(self):
        sources = "".join(f'[s{i}]\ntitle = "Source {i}"\nauthors = "A. Author"\nyear = 2000\nurl = "https://x/{i}"\n'
                          for i in range(30))
        script = 'title = "t"\n[[line]]\nid = "a"\ntext = "One."\ncite = [' + ", ".join(f'"s{i}"' for i in range(30)) + "]\n"
        video, voices, d = fixtures({"a": 0.5}, script, sources)

        expected = []

        class S(NarratedScene):
            def construct(self):
                # Measured here: Manim wraps text to the render's pixel width.
                expected.append(credits_seconds(self.video))
                self.say("a")
                self.roll_credits()

        S.video, S.voices = video, voices
        movie = render(S, d / "media")
        self.assertGreater(expected[0], PAGE_SECONDS)
        self.assertAlmostEqual(float(probe(movie, "format=duration")["format"]["duration"]),
                               0.5 + 2.0 + expected[0], delta=0.1)


def beat(id, role=None, extra=""):
    r = f'role = "{role}"\n' if role else ""
    return f'[[section.line]]\nid = "{id}"\n{r}text = "{id}."\nboard = "{id} drawn"\nframing = true\n{extra}'


CHAPTERS = ('title = "t"\n'
            '[[section]]\nkind = "introduction"\ntitle = "Introduction"\n' + beat("h") +
            '[[section]]\nkind = "chapter"\ntitle = "One"\n' + beat("i1", "intro", "pause = 0.5\n") + beat("c1", "conclusion") +
            '[[section]]\nkind = "chapter"\ntitle = "Two"\n' + beat("i2", "intro") + beat("c2", "conclusion") +
            '[[section]]\nkind = "conclusions"\ntitle = "Conclusions"\n' + beat("t"))
BEATS = {"introduction": ("h",), "chapter-1": ("i1", "c1"), "chapter-2": ("i2", "c2"), "conclusions": ("t",)}


def chapters(seconds=0.5):
    return fixtures({i: seconds for ids in BEATS.values() for i in ids}, CHAPTERS)


def say_all(scene, after_title=None):
    """Title and say every section in order; `after_title(id)` runs after each title screen."""
    for section_id, ids in BEATS.items():
        scene.show_title(section_id)
        if after_title:
            after_title(section_id)
        for i in ids:
            scene.say(i)


class Chapters(unittest.TestCase):
    def test_a_beat_can_shorten_its_stillness(self):
        video, voices, d = chapters(1.0)
        times = []

        class S(NarratedScene):
            def construct(self):
                say_all(self, lambda sid: times.append(self.time) if sid == "chapter-1" else None)
                times.append(self.section_lengths["chapter-1"])

        S.video, S.voices = video, voices
        render(S, d / "media")
        # Chapter 1: its title, then i1 (1 s, then 0.5 s still), then c1 (1 s, then 2 s).
        self.assertAlmostEqual(times[1], 4.0 + 1.5 + 3.0, delta=0.05)

    def test_a_title_screen_is_silent_and_holds(self):
        video, voices, d = chapters(1.0)
        for v in voices.values():
            t = np.arange(int(v.seconds * RATE)) / RATE
            soundfile.write(v.path, (0.5 * np.sin(2 * np.pi * 440 * t)).astype(np.float32), RATE)
        times = []

        class S(NarratedScene):
            def construct(self):
                say_all(self, lambda sid: times.append((self.time, len(self.said))) if sid == "introduction" else None)

        S.video, S.voices = video, voices
        movie = render(S, d / "media")
        self.assertAlmostEqual(times[0][0], 4.0, delta=0.05)
        self.assertEqual(times[0][1], 0)
        # Four titles, six 1 s voices, stillness 2 + 0.5 + 2 + 2 + 2 + 2.
        self.assertAlmostEqual(float(probe(movie, "format=duration")["format"]["duration"]), 16.0 + 6.0 + 10.5, delta=0.1)
        # The title screen is silent; the first beat is not.
        pcm = subprocess.run(["ffmpeg", "-v", "error", "-i", str(movie), "-vn", "-ac", "1", "-ar", str(RATE),
                              "-f", "f32le", "-"], capture_output=True, check=True).stdout
        audio = np.frombuffer(pcm, dtype=np.float32)
        self.assertLess(np.abs(audio[: int(3.9 * RATE)]).max(), 1e-3)
        self.assertGreater(np.abs(audio[int(4.1 * RATE): int(4.9 * RATE)]).max(), 0.1)

    def test_a_title_screen_starts_from_a_clear_screen(self):
        video, voices, d = chapters()
        left = []

        class S(NarratedScene):
            def construct(self):
                dot = Dot()

                def check(section_id):
                    if section_id == "chapter-1":
                        self.add(dot)
                    elif section_id == "chapter-2":
                        left.append(dot in self.mobjects)

                say_all(self, check)

        S.video, S.voices = video, voices
        render(S, d / "media")
        self.assertEqual(left, [False])

    def test_a_sections_first_beat_needs_its_title_screen(self):
        video, voices, d = chapters()

        class NoTitle(NarratedScene):
            def construct(self):
                self.say("h")

        class NoLaterTitle(NarratedScene):
            def construct(self):
                for section_id in ("introduction", "chapter-1"):
                    self.show_title(section_id)
                    for i in BEATS[section_id]:
                        self.say(i)
                self.say("i2")

        class TwoTitles(NarratedScene):
            def construct(self):
                self.show_title("introduction")
                self.show_title("introduction")

        class Unknown(NarratedScene):
            def construct(self):
                self.show_title("chapter-9")

        for cls, name in ((NoTitle, "introduction"), (NoLaterTitle, "chapter-2"), (TwoTitles, "introduction"),
                          (Unknown, "chapter-9")):
            cls.video, cls.voices = video, voices
            with self.assertRaises(ScriptOrderError) as e:
                render(cls, d / "media")
            self.assertIn(name, str(e.exception))

    def test_titles_come_in_order_just_before_their_sections(self):
        video, voices, d = chapters()

        class Early(NarratedScene):
            def construct(self):
                self.show_title("introduction")
                self.show_title("chapter-1")

        class Skipped(NarratedScene):
            def construct(self):
                self.show_title("chapter-2")

        class MidChapter(NarratedScene):
            def construct(self):
                self.show_title("introduction")
                self.say("h")
                self.show_title("chapter-1")
                self.say("i1")
                self.show_title("chapter-2")

        for cls in (Early, Skipped, MidChapter):
            cls.video, cls.voices = video, voices
            with self.assertRaises(ScriptOrderError) as e:
                render(cls, d / "media")
            self.assertIn("chapter-", str(e.exception))

    def test_the_storyboard_shows_every_title(self):
        video, voices, d = chapters()

        class S(StoryboardScene):
            pass

        S.video, S.voices = video, voices
        expected = 4 * 4.0 + 6 * 0.5 + (0.5 + 5 * 2.0) + credits_seconds(video)
        self.assertAlmostEqual(float(probe(render(S, d / "media"), "format=duration")["format"]["duration"]),
                               expected, delta=0.2)

    def test_title_card_numbers_only_chapters(self):
        video = chapters()[0]
        # Manim's Text keeps its words without their spaces.
        texts = lambda card: [m.text for m in card.get_family() if isinstance(m, Text)]
        chapter = title_card(video.sections[1], [])
        self.assertIn("Chapter1", texts(chapter))
        intro = Section(id="introduction", kind="introduction", number=None, title="Introduction", lines=())
        self.assertFalse(any("Chapter" in t for t in texts(title_card(intro, []))))

    def test_titles_are_in_charter(self):
        video = chapters()[0]
        card = title_card(video.sections[1], board_tiles(video.sections[1]))
        words = [m for m in card.get_family() if isinstance(m, Text)]
        self.assertTrue(words)
        self.assertTrue(all(m.font == style.FONT for m in words))
        # The chapter number is italic, the title roman.
        slant = {m.text: m.slant for m in words}
        self.assertEqual(slant["Chapter1"], "ITALIC")
        self.assertEqual(slant["One"], "NORMAL")

    def test_the_wall_leaves_the_ground_showing(self):
        from manim import Rectangle
        from series.title import wall
        for row in wall([Rectangle(width=2.0, height=1.0)]):
            for slot in row:
                self.assertEqual(slot[0].fill_opacity, 0)

    def test_every_scene_starts_on_the_ground(self):
        with tempconfig({"pixel_width": 160, "pixel_height": 90, "verbosity": "ERROR"}):
            scene = NarratedScene()
            scene.setup()
            self.assertIsInstance(scene.mobjects[0], ImageMobject)
            self.assertIs(scene.mobjects[0], scene.ground)
            scene.add(Dot())
            scene.clear()
            self.assertEqual(scene.mobjects, [scene.ground])

    def test_placeholders_are_in_charter(self):
        video, voices, d = fixtures({"a": 0.2, "b": 0.2})
        seen = []

        class S(NarratedScene):
            def construct(self):
                original = self.say
                self.say = lambda *a, **k: (seen.extend(m for m in self.mobjects if isinstance(m, VGroup)), original(*a, **k))
                self.placeholder("a")
                self.placeholder("b")

        S.video, S.voices = video, voices
        render(S, d / "media")
        texts = [t for g in seen for t in g.get_family() if isinstance(t, Text)]
        self.assertTrue(texts)
        self.assertTrue(all(t.font == style.FONT for t in texts))

    def test_a_placeholder_slants_its_header_and_note_and_sets_its_line_roman(self):
        script = SCRIPT.replace('framing = true\n[[line]]\nid = "b"', 'board = "a drawing"\nframing = true\n[[line]]\nid = "b"')
        video, voices, d = fixtures({"a": 0.2, "b": 0.2}, script)
        seen = []

        class S(NarratedScene):
            def construct(self):
                original = self.say
                self.say = lambda *a, **k: (seen.extend(m for m in self.mobjects if isinstance(m, VGroup)), original(*a, **k))
                self.placeholder("a")
                self.placeholder("b")

        S.video, S.voices = video, voices
        render(S, d / "media")
        words = {t.text: t for g in seen for t in g.get_family() if isinstance(t, Text)}
        self.assertEqual(words["One."].slant, "NORMAL")  # what is said, in roman
        header = next(t for k, t in words.items() if k.startswith("a") and len(k) == 1)
        self.assertEqual(header.slant, "ITALIC")
        note = next(t for k, t in words.items() if k.startswith("Animation:"))
        self.assertEqual(note.slant, "ITALIC")
        self.assertEqual(note.color.to_hex().upper(), style.ASH)  # cool means below zero; this is neither

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
