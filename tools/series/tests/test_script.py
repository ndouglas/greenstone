import tempfile
import unittest
from pathlib import Path

from series.script import REPO, ScriptError, load_video

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


if __name__ == "__main__":
    unittest.main()
