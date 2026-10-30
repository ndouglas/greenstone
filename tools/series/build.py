"""Build a series video: voices, render, music, description.

Usage: cd tools && series/.venv/bin/python -m series.build NAME [--draft] [--storyboard] [--release]
"""

import argparse
import importlib.util
import shutil
import statistics
import subprocess
from pathlib import Path

from manim import tempconfig

from series.credits import description
from series.pacing import clock, lengths, too_long
from series.scene import NarratedScene, ScriptOrderError, StoryboardScene
from series.script import REPO, VIDEOS, load_video
from series.sound import process_room
from series.takes import TakeStore
from series.voice import TAKES, Engine, KokoroEngine, Voice, take_problems, voices

OUT = REPO / "target" / "series"
# The music bed sits this far under the narration.
MUSIC_DB = -18
FADE_SECONDS = 2.0
# Peak level of the final mix (about -1 dBFS).
LIMIT = 0.89
__all__ = ["NotNarrated", "SectionTooLong", "build"]


class SectionTooLong(Exception):
    pass


class NotNarrated(Exception):
    """A release build with beats not in Nate's voice."""


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


def mix(movie: Path, music: Path | None, out: Path, room: Path | None = None, room_until: float | None = None) -> None:
    """Lay the room tone under the narration, up to the credits, and the
    music bed under both: looped to the video's length, faded out over its
    last FADE_SECONDS, and limited so the mix can't clip."""
    if music is None and room is None:
        shutil.copyfile(movie, out)
        return
    # Manim's audio ends with the last line, so pad it to the whole video
    # (the credits run on after it) before laying anything under it.
    length = duration(movie)
    inputs, graph, voice = ["-i", str(movie)], [f"[0:a]apad=whole_dur={length:.3f}[voice]"], "[voice]"
    if room is not None:
        until = length if room_until is None else room_until
        inputs += ["-stream_loop", "-1", "-i", str(room)]
        graph.append(f"[{inputs.count('-i') - 1}:a]atrim=end={until:.3f},afade=t=out:st={max(0.0, until - 0.5):.3f}:d=0.5,"
                     f"apad=whole_dur={length:.3f}[room]")
        graph.append("[voice][room]amix=inputs=2:duration=first:normalize=0[spoken]")
        voice = "[spoken]"
    if music is not None:
        fade_at = max(0.0, length - FADE_SECONDS)
        inputs += ["-stream_loop", "-1", "-i", str(music)]
        graph.append(f"[{inputs.count('-i') - 1}:a]volume={MUSIC_DB}dB,afade=t=out:st={fade_at:.3f}:d={FADE_SECONDS}[bed]")
        graph.append(f"{voice}[bed]amix=inputs=2:duration=first:normalize=0[mixed]")
        voice = "[mixed]"
    graph.append(f"{voice}alimiter=limit={LIMIT}[a]")
    subprocess.run(["ffmpeg", "-y", "-v", "error", *inputs, "-filter_complex", ";".join(graph),
                    "-map", "0:v", "-map", "[a]", "-c:v", "copy", "-c:a", "aac", str(out)], check=True)


def room_tone(video, found: dict[str, Voice], out: Path) -> Path | None:
    """The room's quiet at the takes' typical gain, when every beat is a take."""
    room = TakeStore(video.dir / TAKES).room()
    if room is None or any(v.source != "take" for v in found.values()):
        return None
    return process_room(room, statistics.median(v.gain_db for v in found.values()), out / "room")


def build(name: str, engine: Engine, draft: bool = False, storyboard: bool = False, release: bool = False,
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
    narrated = [line.id for line in video.lines if found[line.id].source == "synth"]
    if not storyboard:
        print(f"{len(narrated)} of {len(video.lines)} beats narrated by Kokoro")
    if release and narrated:
        raise NotNarrated(f"a release is in Nate's voice, but {len(narrated)} of {len(video.lines)} beats "
                          f"aren't current takes: {', '.join(narrated)}")
    estimate = lengths(video, found)
    for section in video.sections:
        print(f"{section.id} {section.title} {clock(estimate[section.id])}")
    if problems := too_long(video, estimate):
        raise SectionTooLong("; ".join(problems))
    cls = type(base.__name__, (base,), {"video": video, "voices": found, "require_credits": True})
    movie, scene = render(cls, out / "media", draft)
    # The scene's own waits and animations count too: check the lengths it really ran.
    if problems := too_long(video, scene.section_lengths):
        raise SectionTooLong("as rendered: " + "; ".join(problems))
    final = out / f"{name}{'-storyboard' if storyboard else ''}.mp4"
    mix(movie, video.music, final, room_tone(video, found, out), scene.credits_at)
    (out / "description.txt").write_text(description(video))
    print(final)
    return final


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("name")
    parser.add_argument("--draft", action="store_true")
    parser.add_argument("--storyboard", action="store_true")
    parser.add_argument("--release", action="store_true",
                        help="refuse unless every beat is a current take in Nate's voice")
    args = parser.parse_args()
    build(args.name, KokoroEngine(), draft=args.draft, storyboard=args.storyboard, release=args.release)


if __name__ == "__main__":
    main()
