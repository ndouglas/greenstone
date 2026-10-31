"""The chase's music: the opening of Stures schottis (trad., Sweden), melody on the first pulse channel and an
oom-pah bass of our own on the triangle, as the APU register writes to make on every frame.

The tune is read from stures.abc by a reader for the ABC this tune uses: notes A-G and a-g with , and ',
accidentals ^ _ = (lasting to the bar's end), lengths like 2, 3/2 and /, rests z, chord names in quotes, and
bar lines with first and second endings."""

import re
from pathlib import Path

HERE = Path(__file__).parent
CPU = 1789773
FPS = 60.0988
BPM = 150  # quarter notes a minute, about: a brisk schottis; the tempo is stretched so the tune fills the chase
TAIL = 20  # frames of quiet after the last note
KEY = {"B": -1, "E": -1}  # G minor: B flat, E flat
STEPS = {"C": 0, "D": 2, "E": 4, "F": 5, "G": 7, "A": 9, "B": 11}
ROOTS = {"Gm": 7, "D": 2, "D7": 2, "Bb": 10, "F": 5, "Cm7": 0}  # pitch classes of the chord names used
TOKEN = re.compile(r'"([^"]+)"|(\|\]|\|\||:\||\|:|\|)([12]?)|([_^=]?)([A-Ga-gz])([,\']*)(\d*)(/?)(\d*)')
MELODY_VOLUME = 8


def bars(abc: str) -> list[list[tuple[int | None, float, str | None]]]:
    """The tune's bars, in written order, as (MIDI note or None for a rest, length in eighths, chord name
    starting here or None). A first or second ending is a bar of its own, as written."""
    body = "\n".join(line for line in abc.splitlines() if line and not re.match(r"^[A-Za-z]:", line))
    out, bar, accidentals, chord = [], [], {}, None
    for name, barline, _, accidental, letter, octave, num, slash, den in TOKEN.findall(body):
        if name:
            chord = name
        elif barline:
            if bar:
                out.append(bar)
            bar, accidentals = [], {}
        else:
            length = float(num or 1) / (float(den or 2) if slash else 1)
            midi = None
            if letter != "z":
                upper = letter.upper()
                if accidental:
                    accidentals[upper] = {"^": 1, "_": -1, "=": 0}[accidental]
                shift = accidentals.get(upper, KEY.get(upper, 0))
                midi = (60 + STEPS[upper] + shift + (12 if letter.islower() else 0)
                        + 12 * (octave.count("'") - octave.count(",")))
            bar.append((midi, length, chord))
            chord = None
    if bar:
        out.append(bar)
    return out


def excerpt(frames: int) -> list[list[tuple[int | None, float, str | None]]]:
    """The pickup, some of the first part's bars (three, five, six or all seven: places where it can turn for
    home), and the first ending, which comes home to G minor: as many bars as make `frames` nearest BPM."""
    tune = bars((HERE / "stures.abc").read_text())
    k = min((3, 5, 6, 7), key=lambda k: abs(tempo(frames, 2 + 8 * k + 8) - BPM))
    return [tune[0], *tune[1:1 + k], tune[8]]


def eighth(frames: int, eighths: float) -> float:
    """Frames an eighth note lasts, for `eighths` of them to fill `frames` (less the tail)."""
    return (frames - TAIL) / eighths


def tempo(frames: int, eighths: float) -> float:
    return FPS * 60 / eighth(frames, eighths) / 2


def pulse_period(midi: int) -> int:
    return round(CPU / (16 * 440 * 2 ** ((midi - 69) / 12)) - 1)


def triangle_period(midi: int) -> int:
    return round(CPU / (32 * 440 * 2 ** ((midi - 69) / 12)) - 1)


def writes(frames: int) -> list[list[tuple[int, int]]]:
    """For every frame, the (register, value) writes to make: $4000-$4003 the melody, $4008-$400B the bass."""
    out = [[] for _ in range(frames)]

    tune = excerpt(frames)
    frames_per_eighth = eighth(frames, sum(n[1] for bar in tune for n in bar))

    def write(eighths: float, *pairs: tuple[int, int], late: int = 0) -> None:
        frame = round(eighths * frames_per_eighth) + late
        if frame < frames:
            out[frame] += pairs

    t, chord = 0.0, "Gm"
    for i, bar in enumerate(tune):
        start, marks = t, []
        for midi, length, name in bar:
            if name:
                marks.append((t, name))
            if midi is not None:
                p = pulse_period(midi)
                write(t, (0x4000, 0xB0 | MELODY_VOLUME), (0x4002, p & 0xFF), (0x4003, p >> 8))
                write(t + length, (0x4000, 0xB0), late=-1)  # a frame's breath before the next note
            t += length
        # The bass, after the pickup: the chord's root on beats one and three, its fifth on two and four.
        for beat in range(4 if i else 0):
            when = start + 2 * beat
            chord = next((n for at, n in reversed(marks) if at <= when), chord)
            root = 43 + (ROOTS[chord] - 7) % 12  # G2 up to F#3
            p = triangle_period(root if beat % 2 == 0 else root + 7)
            write(when, (0x4008, 0xFF), (0x400A, p & 0xFF), (0x400B, p >> 8))
            write(when, (0x4008, 0x80), late=9)  # cut short: the linear counter reloads to nothing
    return out


def table(frames: int) -> bytes:
    """Per frame: the number of writes, then (register's low byte, value) for each."""
    out = bytearray()
    for frame in writes(frames):
        out.append(len(frame))
        for register, value in frame:
            out += bytes([register & 0xFF, value])
    return bytes(out)
