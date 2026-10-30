"""The sound of a recorded take: one fixed chain of filters, run with
ffmpeg when a video is built, so the raw takes never change and the chain
can be retuned without recording again."""

import hashlib
import json
import re
import subprocess
from pathlib import Path

import numpy as np
import soundfile

from series.takes import Take, write_atomically

# Part of every cache key, with every value below: change any and every take is reprocessed.
CHAIN_VERSION = 1
TARGET_LUFS = -16.0
# Peak level of a processed take (about -1 dBFS).
LIMIT = 0.89
RATE = 48000
# Starting values, to be tuned by ear: rumble, hiss, sibilance, evenness, presence.
FILTERS = ("highpass=f=80,afftdn=nr=10:nf=-50,deesser=i=0.4,"
           "acompressor=threshold=-20dB:ratio=3:attack=5:release=80:makeup=2,"
           "equalizer=f=4000:t=q:w=1:g=2")
# A take with almost no level would otherwise be boosted without limit.
MAX_GAIN_DB = 40.0
# The room tone loops; its end is faded into its start over this long, so the loop has no seam.
LOOP_FADE = 0.5


def ffmpeg(*args: str) -> str:
    return subprocess.run(["ffmpeg", "-hide_banner", "-nostats", *args],
                          capture_output=True, text=True, check=True).stderr


def measure(source: Path, chain: str) -> float:
    """Integrated loudness, in LUFS, of `source` through `chain`."""
    err = ffmpeg("-f", "wav", "-i", str(source), "-af", f"{chain},ebur128", "-f", "null", "-")
    found = re.findall(r"I:\s+(-?[\d.]+|-inf) LUFS", err)
    return float(found[-1])


def loudness(path: Path) -> float:
    return measure(path, "anull")


def render(source: Path, chain: str, out: Path) -> None:
    write_atomically(out, lambda p: ffmpeg("-y", "-f", "wav", "-i", str(source), "-af", chain, "-ac", "1", "-ar", str(RATE),
                                           "-c:a", "pcm_s16le", str(p)))


def key(path: Path, *parts) -> str:
    tuned = (CHAIN_VERSION, FILTERS, TARGET_LUFS, LIMIT, MAX_GAIN_DB, RATE)
    h = hashlib.sha256(f"{tuned}\0{parts}".encode())
    h.update(path.read_bytes())
    return h.hexdigest()[:12]


def limiter() -> str:
    # level=false: limit peaks only, don't renormalize the whole take.
    return f"alimiter=limit={LIMIT}:level=false"


def cache_path(take: Take, cache: Path) -> Path:
    """Where `process` keeps a take (its gain beside it, as .json)."""
    return Path(cache) / f"{take.line_id}-{take.number}-{key(take.path, take.start, take.end)}.wav"


def loopable(audio: np.ndarray, rate: int) -> np.ndarray:
    """The audio shortened by LOOP_FADE, its end faded into its start, so it
    can repeat with no seam (equal-power: room noise doesn't correlate)."""
    n = min(int(LOOP_FADE * rate), len(audio) // 2)
    if n == 0:
        return audio
    t = np.linspace(0.0, 1.0, n, endpoint=False)
    out = audio[:len(audio) - n].copy()
    out[:n] = audio[:n] * np.sin(t * np.pi / 2) + audio[len(audio) - n:] * np.cos(t * np.pi / 2)
    return out


def process(take: Take, cache: Path) -> tuple[Path, float]:
    """The take trimmed and through the chain, matched to TARGET_LUFS; and the
    gain, in dB, that matched it."""
    out = cache_path(take, cache)
    gain_file = out.with_suffix(".json")
    if out.exists() and gain_file.exists():
        return out, json.loads(gain_file.read_text())["gain_db"]
    trimmed = f"atrim=start={take.start}:end={take.end},asetpts=PTS-STARTPTS,{FILTERS}"
    gain = max(-MAX_GAIN_DB, min(MAX_GAIN_DB, TARGET_LUFS - measure(take.path, trimmed)))
    render(take.path, f"{trimmed},volume={gain:.3f}dB,{limiter()}", out)
    write_atomically(gain_file, lambda p: p.write_text(json.dumps({"gain_db": gain})))
    return out, gain


def process_room(path: Path, gain_db: float, cache: Path) -> Path:
    """The room's quiet through the chain at a given gain (the takes' typical
    gain), so it sits at the takes' own noise floor."""
    out = Path(cache) / f"room-{key(path, round(gain_db, 3), LOOP_FADE)}.wav"
    if not out.exists():
        render(path, f"{FILTERS},volume={gain_db:.3f}dB,{limiter()}", out)
        audio, rate = soundfile.read(out, dtype="float32")
        write_atomically(out, lambda p: soundfile.write(p, loopable(audio, rate), rate, subtype="PCM_16"))
    return out
