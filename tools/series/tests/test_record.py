import json
import tempfile
import threading
import unittest
import urllib.error
import urllib.request
from pathlib import Path

import numpy as np
import soundfile

import series.record
from series.record import ROOM_SECONDS, Session, checks, remembered_device, serve
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

    def test_a_take_stays_with_its_beat_when_next_comes_quickly(self):
        s, fake, _ = session()

        class EnterOnRelease:
            """The session's lock; the moment stop() lets go of it, Enter arrives."""

            def __init__(self):
                self.lock, self.armed = threading.Lock(), False

            def __enter__(self):
                self.lock.acquire()

            def __exit__(self, *exc):
                self.lock.release()
                if self.armed:
                    self.armed = False
                    s.next()

        s.lock = EnterOnRelease()
        s.start()
        fake.feed(speech())
        s.lock.armed = True
        s.stop()
        self.assertEqual([t.line_id for t in s.store.takes("h")], ["h"])
        self.assertEqual(s.store.takes("i1"), [])

    def test_dropouts_are_flagged(self):
        s, fake, _ = session()
        fake.overflows = 0
        s.start()
        fake.feed(speech()[:RATE])
        fake.overflows = 2
        fake.feed(speech()[RATE:])
        take = s.stop()
        self.assertIn("dropouts", s.state()["takes"][0]["problems"])
        self.assertEqual(take.number, 1)

    def test_state_reads_the_index_once(self):
        s, fake, _ = session()
        for _ in range(4):
            record(s, fake)
            s.next()
        calls = []
        real = s.store._index
        s.store._index = lambda: calls.append(1) or real()
        s.state()
        self.assertLessEqual(len(calls), 2)

    def test_the_device_is_remembered_for_every_video(self):
        path = Path(tempfile.mkdtemp()) / "recorder.toml"
        self.assertIsNone(remembered_device(None, path))
        self.assertEqual(remembered_device("FIFINE K669B", path), "FIFINE K669B")
        self.assertEqual(remembered_device(None, path), "FIFINE K669B")

    def test_room_tone_is_saved_off_the_audio_thread(self):
        s, fake, _ = session()
        saved = []
        real = s.store.set_room
        s.store.set_room = lambda *a: saved.append((threading.current_thread(), s.lock.locked())) or real(*a)
        s.record_room()
        fake.feed(np.full(int(11 * RATE), 0.001, dtype=np.float32))
        s.saving.join(5)
        self.assertEqual(len(saved), 1)
        self.assertIsNot(saved[0][0], threading.current_thread())
        self.assertFalse(saved[0][1])

    def test_room_tone_waits_for_a_take_to_finish(self):
        s, fake, _ = session()
        s.start()
        fake.feed(speech()[:RATE])
        s.record_room()  # mid-take: ignored
        fake.feed(speech()[RATE:])
        self.assertEqual(s.stop().number, 1)
        self.assertIsNone(s.store.room())

    def test_the_playback_cache_keeps_only_current_takes(self):
        s, fake, d = session()
        record(s, fake)
        kept = s.processed("h", 1)
        stray = s.cache / "h-9-0123456789ab.wav"
        stray.write_bytes(b"old")
        stray.with_suffix(".json").write_text("{}")
        s2, _, _ = session(dir=d)
        self.assertTrue(kept.exists())
        self.assertFalse(stray.exists())
        self.assertFalse(stray.with_suffix(".json").exists())

    def test_room_tone_stops_by_itself(self):
        s, fake, d = session()
        s.record_room()
        fake.feed(np.full(int(12 * RATE), 0.001, dtype=np.float32))
        self.assertFalse(s.recording)
        s.saving.join(5)
        self.assertAlmostEqual(soundfile.info(s.store.room()).duration, ROOM_SECONDS, delta=0.05)
        self.assertEqual(s.store.takes("h"), [])


class Checks(unittest.TestCase):
    def take(self, start=0.0, end=1.0):
        return Take("a", 1, Path("a.wav"), "A.", start, end, RATE)

    def test_checks_flag_problems(self):
        loud = speech(amplitude=1.0)
        self.assertIn("clipped", checks(loud, RATE, self.take(), 1.0))
        quiet = speech(amplitude=10 ** (-25 / 20))
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


class Server(unittest.TestCase):
    def setUp(self):
        self.s, self.fake, _ = session()
        self.server = serve(self.s, 0)
        threading.Thread(target=self.server.serve_forever, daemon=True).start()
        self.url = f"http://127.0.0.1:{self.server.server_address[1]}"

    def tearDown(self):
        self.server.shutdown()
        self.server.server_close()

    def post(self, body):
        request = urllib.request.Request(f"{self.url}/action", data=json.dumps(body).encode(),
                                         headers={"Content-Type": "application/json"})
        return urllib.request.urlopen(request)

    def test_the_server_drives_the_session(self):
        self.post({"do": "start"})
        self.fake.feed(speech())
        self.post({"do": "stop"})
        state = json.load(urllib.request.urlopen(f"{self.url}/state"))
        self.assertEqual(len(state["takes"]), 1)
        wav = Path(tempfile.mkdtemp()) / "take.wav"
        wav.write_bytes(urllib.request.urlopen(f"{self.url}/take/h/1.wav").read())
        self.assertEqual(soundfile.info(wav).samplerate, 48000)
        self.assertGreater(len(urllib.request.urlopen(f"{self.url}/narrator/h.wav").read()), 44)
        self.post({"do": "next"})
        self.post({"do": "goto", "n": 3})
        self.assertEqual(self.s.index, 3)
        self.assertIn(b"<html", urllib.request.urlopen(self.url + "/").read().lower())

    def test_the_server_refuses_unknown_actions(self):
        for body in ({"do": "delete"}, {"do": "choose", "n": "x"}, {"do": "choose", "n": 7}):
            with self.assertRaises(urllib.error.HTTPError) as e:
                self.post(body)
            self.assertEqual(e.exception.code, 400)
        with self.assertRaises(urllib.error.HTTPError) as e:
            urllib.request.urlopen(f"{self.url}/take/../../etc/1.wav")
        self.assertEqual(e.exception.code, 404)
        self.assertEqual(self.s.store.takes("h"), [])

    def test_actions_must_be_json(self):
        request = urllib.request.Request(f"{self.url}/action", data=b'{"do": "start"}',
                                         headers={"Content-Type": "text/plain"})
        with self.assertRaises(urllib.error.HTTPError) as e:
            urllib.request.urlopen(request)
        self.assertEqual(e.exception.code, 415)
        self.assertFalse(self.s.recording)

    def test_other_hosts_are_refused(self):
        # A page elsewhere that rebinds its name to 127.0.0.1 still sends its own Host.
        for path in ("/state", "/narrator/h.wav"):
            request = urllib.request.Request(self.url + path, headers={"Host": "evil.example:80"})
            with self.assertRaises(urllib.error.HTTPError) as e:
                urllib.request.urlopen(request)
            self.assertEqual(e.exception.code, 403)
        port = self.server.server_address[1]
        ok = urllib.request.Request(self.url + "/state", headers={"Host": f"localhost:{port}"})
        self.assertEqual(urllib.request.urlopen(ok).status, 200)

    def test_the_server_listens_only_locally(self):
        self.assertEqual(self.server.server_address[0], "127.0.0.1")

    def test_the_checkbox_gives_the_keys_back(self):
        page = (Path(series.record.__file__).parent / "record.html").read_text()
        self.assertIn("playback.blur()", page)

    def test_the_page_names_every_key(self):
        page = (Path(series.record.__file__).parent / "record.html").read_text()
        for key in ("Space", "Enter", "R", "P", "N", "Esc", "1–9", "room tone"):
            self.assertIn(key, page)


if __name__ == "__main__":
    unittest.main()
