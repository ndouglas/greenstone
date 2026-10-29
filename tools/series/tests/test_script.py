import tempfile
import unittest
from pathlib import Path

from series.script import REPO, VIDEOS, ScriptError, load_video

CITATIONS = """
[rosenblatt1958]
title = "The perceptron"
authors = "F. Rosenblatt"
year = 1958
url = "https://example.org/perceptron"
"""


def video(script, citations=CITATIONS):
    d = Path(tempfile.mkdtemp()) / "001-test"
    d.mkdir()
    (d / "script.toml").write_text(script)
    (d / "citations.toml").write_text(citations)
    return d


class Script(unittest.TestCase):
    def test_loads_lines_in_order(self):
        v = load_video(video('''
title = "A test"
[[line]]
id = "first"
text = "One."
framing = true
[[line]]
id = "second"
text = "Two."
cite = ["rosenblatt1958"]
'''))
        self.assertEqual(v.name, "001-test")
        self.assertEqual(v.title, "A test")
        self.assertEqual([l.id for l in v.lines], ["first", "second"])
        self.assertEqual(v.lines[1].cite, ("rosenblatt1958",))
        self.assertEqual(v.citations["rosenblatt1958"].year, 1958)

    def test_spoken_text_is_what_gets_voiced(self):
        v = load_video(video('''
title = "t"
[[line]]
id = "a"
text = "w·x + b"
spoken = "w times x, plus b"
framing = true
[[line]]
id = "b"
text = "Plain."
framing = true
'''))
        self.assertEqual(v.lines[0].voiced, "w times x, plus b")
        self.assertEqual(v.lines[1].voiced, "Plain.")

    def test_board_notes_are_optional(self):
        v = load_video(video('''
title = "t"
[[line]]
id = "a"
text = "One."
board = "a line tilts"
framing = true
[[line]]
id = "b"
text = "Two."
framing = true
'''))
        self.assertEqual(v.lines[0].board, "a line tilts")
        self.assertIsNone(v.lines[1].board)

    def test_every_line_needs_a_citation_or_framing(self):
        with self.assertRaises(ScriptError) as e:
            load_video(video('''
title = "t"
[[line]]
id = "bare-claim"
text = "Neurons are lines."
'''))
        self.assertIn("bare-claim", str(e.exception))

    def test_unknown_citation_key_is_an_error(self):
        with self.assertRaises(ScriptError) as e:
            load_video(video('''
title = "t"
[[line]]
id = "a"
text = "One."
cite = ["nope"]
'''))
        self.assertIn("nope", str(e.exception))
        self.assertIn("a", str(e.exception))

    def test_line_ids_are_unique_and_simple(self):
        for script in ['''
title = "t"
[[line]]
id = "a"
text = "One."
framing = true
[[line]]
id = "a"
text = "Two."
framing = true
''', '''
title = "t"
[[line]]
id = "Intro 1"
text = "One."
framing = true
''']:
            with self.assertRaises(ScriptError):
                load_video(video(script))

    def test_music_is_resolved_against_the_repo(self):
        base = '''
[[line]]
id = "a"
text = "One."
framing = true
'''
        v = load_video(video('title = "t"\nmusic = "target/x.wav"\n' + base))
        self.assertEqual(v.music, REPO / "target" / "x.wav")
        self.assertIsNone(load_video(video('title = "t"\n' + base)).music)


class Numbers(unittest.TestCase):
    def test_numbers_fill_the_script(self):
        d = video('''
title = "t"
[[line]]
id = "a"
text = "It was right {accuracy} of the time."
spoken = "It was right {accuracy_spoken} of the time."
framing = true
''')
        (d / "numbers.json").write_text('{"accuracy": "93%", "accuracy_spoken": "ninety-three percent"}')
        line = load_video(d).lines[0]
        self.assertEqual(line.text, "It was right 93% of the time.")
        self.assertEqual(line.voiced, "It was right ninety-three percent of the time.")

    def test_numbers_fill_the_boards(self):
        d = video('''
title = "t"
[[line]]
id = "a"
text = "It was right {accuracy} of the time."
board = "A bar at {accuracy}"
framing = true
''')
        (d / "numbers.json").write_text('{"accuracy": "93%"}')
        self.assertEqual(load_video(d).lines[0].board, "A bar at 93%")

    def test_every_video_loads(self):
        dirs = [d for d in VIDEOS.iterdir() if (d / "script.toml").exists()]
        self.assertTrue(dirs)
        for d in dirs:
            with self.subTest(video=d.name):
                load_video(d)

    def test_an_unknown_number_names_the_line(self):
        d = video('''
title = "t"
[[line]]
id = "result"
text = "It was right {accuracy} of the time."
framing = true
''')
        with self.assertRaises(ScriptError) as e:
            load_video(d)
        self.assertIn("result", str(e.exception))
        self.assertIn("accuracy", str(e.exception))



def beat(id, role=None, extra=""):
    r = f'role = "{role}"\n' if role else ""
    return f'[[section.line]]\nid = "{id}"\n{r}text = "{id}."\nframing = true\n{extra}'


def chapter(title, *ids):
    first, *rest = ids
    body = beat(first, "intro") + "".join(beat(i) for i in rest[:-1]) + beat(rest[-1], "conclusion")
    return f'[[section]]\nkind = "chapter"\ntitle = "{title}"\n{body}'


SECTIONED = (
    'title = "t"\n'
    '[[section]]\nkind = "introduction"\ntitle = "Introduction"\n' + beat("hello") +
    chapter("One", "a1", "a2", "a3") + chapter("Two", "b1", "b2") +
    '[[section]]\nkind = "conclusions"\ntitle = "Conclusions"\n' + beat("tldr") +
    '[[line]]\nid = "closing"\ntext = "Bye."\nframing = true\n'
)


class Sections(unittest.TestCase):
    def test_sections_hold_their_beats_in_order(self):
        v = load_video(video(SECTIONED))
        self.assertEqual([s.id for s in v.sections], ["introduction", "chapter-1", "chapter-2", "conclusions"])
        self.assertEqual([s.number for s in v.sections], [None, 1, 2, None])
        self.assertEqual([s.title for s in v.sections], ["Introduction", "One", "Two", "Conclusions"])
        self.assertEqual([l.id for l in v.lines], ["hello", "a1", "a2", "a3", "b1", "b2", "tldr", "closing"])
        self.assertEqual([l.section for l in v.lines],
                         ["introduction"] + ["chapter-1"] * 3 + ["chapter-2"] * 2 + ["conclusions", None])
        self.assertEqual([l.id for l in v.sections[1].lines], ["a1", "a2", "a3"])
        self.assertEqual(v.sections[1].lines[0].role, "intro")
        self.assertEqual(v.sections[1].lines[-1].role, "conclusion")

    def test_a_chapter_needs_its_intro_first_and_conclusion_last(self):
        no_intro = 'title = "t"\n[[section]]\nkind = "chapter"\ntitle = "One"\n' + beat("a") + beat("b", "conclusion")
        no_end = 'title = "t"\n[[section]]\nkind = "chapter"\ntitle = "One"\n' + beat("a", "intro") + beat("b")
        for script in (no_intro, no_end):
            with self.assertRaises(ScriptError) as e:
                load_video(video(script))
            self.assertIn("chapter-1", str(e.exception))

    def test_sections_must_be_in_their_places(self):
        intro = '[[section]]\nkind = "introduction"\ntitle = "I"\n' + beat("i")
        ends = '[[section]]\nkind = "conclusions"\ntitle = "C"\n' + beat("c")
        for script in ('title = "t"\n' + ends + chapter("One", "a", "b"),
                       'title = "t"\n' + intro + intro.replace('"i"', '"j"'),
                       'title = "t"\n[[section]]\nkind = "appendix"\ntitle = "A"\n' + beat("x"),
                       'title = "t"\n[[section]]\nkind = "chapter"\n' + beat("a", "intro") + beat("b", "conclusion")):
            with self.assertRaises(ScriptError):
                load_video(video(script))

    def test_roles_belong_to_chapters(self):
        script = 'title = "t"\n[[section]]\nkind = "introduction"\ntitle = "I"\n' + beat("i", "intro")
        with self.assertRaises(ScriptError):
            load_video(video(script))

    def test_a_beat_can_set_its_own_pause(self):
        script = SECTIONED.replace('id = "a2"\ntext = "a2."\nframing = true\n', 'id = "a2"\ntext = "a2."\nframing = true\npause = 0.5\n')
        v = load_video(video(script))
        self.assertEqual(next(l for l in v.lines if l.id == "a2").pause, 0.5)
        self.assertIsNone(next(l for l in v.lines if l.id == "a1").pause)

    def test_a_script_without_sections_still_loads(self):
        v = load_video(video('title = "t"\n[[line]]\nid = "a"\ntext = "A."\nframing = true\n'))
        self.assertEqual(v.sections, ())
        self.assertEqual([l.id for l in v.lines], ["a"])
        self.assertIsNone(v.lines[0].section)

    def test_numbers_fill_section_beats(self):
        d = video(SECTIONED.replace('text = "a2."', 'text = "{n}."'))
        (d / "numbers.json").write_text('{"n": "94%"}')
        self.assertEqual(next(l for l in load_video(d).lines if l.id == "a2").text, "94%.")


if __name__ == "__main__":
    unittest.main()
