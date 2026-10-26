"""Build a film: one cached segment per chapter, then the score, the chapter
markers, and the final MP4. Run as a module, it builds the Tetris film:

    cd tools && python3 -m reservoir_film.build [--chapter N]... [--quick] [--no-music] [--contact]
"""

import argparse
import hashlib
import json
import subprocess
import sys
from pathlib import Path

import numpy as np
from PIL import Image

from . import draw
from .model import (FPS, H, TITLE_SECONDS, W, Context, Takes, caption_spans, chapter_seconds, check_captions,
                    load_numbers, shot_seconds, shot_stings)

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


def ffmetadata(chapters, ctx, title):
    lines = [";FFMETADATA1", f"title={title}"]
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
    h.update(json.dumps(ctx.numbers, sort_keys=True).encode())
    return h.hexdigest()


def _span_starts(chapter, spans, ctx):
    """Seconds into the chapter (title card included) where each span begins."""
    title = TITLE_SECONDS if chapter.number > 0 else 0.0
    shot_at, t = {}, title
    for shot in chapter.shots:
        shot_at[shot.name] = (t, shot_seconds(shot, ctx))
        t += shot_seconds(shot, ctx)
    starts = []
    for shot, offset, _ in spans:
        if shot not in shot_at:
            raise ValueError(f"chapter {chapter.number} has no shot {shot}")
        offset = offset(ctx) if callable(offset) else offset
        at, length = shot_at[shot]
        if not 0 <= offset < length:
            raise ValueError(f"span at {chapter.number}/{shot} starts {offset}s in, past the shot's {length}s")
        starts.append(at + offset)
    if starts[0] != title or spans[0][0] != chapter.shots[0].name:
        raise ValueError(f"chapter {chapter.number}'s first span must start its first shot")
    if starts != sorted(starts):
        raise ValueError(f"chapter {chapter.number}'s spans are out of order")
    return [0.0] + starts[1:]


def cue_plan(chapters, chapter_cues, cues, ctx):
    """(cue, seconds, label) for each stretch of the film with one cue. A
    chapter's entry is a cue name for the whole chapter, or a list of
    (shot, offset, cue) spans, each running until the next begins; the
    title card belongs to the first."""
    plan = []
    for ch in chapters:
        entry = chapter_cues[ch.number]
        total = chapter_seconds(ch, ctx)
        if isinstance(entry, str):
            plan.append((cues[entry], total, f"{ch.number}/{ch.shots[0].name}"))
            continue
        starts = _span_starts(ch, entry, ctx)
        for (shot, _, name), start, end in zip(entry, starts, starts[1:] + [total]):
            plan.append((cues[name], end - start, f"{ch.number}/{shot}"))
    return plan


def unfit_spans(plan):
    """The spans whose cue can't be stretched to fit without trimming or padding."""
    from .music.score import fit
    return [f"{label} ({cue.name})" for cue, seconds, label in plan if fit(cue, seconds)[2]]


def _sources(film):
    # Every module the film's chapters draw with, since chapters borrow
    # helpers from each other and from the shared widgets.
    files = []
    for path in film.sources:
        path = Path(path)
        files += sorted(path.rglob("*.py")) if path.is_dir() else [path]
    return "".join(p.read_text() for p in files)


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


def render_segment(film, chapter, ctx, quick):
    film.segments.mkdir(parents=True, exist_ok=True)
    name = f"{'quick-' if quick else ''}c{chapter.number:02d}"
    out, keyfile = film.segments / f"{name}.mp4", film.segments / f"{name}.key"
    key = segment_key(chapter, ctx, _sources(film), _take_times(chapter, ctx))
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


def contact_sheet(film, chapter, ctx):
    out = film.segments.parent / "contact"
    out.mkdir(parents=True, exist_ok=True)
    thumbs = [img.resize((320, 180)) for img in sampled_frames(chapter, ctx, 2.0)]
    rows = (len(thumbs) + 7) // 8
    sheet = Image.new("RGB", (8 * 320, rows * 180))
    for k, th in enumerate(thumbs):
        sheet.paste(th, ((k % 8) * 320, (k // 8) * 180))
    sheet.save(out / f"c{chapter.number:02d}.png")


def run(film, argv):
    from .music.score import score, write_wav

    CHAPTERS = film.chapters()
    CUES, CHAPTER_CUES, STINGS = film.cues()
    SEGMENTS = film.segments
    ap = argparse.ArgumentParser()
    ap.add_argument("--chapter", type=int, action="append")
    ap.add_argument("--quick", action="store_true")
    ap.add_argument("--no-music", action="store_true")
    ap.add_argument("--contact", action="store_true")
    args = ap.parse_args(argv)
    ctx = Context(takes=Takes(film.takes_dir, film.record_hint), numbers=load_numbers(film.numbers))
    chosen = [c for c in CHAPTERS if not args.chapter or c.number in args.chapter]
    problems = [p for c in chosen for p in check_captions(c, ctx)]
    if problems:
        sys.exit("caption problems:\n" + "\n".join(problems))
    if args.contact:
        for c in chosen:
            contact_sheet(film, c, ctx)
        return
    segments = [render_segment(film, c, ctx, args.quick) for c in chosen]
    if args.chapter or args.quick:
        return
    listing = SEGMENTS / "segments.txt"
    listing.write_text("".join(f"file '{s}'\n" for s in segments))
    video = SEGMENTS / "video.mp4"
    subprocess.run(["ffmpeg", "-y", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i", str(listing),
                    "-c", "copy", str(video)], check=True)
    meta = SEGMENTS / "chapters.ffmeta"
    meta.write_text(ffmetadata(CHAPTERS, ctx, film.title))
    (film.out.parent / "chapters.txt").write_text(youtube_chapters(CHAPTERS, ctx))
    cmd = ["ffmpeg", "-y", "-loglevel", "error", "-i", str(video), "-i", str(meta)]
    if not args.no_music:
        starts = chapter_starts(CHAPTERS, ctx)
        spans = cue_plan(CHAPTERS, CHAPTER_CUES, CUES, ctx)
        unfit = unfit_spans(spans)
        if unfit and film.strict_score:
            sys.exit("cues that don't fit their spans:\n" + "\n".join(unfit))
        plan = [(cue, seconds) for cue, seconds, _ in spans]
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
    cmd += ["-map_metadata", "1", "-map_chapters", "1", "-c:v", "copy", str(film.out)]
    subprocess.run(cmd, check=True)
    print(f"wrote {film.out}")


def main():
    from .film import TETRIS
    run(TETRIS, sys.argv[1:])


if __name__ == "__main__":
    main()
