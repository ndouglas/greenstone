import re
import tempfile
import unittest
from pathlib import Path

from manim import Text, config

from series import style
from series.credits import DISCLAIMER, PAGE_SECONDS, credit_pages, credits_seconds, description
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


def many(n=30):
    sources = "".join(f'[s{i}]\ntitle = "Source {i}"\nauthors = "A. Author"\nyear = 2000\nurl = "https://x/{i}"\n'
                      for i in range(n))
    lines = "".join(f'[[line]]\nid = "l{i}"\ntext = "Line."\ncite = ["s{i}"]\n' for i in range(n))
    d = Path(tempfile.mkdtemp()) / "001-many"
    d.mkdir()
    (d / "script.toml").write_text('title = "t"\n' + lines)
    (d / "citations.toml").write_text(sources)
    return load_video(d)


def texts(page):
    # Manim's Text keeps its words without their spaces.
    return [m.text for m in page.get_family() if isinstance(m, Text)]


class Pages(unittest.TestCase):
    def test_a_few_sources_fit_one_page_with_the_disclaimer(self):
        pages = credit_pages(video())
        self.assertEqual(len(pages), 1)
        self.assertTrue(any("Rosenblatt" in t for t in texts(pages[0])))
        self.assertTrue(any(t.startswith("I'mnotanexpert") for t in texts(pages[0])))

    def test_many_sources_are_split_across_pages(self):
        pages = credit_pages(many())
        self.assertGreater(len(pages), 2)
        found = [int(m) for page in pages for t in texts(page) for m in re.findall(r"Source(\d+)\.", t)]
        self.assertEqual(sorted(found), list(range(30)))
        for page in pages:
            self.assertLessEqual(page.height, config.frame_height - 1.0)
            self.assertLessEqual(page.width, config.frame_width - 1.0)
        self.assertTrue(any(t.startswith("I'mnotanexpert") for t in texts(pages[-1])))
        self.assertFalse(any(t.startswith("I'mnotanexpert") for page in pages[:-1] for t in texts(page)))
        numbered = [page for page in pages if texts(page)[0].startswith("Sources")]
        self.assertEqual(texts(pages[0])[0], f"Sources(1of{len(numbered)})")

    def test_credits_are_in_charter(self):
        pages = credit_pages(many())
        for page in pages:
            words = [m for m in page.get_family() if isinstance(m, Text)]
            self.assertTrue(all(m.font == style.FONT for m in words))
            heading = [m for m in words if m.text.startswith("Sources")]
            if heading:  # the lone disclaimer page has none
                self.assertEqual(heading[0].slant, "ITALIC")
        last = [m for m in pages[-1].get_family() if isinstance(m, Text)]
        self.assertEqual(last[-1].slant, "ITALIC")  # the disclaimer

    def test_the_sources_themselves_are_roman(self):
        entries = [m for page in credit_pages(many()) for m in page.get_family()
                   if isinstance(m, Text) and re.search(r"Source\d+\.$", m.text)]
        self.assertEqual(len(entries), 30)
        self.assertTrue(all(m.slant == "NORMAL" for m in entries))

    def test_every_page_is_the_same_readable_size(self):
        def row(page, i):
            return next(m for m in page.get_family() if isinstance(m, Text) and m.text.endswith(f"Source{i}."))
        crowded, sparse = credit_pages(many()), credit_pages(many(1))
        heights = [row(page, i).height for page in crowded for i in range(30)
                   if any(t.endswith(f"Source{i}.") for t in texts(page))]
        self.assertAlmostEqual(max(heights), min(heights), delta=min(heights) * 0.01)
        self.assertAlmostEqual(heights[0], row(sparse[0], 0).height, delta=heights[0] * 0.01)

    def test_the_credits_hold_each_page(self):
        self.assertEqual(credits_seconds(many()), len(credit_pages(many())) * PAGE_SECONDS)
        self.assertEqual(credits_seconds(video()), PAGE_SECONDS)


if __name__ == "__main__":
    unittest.main()
