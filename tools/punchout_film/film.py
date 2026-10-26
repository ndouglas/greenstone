"""The Punch-Out!! reservoir film, as the compositor builds it."""

from pathlib import Path

from reservoir_film.model import REPO, Film

HERE = Path(__file__).resolve().parent
OUT_DIR = REPO / "target" / "punchout-film"


def _chapters():
    from .chapters import CHAPTERS
    return CHAPTERS


def _cues():
    from .music import CHAPTER_CUES, CUES, STINGS
    return CUES, CHAPTER_CUES, STINGS


PUNCHOUT = Film(name="punchout", title="Reservoir Punch-Out!!", takes_dir=OUT_DIR / "takes",
                out=OUT_DIR / "punchout-reservoir.mp4", segments=OUT_DIR / "segments",
                record_hint="cargo run --release --example punchout -- film-takes --only {name}",
                numbers=HERE / "numbers.json", chapters=_chapters, cues=_cues,
                sources=[REPO / "tools" / "reservoir_film" / "draw.py", REPO / "tools" / "reservoir_film" / "model.py",
                         HERE],
                strict_score=True)
