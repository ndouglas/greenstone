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


@unittest.skipUnless(HAVE_TAKES, "record the takes first: cargo run --release --example reservoir_film")
class CaptionsMatchTakes(unittest.TestCase):
    def setUp(self):
        self.ctx = Context(takes=Takes(), numbers=load_numbers())

    def texts(self, n, shot_name):
        from reservoir_film.model import caption_spans
        shot = next(s for s in chapter(n).shots if s.name == shot_name)
        return " ".join(t for _, _, t in caption_spans(shot, self.ctx))

    def test_the_noise_shot_names_the_bank_it_shook(self):
        if self.ctx.takes["bank_noise"].meta["condition"] == "bank-tidy":
            self.assertIn("tidy", self.texts(6, "noise"))

    def test_the_tidy_caption_follows_the_higher_mean(self):
        conds = {c["name"]: c["mean_x"] for c in self.ctx.takes["bank_drive"].meta["conditions"]}
        text = self.texts(6, "tidy")
        if conds["bank-tetris"] > conds["bank-tidy"]:
            self.assertIn("plain four drove further", text)
        else:
            self.assertIn("helped", text)

    def test_the_teacher_is_not_said_to_clear_every_start(self):
        self.assertNotIn("clears the level, the same way every time", self.texts(4, "rule"))

    def test_one_drive_is_not_called_every_time(self):
        self.assertNotIn("every time", self.texts(2, "drives"))
