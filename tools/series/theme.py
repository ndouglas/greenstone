"""The series' theme, under the opening: Glass-like, in the NES's voices. Four chords (A minor, F, C, E), the
first pulse in rising-and-falling triplets that grow louder, the second joining on the second chord in pairs,
two against three; then A minor circling on, quieter each beat, fading with the picture to black."""

import hashlib
from pathlib import Path

import numpy as np
import soundfile

from series import chiptune
from series.opening import OPENING_SECONDS

CHORDS = [  # (first pulse's arpeggio, second pulse's pair, the triangle's bass)
    (["A4", "C5", "E5"], ["A3", "E4"], "A2"),
    (["A4", "C5", "F5"], ["F3", "C4"], "F2"),
    (["G4", "C5", "E5"], ["C4", "G4"], "C3"),
    (["G#4", "B4", "E5"], ["E3", "B3"], "E2"),
]
CHORD = 1.1  # seconds
BEAT = CHORD / 4
FADE = 3.0  # the last seconds fade out, with the picture
PEAK = 0.7


def score(seconds: float):
    first, second, bass = [], [], []
    for c, (arp, pair, root) in enumerate(CHORDS):
        t0 = c * CHORD
        up = arp + [arp[0][:-1] + str(int(arp[0][-1]) + 1)]
        cycle = up + up[-2:0:-1]  # up and back down: six notes, two beats of triplets
        for i in range(12):
            first.append((t0 + i * BEAT / 3, BEAT / 3, cycle[i % 6], min(12, 6 + 2 * c + i // 6)))
        if c >= 1:
            for i in range(8):
                second.append((t0 + i * BEAT / 2, BEAT / 2, pair[i % 2], 5 + c))
        bass.append((t0, CHORD, root))
    # Home: A minor, circling on, quieter each beat; the second voice thins to one note every other beat.
    t0, arp, pair = 4 * CHORD, ["A4", "C5", "E5", "A5", "E5", "C5"], ["A3", "E4"]
    beats = int((seconds - t0) / BEAT)
    for b in range(beats):
        volume = max(1, round(12 - 11 * b / beats))
        for i in range(3):
            first.append((t0 + b * BEAT + i * BEAT / 3, BEAT / 3, arp[(3 * b + i) % 6], volume))
        if b % 2 == 0:
            second.append((t0 + b * BEAT, BEAT, pair[(b // 2) % 2], max(1, volume - 3)))
    bass.append((t0, 2 * CHORD, "A2"))  # the triangle can't fade; it stops
    return first, second, bass


def render(seconds: float = OPENING_SECONDS - 0.1) -> np.ndarray:
    """The theme as samples at chiptune.RATE, peaking at PEAK, silent at its end."""
    first, second, bass = score(seconds)
    x = chiptune.output(chiptune.mix(chiptune.pulse(first, seconds, 25, 0.18),
                                     chiptune.pulse(second, seconds, 12, 0.25),
                                     chiptune.triangle(bass, seconds)))
    x = x / np.abs(x).max() * PEAK
    fade = int(FADE * chiptune.RATE)
    x[-fade:] *= np.linspace(1, 0, fade) ** 2
    return x


def path(root: Path | None = None) -> Path:
    """The theme as a 24-bit WAV, written the first time it's wanted. Its name carries a hash of the code that
    makes it, so a changed score is written afresh."""
    if root is None:
        from series.build import OUT  # here, not above: the build imports the scene, which imports this
        root = OUT / "theme"
    made_by = Path(chiptune.__file__).read_bytes() + Path(__file__).read_bytes()
    out = Path(root) / f"theme-{hashlib.sha256(made_by).hexdigest()[:12]}.wav"
    if not out.exists():
        out.parent.mkdir(parents=True, exist_ok=True)
        soundfile.write(out, render(), chiptune.RATE, subtype="PCM_24")
    return out
