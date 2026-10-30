"""The beat recorder: Nate reads a video's beats one at a time, as many
takes as each needs (docs/superpowers/specs/2026-10-30-beat-recorder-design.md).

Usage: cd tools && series/.venv/bin/python -m series.record NAME [--device NAME] [--list-devices]
"""

import threading
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
