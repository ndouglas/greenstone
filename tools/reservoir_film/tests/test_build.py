import unittest

from reservoir_film import build
from reservoir_film.model import Caption, Chapter, Context, Shot

CTX = Context(takes=None, numbers={})


def chapter(n, seconds, title="T"):
    return Chapter(n, title, "", [Shot(name="s", seconds=seconds, draw=None, captions=[])])


class Build(unittest.TestCase):
    def test_chapter_starts_include_title_cards_after_the_cold_open(self):
        chs = [chapter(0, 39), chapter(1, 100), chapter(2, 60)]
        self.assertEqual(build.chapter_starts(chs, CTX), [0.0, 39.0, 143.0])

    def test_youtube_chapters(self):
        chs = [chapter(0, 39, "Cold open"), chapter(1, 100, "Mario is just numbers"), chapter(2, 600, "Long")]
        self.assertEqual(build.youtube_chapters(chs, CTX),
                         "0:00 Cold open\n0:39 1. Mario is just numbers\n2:23 2. Long\n")

    def test_ffmetadata_has_one_chapter_each(self):
        text = build.ffmetadata([chapter(0, 10), chapter(1, 20)], CTX)
        self.assertTrue(text.startswith(";FFMETADATA1"))
        self.assertEqual(text.count("[CHAPTER]"), 2)
        self.assertIn("START=10000", text)
        self.assertIn("END=34000", text)

    def test_segment_key_changes_with_the_chapter_and_its_takes(self):
        a = build.segment_key(chapter(1, 10), CTX, sources="code v1", take_times={"t": 1.0})
        self.assertEqual(a, build.segment_key(chapter(1, 10), CTX, sources="code v1", take_times={"t": 1.0}))
        self.assertNotEqual(a, build.segment_key(chapter(1, 11), CTX, sources="code v1", take_times={"t": 1.0}))
        self.assertNotEqual(a, build.segment_key(chapter(1, 10), CTX, sources="code v2", take_times={"t": 1.0}))
        self.assertNotEqual(a, build.segment_key(chapter(1, 10), CTX, sources="code v1", take_times={"t": 2.0}))
