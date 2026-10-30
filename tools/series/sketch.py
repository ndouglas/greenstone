"""Render one animation alone, with a contact sheet of stills to judge it by.

A sketch file holds one Manim scene named `Sketch`, with no narration. The
mp4 is for watching; the contact sheet (evenly spaced stills, each labelled
with its time) is for checking composition, clutter and legibility frame by
frame.

Usage: cd tools && series/.venv/bin/python -m series.sketch PATH [--final] [--stills N]
"""

import argparse
import importlib.util
import math
import subprocess
import tempfile
from pathlib import Path

from manim import tempconfig
from PIL import Image, ImageDraw

from series.build import ENCODE, OUT

STILL_WIDTH = 480


def still_times(duration: float, n: int) -> list[float]:
    """The middle of each of `n` equal slices of the clip."""
    return [(i + 0.5) * duration / n for i in range(n)]


def load_sketch(path: Path):
    spec = importlib.util.spec_from_file_location(f"series_sketch_{path.stem}", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    if not hasattr(module, "Sketch"):
        raise LookupError(f"{path} has no scene named Sketch")
    return module.Sketch


def contact_sheet(movie: Path, duration: float, n: int, out: Path) -> None:
    cols = math.ceil(math.sqrt(n))
    rows = math.ceil(n / cols)
    height = round(STILL_WIDTH * 9 / 16)
    sheet = Image.new("RGB", (cols * STILL_WIDTH, rows * height))
    with tempfile.TemporaryDirectory() as tmp:
        for i, t in enumerate(still_times(duration, n)):
            png = Path(tmp) / f"{i}.png"
            subprocess.run(["ffmpeg", "-y", "-v", "error", "-ss", f"{t:.3f}", "-i", str(movie), "-frames:v", "1",
                            "-vf", f"scale={STILL_WIDTH}:{height}", str(png)], check=True)
            still = Image.open(png).convert("RGB")
            ImageDraw.Draw(still).text((8, 6), f"{t:.2f}s", fill=(255, 255, 0))
            sheet.paste(still, ((i % cols) * STILL_WIDTH, (i // cols) * height))
    sheet.save(out)


def sketch(path: Path, draft: bool = True, stills: int = 9, out_root: Path = OUT) -> tuple[Path, Path]:
    path = Path(path).resolve()
    cls = load_sketch(path)
    out = Path(out_root) / path.parent.parent.name / "sketches"
    quality = ({"quality": "low_quality", "pixel_width": 854, "pixel_height": 480, "frame_rate": 30} if draft
               else {"quality": "high_quality", "frame_rate": 60})
    with tempconfig({**quality, "video_encoder_options": ENCODE, "media_dir": str(out / "media"), "disable_caching": True,
                     "verbosity": "WARNING", "progress_bar": "none"}):
        scene = cls()
        scene.render()
        rendered = Path(scene._get_manager().file_writer.movie_file_path)
        duration = scene.time
    movie = out / f"{path.stem}.mp4"
    movie.write_bytes(rendered.read_bytes())
    sheet = out / f"{path.stem}-sheet.png"
    contact_sheet(movie, duration, stills, sheet)
    return movie, sheet


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("path")
    parser.add_argument("--final", action="store_true")
    parser.add_argument("--stills", type=int, default=9)
    args = parser.parse_args()
    for p in sketch(Path(args.path), draft=not args.final, stills=args.stills):
        print(p)


if __name__ == "__main__":
    main()
