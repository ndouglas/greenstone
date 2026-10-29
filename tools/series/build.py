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
FADE_SECONDS = 2.0
# Peak level of the final mix (about -1 dBFS).
LIMIT = 0.89
__all__ = ["CREDITS_SECONDS", "build"]


def scene_class(dir: Path) -> type[NarratedScene]:
    """The video's own scene: `VideoScene` in its scene.py."""
    path = dir / "scene.py"
    if not path.exists():
        raise FileNotFoundError(f"{path} doesn't exist; build with --storyboard until it does")
    spec = importlib.util.spec_from_file_location(f"series_video_{dir.name.replace('-', '_')}", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    if not hasattr(module, "VideoScene"):
        raise LookupError(f"{path} has no scene named VideoScene")
    return module.VideoScene


def render(cls: type[NarratedScene], media: Path, draft: bool) -> tuple[Path, NarratedScene]:
    quality = {"quality": "low_quality", "pixel_width": 854, "pixel_height": 480} if draft else {"quality": "high_quality"}
    with tempconfig({**quality, "frame_rate": 60, "media_dir": str(media), "disable_caching": True,
                     "verbosity": "WARNING", "progress_bar": "none"}):
        scene = cls()
        scene.render()
        return Path(scene._get_manager().file_writer.movie_file_path), scene


def duration(path: Path) -> float:
    out = subprocess.run(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", str(path)],
                         capture_output=True, text=True, check=True).stdout
    return float(out)


def mix(movie: Path, music: Path | None, out: Path) -> None:
    """Lay the music bed under the narration: looped to the video's length,
    faded out over its last FADE_SECONDS, and limited so the mix can't clip."""
    if music is None:
        shutil.copyfile(movie, out)
        return
    # Manim's audio ends with the last line, so pad it to the whole video
    # (the credits run on after it) before laying the bed under it.
    length = duration(movie)
    fade_at = max(0.0, length - FADE_SECONDS)
    graph = (f"[0:a]apad=whole_dur={length:.3f}[voice];"
             f"[1:a]volume={MUSIC_DB}dB,afade=t=out:st={fade_at:.3f}:d={FADE_SECONDS}[bed];"
             f"[voice][bed]amix=inputs=2:duration=first:normalize=0,alimiter=limit={LIMIT}[a]")
    subprocess.run(["ffmpeg", "-y", "-v", "error", "-i", str(movie), "-stream_loop", "-1", "-i", str(music),
                    "-filter_complex", graph,
                    "-map", "0:v", "-map", "[a]", "-c:v", "copy", "-c:a", "aac", str(out)], check=True)


def build(name: str, engine: Engine, draft: bool = False, storyboard: bool = False,
          videos: Path = VIDEOS, out_root: Path = OUT) -> Path:
    video = load_video(Path(videos) / name)
    if video.music is not None and not video.music.exists():
        raise FileNotFoundError(f"{name}'s music, {video.music}, doesn't exist; render it, or remove music from script.toml")
    out = Path(out_root) / name
    base = StoryboardScene if storyboard else scene_class(video.dir)
    for problem in take_problems(video):
        print(f"TAKE: {problem}")
    found, stale = voices(video, engine, out)
    # Report the voices before rendering, so a failed render doesn't hide them.
    for line in video.lines:
        print(f"{line.id}: {found[line.id].source}")
    for line_id in stale:
        print(f"STALE take: {line_id} was recorded from different text; re-record it (narrator used)")
    cls = type(base.__name__, (base,), {"video": video, "voices": found, "require_credits": True})
    movie, _ = render(cls, out / "media", draft)
    final = out / f"{name}{'-storyboard' if storyboard else ''}.mp4"
    mix(movie, video.music, final)
    (out / "description.txt").write_text(description(video))
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
