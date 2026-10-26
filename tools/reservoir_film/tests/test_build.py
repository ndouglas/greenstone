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
        text = build.ffmetadata([chapter(0, 10), chapter(1, 20)], CTX, "T")
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

    def test_segment_key_changes_with_the_numbers(self):
        a = build.segment_key(chapter(1, 10), Context(takes=None, numbers={"n": 1}), sources="v", take_times={})
        b = build.segment_key(chapter(1, 10), Context(takes=None, numbers={"n": 2}), sources="v", take_times={})
        self.assertNotEqual(a, b)

    def test_sources_include_modules_a_chapter_borrows_from(self):
        from reservoir_film.film import TETRIS
        c00 = (build.HERE / "chapters" / "c00_cold_open.py").read_text()
        self.assertIn(c00, build._sources(TETRIS))


def two_shots():
    return Chapter(1, "T", "", [Shot(name="a", seconds=10, draw=None), Shot(name="b", seconds=20, draw=None)])


class CuePlan(unittest.TestCase):
    def setUp(self):
        from reservoir_film.music.core import Cue, Instrument, Part, Section
        one_bar = Section([Part("p1", Instrument(), "C4/1")])
        self.cues = {n: Cue(name=n, title=n, bpm=120, loop=one_bar, ending=one_bar) for n in ("x", "y", "z")}

    def plan(self, chapters, chapter_cues):
        return build.cue_plan(chapters, chapter_cues, self.cues, CTX)

    def test_spans_split_a_chapter_at_shots_and_offsets(self):
        for offset in (5, lambda ctx: 5):
            plan = self.plan([two_shots()], {1: [("a", 0, "x"), ("b", offset, "y")]})
            self.assertEqual([(c.name, s, label) for c, s, label in plan], [("x", 19.0, "1/a"), ("y", 15.0, "1/b")])

    def test_plan_covers_the_film_exactly(self):
        chs = [two_shots(), chapter(2, 30)]
        plan = self.plan(chs, {1: [("a", 0, "x"), ("b", 5, "y")], 2: "z"})
        self.assertAlmostEqual(sum(s for _, s, _ in plan), sum(build.chapter_seconds(c, CTX) for c in chs))
        self.assertEqual(plan[-1][2], "2/s")

    def test_a_span_must_start_its_chapter(self):
        for spans in ([("b", 0, "x")], [("a", 1, "x")]):
            with self.assertRaises(ValueError):
                self.plan([two_shots()], {1: spans})

    def test_strict_film_rejects_an_unfit_span(self):
        plan = [(self.cues["x"], 0.5, "1/a"), (self.cues["y"], 60.0, "1/b")]
        self.assertEqual(build.unfit_spans(plan), ["1/a (x)"])


@unittest.skipUnless(__import__("reservoir_film.model", fromlist=["TAKES_DIR"]).TAKES_DIR.exists(), "no Tetris takes")
class TetrisPlan(unittest.TestCase):
    def test_tetris_plan_is_unchanged(self):
        from reservoir_film.film import TETRIS
        from reservoir_film.model import Takes, load_numbers
        ctx = Context(takes=Takes(), numbers=load_numbers())
        chapters = TETRIS.chapters()
        cues, chapter_cues, _ = TETRIS.cues()
        got = [(c.name, s) for c, s, _ in build.cue_plan(chapters, chapter_cues, cues, ctx)]
        self.assertEqual(got, [(cues[chapter_cues[ch.number]].name, build.chapter_seconds(ch, ctx)) for ch in chapters])
