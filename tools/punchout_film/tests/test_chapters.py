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

    def test_chapter_9(self):
        self.check(9)

    def test_chapter_10(self):
        self.check(10)

    def test_chapter_11(self):
        self.check(11)

    def test_a_finished_fight_holds_its_last_lit_frame(self):
        from punchout_film.chapters.common import FIGHT_BOX
        ctx = context()
        shot = next(s for s in chapter(3).shots if s.name == "knockout")
        last = int(shot_seconds(shot, ctx) * 60) - 1
        x, y, w, h = FIGHT_BOX
        fight = shot.draw(ctx, last / 60, last).crop((x, y, x + w, y + h)).convert("L")
        self.assertGreater(sum(fight.getdata()) / (w * h), 40)

    def test_a_finished_fight_side_by_side_holds_its_last_lit_frame(self):
        ctx = context()
        shot = chapter(9).shots[0]
        last = int(shot_seconds(shot, ctx) * 60) - 1
        # Round 4 is on the right, and its take ends first.
        right = shot.draw(ctx, last / 60, last).crop((660, 20, 1220, 545)).convert("L")
        self.assertGreater(sum(right.getdata()) / (560 * 525), 40)

    def test_the_pixel_count_adds_up(self):
        # 84x84 is 7,056; the score is cropped off, leaving 68x84 = 5,712.
        self.assertIn("score cropped off, is 5,712 pixels", self.text(4))

    def test_the_verdict_claims_only_the_teachers_fight_knockouts(self):
        self.assertNotIn("knocks everyone out", self.text(11))

    def test_funeral_march_starts_at_macs_last_knockdown(self):
        from punchout_film.chapters.c02_idle import funeral_offset
        from punchout_film.chapters.common import downs
        ctx = context()
        self.assertEqual(downs(ctx.takes["p1-idle-glass-joe"], "mac_health")[-1], 1106)
        self.assertLess(abs(funeral_offset(ctx) - 18.43), 0.05)

    def test_knockdowns_get_a_roar(self):
        from reservoir_film.model import shot_stings
        ctx = context()
        stings = shot_stings(chapter(8).shots[0], ctx)
        bells = sorted(t for t, n in stings if n == "bell")
        self.assertTrue(bells)
        self.assertEqual(sorted(t for t, n in stings if n == "roar"), bells)
        knockout = next(s for s in chapter(3).shots if s.name == "knockout")
        stings = shot_stings(knockout, ctx)
        bells = [t for t, n in stings if n == "bell"]
        roars = sorted(t for t, n in stings if n == "roar")
        self.assertEqual(len(bells), 1)
        self.assertEqual(len(roars), 4)
        self.assertTrue(all(t < bells[0] for t in roars[:3]))
        self.assertEqual(roars[3], bells[0])

    def test_the_teachers_ko_is_said_with_its_real_length(self):
        self.assertIn("801 moves", self.text(3))
