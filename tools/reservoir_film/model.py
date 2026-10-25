"""The film's model: takes recorded by examples/reservoir_film.rs, the
long-run numbers, and chapters made of shots with captions."""

import json
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable

import numpy as np

REPO = Path(__file__).resolve().parents[2]
TAKES_DIR = REPO / "target" / "reservoir-film"
NUMBERS = Path(__file__).resolve().parent / "numbers.json"
W, H, FPS = 1280, 720, 60
TITLE_SECONDS = 4.0
NES_W, NES_H = 256, 240
REF = re.compile(r"\{([a-z0-9_]+)\.([A-Za-z0-9_.\-]+)(?::([^}]+))?\}")
UNFILLED = re.compile(r"\{[^}]*\}")


class Frames:
    def __init__(self, path, n):
        self.n = n
        self.data = np.memmap(path, dtype=np.uint8, mode="r", shape=(n, NES_H, NES_W, 3)) if n else None

    def __len__(self):
        return self.n

    def __getitem__(self, i):
        return np.asarray(self.data[min(max(int(i), 0), self.n - 1)])


class Take:
    def __init__(self, path):
        self.path = Path(path)
        self.name = self.path.name
        self.meta = json.loads((self.path / "take.json").read_text())
        self.mtime = (self.path / "take.json").stat().st_mtime

    def frames(self, stream):
        n = self.meta["streams"].get(stream)
        if n is None:
            raise KeyError(f"take {self.name} has no stream {stream}; it has {sorted(self.meta['streams'])}")
        return Frames(self.path / f"{stream}.rgb", n)


RECORD_HINT = "cargo run --release --example reservoir_film -- --take {name}"


class Takes:
    def __init__(self, root=TAKES_DIR, hint=RECORD_HINT):
        self.root = Path(root)
        self.hint = hint
        self.cache = {}

    def __getitem__(self, name):
        if name not in self.cache:
            path = self.root / name
            if not (path / "take.json").exists():
                raise FileNotFoundError(f"no take at {path}; record it with: {self.hint.format(name=name)}")
            self.cache[name] = Take(path)
        return self.cache[name]


def load_numbers(path=NUMBERS):
    return json.loads(Path(path).read_text())


@dataclass
class Film:
    """One film the compositor can build: where its takes and output live,
    its chapters and score, and the sources whose text keys its cache."""
    name: str
    title: str
    takes_dir: Path
    out: Path
    segments: Path
    record_hint: str
    numbers: Path
    chapters: Callable
    cues: Callable
    sources: list


def reading_seconds(text):
    words = len(text.split())
    return 0 if words == 0 else max(3.0, 4.0 * words / 15)


def _walk(value, path, ref):
    for part in path.split("."):
        try:
            value = value[int(part)] if isinstance(value, list) else value[part]
        except (KeyError, IndexError, ValueError, TypeError):
            raise KeyError(f"caption reference {{{ref}}} not found")
    if value is None:
        raise KeyError(f"caption reference {{{ref}}} is null")
    return value


def fill(text, takes, numbers):
    def one(m):
        head, path, spec = m.groups()
        ref = f"{head}.{path}"
        if head == "numbers":
            value = _walk(numbers, path, ref)
        else:
            try:
                meta = takes[head].meta
            except FileNotFoundError as e:
                raise KeyError(f"caption reference {{{ref}}}: {e}")
            value = _walk(meta, path, ref)
        return format(value, spec) if spec else str(value)
    return REF.sub(one, text)


@dataclass
class Context:
    takes: object
    numbers: dict


@dataclass
class Caption:
    at: float
    text: object


@dataclass
class Shot:
    name: str
    seconds: object
    draw: Callable
    captions: list = field(default_factory=list)
    takes: tuple = ()
    stings: object = field(default_factory=list)


@dataclass
class Chapter:
    number: int
    title: str
    subtitle: str
    shots: list


def shot_seconds(shot, ctx):
    return float(shot.seconds(ctx) if callable(shot.seconds) else shot.seconds)


def chapter_seconds(chapter, ctx):
    title = TITLE_SECONDS if chapter.number > 0 else 0.0
    return title + sum(shot_seconds(s, ctx) for s in chapter.shots)


def caption_text(caption, ctx):
    text = caption.text(ctx) if callable(caption.text) else caption.text
    return fill(text, ctx.takes, ctx.numbers)


def caption_spans(shot, ctx):
    end = shot_seconds(shot, ctx)
    caps = sorted(shot.captions, key=lambda c: c.at)
    out = []
    for i, c in enumerate(caps):
        stop = caps[i + 1].at if i + 1 < len(caps) else end
        out.append((c.at, stop, caption_text(c, ctx)))
    return out


def check_captions(chapter, ctx):
    problems = []
    for shot in chapter.shots:
        for start, stop, text in caption_spans(shot, ctx):
            if UNFILLED.search(text):
                problems.append(f"chapter {chapter.number}, shot {shot.name}: '{text}' has an unfilled reference")
            need = reading_seconds(text)
            if stop - start + 1e-9 < need:
                problems.append(f"chapter {chapter.number}, shot {shot.name}: '{text}' shows {stop - start:.1f} s, needs {need:.1f} s")
    return problems


def shot_stings(shot, ctx):
    return shot.stings(ctx) if callable(shot.stings) else list(shot.stings)
