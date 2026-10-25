import unittest

from punchout_film.film import PUNCHOUT
from reservoir_film.model import Context, Takes, caption_spans, check_captions, load_numbers, shot_seconds

HAVE_TAKES = (PUNCHOUT.takes_dir / "boot" / "take.json").exists()


def chapter(n):
    from punchout_film.chapters import CHAPTERS
    found = [c for c in CHAPTERS if c.number == n]
    if not found:
        raise AssertionError(f"chapter {n} is not in CHAPTERS")
    return found[0]


def context():
    return Context(takes=Takes(PUNCHOUT.takes_dir, PUNCHOUT.record_hint), numbers=load_numbers(PUNCHOUT.numbers))


@unittest.skipUnless(HAVE_TAKES, "record the takes first: cargo run --release --example punchout -- film-takes")
class Chapters(unittest.TestCase):
    def check(self, n):
        ctx = context()
        ch = chapter(n)
        self.assertEqual(check_captions(ch, ctx), [])
        for shot in ch.shots:
            last = int(shot_seconds(shot, ctx) * 60) - 1
            for i in (0, last // 2, last):
                img = shot.draw(ctx, i / 60, i)
                self.assertEqual(img.size, (1280, 720), f"{shot.name} frame {i}")

    def text(self, n):
        ctx = context()
        return " ".join(t for shot in chapter(n).shots for _, _, t in caption_spans(shot, ctx))

    def test_chapter_1(self):
        self.check(1)

    def test_chapter_2(self):
        self.check(2)

    def test_chapter_3(self):
        self.check(3)

    def test_chapter_4(self):
        self.check(4)

    def test_chapter_5(self):
        self.check(5)

    def test_chapter_6(self):
        self.check(6)

    def test_chapter_7(self):
        self.check(7)

    def test_chapter_8(self):
        self.check(8)

    def test_the_teachers_ko_is_said_with_its_real_length(self):
        self.assertIn("801 moves", self.text(3))
