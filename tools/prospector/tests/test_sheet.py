import unittest

from prospector import sheet


def entry(offset, cls, **kw):
    return dict({"offset": offset, "hex": f"${offset:04X}", "class": cls}, **kw)


class Sections(unittest.TestCase):
    def setUp(self):
        self.entries = [
            entry(0x8000, "crashed", message="Opcode 0x02 is not recognized"),
            entry(0x8001, "blank"),
            entry(0x8002, "walked", walk_x=900, fingerprint="aa"),
            entry(0x8003, "duplicate", duplicate_of=0x8002, fingerprint="aa"),
            entry(0x8004, "stuck", fingerprint="bb"),
            entry(0x8005, "unsolved", fingerprint="cc"),
            entry(0x8006, "cleared_distance", fingerprint="dd"),
            entry(0x8007, "cleared_flag", fingerprint="ee"),
            entry(0x8008, "instant_death"),
        ]
        self.ranking = [{"rank": 1, "offset": 0x8007, "hex": "$8007", "bottleneck": 1, "kill_rate": 0.9, "rounds": 99},
                        {"rank": 2, "offset": 0x8006, "hex": "$8006", "bottleneck": 2, "kill_rate": 0.5, "rounds": 80}]

    def test_sections_come_in_order_and_skip_duplicates(self):
        got = sheet.sections(self.entries, self.ranking)
        self.assertEqual([t for t, _ in got], list(sheet.SECTION_TITLES))
        flat = [e["offset"] for _, es in got for e in es]
        self.assertNotIn(0x8003, flat)
        self.assertEqual(sorted(flat), sorted(e["offset"] for e in self.entries if e["class"] != "duplicate"))

    def test_the_leaderboard_follows_the_ranking(self):
        title, leaders = sheet.sections(self.entries, self.ranking)[0]
        self.assertEqual([e["offset"] for e in leaders], [0x8007, 0x8006])

    def test_render_shows_counts_messages_and_calibration(self):
        html = sheet.render(self.entries, self.ranking, {"1-1": {"bottleneck": 4, "kill_rate": 0.5, "rounds": 80}})
        self.assertIn("Opcode 0x02 is not recognized", html)
        self.assertIn("crashed: 1", html)
        self.assertIn("1-1", html)
        self.assertIn("$8007", html)
        self.assertTrue(html.startswith("<!doctype html>"))


if __name__ == "__main__":
    unittest.main()
