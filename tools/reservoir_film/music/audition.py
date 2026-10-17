"""Render cues and stings alone, with a spectrogram each, for listening and checking.

    cd tools && python3 -m reservoir_film.music.audition [name ...]
"""

import pathlib
import sys

import numpy as np
from PIL import Image

from .cues import CUES, STINGS
from .score import REPO, SAMPLE_RATE, render_cue, write_wav

OUT = REPO / "target" / "reservoir-film" / "music"


def spectrogram(x, path, width=1200, height=300):
    n = 2048
    hop = max(1, (len(x) - n) // width)
    cols = []
    for i in range(0, max(1, len(x) - n), hop):
        frame = x[i:i + n] * np.hanning(n)
        cols.append(np.abs(np.fft.rfft(frame))[: n // 4])
    s = np.log1p(np.array(cols).T)
    s = (255 * s / (s.max() or 1)).astype(np.uint8)[::-1]
    Image.fromarray(s).resize((width, height)).save(path)


def main(names):
    OUT.mkdir(parents=True, exist_ok=True)
    every = {**CUES, **STINGS}
    for name in names or list(every):
        cue = every[name]
        x = render_cue(cue, 1 if name in STINGS else 2, cue.bpm, OUT / "work")
        write_wav(x, OUT / f"{name}.wav")
        spectrogram(x, OUT / f"{name}.png")
        peak = float(np.max(np.abs(x)))
        silent = float(np.mean(np.abs(x) < 1e-3))
        print(f"{name}: {len(x) / SAMPLE_RATE:.1f} s, peak {peak:.2f}, silent {silent:.0%}")


if __name__ == "__main__":
    main(sys.argv[1:])
