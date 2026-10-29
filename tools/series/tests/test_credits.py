import tempfile
import unittest
from pathlib import Path

from series.credits import DISCLAIMER, credits, description
from series.script import load_video

CITATIONS = '''
[k]
title = "The perceptron"
authors = "F. Rosenblatt"
year = 1958
url = "https://doi.org/10.1037/h0042519"
[j]
title = "A logical calculus"
authors = "W. McCulloch and W. Pitts"
year = 1943
url = "https://example.org/mp"
'''
SCRIPT = '''
title = "One neuron"
[[line]]
id = "a"
text = "One."
cite = ["j"]
[[line]]
id = "b"
text = "Two."
cite = ["k"]
[[line]]
id = "c"
text = "Three."
cite = ["k", "j"]
'''


def video():
    d = Path(tempfile.mkdtemp()) / "001-test"
    d.mkdir()
    (d / "script.toml").write_text(SCRIPT)
    (d / "citations.toml").write_text(CITATIONS)
    return load_video(d)


class Description(unittest.TestCase):
    def test_description_lists_every_cited_source_once(self):
        text = description(video())
        for part in ("The perceptron", "F. Rosenblatt", "1958", "https://doi.org/10.1037/h0042519"):
            self.assertEqual(text.count(part), 1, part)
        self.assertLess(text.index("McCulloch"), text.index("Rosenblatt"))

    def test_description_ends_with_the_disclaimer(self):
        self.assertEqual(
            DISCLAIMER,
            "I'm not an expert, just a guy learning this stuff, in public. If you spot a mistake, please tell me.",
        )
        self.assertTrue(description(video()).rstrip().endswith(DISCLAIMER))


class Card(unittest.TestCase):
    def test_the_disclaimer_stays_readable_with_many_sources(self):
        many = "".join(f'[s{i}]\ntitle = "Source {i}"\nauthors = "A. Author"\nyear = 2000\nurl = "https://x/{i}"\n'
                       for i in range(30))
        lines = "".join(f'[[line]]\nid = "l{i}"\ntext = "Line."\ncite = ["s{i}"]\n' for i in range(30))
        d = Path(tempfile.mkdtemp()) / "001-many"
        d.mkdir()
        (d / "script.toml").write_text('title = "t"\n' + lines)
        (d / "citations.toml").write_text(many)
        crowded, sparse = credits(load_video(d))[-1], credits(video())[-1]
        self.assertAlmostEqual(crowded.height, sparse.height, delta=sparse.height * 0.01)


if __name__ == "__main__":
    unittest.main()
