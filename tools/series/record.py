"""The beat recorder: Nate reads a video's beats one at a time, as many
takes as each needs (docs/superpowers/specs/2026-10-30-beat-recorder-design.md).

Usage: cd tools && series/.venv/bin/python -m series.record NAME [--device NAME] [--list-devices] [--no-browser]
"""

import argparse
import json
import re
import threading
import tomllib
import webbrowser
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Callable, Protocol

import numpy as np
import soundfile

from series.script import Video
from series.sound import process
from series.takes import Take, TakeStore, trim

ROOM_SECONDS = 10.0
# A take this much longer or shorter than the narrator's reading gets a hint.
LENGTH_HINT = 0.5
QUIET_DB = -20.0
CLIP = 0.999


class Input(Protocol):
    rate: int

    def start(self, callback: Callable[[np.ndarray], None]) -> None: ...

    def stop(self) -> None: ...


class DeviceInput:
    """A microphone, through PortAudio: mono, 48 kHz if it can, else its own rate."""

    def __init__(self, name: str | None = None):
        import sounddevice

        self.sd = sounddevice
        self.device = name
        try:
            sounddevice.check_input_settings(device=name, channels=1, samplerate=48000, dtype="float32")
            self.rate = 48000
        except Exception:
            self.rate = int(sounddevice.query_devices(name, "input")["default_samplerate"])
        self.stream = None

    def start(self, callback: Callable[[np.ndarray], None]) -> None:
        def on_audio(indata, frames, time, status):
            callback(indata[:, 0].copy())

        self.stream = self.sd.InputStream(device=self.device, channels=1, samplerate=self.rate, dtype="float32",
                                          callback=on_audio)
        self.stream.start()

    def stop(self) -> None:
        if self.stream is not None:
            self.stream.stop()
            self.stream.close()
            self.stream = None


def peak_db(audio: np.ndarray) -> float:
    return 20 * np.log10(max(float(np.abs(audio).max(initial=0.0)), 1e-10))


def checks(audio: np.ndarray, rate: int, take: Take, narrator_seconds: float) -> list[str]:
    """What's wrong with a take, or might be."""
    problems = []
    if trim(audio, rate) is None:
        return ["no speech"]
    if np.abs(audio).max() >= CLIP:
        problems.append("clipped")
    if peak_db(audio) < QUIET_DB:
        problems.append("too quiet")
    if take.seconds > narrator_seconds * (1 + LENGTH_HINT):
        problems.append("much longer than the narrator")
    elif take.seconds < narrator_seconds * (1 - LENGTH_HINT):
        problems.append("much shorter than the narrator")
    return problems


class Session:
    """The recorder's state: which beat, the takes, and the microphone. The
    input runs from open() to close(), so the level meter always moves; a
    take keeps only what arrives between start() and stop()."""

    def __init__(self, video: Video, store: TakeStore, input: Input, narration: dict[str, Path], cache: Path):
        self.video, self.store, self.input = video, store, input
        self.narration, self.cache = narration, Path(cache)
        self.index = 0
        self.recording = False
        self.room_mode = False
        self.level = -100.0
        self.blocks: list[np.ndarray] = []
        self.lock = threading.Lock()
        self.problems: dict[tuple[str, int], list[str]] = {}
        self.narrator_seconds = {i: soundfile.info(p).duration for i, p in narration.items()}

    @property
    def line(self):
        return self.video.lines[self.index]

    def open(self) -> None:
        self.input.start(self._audio)

    def close(self) -> None:
        self.abandon()
        self.input.stop()

    def _audio(self, block: np.ndarray) -> None:
        self.level = peak_db(block)
        with self.lock:
            if not self.recording:
                return
            self.blocks.append(block)
            if self.room_mode and sum(len(b) for b in self.blocks) >= ROOM_SECONDS * self.input.rate:
                audio = np.concatenate(self.blocks)[:int(ROOM_SECONDS * self.input.rate)]
                self.blocks, self.recording, self.room_mode = [], False, False
                self.store.set_room(audio, self.input.rate)

    def goto(self, i: int) -> None:
        if not self.recording:
            self.index = max(0, min(len(self.video.lines) - 1, i))

    def next(self) -> None:
        self.goto(self.index + 1)

    def prev(self) -> None:
        self.goto(self.index - 1)

    def start(self) -> None:
        with self.lock:
            self.blocks, self.recording, self.room_mode = [], True, False

    def record_room(self) -> None:
        """Record ROOM_SECONDS of the room's quiet; it stops by itself."""
        with self.lock:
            self.blocks, self.recording, self.room_mode = [], True, True

    def stop(self) -> Take | None:
        with self.lock:
            if not self.recording or self.room_mode:
                return None
            blocks, self.blocks, self.recording = self.blocks, [], False
        audio = np.concatenate(blocks) if blocks else np.zeros(0, dtype=np.float32)
        return self.store.add(self.line.id, audio, self.input.rate, self.line.text)

    def abandon(self) -> None:
        with self.lock:
            self.blocks, self.recording, self.room_mode = [], False, False

    def choose(self, number: int) -> None:
        self.store.choose(self.line.id, number)

    def processed(self, line_id: str, number: int) -> Path:
        take = next(t for t in self.store.takes(line_id) if t.number == number)
        return process(take, self.cache)[0]

    def _problems(self, take: Take) -> list[str]:
        key = (take.line_id, take.number)
        if key not in self.problems:
            audio, rate = soundfile.read(take.path, dtype="float32")
            self.problems[key] = checks(audio, rate, take, self.narrator_seconds.get(take.line_id, 0.0))
        return self.problems[key]

    def _position(self) -> tuple[str | None, str]:
        line = self.line
        section = next((s for s in self.video.sections if s.id == line.section), None)
        if section is None:
            closing = [l for l in self.video.lines if l.section is None]
            return None, f"Closing · line {closing.index(line) + 1} of {len(closing)}"
        name = f"Chapter {section.number}" if section.number is not None else section.title
        ids = [l.id for l in section.lines]
        return section.title, f"{name} · beat {ids.index(line.id) + 1} of {len(ids)}"

    def _status(self, line) -> str:
        take = self.store.chosen(line.id)
        if take is None:
            return "unrecorded"
        return "recorded" if take.text == line.text else "stale"

    def state(self) -> dict:
        line, lines = self.line, self.video.lines
        chosen = self.store.chosen(line.id)
        section, position = self._position()
        return {
            "beat": {"id": line.id, "text": line.text, "board": line.board, "section": section, "position": position},
            "index": self.index,
            "before": lines[self.index - 1].text if self.index > 0 else None,
            "after": lines[self.index + 1].text if self.index + 1 < len(lines) else None,
            "takes": [{"number": t.number, "seconds": round(t.seconds, 2),
                       "chosen": chosen is not None and chosen.number == t.number,
                       "stale": t.text != line.text, "problems": self._problems(t)}
                      for t in self.store.takes(line.id)],
            "narrator_seconds": round(self.narrator_seconds.get(line.id, 0.0), 2),
            "recording": self.recording,
            "room_recording": self.room_mode,
            "level": round(self.level, 1),
            "strip": [self._status(l) for l in lines],
            "room": self.store.room() is not None,
        }


ACTIONS = {"start", "stop", "abandon", "next", "prev", "goto", "choose", "room"}
PAGE = Path(__file__).resolve().parent / "record.html"
TAKE_URL = re.compile(r"^/take/([a-z0-9-]+)/(\d+)\.wav$")
NARRATOR_URL = re.compile(r"^/narrator/([a-z0-9-]+)\.wav$")


def serve(session: Session, port: int) -> ThreadingHTTPServer:
    """The recorder's page and its actions, on this machine only."""

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def send(self, code: int, body: bytes = b"", kind: str = "text/plain") -> None:
            self.send_response(code)
            self.send_header("Content-Type", kind)
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Cache-Control", "no-store")
            self.end_headers()
            self.wfile.write(body)

        def do_GET(self):
            path = self.path.split("?")[0]
            if path == "/":
                return self.send(200, PAGE.read_bytes(), "text/html; charset=utf-8")
            if path == "/state":
                return self.send(200, json.dumps(session.state()).encode(), "application/json")
            wav = None
            if m := TAKE_URL.match(path):
                try:
                    wav = session.processed(m.group(1), int(m.group(2)))
                except StopIteration:
                    wav = None
            elif m := NARRATOR_URL.match(path):
                wav = session.narration.get(m.group(1))
            if wav is None:
                return self.send(404, b"not found")
            self.send(200, Path(wav).read_bytes(), "audio/wav")

        def do_POST(self):
            if self.path != "/action":
                return self.send(404, b"not found")
            try:
                body = json.loads(self.rfile.read(int(self.headers.get("Content-Length", 0))))
                do, n = body.get("do"), body.get("n")
                if do not in ACTIONS or (do in ("goto", "choose") and not isinstance(n, int)):
                    raise ValueError(f"unknown action {body!r}")
                if do == "goto":
                    session.goto(n)
                elif do == "choose":
                    session.choose(n)
                elif do == "room":
                    session.record_room()
                else:
                    getattr(session, do)()
            except (ValueError, TypeError, AttributeError) as e:
                return self.send(400, str(e).encode())
            self.send(200, json.dumps(session.state()).encode(), "application/json")

    return ThreadingHTTPServer(("127.0.0.1", port), Handler)


def remembered_device(store: TakeStore, name: str | None) -> str | None:
    """The input named on the command line, remembered in takes/recorder.toml; else the last one used."""
    path = store.dir / "recorder.toml"
    if name is not None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(f"device = {json.dumps(name)}\n")
        return name
    return tomllib.loads(path.read_text()).get("device") if path.exists() else None


def main() -> None:
    from series.build import OUT
    from series.script import VIDEOS, load_video
    from series.voice import TAKES, KokoroEngine, narrator

    parser = argparse.ArgumentParser()
    parser.add_argument("name", nargs="?")
    parser.add_argument("--device", help="the input device's name (remembered)")
    parser.add_argument("--list-devices", action="store_true")
    parser.add_argument("--port", type=int, default=8770)
    parser.add_argument("--no-browser", action="store_true", help="don't open the page")
    args = parser.parse_args()
    if args.list_devices:
        import sounddevice
        print(sounddevice.query_devices())
        return
    if args.name is None:
        parser.error("name the video, e.g. 001-one-neuron")
    video = load_video(VIDEOS / args.name)
    store = TakeStore(video.dir / TAKES)
    out = OUT / args.name
    print("Voicing the narrator's guide readings (cached after the first time)…")
    session = Session(video, store, DeviceInput(remembered_device(store, args.device)), narrator(video, KokoroEngine(), out),
                      out / "recorder")
    session.open()
    server = serve(session, args.port)
    url = f"http://127.0.0.1:{args.port}/"
    print(f"Recording {args.name} at {url} (Ctrl-C to finish)")
    if not args.no_browser:
        webbrowser.open(url)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        session.close()
        server.server_close()


if __name__ == "__main__":
    main()
