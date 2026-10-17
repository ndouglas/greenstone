import unittest

from reservoir_film.model import TAKES_DIR, Context, Takes, check_captions, load_numbers, shot_seconds

HAVE_TAKES = (TAKES_DIR / "bank_drive" / "take.json").exists()


def chapter(n):
    from reservoir_film.chapters import CHAPTERS
    found = [c for c in CHAPTERS if c.number == n]
    if not found:
        raise AssertionError(f"chapter {n} is not in CHAPTERS")
    return found[0]


@unittest.skipUnless(HAVE_TAKES, "record the takes first: cargo run --release --example reservoir_film")
class Chapters(unittest.TestCase):
    def check(self, n):
        ctx = Context(takes=Takes(), numbers=load_numbers())
        ch = chapter(n)
        self.assertEqual(check_captions(ch, ctx), [])
        for shot in ch.shots:
            last = int(shot_seconds(shot, ctx) * 60) - 1
            for i in (0, last // 2, last):
                img = shot.draw(ctx, i / 60, i)
                self.assertEqual(img.size, (1280, 720), f"{shot.name} frame {i}")

    def test_chapter_0(self):
        self.check(0)

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

    def test_chapter_9(self):
        self.check(9)

    def test_all_ten_chapters_in_order(self):
        from reservoir_film.chapters import CHAPTERS
        self.assertEqual([c.number for c in CHAPTERS], list(range(10)))
