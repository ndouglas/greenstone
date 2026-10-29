"""A video's script: its lines, what supports each one, and its sources."""

import json
import re
import tomllib
from dataclasses import dataclass
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
VIDEOS = Path(__file__).resolve().parent / "videos"
ID = re.compile(r"^[a-z0-9-]+$")
# A number from the video's experiments, filled from numbers.json.
NUMBER = re.compile(r"\{([a-z0-9_]+)\}")


class ScriptError(Exception):
    pass


@dataclass(frozen=True)
class Line:
    id: str
    text: str
    spoken: str | None
    cite: tuple[str, ...]
    framing: bool
    board: str | None = None

    @property
    def voiced(self) -> str:
        """What the narrator says: `spoken` when the text has symbols in it."""
        return self.spoken or self.text


@dataclass(frozen=True)
class Citation:
    key: str
    title: str
    authors: str
    year: int
    url: str


@dataclass(frozen=True)
class Video:
    name: str
    dir: Path
    title: str
    music: Path | None
    lines: tuple[Line, ...]
    citations: dict[str, Citation]


def load_video(dir: Path) -> Video:
    dir = Path(dir)
    script = tomllib.loads((dir / "script.toml").read_text())
    path = dir / "citations.toml"
    sources = tomllib.loads(path.read_text()) if path.exists() else {}
    citations = {k: Citation(key=k, **v) for k, v in sources.items()}
    path = dir / "numbers.json"
    numbers = json.loads(path.read_text()) if path.exists() else {}

    def fill(line_id: str, text: str | None) -> str | None:
        """Put each {name} from numbers.json into the text; numbers are never typed in."""
        if text is None:
            return None
        missing = [m for m in NUMBER.findall(text) if m not in numbers]
        if missing:
            raise ScriptError(f"line {line_id} uses {', '.join(missing)}, which numbers.json doesn't have")
        return NUMBER.sub(lambda m: str(numbers[m.group(1)]), text)

    lines, seen = [], set()
    for raw in script.get("line", []):
        line = Line(
            id=raw["id"],
            text=fill(raw["id"], raw["text"]),
            spoken=fill(raw["id"], raw.get("spoken")),
            cite=tuple(raw.get("cite", ())),
            framing=raw.get("framing", False),
            board=raw.get("board"),
        )
        if not ID.match(line.id):
            raise ScriptError(f"line id {line.id!r} must be lowercase letters, digits and hyphens")
        if line.id in seen:
            raise ScriptError(f"line id {line.id!r} is used twice")
        seen.add(line.id)
        if not line.text.strip():
            raise ScriptError(f"line {line.id} has no text")
        if not line.cite and not line.framing:
            raise ScriptError(f"line {line.id} cites nothing; cite a source or mark it framing = true")
        for key in line.cite:
            if key not in citations:
                raise ScriptError(f"line {line.id} cites {key!r}, which citations.toml doesn't list")
        lines.append(line)
    music = script.get("music")
    return Video(
        name=dir.name,
        dir=dir,
        title=script["title"],
        music=REPO / music if music else None,
        lines=tuple(lines),
        citations=citations,
    )
