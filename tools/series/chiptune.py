"""Music in the NES's own voices: the 2A03's two pulse channels and its triangle, at the pitches its period
registers can make, mixed by its nonlinear mixer and its output filters. A note is (start s, length s, name,
volume 0-15); the triangle has no volume, only on or off."""

import numpy as np
from scipy.signal import lfilter

RATE = 48000
CPU = 1789773  # the NTSC console's CPU clock, Hz
DUTY = {12: [0, 1, 0, 0, 0, 0, 0, 0], 25: [0, 1, 1, 0, 0, 0, 0, 0], 50: [0, 1, 1, 1, 1, 0, 0, 0]}
TRIANGLE = list(range(15, -1, -1)) + list(range(16))
STEPS = {"C": 0, "C#": 1, "D": 2, "D#": 3, "E": 4, "F": 5, "F#": 6, "G": 7, "G#": 8, "A": 9, "A#": 10, "B": 11}


def hz(note: str) -> float:
    """Equal-tempered pitch of a note like "A4" or "G#3"."""
    name, octave = note[:-1], int(note[-1])
    return 440.0 * 2 ** ((STEPS[name] + 12 * (octave - 4) - 9) / 12)


def pulse_hz(f: float) -> float:
    """The nearest pitch a pulse channel can play: its period register counts 16 CPU cycles per step."""
    return CPU / (16 * (round(CPU / (16 * f) - 1) + 1))


def triangle_hz(f: float) -> float:
    """The nearest pitch the triangle can play: 32 steps a cycle."""
    return CPU / (32 * (round(CPU / (32 * f) - 1) + 1))


def _steps(seconds: float, pitch: float, sequence: list[int]) -> np.ndarray:
    k = np.arange(int(seconds * RATE))
    return np.array(sequence, float)[(k * pitch * len(sequence) / RATE).astype(int) % len(sequence)]


def pulse(notes, seconds: float, duty: int = 25, decay: float | None = 0.2) -> np.ndarray:
    """A pulse channel's 4-bit level, sample by sample; each note's volume falls away with time constant
    `decay` (seconds), as the chip's envelope does, but never below 1 until the note ends."""
    out = np.zeros(int(seconds * RATE))
    for start, length, note, volume in notes:
        a = int(start * RATE)
        wave = _steps(min(length, seconds - start), pulse_hz(hz(note)), DUTY[duty])
        if decay is not None:
            t = np.arange(len(wave)) / RATE
            wave = wave * np.maximum(np.round(volume * np.exp(-t / decay)), 1 if volume else 0)
        else:
            wave = wave * volume
        out[a:a + len(wave)] = wave
    return out


def triangle(notes, seconds: float) -> np.ndarray:
    """The triangle's level: it steps through 16 levels and cannot be made quieter, only stopped."""
    out = np.zeros(int(seconds * RATE))
    for start, length, note in notes:
        a = int(start * RATE)
        wave = _steps(min(length, seconds - start), triangle_hz(hz(note)), TRIANGLE)
        out[a:a + len(wave)] = wave
    return out


def mix(pulse_1: np.ndarray, pulse_2: np.ndarray, tri: np.ndarray) -> np.ndarray:
    """The chip's nonlinear mixer (the noise and sample channels silent)."""
    pulses = pulse_1 + pulse_2
    with np.errstate(divide="ignore"):
        out = np.where(pulses > 0, 95.88 / (8128 / pulses + 100), 0.0)
        return out + np.where(tri > 0, 159.79 / (8227 / tri + 100), 0.0)


def output(x: np.ndarray) -> np.ndarray:
    """The console's output stage: high-pass at 90 Hz and 440 Hz, low-pass at 14 kHz, each first order."""
    for fc in (90, 440):
        a = 1 / (1 + 2 * np.pi * fc / RATE)
        x = lfilter([a, -a], [1, -a], x)
    a = 2 * np.pi * 14000 / RATE / (1 + 2 * np.pi * 14000 / RATE)
    return lfilter([a], [1, -(1 - a)], x)
