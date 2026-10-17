import unittest

from reservoir_film.music.core import (
    NES_FRAME_HZ, Cue, Instrument, Part, Section, compile_cue, midi, notes_to_events, pulse_period, triangle_period,
)

LEAD = Instrument(volume=(12, 10, 8), duty=(2,))
BASS = Instrument()


class Pitches(unittest.TestCase):
    def test_midi_numbers(self):
        self.assertEqual(midi("A4"), 69)
        self.assertEqual(midi("C4"), 60)
        self.assertEqual(midi("C#4"), 61)
        self.assertEqual(midi("Bb3"), 58)

    def test_periods_match_the_ntsc_formulas(self):
        self.assertEqual(pulse_period(440.0), 253)
        self.assertEqual(triangle_period(220.0), 253)


class Notes(unittest.TestCase):
    def test_lengths_and_dots(self):
        events = notes_to_events("C5/4 | r/8 E5/8. k/16", bpm=120, start_beat=2)
        self.assertEqual(events, [(2, 1, "C5"), (3, 0.5, "r"), (3.5, 0.75, "E5"), (4.25, 0.25, "k")])

    def test_bad_token_names_itself(self):
        with self.assertRaisesRegex(ValueError, "C5"):
            notes_to_events("C5", bpm=120, start_beat=0)


class Compile(unittest.TestCase):
    def cue(self):
        loop = Section([Part("p1", LEAD, "A4/4 r/4"), Part("tri", BASS, "A3/2")])
        return Cue(name="t", title="T", bpm=120, loop=loop)

    def test_section_parts_must_agree(self):
        with self.assertRaises(ValueError):
            Section([Part("p1", LEAD, "A4/4"), Part("tri", BASS, "A3/2")]).beats(120)

    def test_frames_follow_the_tempo(self):
        frames, _ = compile_cue(self.cue(), loops=2)
        self.assertEqual(frames, round(4 * 60 / 120 * NES_FRAME_HZ))

    def test_writes_start_the_chip_and_set_the_pitch(self):
        _, writes = compile_cue(self.cue(), loops=1)
        first = [w for w in writes if w[0] == 0]
        self.assertIn((0, 0x4015, 0x0F), first)
        self.assertIn((0, 0x4001, 0x08), first)
        self.assertIn((0, 0x4002, 253 & 0xFF), first)
        self.assertIn((0, 0x400A, 253 & 0xFF), first)

    def test_the_volume_envelope_steps_then_holds(self):
        _, writes = compile_cue(self.cue(), loops=1)
        vols = [(f, v & 0x0F) for f, a, v in writes if a == 0x4000]
        self.assertEqual(vols[:3], [(0, 12), (1, 10), (2, 8)])
        rest = round(60 / 120 * NES_FRAME_HZ)
        self.assertIn((rest, 0), vols, "the rest silences the pulse")

    def test_writes_are_sorted_and_deduplicated(self):
        _, writes = compile_cue(self.cue(), loops=2)
        self.assertEqual(writes, sorted(writes, key=lambda w: w[0]))
        last = {}
        for f, a, v in writes:
            if a in (0x4003, 0x4007, 0x400B, 0x400F):
                continue
            self.assertNotEqual(last.get(a), v, f"redundant write to {a:04X} at {f}")
            last[a] = v
