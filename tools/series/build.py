"""Build a series video: voices, render, music, description.

Usage: cd tools && series/.venv/bin/python -m series.build NAME [--draft] [--storyboard]
"""

import argparse
import importlib.util
import shutil
import subprocess
from pathlib import Path

from manim import tempconfig

from series.credits import description
from series.scene import CREDITS_SECONDS, NarratedScene, ScriptOrderError, StoryboardScene
from series.script import REPO, VIDEOS, load_video
from series.voice import Engine, KokoroEngine, take_problems, voices

OUT = REPO / "target" / "series"
# The music bed sits this far under the narration.
MUSIC_DB = -18
__all__ = ["CREDITS_SECONDS", "build"]


def scene_class(dir: Path) -> type[NarratedScene]:
    """The video's own scene: `VideoScene` in its scene.py."""
    path = dir / "scene.py"
    if not path.exists():
        raise FileNotFoundError(f"{path} doesn't exist; build with --storyboard until it does")
    spec = importlib.util.spec_from_file_location(f"series_video_{dir.name.replace('-', '_')}", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.VideoScene


def render(cls: type[NarratedScene], media: Path, draft: bool) -> tuple[Path, NarratedScene]:
    quality = {"quality": "low_quality", "pixel_width": 854, "pixel_height": 480} if draft else {"quality": "high_quality"}
    with tempconfig({**quality, "frame_rate": 60, "media_dir": str(media), "disable_caching": True,
                     "verbosity": "WARNING", "progress_bar": "none"}):
        scene = cls()
        scene.render()
        return Path(scene._get_manager().file_writer.movie_file_path), scene


def mix(movie: Path, music: Path | None, out: Path) -> None:
    if music is None:
        shutil.copyfile(movie, out)
        return
    graph = f"[1:a]volume={MUSIC_DB}dB[bed];[0:a][bed]amix=inputs=2:duration=first:normalize=0[a]"
    subprocess.run(["ffmpeg", "-y", "-v", "error", "-i", str(movie), "-i", str(music), "-filter_complex", graph,
                    "-map", "0:v", "-map", "[a]", "-c:v", "copy", "-c:a", "aac", str(out)], check=True)


def build(name: str, engine: Engine, draft: bool = False, storyboard: bool = False,
          videos: Path = VIDEOS, out_root: Path = OUT) -> Path:
    video = load_video(Path(videos) / name)
    if video.music is not None and not video.music.exists():
        raise FileNotFoundError(f"{name}'s music, {video.music}, doesn't exist; render it, or remove music from script.toml")
    out = Path(out_root) / name
    problems = take_problems(video)
    for problem in problems:
        print(f"TAKE: {problem}")
    found, stale = voices(video, engine, out)
    base = StoryboardScene if storyboard else scene_class(video.dir)
    cls = type(base.__name__, (base,), {"video": video, "voices": found})
    movie, scene = render(cls, out / "media", draft)
    if not scene.credits_rolled:
        raise ScriptOrderError(f"{name}'s scene never rolls the credits")
    final = out / f"{name}{'-storyboard' if storyboard else ''}.mp4"
    mix(movie, video.music, final)
    (out / "description.txt").write_text(description(video))
    for line in video.lines:
        print(f"{line.id}: {found[line.id].source}")
    for line_id in stale:
        print(f"STALE take: {line_id} was recorded from different text; re-record it (narrator used)")
    print(final)
    return final


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("name")
    parser.add_argument("--draft", action="store_true")
    parser.add_argument("--storyboard", action="store_true")
    args = parser.parse_args()
    build(args.name, KokoroEngine(), draft=args.draft, storyboard=args.storyboard)


if __name__ == "__main__":
    main()
