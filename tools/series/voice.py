"""Each line's voice: a recorded take when one matches, else the narrator."""

import hashlib
import tomllib
from importlib.metadata import version
from dataclasses import dataclass
from pathlib import Path
from typing import Literal, Protocol

import soundfile

from series.script import Video

# Per video: <id>.wav takes, and takes.toml mapping each id to the text it
# was recorded from.
TAKES = "takes"
DEFAULT_VOICE = "af_heart"
KOKORO_RATE = 24000


class Engine(Protocol):
    name: str

    def synthesize(self, text: str, out: Path) -> None: ...


class KokoroEngine:
    """Kokoro-82M, run locally."""

    def __init__(self, voice: str = DEFAULT_VOICE):
        self.voice = voice
        # The version is in the name, so a new Kokoro re-voices the cache.
        self.name = f"kokoro-{version('kokoro')}-{voice}"
        self.pipeline = None

    def synthesize(self, text: str, out: Path) -> None:
        import numpy as np
        from kokoro import KPipeline

        if self.pipeline is None:
            self.pipeline = KPipeline(lang_code="a", repo_id="hexgrad/Kokoro-82M")
        audio = [chunk.audio.numpy() for chunk in self.pipeline(text, voice=self.voice)]
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


def seconds(path: Path) -> float:
    info = soundfile.info(path)
    return info.frames / info.samplerate


def take_problems(video: Video) -> list[str]:
    """Takes that can't be used for a reason other than being stale: a wav
    with no takes.toml entry, an entry with no wav, or an id the script
    doesn't have."""
    takes_dir = video.dir / TAKES
    if not takes_dir.exists():
        return []
    toml = takes_dir / "takes.toml"
    recorded = tomllib.loads(toml.read_text()) if toml.exists() else {}
    ids = {line.id for line in video.lines}
    wavs = {p.stem for p in takes_dir.glob("*.wav")}
    problems = [f"{TAKES}/{w}.wav isn't listed in takes.toml (narrator used)" for w in sorted(wavs - recorded.keys())]
    problems += [f"takes.toml lists {i}, but {TAKES}/{i}.wav doesn't exist" for i in sorted((recorded.keys() & ids) - wavs)]
    problems += [f"takes.toml lists {i}, which isn't a line in the script" for i in sorted(recorded.keys() - ids)]
    return problems


def voices(video: Video, engine: Engine, out: Path) -> tuple[dict[str, Voice], list[str]]:
    """Every line's voice, and the ids of takes recorded from other text."""
    takes_dir = video.dir / TAKES
    toml = takes_dir / "takes.toml"
    recorded = tomllib.loads(toml.read_text()) if toml.exists() else {}
    found, stale = {}, []
    for line in video.lines:
        take = takes_dir / f"{line.id}.wav"
        if take.exists() and line.id in recorded:
            if recorded[line.id] == line.text:
                found[line.id] = Voice(line.id, take, seconds(take), "take")
                continue
            stale.append(line.id)
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
        found[line.id] = Voice(line.id, path, seconds(path), "synth")
    prune(Path(out) / "tts", {v.path for v in found.values()})
    return found, stale


def prune(tts: Path, keep: set[Path]) -> None:
    """Remove cached voices no line uses any more."""
    for path in tts.glob("*.wav") if tts.exists() else ():
        if path not in keep:
            path.unlink()
