"""Each line's voice: Nate's chosen take, through the sound chain, when it
was read from the line's text; else the narrator."""

import hashlib
import tomllib
from importlib.metadata import version
from dataclasses import dataclass
from pathlib import Path
from typing import Literal, Protocol

import soundfile

from series.pacing import NARRATION_SPEED
from series.script import Video
from series.sound import process
from series.takes import RAW, ROOM, TakeStore

# Per video: the take store (series/takes.py).
TAKES = "takes"
DEFAULT_VOICE = "af_heart"
KOKORO_RATE = 24000


class Engine(Protocol):
    name: str

    def synthesize(self, text: str, out: Path) -> None: ...


class KokoroEngine:
    """Kokoro-82M, run locally."""

    def __init__(self, voice: str = DEFAULT_VOICE, speed: float = NARRATION_SPEED):
        self.voice = voice
        self.speed = speed
        # The version and speed are in the name, so changing either re-voices the cache.
        self.name = f"kokoro-{version('kokoro')}-{voice}-speed{speed}"
        self.pipeline = None

    def synthesize(self, text: str, out: Path) -> None:
        import numpy as np
        from kokoro import KPipeline

        if self.pipeline is None:
            self.pipeline = KPipeline(lang_code="a", repo_id="hexgrad/Kokoro-82M")
        audio = [chunk.audio.numpy() for chunk in self.pipeline(text, voice=self.voice, speed=self.speed)]
        if not audio:
            raise ValueError(f"nothing to pronounce in {text!r}; give the line a `spoken` version")
        out.parent.mkdir(parents=True, exist_ok=True)
        soundfile.write(out, np.concatenate(audio), KOKORO_RATE)


@dataclass(frozen=True)
class Voice:
    line_id: str
    path: Path
    seconds: float
    source: Literal["take", "synth"]
    # A take's gain through the chain, in dB (None for the narrator).
    gain_db: float | None = None


def seconds(path: Path) -> float:
    info = soundfile.info(path)
    return info.frames / info.samplerate


def take_problems(video: Video) -> list[str]:
    """Takes that can't be used for a reason other than being stale: an
    old-form wav with no takes.toml entry, an entry with no wav, or takes
    for an id the script doesn't have."""
    takes_dir = video.dir / TAKES
    if not takes_dir.exists():
        return []
    toml = takes_dir / "takes.toml"
    recorded = tomllib.loads(toml.read_text()) if toml.exists() else {}
    ids = {line.id for line in video.lines}
    wavs = {p.stem for p in takes_dir.glob("*.wav") if p.name != ROOM}
    old = {k for k, v in recorded.items() if isinstance(v, str)}
    raw = {p.name for p in (takes_dir / RAW).iterdir() if p.is_dir()} if (takes_dir / RAW).exists() else set()
    problems = [f"{TAKES}/{w}.wav isn't listed in takes.toml (narrator used)" for w in sorted(wavs - recorded.keys())]
    problems += [f"takes.toml lists {i}, but {TAKES}/{i}.wav doesn't exist" for i in sorted((old & ids) - wavs)]
    problems += [f"takes.toml lists {i}, which isn't a line in the script" for i in sorted(recorded.keys() - ids)]
    problems += [f"{TAKES}/{RAW}/{i} holds takes for {i}, which isn't a line in the script"
                 for i in sorted(raw - ids - recorded.keys())]
    return problems


def narrator(video: Video, engine: Engine, out: Path) -> dict[str, Path]:
    """Every line's narrator reading, cached by the engine and the voiced text;
    kept for recorded lines too, as the recorder's pacing guide."""
    paths = {}
    for line in video.lines:
        key = hashlib.sha256(f"{engine.name}\0{line.voiced}".encode()).hexdigest()[:12]
        path = Path(out) / "tts" / f"{line.id}-{key}.wav"
        if not path.exists():
            # Synthesize beside the cache and move it in, so a crash can't
            # leave a half-written file that looks cached.
            partial = path.with_name(f".{path.stem}.partial.wav")
            try:
                engine.synthesize(line.voiced, partial)
            except Exception as e:
                partial.unlink(missing_ok=True)
                raise RuntimeError(f"couldn't voice line {line.id}: {e}") from e
            partial.replace(path)
        paths[line.id] = path
    return paths


def voices(video: Video, engine: Engine, out: Path) -> tuple[dict[str, Voice], list[str]]:
    """Every line's voice, and the ids of chosen takes read from other text."""
    store = TakeStore(video.dir / TAKES)
    readings = narrator(video, engine, out)
    found, stale = {}, []
    for line in video.lines:
        take = store.chosen(line.id)
        if take is not None and take.text == line.text:
            path, gain = process(take, Path(out) / "takes")
            found[line.id] = Voice(line.id, path, seconds(path), "take", gain)
            continue
        if take is not None:
            stale.append(line.id)
        found[line.id] = Voice(line.id, readings[line.id], seconds(readings[line.id]), "synth")
    prune(Path(out) / "tts", set(readings.values()))
    used = {v.path for v in found.values() if v.source == "take"}
    prune(Path(out) / "takes", used | {p.with_suffix(".json") for p in used}, "*")
    return found, stale


def prune(folder: Path, keep: set[Path], pattern: str = "*.wav") -> None:
    """Remove cached files nothing uses any more."""
    for path in folder.glob(pattern) if folder.exists() else ():
        if path not in keep:
            path.unlink()
