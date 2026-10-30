"""Nate's recorded takes of a video's beats: every take kept raw, with the
text it was read from and where its speech starts and ends, and which one
each beat uses (docs/superpowers/specs/2026-10-30-beat-recorder-design.md).

takes/
  takes.toml            [<id>] text = "...", take = <n>   (or the old <id> = "<text>")
  raw/<id>/<n>.wav      a raw take, 24-bit, never altered
  raw/<id>/<n>.toml     its text, rate, and trim (or silent = true)
  room.wav              the room's own quiet
  <id>.wav              an old-form take
"""

import json
import tomllib
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import soundfile

RAW = "raw"
ROOM = "room.wav"
# Seconds kept either side of the speech, so breaths and consonants aren't clipped.
PAD = 0.15
FRAME = 0.02
# Speech is any frame above this, or within RANGE dB of the loudest, whichever is higher.
FLOOR_DB = -50.0
RANGE_DB = 35.0


@dataclass(frozen=True)
class Take:
    line_id: str
    # 0 for an old-form take (takes/<id>.wav, used whole).
    number: int
    path: Path
    text: str
    start: float
    end: float
    rate: int

    @property
    def seconds(self) -> float:
        return self.end - self.start


def trim(audio: np.ndarray, rate: int) -> tuple[float, float] | None:
    """Where the speech starts and ends, in seconds, padded; None if there's none."""
    size = max(1, int(FRAME * rate))
    frames = len(audio) // size
    if frames == 0:
        return None
    rms = np.sqrt(np.mean(audio[:frames * size].astype(np.float64).reshape(frames, size) ** 2, axis=1))
    db = 20 * np.log10(np.maximum(rms, 1e-10))
    loud = np.flatnonzero(db > max(FLOOR_DB, db.max() - RANGE_DB))
    if len(loud) == 0:
        return None
    start = max(0.0, loud[0] * size / rate - PAD)
    end = min(len(audio) / rate, (loud[-1] + 1) * size / rate + PAD)
    return start, end


def quote(text: str) -> str:
    # A JSON string is a valid TOML basic string.
    return json.dumps(text, ensure_ascii=False)


def write_atomically(path: Path, write) -> None:
    """Write beside the file and move it in, so nothing half-written is left."""
    path.parent.mkdir(parents=True, exist_ok=True)
    partial = path.with_name(f".{path.stem}.partial{path.suffix}")
    try:
        write(partial)
        partial.replace(path)
    finally:
        partial.unlink(missing_ok=True)


class TakeStore:
    def __init__(self, dir: Path):
        self.dir = Path(dir)

    def _index(self) -> dict:
        toml = self.dir / "takes.toml"
        return tomllib.loads(toml.read_text()) if toml.exists() else {}

    def _write_index(self, index: dict) -> None:
        old = {k: v for k, v in index.items() if isinstance(v, str)}
        tables = {k: v for k, v in index.items() if isinstance(v, dict)}
        text = "".join(f"{k} = {quote(v)}\n" for k, v in old.items())
        text += "".join(f"\n[{k}]\ntext = {quote(v['text'])}\ntake = {v['take']}\n" for k, v in tables.items())
        write_atomically(self.dir / "takes.toml", lambda p: p.write_text(text))

    def _take(self, line_id: str, number: int) -> Take:
        meta = tomllib.loads((self.dir / RAW / line_id / f"{number}.toml").read_text())
        return Take(line_id, number, self.dir / RAW / line_id / f"{number}.wav", meta["text"],
                    meta.get("start", 0.0), meta.get("end", 0.0), meta["rate"])

    def takes(self, line_id: str) -> list[Take]:
        folder = self.dir / RAW / line_id
        numbers = sorted(int(p.stem) for p in folder.glob("*.toml") if p.stem.isdigit()) if folder.exists() else []
        return [self._take(line_id, n) for n in numbers]

    def silent(self, take: Take) -> bool:
        if take.number == 0:
            return False
        meta = tomllib.loads((self.dir / RAW / take.line_id / f"{take.number}.toml").read_text())
        return meta.get("silent", False)

    def add(self, line_id: str, audio: np.ndarray, rate: int, text: str) -> Take:
        """Keep a new take, and choose it unless there's no speech in it."""
        number = max((t.number for t in self.takes(line_id)), default=0) + 1
        folder = self.dir / RAW / line_id
        found = trim(audio, rate)
        meta = f"text = {quote(text)}\nrate = {rate}\n"
        meta += f"start = {float(found[0])!r}\nend = {float(found[1])!r}\n" if found else "silent = true\n"
        # The wav first: a take's .toml is what makes it exist.
        write_atomically(folder / f"{number}.wav", lambda p: soundfile.write(p, audio, rate, subtype="PCM_24"))
        write_atomically(folder / f"{number}.toml", lambda p: p.write_text(meta))
        if found:
            self.choose(line_id, number)
        return self._take(line_id, number)

    def choose(self, line_id: str, number: int) -> None:
        take = next((t for t in self.takes(line_id) if t.number == number), None)
        if take is None:
            raise ValueError(f"{line_id} has no take {number}")
        if self.silent(take):
            raise ValueError(f"{line_id}'s take {number} has no speech in it")
        index = self._index()
        index[line_id] = {"text": take.text, "take": number}
        self._write_index(index)

    def chosen(self, line_id: str) -> Take | None:
        entry = self._index().get(line_id)
        if isinstance(entry, dict):
            return self._take(line_id, entry["take"])
        path = self.dir / f"{line_id}.wav"
        if isinstance(entry, str) and path.exists():
            info = soundfile.info(path)
            return Take(line_id, 0, path, entry, 0.0, info.frames / info.samplerate, info.samplerate)
        return None

    def set_room(self, audio: np.ndarray, rate: int) -> None:
        write_atomically(self.dir / ROOM, lambda p: soundfile.write(p, audio, rate, subtype="PCM_24"))

    def room(self) -> Path | None:
        path = self.dir / ROOM
        return path if path.exists() else None
