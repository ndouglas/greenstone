"""Build the reservoir film: one cached segment per chapter, then the score,
the chapter markers, and the final MP4.

    cd tools && python3 -m reservoir_film.build [--chapter N]... [--quick] [--no-music] [--contact]
"""

import argparse
import hashlib
import inspect
import subprocess
import sys
from pathlib import Path

import numpy as np
from PIL import Image

from . import draw
from .model import (FPS, H, TAKES_DIR, TITLE_SECONDS, W, Context, Takes, caption_spans, chapter_seconds,
                    check_captions, load_numbers, shot_seconds, shot_stings)

SEGMENTS = TAKES_DIR / "segments"
OUT = TAKES_DIR / "tetris-reservoir.mp4"
HERE = Path(__file__).resolve().parent


def chapter_starts(chapters, ctx):
    starts, t = [], 0.0
    for ch in chapters:
        starts.append(t)
        t += chapter_seconds(ch, ctx)
    return starts


def _stamp(seconds):
    s = int(round(seconds))
    return f"{s // 3600}:{s // 60 % 60:02d}:{s % 60:02d}" if s >= 3600 else f"{s // 60}:{s % 60:02d}"


def _label(ch):
    return ch.title if ch.number == 0 else f"{ch.number}. {ch.title}"


def youtube_chapters(chapters, ctx):
    return "".join(f"{_stamp(t)} {_label(ch)}\n" for t, ch in zip(chapter_starts(chapters, ctx), chapters))


def ffmetadata(chapters, ctx):
    lines = [";FFMETADATA1", "title=Can Tetris play Mario?"]
    starts = chapter_starts(chapters, ctx)
    for ch, start in zip(chapters, starts):
        end = start + chapter_seconds(ch, ctx)
        lines += ["[CHAPTER]", "TIMEBASE=1/1000", f"START={round(start * 1000)}", f"END={round(end * 1000)}",
                  f"title={_label(ch)}"]
    return "\n".join(lines) + "\n"


def segment_key(chapter, ctx, sources, take_times):
    h = hashlib.sha1()
    h.update(sources.encode())
    h.update(repr([(s.name, shot_seconds(s, ctx), [(c.at, c.text if isinstance(c.text, str) else "fn")
                                                  for c in s.captions]) for s in chapter.shots]).encode())
    h.update(repr(sorted(take_times.items())).encode())
    return h.hexdigest()


def _sources(chapter):
    module = sys.modules[chapter.shots[0].draw.__module__] if chapter.shots and chapter.shots[0].draw else None
    text = (HERE / "draw.py").read_text() + (HERE / "model.py").read_text()
    return text + (inspect.getsource(module) if module else "")


def _take_times(chapter, ctx):
    return {name: ctx.takes[name].mtime for shot in chapter.shots for name in shot.takes}


def frames_of(chapter, ctx, quick):
    step = 10 if quick else 1
    if chapter.number > 0:
        card = draw.title_card(chapter.number, chapter.title, chapter.subtitle)
        for _ in range(0, round(TITLE_SECONDS * FPS), step):
            yield card
    for shot in chapter.shots:
        spans = caption_spans(shot, ctx)
        for i in range(0, round(shot_seconds(shot, ctx) * FPS), step):
            t = i / FPS
            img = shot.draw(ctx, t, i)
            text = next((s[2] for s in spans if s[0] <= t < s[1]), "")
            if text:
                draw.caption(img, text)
            yield img


def render_segment(chapter, ctx, quick):
    SEGMENTS.mkdir(parents=True, exist_ok=True)
    name = f"{'quick-' if quick else ''}c{chapter.number:02d}"
    out, keyfile = SEGMENTS / f"{name}.mp4", SEGMENTS / f"{name}.key"
    key = segment_key(chapter, ctx, _sources(chapter), _take_times(chapter, ctx))
    if out.exists() and keyfile.exists() and keyfile.read_text() == key:
        print(f"chapter {chapter.number}: cached")
        return out
    fps = FPS // 10 if quick else FPS
    ff = subprocess.Popen(["ffmpeg", "-y", "-loglevel", "error", "-f", "rawvideo", "-pix_fmt", "rgb24",
                           "-s", f"{W}x{H}", "-r", str(fps), "-i", "-", "-c:v", "libx264", "-crf", "20",
                           "-pix_fmt", "yuv420p", str(out)], stdin=subprocess.PIPE)
    n = 0
    for img in frames_of(chapter, ctx, quick):
        ff.stdin.write(img.tobytes())
        n += 1
    ff.stdin.close()
    if ff.wait() != 0:
        raise RuntimeError(f"ffmpeg failed on chapter {chapter.number}")
    keyfile.write_text(key)
    print(f"chapter {chapter.number}: {n} frames")
    return out


def sampled_frames(chapter, ctx, every):
    """Frames every `every` seconds, captions included, drawing only those."""
    if chapter.number > 0:
        card = draw.title_card(chapter.number, chapter.title, chapter.subtitle)
        for _ in range(int(TITLE_SECONDS // every)):
            yield card
    for shot in chapter.shots:
        spans = caption_spans(shot, ctx)
        t = 0.0
        while t < shot_seconds(shot, ctx):
            img = shot.draw(ctx, t, round(t * FPS))
            text = next((s[2] for s in spans if s[0] <= t < s[1]), "")
            if text:
                draw.caption(img, text)
            yield img
            t += every


def contact_sheet(chapter, ctx):
    out = TAKES_DIR / "contact"
    out.mkdir(parents=True, exist_ok=True)
    thumbs = [img.resize((320, 180)) for img in sampled_frames(chapter, ctx, 2.0)]
    rows = (len(thumbs) + 7) // 8
    sheet = Image.new("RGB", (8 * 320, rows * 180))
    for k, th in enumerate(thumbs):
        sheet.paste(th, ((k % 8) * 320, (k // 8) * 180))
    sheet.save(out / f"c{chapter.number:02d}.png")


def main():
    from .chapters import CHAPTERS
    from .music.cues import CHAPTER_CUES, CUES, STINGS
    from .music.score import score, write_wav

    ap = argparse.ArgumentParser()
    ap.add_argument("--chapter", type=int, action="append")
    ap.add_argument("--quick", action="store_true")
    ap.add_argument("--no-music", action="store_true")
    ap.add_argument("--contact", action="store_true")
    args = ap.parse_args()
    ctx = Context(takes=Takes(), numbers=load_numbers())
    chosen = [c for c in CHAPTERS if not args.chapter or c.number in args.chapter]
    problems = [p for c in chosen for p in check_captions(c, ctx)]
    if problems:
        sys.exit("captions too short to read:\n" + "\n".join(problems))
    if args.contact:
        for c in chosen:
            contact_sheet(c, ctx)
        return
    segments = [render_segment(c, ctx, args.quick) for c in chosen]
    if args.chapter or args.quick:
        return
    listing = SEGMENTS / "segments.txt"
    listing.write_text("".join(f"file '{s}'\n" for s in segments))
    video = SEGMENTS / "video.mp4"
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i", str(listing),
                    "-c", "copy", str(video)], check=True)
    meta = SEGMENTS / "chapters.ffmeta"
    meta.write_text(ffmetadata(CHAPTERS, ctx))
    (TAKES_DIR / "chapters.txt").write_text(youtube_chapters(CHAPTERS, ctx))
    cmd = ["ffmpeg", "-y", "-loglevel", "error", "-i", str(video), "-i", str(meta)]
    if not args.no_music:
        starts = chapter_starts(CHAPTERS, ctx)
        plan = [(CUES[CHAPTER_CUES[c.number]], chapter_seconds(c, ctx)) for c in CHAPTERS]
        stings = []
        for c, start in zip(CHAPTERS, starts):
            at = start + (TITLE_SECONDS if c.number > 0 else 0)
            for shot in c.shots:
                stings += [(at + s, STINGS[name]) for s, name in shot_stings(shot, ctx)]
                at += shot_seconds(shot, ctx)
        wav = SEGMENTS / "score.wav"
        write_wav(score(plan, stings, SEGMENTS / "music"), wav)
        cmd += ["-i", str(wav), "-map", "0:v", "-map", "2:a", "-af", "loudnorm=I=-16:TP=-1.5:LRA=11",
                "-c:a", "aac", "-b:a", "192k"]
    else:
        cmd += ["-map", "0:v"]
    cmd += ["-map_metadata", "1", "-map_chapters", "1", "-c:v", "copy", str(OUT)]
    subprocess.run(cmd, check=True)
    print(f"wrote {OUT}")


if __name__ == "__main__":
    main()
