"""The Tetris reservoir film, as the compositor builds it."""

from pathlib import Path

from .model import NUMBERS, RECORD_HINT, TAKES_DIR, Film

HERE = Path(__file__).resolve().parent


def _chapters():
    from .chapters import CHAPTERS
    return CHAPTERS


def _cues():
    from .music.cues import CHAPTER_CUES, CUES, STINGS
    return CUES, CHAPTER_CUES, STINGS


TETRIS = Film(name="tetris", title="Can Tetris play Mario?", takes_dir=TAKES_DIR,
              out=TAKES_DIR / "tetris-reservoir.mp4", segments=TAKES_DIR / "segments", record_hint=RECORD_HINT,
              numbers=NUMBERS, chapters=_chapters, cues=_cues, sources=[HERE / "draw.py", HERE / "model.py",
                                                                        HERE / "chapters"])
