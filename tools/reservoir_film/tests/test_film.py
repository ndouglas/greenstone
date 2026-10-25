import tempfile
import unittest
from pathlib import Path

from reservoir_film import build
from reservoir_film.film import TETRIS
from reservoir_film.model import TAKES_DIR, Context, Film, Takes

CTX = Context(takes=None, numbers={})


class FilmDescriptor(unittest.TestCase):
    def test_tetris_film_is_the_default(self):
        self.assertEqual(TETRIS.out, TAKES_DIR / "tetris-reservoir.mp4")
        self.assertEqual(TETRIS.title, "Can Tetris play Mario?")
        self.assertEqual(TETRIS.segments, TAKES_DIR / "segments")

    def test_missing_take_hint_names_the_films_command(self):
        with tempfile.TemporaryDirectory() as tmp:
            takes = Takes(tmp, hint="cargo run --release --example punchout -- film-takes --only {name}")
            with self.assertRaises(FileNotFoundError) as caught:
                takes["nope"]
            self.assertIn("--only nope", str(caught.exception))

    def test_ffmetadata_uses_the_films_title(self):
        self.assertIn("title=X", build.ffmetadata([], CTX, "X"))

    def test_sources_include_the_diagram_and_shared_modules(self):
        with tempfile.TemporaryDirectory() as tmp:
            (Path(tmp) / "diagram.py").write_text("# the diagram\n")
            film = Film(name="t", title="T", takes_dir=Path(tmp), out=Path(tmp) / "o.mp4", segments=Path(tmp) / "s",
                        record_hint="{name}", numbers=Path(tmp) / "n.json", chapters=list, cues=tuple,
                        sources=[build.HERE, Path(tmp)])
            text = build._sources(film)
            self.assertIn((build.HERE / "draw.py").read_text(), text)
            self.assertIn("# the diagram", text)
