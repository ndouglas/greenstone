import unittest

from series import pacing


class Pacing(unittest.TestCase):
    def test_pacing_defaults(self):
        self.assertEqual(pacing.STILLNESS, 2.0)
        self.assertEqual(pacing.TITLE_HOLD, 4.0)
        self.assertEqual(pacing.SECTION_LIMIT, 240.0)
        self.assertEqual(pacing.NARRATION_SPEED, 0.9)


if __name__ == "__main__":
    unittest.main()
