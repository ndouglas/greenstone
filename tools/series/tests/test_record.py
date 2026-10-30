import json
import tempfile
import unittest
from pathlib import Path

import numpy as np
import soundfile

from series.record import ROOM_SECONDS, Session, checks
from series.script import load_video
from series.takes import Take, TakeStore
from series.tests.test_takes import speech
from series.voice import TAKES

RATE = 48000


def beat(id, role=None):
    r = f'role = "{role}"\n' if role else ""
    return f'[[section.line]]\nid = "{id}"\n{r}text = "{id} text."\nboard = "{id} picture"\nframing = true\n'


SCRIPT = ('title = "t"\n'
          '[[section]]\nkind = "introduction"\ntitle = "Introduction"\n' + beat("h") +
          '[[section]]\nkind = "chapter"\ntitle = "One"\n' + beat("i1", "intro") + beat("m1") + beat("c1", "conclusion") +
          '[[section]]\nkind = "conclusions"\ntitle = "Conclusions"\n' + beat("t"))


class FakeInput:
    """Audio arrives only when the test feeds it, in 1024-sample blocks."""

    rate = RATE

    def __init__(self):
        self.callback = None
        self.stops = 0

    def start(self, callback):
        self.callback = callback

    def stop(self):
        self.stops += 1

    def feed(self, audio):
        for i in range(0, len(audio), 1024):
            if self.callback:
                self.callback(audio[i:i + 1024])


def session(script=SCRIPT, dir=None, narrator_seconds=1.0):
    d = dir or Path(tempfile.mkdtemp()) / "001-test"
    d.mkdir(parents=True, exist_ok=True)
    (d / "script.toml").write_text(script)
    video = load_video(d)
    narration = {}
    for line in video.lines:
        path = d.parent / f"{line.id}-narrator.wav"
        soundfile.write(path, np.zeros(int(narrator_seconds * 24000), dtype=np.float32), 24000)
        narration[line.id] = path
    fake = FakeInput()
    s = Session(video, TakeStore(d / TAKES), fake, narration, d.parent / "cache")
    s.open()
    return s, fake, d


def record(s, fake, audio=None):
    s.start()
    fake.feed(speech() if audio is None else audio)
    return s.stop()


class Recording(unittest.TestCase):
    def test_a_take_is_recorded_into_the_store(self):
        s, fake, d = session()
        take = record(s, fake)
        self.assertEqual((take.line_id, take.number, take.text), ("h", 1, "h text."))
        self.assertEqual(s.store.chosen("h").number, 1)
        self.assertAlmostEqual(soundfile.info(take.path).duration, 2.5, delta=0.03)

    def test_an_abandoned_take_leaves_nothing(self):
        s, fake, d = session()
        s.start()
        fake.feed(speech())
        s.abandon()
        self.assertFalse(s.recording)
        self.assertEqual(s.store.takes("h"), [])
        self.assertFalse((d / TAKES / "raw").exists() and any((d / TAKES / "raw").rglob("*.wav")))
        self.assertEqual(record(s, fake).number, 1)

    def test_audio_outside_a_take_is_not_kept(self):
        s, fake, _ = session()
        fake.feed(speech(at=0.0, length=1.0, total=1.0))  # before start: only the level meter hears it
        self.assertGreater(s.level, -30)
        take = record(s, fake, speech(at=0.2, total=1.0))
        self.assertAlmostEqual(soundfile.info(take.path).duration, 1.0, delta=0.03)

    def test_moving_between_beats(self):
        s, _, _ = session()
        s.next()
        s.next()
        s.prev()
        self.assertEqual(s.index, 1)
        s.goto(0)
        s.prev()
        self.assertEqual(s.index, 0)
        s.goto(999)
        self.assertEqual(s.index, len(s.video.lines) - 1)

    def test_no_moving_mid_take(self):
        s, _, _ = session()
        s.start()
        s.next()
        self.assertEqual(s.index, 0)
        s.abandon()

    def test_choosing_a_take(self):
        s, fake, _ = session()
        record(s, fake)
        record(s, fake)
        s.choose(1)
        self.assertEqual(s.store.chosen("h").number, 1)

    def test_room_tone_stops_by_itself(self):
        s, fake, d = session()
        s.record_room()
        fake.feed(np.full(int(12 * RATE), 0.001, dtype=np.float32))
        self.assertFalse(s.recording)
        self.assertAlmostEqual(soundfile.info(s.store.room()).duration, ROOM_SECONDS, delta=0.05)
        self.assertEqual(s.store.takes("h"), [])


class Checks(unittest.TestCase):
    def take(self, start=0.0, end=1.0):
        return Take("a", 1, Path("a.wav"), "A.", start, end, RATE)

    def test_checks_flag_problems(self):
        loud = speech(amplitude=1.0)
        self.assertIn("clipped", checks(loud, RATE, self.take(), 1.0))
        quiet = speech(amplitude=10 ** (-30 / 20))
        self.assertIn("too quiet", checks(quiet, RATE, self.take(), 1.0))
        self.assertIn("much shorter than the narrator", checks(speech(), RATE, self.take(0.0, 0.5), 2.0))
        self.assertIn("much longer than the narrator", checks(speech(), RATE, self.take(0.0, 3.5), 2.0))
        self.assertIn("no speech", checks(np.zeros(RATE, dtype=np.float32), RATE, self.take(0.0, 0.0), 1.0))
        self.assertEqual(checks(speech(), RATE, self.take(0.0, 1.2), 1.0), [])


class State(unittest.TestCase):
    def test_the_strip_shows_each_beat(self):
        s, fake, d = session()
        record(s, fake)  # h, current
        s.next()
        record(s, fake)  # i1, about to go stale
        s, fake, _ = session(SCRIPT.replace('"i1 text."', '"i1 rewritten."'), dir=d)
        self.assertEqual(s.state()["strip"], ["recorded", "stale", "unrecorded", "unrecorded", "unrecorded"])

    def test_state_is_json(self):
        s, fake, _ = session()
        s.goto(2)
        record(s, fake)
        state = json.loads(json.dumps(s.state()))
        self.assertEqual(state["beat"]["position"], "Chapter 1 · beat 2 of 3")
        self.assertEqual(state["beat"]["section"], "One")
        self.assertEqual(state["beat"]["board"], "m1 picture")
        self.assertEqual((state["before"], state["after"]), ("i1 text.", "c1 text."))
        self.assertEqual(state["takes"][0]["number"], 1)
        self.assertTrue(state["takes"][0]["chosen"])
        self.assertFalse(state["takes"][0]["stale"])
        self.assertAlmostEqual(state["narrator_seconds"], 1.0, delta=0.01)
        self.assertFalse(state["recording"])
        self.assertFalse(state["room"])
        s.goto(0)
        self.assertEqual(s.state()["beat"]["position"], "Introduction · beat 1 of 1")
        self.assertIsNone(s.state()["before"])


if __name__ == "__main__":
    unittest.main()
