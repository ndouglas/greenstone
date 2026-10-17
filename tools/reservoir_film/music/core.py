"""A small tracker: tunes as note strings per channel, instruments as
per-frame macros, compiled to a log of APU register writes for
examples/apu_render.rs (format in src/nes/apu/render.rs)."""

import math
import re
from dataclasses import dataclass, field

CPU_HZ = 1789773
NES_FRAME_HZ = CPU_HZ / 29781
NAMES = {"C": 0, "D": 2, "E": 4, "F": 5, "G": 7, "A": 9, "B": 11}
TOKEN = re.compile(r"^(r|[kosh]|[A-G](?:#|b)?-?\d)/(\d+)(\.?)$")

# Noise drums: (period index into the noise table, mode bit, volume per frame).
DRUMS = {
    "k": (12, 0, (15, 13, 10, 7, 4, 2, 0)),
    "s": (6, 0, (14, 12, 9, 7, 5, 3, 2, 1, 0)),
    "h": (2, 0, (9, 5, 2, 0)),
    "o": (3, 0, (9, 8, 7, 6, 5, 4, 3, 2, 1, 0)),
}

PULSE_BASE = {"p1": 0x4000, "p2": 0x4004}


def midi(name):
    m = re.match(r"^([A-G])(#|b)?(-?\d)$", name)
    if not m:
        raise ValueError(f"not a pitch: {name}")
    letter, accidental, octave = m.groups()
    return 12 * (int(octave) + 1) + NAMES[letter] + {"#": 1, "b": -1, None: 0}[accidental]


def hz(m):
    return 440.0 * 2 ** ((m - 69) / 12)


def pulse_period(f):
    return max(8, min(0x7FF, round(CPU_HZ / (16 * f) - 1)))


def triangle_period(f):
    return max(2, min(0x7FF, round(CPU_HZ / (32 * f) - 1)))


@dataclass(frozen=True)
class Instrument:
    volume: tuple = (10,)
    duty: tuple = (2,)
    vibrato: tuple = (0, 0.0, 6)
    arp: tuple = (0,)
    release: tuple = ()
    sweep: int | None = None


@dataclass
class Part:
    channel: str
    instrument: Instrument
    notes: str


@dataclass
class Section:
    parts: list

    def beats(self, bpm):
        lengths = {round(sum(e[1] for e in notes_to_events(p.notes, bpm, 0)), 6) for p in self.parts}
        if len(lengths) != 1:
            raise ValueError(f"parts differ in length: {sorted(lengths)} beats")
        return lengths.pop()


@dataclass
class Cue:
    name: str
    title: str
    bpm: float
    loop: Section
    intro: Section | None = None
    ending: Section | None = None
    credit: str = "original"


def notes_to_events(notes, bpm, start_beat):
    events, beat = [], start_beat
    for token in notes.replace("|", " ").split():
        m = TOKEN.match(token)
        if not m:
            raise ValueError(f"bad note token: {token}")
        pitch, denominator, dot = m.groups()
        length = 4 / int(denominator) * (1.5 if dot else 1)
        events.append((beat, length, pitch))
        beat += length
    return events


def _frame(beat, bpm):
    return round(beat * 60 / bpm * NES_FRAME_HZ)


def _sections(cue, loops):
    return ([cue.intro] if cue.intro else []) + [cue.loop] * loops + ([cue.ending] if cue.ending else [])


def _channel_frames(channel, events, bpm, total):
    """Per frame: None for silence, else (pitch or drum, frames since onset,
    instrument, frames since release or None while held, whether this is the onset)."""
    state = [None] * total
    for start, length, pitch, inst in events:
        if pitch == "r":
            continue
        a, b = _frame(start, bpm), _frame(start + length, bpm)
        for f in range(a, min(b + len(inst.release), total)):
            state[f] = (pitch, f - a, inst, f - b if f >= b else None, f == a)
    return state


def _seq(values, i):
    return values[min(i, len(values) - 1)]


def compile_cue(cue, loops, bpm=None):
    bpm = bpm or cue.bpm
    by_channel = {"p1": [], "p2": [], "tri": [], "noise": []}
    beat = 0.0
    for section in _sections(cue, loops):
        length = section.beats(bpm)
        for part in section.parts:
            for start, l, pitch in notes_to_events(part.notes, bpm, beat):
                by_channel[part.channel].append((start, l, pitch, part.instrument))
        beat += length
    total = _frame(beat, bpm)
    writes, last = [(0, 0x4015, 0x0F), (0, 0x4017, 0x40)], {}

    def put(frame, address, value, force=False):
        if force or last.get(address) != value:
            writes.append((frame, address, value))
            last[address] = value

    for channel, events in by_channel.items():
        frames = _channel_frames(channel, events, bpm, total)
        for f, s in enumerate(frames):
            if channel in PULSE_BASE:
                base = PULSE_BASE[channel]
                if s is None:
                    put(f, base, 0x30 | (_seq((2,), 0) << 6))
                    continue
                pitch, i, inst, released, onset = s
                vol = _seq(inst.volume, i) if released is None else _seq(inst.release, released)
                duty = _seq(inst.duty, i)
                period = _pitched(pulse_period, pitch, inst, i)
                put(f, base, (duty << 6) | 0x30 | vol)
                put(f, base + 1, 0x08 if inst.sweep is None else inst.sweep, force=onset and inst.sweep is not None)
                put(f, base + 2, period & 0xFF)
                put(f, base + 3, 0x08 | (period >> 8), force=onset and last.get(base + 3) != 0x08 | (period >> 8))
            elif channel == "tri":
                if s is None or s[3] is not None:
                    put(f, 0x4008, 0x80)
                    continue
                pitch, i, inst, _, onset = s
                period = _pitched(triangle_period, pitch, inst, i)
                put(f, 0x4008, 0xFF)
                put(f, 0x400A, period & 0xFF)
                put(f, 0x400B, 0x08 | (period >> 8), force=onset)
            else:
                if s is None:
                    put(f, 0x400C, 0x30)
                    continue
                drum, i, inst, _, onset = s
                period, mode, vols = DRUMS[drum]
                put(f, 0x400C, 0x30 | _seq(vols, i))
                put(f, 0x400E, (mode << 7) | period)
                if onset:
                    put(f, 0x400F, 0x08, force=True)
    writes.sort(key=lambda w: w[0])
    return total, writes


def _pitched(period_of, pitch, inst, i):
    m = midi(pitch) + inst.arp[i % len(inst.arp)]
    delay, depth, cycle = inst.vibrato
    if depth and i >= delay:
        m += depth * math.sin(2 * math.pi * (i - delay) / cycle)
    return period_of(hz(m))


def write_log(frames, writes, path):
    with open(path, "w") as f:
        f.write(f"frames {frames}\n")
        for frame, address, value in writes:
            f.write(f"{frame} {address:04X} {value:02X}\n")
