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
# A video opens with an introduction, has numbered chapters, and closes
# with conclusions; each chapter opens with an intro beat and ends with a
# conclusion beat.
SECTION_KINDS = ("introduction", "chapter", "conclusions")
ROLES = ("intro", "conclusion")


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
    # "intro" or "conclusion", for a chapter's first and last beats.
    role: str | None = None
    # Seconds of stillness after this beat, if not the default.
    pause: float | None = None
    # The id of the section the beat is in (None for the closing lines).
    section: str | None = None

    @property
    def voiced(self) -> str:
        """What the narrator says: `spoken` when the text has symbols in it."""
        return self.spoken or self.text


@dataclass(frozen=True)
class Section:
    id: str
    kind: str
    number: int | None
    title: str
    lines: tuple[Line, ...]


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
    sections: tuple[Section, ...] = ()


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

    seen = set()

    def make_line(raw: dict, section: str | None) -> Line:
        line = Line(
            id=raw["id"],
            text=fill(raw["id"], raw["text"]),
            spoken=fill(raw["id"], raw.get("spoken")),
            cite=tuple(raw.get("cite", ())),
            framing=raw.get("framing", False),
            board=fill(raw["id"], raw.get("board")),
            role=raw.get("role"),
            pause=raw.get("pause"),
            section=section,
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
        if line.role is not None and line.role not in ROLES:
            raise ScriptError(f"line {line.id}'s role is {line.role!r}; it must be one of {', '.join(ROLES)}")
        return line

    sections, chapters = [], 0
    raw_sections = script.get("section", [])
    for i, raw in enumerate(raw_sections):
        kind = raw.get("kind")
        if kind not in SECTION_KINDS:
            raise ScriptError(f"section {i + 1}'s kind is {kind!r}; it must be one of {', '.join(SECTION_KINDS)}")
        if kind == "chapter":
            chapters += 1
            sid, number = f"chapter-{chapters}", chapters
        else:
            sid, number = kind, None
        if kind == "introduction" and i != 0:
            raise ScriptError("the introduction must be the first section, and there can be only one")
        if kind == "conclusions" and i != len(raw_sections) - 1:
            raise ScriptError("the conclusions must be the last section, and there can be only one")
        if not raw.get("title", "").strip():
            raise ScriptError(f"{sid} has no title")
        beats = tuple(make_line(b, sid) for b in raw.get("line", []))
        if not beats:
            raise ScriptError(f"{sid} has no beats")
        roles = [b.role for b in beats]
        if kind == "chapter":
            if roles[0] != "intro" or roles[-1] != "conclusion" or len(beats) < 2:
                raise ScriptError(f"{sid} must open with a role = \"intro\" beat and end with a role = \"conclusion\" beat")
            if "intro" in roles[1:] or "conclusion" in roles[:-1]:
                raise ScriptError(f"{sid} has an intro or conclusion beat out of place")
        elif any(roles):
            raise ScriptError(f"{sid} isn't a chapter, so its beats don't take intro or conclusion roles")
        sections.append(Section(id=sid, kind=kind, number=number, title=raw["title"], lines=beats))
    lines = [l for sec in sections for l in sec.lines]
    for raw in script.get("line", []):
        # [[line]] is always top-level in TOML, even written inside a
        # chapter, where it would silently play after the conclusions.
        if sections and not raw.get("closing", False):
            raise ScriptError(f"line {raw['id']} is a top-level [[line]] in a script with sections: "
                              "if it belongs in a section, write [[section.line]]; if it closes the video, "
                              "add closing = true")
        lines.append(make_line(raw, None))
    music = script.get("music")
    return Video(
        name=dir.name,
        dir=dir,
        title=script["title"],
        music=REPO / music if music else None,
        lines=tuple(lines),
        citations=citations,
        sections=tuple(sections),
    )
