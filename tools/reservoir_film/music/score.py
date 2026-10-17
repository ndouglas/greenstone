"""Fit cues to chapter lengths, render them on the APU, and mix the score."""

import hashlib
import pathlib
import subprocess
import wave

import numpy as np

from .core import NES_FRAME_HZ, compile_cue, write_log

SAMPLE_RATE = 44100
CROSSFADE = 0.3
# How far a cue's tempo may stretch to end with its chapter. An 8-bar loop
# is about 15 s, so a 40 s chapter can need 15%.
TEMPO_RANGE = (0.85, 1.15)
REPO = pathlib.Path(__file__).resolve().parents[3]


def cue_seconds(cue, loops, bpm):
    frames, _ = compile_cue(cue, loops, bpm)
    return frames / NES_FRAME_HZ


def fit(cue, seconds):
    lo, hi = TEMPO_RANGE
    best = None
    loops = 0
    while True:
        nominal = cue_seconds(cue, loops, cue.bpm)
        if nominal / hi > seconds and loops == 0:
            return 0, cue.bpm * hi, True
        if nominal / hi > seconds:
            break
        scale = nominal / seconds
        if lo <= scale <= hi:
            cand = (abs(scale - 1), loops, cue.bpm * scale)
            best = min(best, cand) if best else cand
        loops += 1
    if best is None:
        # Nothing fits: take the loop count that comes closest, at the
        # nearest allowed tempo, and let trim() pad or fade the rest.
        n = max(0, loops - 1)
        scale = min(hi, max(lo, cue_seconds(cue, n, cue.bpm) / seconds))
        return n, cue.bpm * scale, True
    _, loops, bpm = best
    # Rounding to frames can leave the cue a frame off; nudge the tempo.
    for _ in range(20):
        err = cue_seconds(cue, loops, bpm) - seconds
        if abs(err) < 1 / NES_FRAME_HZ:
            break
        bpm *= cue_seconds(cue, loops, bpm) / seconds
    return loops, bpm, False


def trim(samples, seconds, fade=1.0):
    n = round(seconds * SAMPLE_RATE)
    out = np.zeros(n, dtype=np.float32)
    k = min(n, len(samples))
    out[:k] = samples[:k]
    if len(samples) > n:
        f = min(n, round(fade * SAMPLE_RATE))
        out[n - f:] *= np.linspace(1.0, 0.0, f, dtype=np.float32)
    return out


def crossfade_join(parts):
    """Each part keeps its own slot (the sum of the earlier parts' lengths)
    but starts CROSSFADE seconds early, fading in over the previous part's
    last CROSSFADE seconds as that part fades out."""
    out = np.zeros(sum(len(p) for p in parts), dtype=np.float32)
    c = round(CROSSFADE * SAMPLE_RATE)
    t = np.linspace(0, np.pi / 2, c, dtype=np.float32)
    at = 0
    for i, p in enumerate(parts):
        seg = p.astype(np.float32).copy()
        start = at - c if i > 0 and at >= c and len(seg) > c else at
        if start < at:
            seg[:c] *= np.sin(t)
            out[start:at] *= np.cos(t)
        end = min(len(out), start + len(seg))
        out[start:end] += seg[: end - start]
        at += len(p)
    return out


def render_cue(cue, loops, bpm, work_dir):
    work = pathlib.Path(work_dir)
    work.mkdir(parents=True, exist_ok=True)
    frames, writes = compile_cue(cue, loops, bpm)
    log = work / f"{cue.name}-{loops}-{bpm:.3f}.log"
    write_log(frames, writes, log)
    key = hashlib.sha1(log.read_bytes()).hexdigest()[:12]
    wav = work / f"{cue.name}-{key}.wav"
    if not wav.exists():
        subprocess.run([str(renderer()), str(log), str(wav)], check=True)
    return read_wav(wav)


_built = []


def renderer():
    """Build examples/apu_render.rs once per process, quietly, and return its path."""
    if not _built:
        done = subprocess.run(["cargo", "build", "-q", "--release", "--example", "apu_render"], cwd=REPO,
                              capture_output=True, text=True)
        if done.returncode != 0:
            raise RuntimeError("building apu_render failed:\n" + done.stderr)
        _built.append(REPO / "target" / "release" / "examples" / "apu_render")
    return _built[0]


def read_wav(path):
    with wave.open(str(path)) as w:
        data = np.frombuffer(w.readframes(w.getnframes()), dtype="<i2")
    return data.astype(np.float32) / 32767


def write_wav(samples, path):
    data = (np.clip(samples, -1, 1) * 32767).astype("<i2")
    with wave.open(str(path), "w") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(SAMPLE_RATE)
        w.writeframes(data.tobytes())


def score(plan, stings, work_dir):
    parts = []
    for cue, seconds in plan:
        loops, bpm, _ = fit(cue, seconds)
        parts.append(trim(render_cue(cue, loops, bpm, work_dir), seconds))
    mix = crossfade_join(parts)
    for at, sting in stings:
        s = render_cue(sting, 1, sting.bpm, work_dir)
        i = round(at * SAMPLE_RATE)
        j = min(len(mix), i + len(s))
        mix[i:j] += s[: j - i]
    peak = float(np.max(np.abs(mix))) or 1.0
    return mix * (10 ** (-1 / 20) / peak)
