# Reservoir Film Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A 10 to 12 minute chaptered video, *Can Tetris play Mario?*, built from live emulator experiments and scored with music played on Greenstone's own APU.

**Architecture:** Three phases, each testable alone. (A) Music: the APU sweep fix, a Rust renderer that plays a register log through `APU`, and a Python tracker that compiles tunes to register logs and fits them to chapter lengths. (B) Takes: a library module `src/nes/gym/reservoir/film.rs` whose functions run the emulators with an intervention and write frames plus `take.json`, driven by `examples/reservoir_film.rs`. (C) Compositor: a Python package `tools/reservoir_film/` with shared widgets, one module per chapter, per-chapter cached segments, music lay-in and chapter markers.

**Tech Stack:** Rust 2021 (this crate, serde_json), Python 3 with Pillow and numpy only, ffmpeg.

**Spec:** `docs/superpowers/specs/2026-10-17-reservoir-film-design.md`

## Global Constraints

- Rust uses 2-space indent. Format only touched files with `rustfmt --edition 2021 <files>`; never `cargo fmt`.
- Run Rust tests with `RUSTFLAGS=-Awarnings`. Never chain `cargo build` behind a filtered pipeline.
- Never stage `src/main.rs`, `src/nes/bus/mod.rs`, `.claude/` or `AGENTS.md`.
- Every commit message ends with `Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e`. Write messages to a file and use `git commit -F`. Nothing is pushed.
- Python: standard library, Pillow and numpy only. Tests are `unittest`, run from `tools/` with `python3 -m unittest discover -s reservoir_film/tests -t .`.
- ROMs: `roms2/super_mario_bros.nes`, `roms2/tetris.nes`. Take output: `target/reservoir-film/<take>/`. Segments: `target/reservoir-film/segments/`. Final video: `target/reservoir-film/tetris-reservoir.mp4`.
- Video: 1280x720, 60 fps, H.264 (`-crf 20 -pix_fmt yuv420p`), AAC 192k.
- Captions: one sentence each; on screen at least `max(3.0, 4.0 * words / 15)` seconds.
- Mix: ffmpeg `loudnorm=I=-16:TP=-1.5:LRA=11`.
- Music: melodies original except Korobeiniki (traditional) and the Minuet in G (Petzold, long attributed to Bach). Nothing from Koji Kondo's or Hirokazu Tanaka's scores.
- Every number shown on screen comes from a `take.json` or `tools/reservoir_film/numbers.json`, never typed into a caption.
- The buggy builds (leaky window, permissive search pool) are off by default and used only by the film.

## Review Focus

1. A missing ROM or demonstration cache: recording a take should stop with a message naming the missing path and the command that creates it, not a panic. Pinned in Task 7.
2. A caption that quotes a number whose key is missing from its take: the build should stop naming the key, not render "{bank_drive.max_x}". Pinned in Task 12.
3. A chapter shorter than its cue's intro plus ending: the score must still end with the chapter (faded), never run into the next. Pinned in Task 4.
4. A take whose run ends before its shot does (Mario dies at step 40 of an 80-step shot): the shot should hold the last frame, not raise IndexError. Pinned in Task 12.
5. Editing one chapter, or re-recording one take, should re-render only the chapters that use it. Pinned in Task 14.

---

## Phase A: Music

### Task 1: Fix the pulse sweep mute under negate

**Files:**
- Modify: `src/nes/apu/pulse.rs` (`calculate_sweep_target`, and a new `#[cfg(test)] mod tests` at the end)

**Interfaces:**
- Consumes: nothing.
- Produces: pulse 1 audible with `$4001 = $08`. Later tasks write `$08` to `$4001`/`$4005` to mean "sweep off".

- [ ] **Step 1: Write the failing tests**

Append to `src/nes/apu/pulse.rs`:

```rust
#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::apu::LENGTH_TABLE;

  /// A channel set to constant volume 15, 50% duty, halted length, at `period`.
  fn playing(channel: u8, sweep: u8, period: u16) -> Pulse {
    let mut p = Pulse::new(channel);
    p.set_enabled(true);
    p.write_duty_envelope(0b1011_1111);
    p.write_sweep(sweep);
    p.write_timer_low((period & 0xFF) as u8);
    p.write_length_timer_high(0x08 | (period >> 8) as u8, &LENGTH_TABLE);
    p
  }

  fn audible(p: &mut Pulse) -> bool {
    let mut heard = false;
    for _ in 0..8192 {
      p.tick_timer();
      heard |= p.output() > 0;
    }
    heard
  }

  #[test]
  fn a_negated_sweep_with_shift_zero_does_not_mute_either_channel() {
    assert!(audible(&mut playing(1, 0x08, 253)));
    assert!(audible(&mut playing(2, 0x08, 253)));
  }

  #[test]
  fn a_period_under_eight_mutes_even_when_negating() {
    assert!(!audible(&mut playing(1, 0x08, 5)));
  }

  #[test]
  fn an_upward_target_past_7ff_still_mutes() {
    // Shift 0, no negate: the target is twice the period, 0x800.
    assert!(!audible(&mut playing(1, 0x00, 0x400)));
  }
}
```

- [ ] **Step 2: Run them to see the first fail**

Run: `RUSTFLAGS=-Awarnings cargo test --lib nes::apu::pulse::tests`
Expected: `a_negated_sweep_with_shift_zero_does_not_mute_either_channel` FAILS; the other two pass.

- [ ] **Step 3: Fix the target calculation**

In `calculate_sweep_target`, replace the negate branch and the muting line:

```rust
    let negate = (self.sweep & 0x08) != 0;
    if negate {
      // Pulse 1 subtracts one more than pulse 2 (ones' complement). A
      // negated target cannot overflow on hardware, so it saturates at 0.
      let extra = if self.channel == 1 { 1 } else { 0 };
      self.sweep_target_period = self.timer_period.saturating_sub(change + extra);
    } else {
      self.sweep_target_period = self.timer_period.wrapping_add(change);
    }

    // Muting occurs if period < 8, or if an upward target passes 0x7FF.
    self.sweep_muting = self.timer_period < 8 || (!negate && self.sweep_target_period > 0x7FF);
```

- [ ] **Step 4: Run the tests**

Run: `RUSTFLAGS=-Awarnings cargo test --lib nes::apu`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
rustfmt --edition 2021 src/nes/apu/pulse.rs
git add src/nes/apu/pulse.rs
# message: "Stop muting pulse 1 when its sweep negates with shift 0."
git commit -F <message file>
```

### Task 2: Render a register log through the APU

**Files:**
- Create: `src/nes/apu/render.rs`
- Modify: `src/nes/apu/mod.rs` (add `pub mod render;`)
- Create: `examples/apu_render.rs`

**Interfaces:**
- Consumes: `APU::new`, `APU::write_register`, `APU::tick`, `APU::take_samples` (all in `src/nes/apu/mod.rs`).
- Produces:
  - `pub struct RegisterWrite { pub frame: u32, pub address: u16, pub value: u8 }`
  - `pub fn parse_log(text: &str) -> Result<(u32, Vec<RegisterWrite>), String>`: returns (total frames, writes). Format: a first line `frames N`; then one write per line, `frame address value`, address and value in hex without `$`, e.g. `12 4000 BF`; `#` starts a comment; blank lines are skipped; frames must not decrease and must be under N.
  - `pub fn render_log(frames: u32, writes: &[RegisterWrite]) -> Vec<f32>`: 44.1 kHz mono.
  - `pub fn wav_bytes(samples: &[f32]) -> Vec<u8>`: 16-bit PCM mono 44,100 Hz, samples scaled by 0.9 and clamped.
  - `pub const CYCLES_PER_FRAME: u32 = 29_781;` and `pub const SAMPLE_RATE: u32 = 44_100;`
  - CLI: `cargo run --release --example apu_render -- <log> <wav>`.

- [ ] **Step 1: Write the failing tests**

Create `src/nes/apu/render.rs` with the tests first (the functions after them in Step 3):

```rust
//! Play a log of APU register writes, one batch per NTSC frame, into samples.

#[cfg(test)]
mod tests {
  use super::*;

  /// Strength of `hz` in `samples` by a single-bin DFT.
  fn strength(samples: &[f32], hz: f64) -> f64 {
    let (mut re, mut im) = (0.0, 0.0);
    for (i, &s) in samples.iter().enumerate() {
      let a = 2.0 * std::f64::consts::PI * hz * i as f64 / SAMPLE_RATE as f64;
      re += s as f64 * a.cos();
      im += s as f64 * a.sin();
    }
    (re * re + im * im).sqrt()
  }

  /// The frequency between `lo` and `hi` Hz, in 1 Hz steps, with the most energy.
  fn peak(samples: &[f32], lo: u32, hi: u32) -> u32 {
    (lo..=hi).max_by(|&a, &b| strength(samples, a as f64).total_cmp(&strength(samples, b as f64))).unwrap()
  }

  const LOG: &str = "frames 90
# all channels on, frame IRQ off
0 4015 0F
0 4017 40
# pulse 1, A440: duty 50%, constant volume 15, sweep off
0 4000 BF
0 4001 08
0 4002 FD
0 4003 08
# at frame 30, pulse off and triangle A220
30 4000 B0
30 4008 FF
30 400A FD
30 400B 08
# at frame 60, triangle off and noise on
60 4008 80
60 400C 3F
60 400E 04
60 400F 08
";

  #[test]
  fn parse_log_reads_the_header_and_hex_writes() {
    let (frames, writes) = parse_log(LOG).unwrap();
    assert_eq!(frames, 90);
    assert_eq!(writes.len(), 14);
    assert_eq!(writes[4], RegisterWrite { frame: 0, address: 0x4002, value: 0xFD });
  }

  #[test]
  fn parse_log_rejects_bad_logs() {
    assert!(parse_log("0 4000 BF").is_err(), "no header");
    assert!(parse_log("frames 10\n5 4000 BF\n4 4000 BF").is_err(), "decreasing frames");
    assert!(parse_log("frames 10\n10 4000 BF").is_err(), "past the end");
    assert!(parse_log("frames 10\n1 4000 XYZ").is_err(), "bad hex");
  }

  #[test]
  fn render_log_plays_each_channel_at_its_pitch() {
    let (frames, writes) = parse_log(LOG).unwrap();
    let samples = render_log(frames, &writes);
    let per_frame = SAMPLE_RATE as f64 * CYCLES_PER_FRAME as f64 / 1_789_773.0;
    assert!((samples.len() as f64 - frames as f64 * per_frame).abs() < 2.0);
    let at = |f: f64| (f * per_frame) as usize;
    assert_eq!(peak(&samples[at(5.0)..at(25.0)], 400, 480), 440);
    assert_eq!(peak(&samples[at(35.0)..at(55.0)], 200, 240), 220);
    let noise = &samples[at(65.0)..at(85.0)];
    let mean = noise.iter().sum::<f32>() / noise.len() as f32;
    let rms = (noise.iter().map(|s| (s - mean) * (s - mean)).sum::<f32>() / noise.len() as f32).sqrt();
    assert!(rms > 0.02, "noise rms {}", rms);
  }

  #[test]
  fn wav_bytes_writes_a_canonical_header() {
    let wav = wav_bytes(&[0.0, 1.0, -1.0, 2.0]);
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(&wav[8..16], b"WAVEfmt ");
    assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), SAMPLE_RATE);
    assert_eq!(wav.len(), 44 + 4 * 2);
    let last = i16::from_le_bytes(wav[50..52].try_into().unwrap());
    assert_eq!(last, (0.9 * i16::MAX as f32) as i16, "clamped");
  }
}
```

Add `pub mod render;` to `src/nes/apu/mod.rs` beside the other `pub mod` lines.

- [ ] **Step 2: Run them to see them fail**

Run: `RUSTFLAGS=-Awarnings cargo test --lib nes::apu::render`
Expected: compile errors, `parse_log`, `render_log`, `wav_bytes` not found.

- [ ] **Step 3: Implement**

Above the tests in `src/nes/apu/render.rs`:

```rust
use super::APU;

/// CPU cycles per NTSC frame, rounded: 1,789,773 Hz over 60.0988 frames a second.
pub const CYCLES_PER_FRAME: u32 = 29_781;
/// The APU's output rate (see `APU::new`).
pub const SAMPLE_RATE: u32 = 44_100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisterWrite {
  pub frame: u32,
  pub address: u16,
  pub value: u8,
}

/// Parse `frames N` and then `frame address value` lines (hex address and value).
pub fn parse_log(text: &str) -> Result<(u32, Vec<RegisterWrite>), String> {
  let mut lines = text.lines().map(|l| l.split('#').next().unwrap().trim()).filter(|l| !l.is_empty());
  let header = lines.next().ok_or("empty log")?;
  let frames: u32 = header
    .strip_prefix("frames ")
    .ok_or_else(|| format!("expected 'frames N', got '{}'", header))?
    .trim()
    .parse()
    .map_err(|e| format!("bad frame count: {}", e))?;
  let mut writes = Vec::new();
  for line in lines {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != 3 {
      return Err(format!("expected 'frame address value', got '{}'", line));
    }
    let frame: u32 = parts[0].parse().map_err(|e| format!("bad frame in '{}': {}", line, e))?;
    let address = u16::from_str_radix(parts[1], 16).map_err(|e| format!("bad address in '{}': {}", line, e))?;
    let value = u8::from_str_radix(parts[2], 16).map_err(|e| format!("bad value in '{}': {}", line, e))?;
    if frame >= frames {
      return Err(format!("write at frame {} is past the end ({})", frame, frames));
    }
    if writes.last().map_or(false, |w: &RegisterWrite| w.frame > frame) {
      return Err(format!("frames decrease at '{}'", line));
    }
    writes.push(RegisterWrite { frame, address, value });
  }
  Ok((frames, writes))
}

/// Play the writes into a fresh APU, each frame's writes before its cycles.
pub fn render_log(frames: u32, writes: &[RegisterWrite]) -> Vec<f32> {
  let mut apu = APU::new();
  let mut samples = Vec::new();
  let mut next = 0;
  for frame in 0..frames {
    while next < writes.len() && writes[next].frame == frame {
      apu.write_register(writes[next].address, writes[next].value);
      next += 1;
    }
    for _ in 0..CYCLES_PER_FRAME {
      apu.tick();
    }
    samples.extend(apu.take_samples());
  }
  samples
}

/// 16-bit PCM mono WAV at `SAMPLE_RATE`, samples scaled by 0.9 and clamped.
pub fn wav_bytes(samples: &[f32]) -> Vec<u8> {
  let data_len = (samples.len() * 2) as u32;
  let mut out = Vec::with_capacity(44 + data_len as usize);
  out.extend(b"RIFF");
  out.extend((36 + data_len).to_le_bytes());
  out.extend(b"WAVEfmt ");
  out.extend(16u32.to_le_bytes());
  out.extend(1u16.to_le_bytes());
  out.extend(1u16.to_le_bytes());
  out.extend(SAMPLE_RATE.to_le_bytes());
  out.extend((SAMPLE_RATE * 2).to_le_bytes());
  out.extend(2u16.to_le_bytes());
  out.extend(16u16.to_le_bytes());
  out.extend(b"data");
  out.extend(data_len.to_le_bytes());
  for &s in samples {
    let v = (s * 0.9).clamp(-0.9, 0.9) * i16::MAX as f32;
    out.extend((v as i16).to_le_bytes());
  }
  out
}
```

Create `examples/apu_render.rs`:

```rust
//! Play a register log through Greenstone's APU and write a WAV.
//!
//! Usage: cargo run --release --example apu_render -- <log> <wav>

use greenstone::nes::apu::render::{parse_log, render_log, wav_bytes};

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 3 {
    eprintln!("usage: apu_render <log> <wav>");
    std::process::exit(2);
  }
  let text = std::fs::read_to_string(&args[1]).unwrap_or_else(|e| {
    eprintln!("read {}: {}", args[1], e);
    std::process::exit(1);
  });
  let (frames, writes) = parse_log(&text).unwrap_or_else(|e| {
    eprintln!("{}: {}", args[1], e);
    std::process::exit(1);
  });
  std::fs::write(&args[2], wav_bytes(&render_log(frames, &writes))).expect("write wav");
}
```

- [ ] **Step 4: Run the tests**

Run: `RUSTFLAGS=-Awarnings cargo test --lib nes::apu`
Expected: all pass. If `render_log_plays_each_channel_at_its_pitch` fails on the triangle or noise, that is another APU bug: probe it, write a failing unit test in that channel's file, fix, and record a ruling.

- [ ] **Step 5: Commit**

```bash
rustfmt --edition 2021 src/nes/apu/render.rs src/nes/apu/mod.rs examples/apu_render.rs
git add src/nes/apu/render.rs src/nes/apu/mod.rs examples/apu_render.rs
# message: "Render a log of APU register writes to a WAV."
git commit -F <message file>
```

### Task 3: A tracker that compiles tunes to register logs

**Files:**
- Create: `tools/reservoir_film/__init__.py` (empty), `tools/reservoir_film/music/__init__.py` (empty)
- Create: `tools/reservoir_film/music/core.py`
- Create: `tools/reservoir_film/tests/__init__.py` (empty), `tools/reservoir_film/tests/test_music_core.py`

**Interfaces:**
- Consumes: the log format of Task 2.
- Produces (in `reservoir_film.music.core`):
  - `NES_FRAME_HZ = 1789773 / 29781`
  - `midi(name: str) -> int` ("A4" is 69; sharps "C#4", flats "Bb3")
  - `pulse_period(hz: float) -> int`, `triangle_period(hz: float) -> int`
  - `@dataclass(frozen=True) class Instrument: volume=(10,); duty=(2,); vibrato=(0, 0.0, 6); arp=(0,); release=(); sweep=None` — `volume`/`duty` are per-frame sequences whose last value holds; `vibrato` is (delay frames, depth in semitones, period in frames); `arp` cycles semitone offsets per frame; `release` is volumes played after the note ends; `sweep` is a raw `$4001` value or None for `$08`.
  - `DRUMS: dict[str, tuple[int, int, tuple[int, ...]]]` — name to (noise period index, mode bit, volume sequence): `k` kick, `s` snare, `h` closed hat, `o` open hat.
  - `@dataclass class Part: channel: str; instrument: Instrument; notes: str` with channel one of `p1`, `p2`, `tri`, `noise`.
  - `@dataclass class Section: parts: list[Part]`; `Section.beats(bpm) -> float` raises `ValueError` if parts differ in length.
  - `@dataclass class Cue: name: str; title: str; bpm: float; loop: Section; intro: Section | None = None; ending: Section | None = None; credit: str = "original"`
  - `notes_to_events(notes: str, bpm: float, start_beat: float) -> list[tuple[float, float, str]]`: (start beat, length in beats, pitch or drum or "r").
  - `compile_cue(cue: Cue, loops: int, bpm: float | None = None) -> tuple[int, list[tuple[int, int, int]]]`: (frames, writes as (frame, address, value)), sorted by frame.
  - `write_log(frames, writes, path)`.
- Note syntax: tokens separated by spaces, `|` bar lines ignored. A token is `<pitch>/<denominator>` with an optional `.` for dotted: `E5/4`, `C#5/8.`, `r/2` (rest), `k/8` (drum on the noise channel). `/4` is one beat.

- [ ] **Step 1: Write the failing tests**

`tools/reservoir_film/tests/test_music_core.py`:

```python
import unittest

from reservoir_film.music.core import (
    NES_FRAME_HZ, Cue, Instrument, Part, Section, compile_cue, midi, notes_to_events, pulse_period, triangle_period,
)

LEAD = Instrument(volume=(12, 10, 8), duty=(2,))
BASS = Instrument()


class Pitches(unittest.TestCase):
    def test_midi_numbers(self):
        self.assertEqual(midi("A4"), 69)
        self.assertEqual(midi("C4"), 60)
        self.assertEqual(midi("C#4"), 61)
        self.assertEqual(midi("Bb3"), 58)

    def test_periods_match_the_ntsc_formulas(self):
        self.assertEqual(pulse_period(440.0), 253)
        self.assertEqual(triangle_period(220.0), 253)


class Notes(unittest.TestCase):
    def test_lengths_and_dots(self):
        events = notes_to_events("C5/4 | r/8 E5/8. k/16", bpm=120, start_beat=2)
        self.assertEqual(events, [(2, 1, "C5"), (3, 0.5, "r"), (3.5, 0.75, "E5"), (4.25, 0.25, "k")])

    def test_bad_token_names_itself(self):
        with self.assertRaisesRegex(ValueError, "C5"):
            notes_to_events("C5", bpm=120, start_beat=0)


class Compile(unittest.TestCase):
    def cue(self):
        loop = Section([Part("p1", LEAD, "A4/4 r/4"), Part("tri", BASS, "A3/2")])
        return Cue(name="t", title="T", bpm=120, loop=loop)

    def test_section_parts_must_agree(self):
        with self.assertRaises(ValueError):
            Section([Part("p1", LEAD, "A4/4"), Part("tri", BASS, "A3/2")]).beats(120)

    def test_frames_follow_the_tempo(self):
        frames, _ = compile_cue(self.cue(), loops=2)
        self.assertEqual(frames, round(4 * 60 / 120 * NES_FRAME_HZ))

    def test_writes_start_the_chip_and_set_the_pitch(self):
        _, writes = compile_cue(self.cue(), loops=1)
        first = [w for w in writes if w[0] == 0]
        self.assertIn((0, 0x4015, 0x0F), first)
        self.assertIn((0, 0x4001, 0x08), first)
        self.assertIn((0, 0x4002, 253 & 0xFF), first)
        self.assertIn((0, 0x400A, 253 & 0xFF), first)

    def test_the_volume_envelope_steps_then_holds(self):
        _, writes = compile_cue(self.cue(), loops=1)
        vols = [(f, v & 0x0F) for f, a, v in writes if a == 0x4000]
        self.assertEqual(vols[:3], [(0, 12), (1, 10), (2, 8)])
        rest = round(60 / 120 * NES_FRAME_HZ)
        self.assertIn((rest, 0), vols, "the rest silences the pulse")

    def test_writes_are_sorted_and_deduplicated(self):
        _, writes = compile_cue(self.cue(), loops=2)
        self.assertEqual(writes, sorted(writes, key=lambda w: w[0]))
        last = {}
        for f, a, v in writes:
            self.assertNotEqual(last.get(a), v, f"redundant write to {a:04X} at {f}")
            last[a] = v
```

`$4003`, `$400B` and `$400F` are exempt from the deduplication rule because writing them restarts a note; in the test, skip those three addresses:

```python
        for f, a, v in writes:
            if a in (0x4003, 0x4007, 0x400B, 0x400F):
                continue
```

(Put that `if` as the first line inside the loop of `test_writes_are_sorted_and_deduplicated`.)

- [ ] **Step 2: Run them to see them fail**

Run: `cd tools && python3 -m unittest discover -s reservoir_film/tests -t .`
Expected: ImportError, `reservoir_film.music.core` has no such names.

- [ ] **Step 3: Implement `core.py`**

```python
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
```

The silent pulse write keeps duty 2 so silence does not flip the duty bits back and forth.

- [ ] **Step 4: Run the tests**

Run: `cd tools && python3 -m unittest discover -s reservoir_film/tests -t .`
Expected: all pass. Then render the test cue end to end as a smoke check:

```bash
cd tools && python3 -c "
from reservoir_film.music.core import *
from reservoir_film.tests.test_music_core import Compile
f, w = compile_cue(Compile().cue(), loops=4); write_log(f, w, '/tmp/t.log')" && cd .. \
 && cargo run -q --release --example apu_render -- /tmp/t.log /tmp/t.wav && ls -l /tmp/t.wav
```

Use the scratchpad directory instead of `/tmp` when one is available.
Expected: a WAV of about 8 seconds.

- [ ] **Step 5: Commit**

```bash
git add tools/reservoir_film
# message: "Add a tracker that compiles tunes to APU register logs."
git commit -F <message file>
```

### Task 4: Fit cues to chapter lengths and mix the score

**Files:**
- Create: `tools/reservoir_film/music/score.py`
- Create: `tools/reservoir_film/tests/test_music_score.py`

**Interfaces:**
- Consumes: `compile_cue`, `write_log`, `NES_FRAME_HZ` (Task 3); `examples/apu_render.rs` (Task 2).
- Produces (in `reservoir_film.music.score`):
  - `SAMPLE_RATE = 44100`, `CROSSFADE = 0.3`, `TEMPO_RANGE = (0.94, 1.06)`
  - `cue_seconds(cue, loops, bpm) -> float`
  - `fit(cue, seconds) -> tuple[int, float, bool]`: (loops, bpm, needs_trim). Picks loops ≥ 0 and a bpm within `TEMPO_RANGE` of `cue.bpm` so the cue lasts `seconds` within one NES frame; if even zero loops at the fastest tempo run long, returns `(0, cue.bpm * 1.06, True)`.
  - `trim(samples: np.ndarray, seconds: float, fade: float = 1.0) -> np.ndarray`: exactly `round(seconds * SAMPLE_RATE)` samples, with a fade-out over the last `fade` seconds when it cut anything, padded with silence when short.
  - `render_cue(cue, loops, bpm, work_dir) -> np.ndarray` (float32 in −1..1, via the Rust renderer; caches by a hash of the log text).
  - `crossfade_join(parts: list[np.ndarray]) -> np.ndarray`: equal-power crossfades of `CROSSFADE` seconds, each overlap taken from the start of the next part, total length equal to the sum of the parts' lengths.
  - `score(plan: list[tuple[Cue, float]], stings: list[tuple[float, Cue]], work_dir) -> np.ndarray`: fit, render, trim, join, add each sting at its second, peak-normalise to −1 dBFS.
  - `write_wav(samples, path)`.

- [ ] **Step 1: Write the failing tests**

```python
import unittest

import numpy as np

from reservoir_film.music.core import NES_FRAME_HZ, Cue, Instrument, Part, Section
from reservoir_film.music.score import SAMPLE_RATE, crossfade_join, cue_seconds, fit, trim

I = Instrument()
TOY = Cue(
    name="toy", title="Toy", bpm=120,
    intro=Section([Part("p1", I, "C5/1")]),
    loop=Section([Part("p1", I, "C5/1 D5/1")]),
    ending=Section([Part("p1", I, "C5/1")]),
)


class Fit(unittest.TestCase):
    def test_fit_lands_within_a_frame(self):
        for seconds in (30.0, 61.3, 95.0):
            loops, bpm, needs_trim = fit(TOY, seconds)
            self.assertFalse(needs_trim)
            self.assertLess(abs(cue_seconds(TOY, loops, bpm) - seconds), 1 / NES_FRAME_HZ, seconds)
            self.assertTrue(0.94 * TOY.bpm <= bpm <= 1.06 * TOY.bpm)

    def test_too_short_a_chapter_asks_for_a_trim(self):
        loops, bpm, needs_trim = fit(TOY, 2.0)
        self.assertEqual(loops, 0)
        self.assertTrue(needs_trim)


class Trim(unittest.TestCase):
    def test_trim_is_exact_and_fades_what_it_cuts(self):
        x = np.ones(SAMPLE_RATE * 5, dtype=np.float32)
        y = trim(x, 3.0)
        self.assertEqual(len(y), 3 * SAMPLE_RATE)
        self.assertAlmostEqual(float(y[-1]), 0.0, places=3)
        self.assertAlmostEqual(float(y[0]), 1.0)

    def test_trim_pads_short_audio(self):
        y = trim(np.ones(100, dtype=np.float32), 1.0)
        self.assertEqual(len(y), SAMPLE_RATE)
        self.assertEqual(float(y[-1]), 0.0)


class Join(unittest.TestCase):
    def test_join_keeps_the_total_length(self):
        a = np.ones(SAMPLE_RATE * 2, dtype=np.float32)
        b = np.ones(SAMPLE_RATE * 3, dtype=np.float32)
        self.assertEqual(len(crossfade_join([a, b])), 5 * SAMPLE_RATE)
```

- [ ] **Step 2: Run them to see them fail**

Run: `cd tools && python3 -m unittest discover -s reservoir_film/tests -t .`
Expected: ImportError for `reservoir_film.music.score`.

- [ ] **Step 3: Implement `score.py`**

```python
"""Fit cues to chapter lengths, render them on the APU, and mix the score."""

import hashlib
import pathlib
import subprocess
import wave

import numpy as np

from .core import NES_FRAME_HZ, compile_cue, write_log

SAMPLE_RATE = 44100
CROSSFADE = 0.3
TEMPO_RANGE = (0.94, 1.06)
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
        return 0, cue.bpm * hi, True
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
        subprocess.run(["cargo", "run", "-q", "--release", "--example", "apu_render", "--", str(log), str(wav)],
                       cwd=REPO, check=True)
    return read_wav(wav)


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
        s = render_cue(sting, 0, sting.bpm, work_dir)
        i = round(at * SAMPLE_RATE)
        j = min(len(mix), i + len(s))
        mix[i:j] += s[: j - i]
    peak = float(np.max(np.abs(mix))) or 1.0
    return mix * (10 ** (-1 / 20) / peak)
```

- [ ] **Step 4: Run the tests**

Run: `cd tools && python3 -m unittest discover -s reservoir_film/tests -t .`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add tools/reservoir_film/music/score.py tools/reservoir_film/tests/test_music_score.py
# message: "Fit cues to chapter lengths and mix the score."
git commit -F <message file>
```

### Task 5: Compose the cues and stings, and audition them

**Files:**
- Create: `tools/reservoir_film/music/cues.py`
- Create: `tools/reservoir_film/music/audition.py`
- Create: `tools/reservoir_film/tests/test_cues.py`

**Interfaces:**
- Consumes: `Cue`, `Section`, `Part`, `Instrument`, `compile_cue` (Task 3); `render_cue`, `write_wav` (Task 4).
- Produces (in `reservoir_film.music.cues`):
  - `CUES: dict[str, Cue]` with keys `coin_run`, `underground`, `lab`, `korobeiniki`, `korobeiniki_fast`, `fooled`, `minuet`, `reprise`.
  - `STINGS: dict[str, Cue]` with keys `blip`, `oneup`, `wahwah`, `fanfare`, `death`.
  - `CHAPTER_CUES: dict[int, str]`: `{0: "coin_run", 1: "underground", 2: "lab", 3: "korobeiniki", 4: "lab", 5: "fooled", 6: "korobeiniki_fast", 7: "minuet", 8: "minuet", 9: "reprise"}`.
  - `python3 -m reservoir_film.music.audition [name ...]` (run from `tools/`) writes `target/reservoir-film/music/<name>.wav` and `<name>.png` (a spectrogram) for each cue and sting, two loops each.

Ruling carried from planning: the spec named Bach's Minuet from French Suite No. 3; this plan uses the Minuet in G (BWV Anh. 114, Petzold), equally public domain and one whose melody is written out below. The credits name it correctly. The spec's "triangle kick" is dropped: kicks are on the noise channel so the triangle keeps the bass line. Both deviations get a line in the spec in this task's commit.

- [ ] **Step 1: Write the failing tests**

`tools/reservoir_film/tests/test_cues.py`:

```python
import unittest

from reservoir_film.music.core import compile_cue
from reservoir_film.music.cues import CHAPTER_CUES, CUES, STINGS


class Cues(unittest.TestCase):
    def test_every_cue_and_sting_compiles(self):
        for name, cue in {**CUES, **STINGS}.items():
            with self.subTest(name):
                frames, writes = compile_cue(cue, loops=1)
                self.assertGreater(frames, 0)
                self.assertTrue(any(a in (0x4002, 0x4006, 0x400A) for _, a, _ in writes), "plays a pitch")

    def test_every_chapter_has_a_cue(self):
        self.assertEqual(sorted(CHAPTER_CUES), list(range(10)))
        for name in CHAPTER_CUES.values():
            self.assertIn(name, CUES)

    def test_credits_name_the_two_borrowed_tunes(self):
        self.assertEqual(CUES["korobeiniki"].credit, "traditional")
        self.assertIn("Petzold", CUES["minuet"].credit)
        originals = [n for n, c in CUES.items() if c.credit == "original"]
        self.assertEqual(sorted(originals), ["coin_run", "fooled", "lab", "reprise", "underground"])
```

- [ ] **Step 2: Run them to see them fail**

Run: `cd tools && python3 -m unittest discover -s reservoir_film/tests -t .`
Expected: ImportError for `reservoir_film.music.cues`.

- [ ] **Step 3: Write `cues.py`**

```python
"""The film's music: five originals, Korobeiniki (traditional) and the
Minuet in G (Petzold, long attributed to Bach). Nothing here is taken from
Koji Kondo's or Hirokazu Tanaka's scores; the Mario-flavoured cues borrow
a style (bouncy, syncopated, major), not a tune."""

from .core import Cue, Instrument, Part, Section

LEAD = Instrument(volume=(11, 10, 9, 9, 8), duty=(2,), vibrato=(12, 0.15, 6), release=(5, 3, 1, 0))
SOFT = Instrument(volume=(8, 7, 7, 6), duty=(1,), vibrato=(16, 0.12, 7), release=(4, 2, 1, 0))
HARM = Instrument(volume=(7, 6, 5), duty=(1,), release=(3, 1, 0))
PLINK = Instrument(volume=(10, 6, 4, 2, 1, 0), duty=(1,))
STAC = Instrument(volume=(9, 5, 2, 0), duty=(2,))
OCT = Instrument(volume=(6, 5, 5, 4), duty=(1,), arp=(0, 0, 12, 12))
SWEEP = Instrument(volume=(12, 12, 11, 10, 9, 8, 8, 7, 6, 5, 4, 3, 2, 1, 0), duty=(2,), vibrato=(8, 0.3, 5), sweep=0x94)
BASS = Instrument()
DRUM = Instrument()


def bars(*b):
    return " | ".join(b)


def each(chords, pattern):
    """One bar of `pattern` per chord; `pattern` is a function of the chord."""
    return bars(*(pattern(c) for c in chords))


def rest(n_beats_per_bar, n_bars):
    return bars(*([{4: "r/1", 3: "r/2."}[n_beats_per_bar]] * n_bars))


# --- Coin Run: bouncy, syncopated C major, calypso bass -------------------
CR_CHORDS = ["C", "Am", "F", "G", "C", "Am", "G", "C"]
CR_ROOT = {"C": ("C", "G"), "Am": ("A", "E"), "F": ("F", "C"), "G": ("G", "D")}
CR_THIRD = {"C": "E4", "Am": "C5", "F": "A4", "G": "B4"}
CR_MELODY = bars(
    "G4/8 C5/8 r/8 E5/8 r/8 G5/8 r/8 E5/8",
    "A5/8 G5/8 r/8 E5/8 C5/4 r/4",
    "A4/8 C5/8 r/8 F5/8 r/8 A5/8 r/8 F5/8",
    "G5/8 F5/8 r/8 D5/8 B4/4 r/4",
    "E5/8 E5/8 r/8 E5/8 r/8 C5/8 E5/4",
    "G5/4 r/4 G4/4 r/4",
    "F5/8 E5/8 r/8 D5/8 r/8 B4/8 G4/4",
    "C5/4 G4/8 C5/8 r/2",
)


def calypso(c):
    r, f = CR_ROOT[c]
    return f"{r}3/8. {f}3/16 r/8 {r}3/8 r/8 {f}2/8 {r}3/8 r/8"


def skank(c):
    t = CR_THIRD[c]
    return f"r/8 {t}/8 r/8 {t}/8 r/8 {t}/8 r/8 {t}/8"


CR_DRUMS = "k/8 h/8 s/8 h/16 h/16 k/8 h/8 s/8 h/8"
coin_run = Cue(
    name="coin_run", title="Coin Run", bpm=132,
    intro=Section([
        Part("p1", LEAD, rest(4, 2)), Part("p2", STAC, rest(4, 2)),
        Part("tri", BASS, each(["C", "C"], calypso)), Part("noise", DRUM, bars(CR_DRUMS, CR_DRUMS)),
    ]),
    loop=Section([
        Part("p1", LEAD, CR_MELODY), Part("p2", STAC, each(CR_CHORDS, skank)),
        Part("tri", BASS, each(CR_CHORDS, calypso)), Part("noise", DRUM, bars(*[CR_DRUMS] * 8)),
    ]),
    ending=Section([
        Part("p1", LEAD, "G4/8 C5/8 E5/8 G5/8 C6/2"), Part("p2", STAC, "r/2 E5/2"),
        Part("tri", BASS, "C3/8 C3/8 C3/8 C3/8 C2/2"), Part("noise", DRUM, "k/8 k/8 k/8 k/8 o/2"),
    ]),
)

reprise = Cue(
    name="reprise", title="Coin Run (reprise)", bpm=96,
    loop=Section([
        Part("p1", SOFT, CR_MELODY),
        Part("tri", BASS, each(CR_CHORDS, lambda c: f"{CR_ROOT[c][0]}3/2 {CR_ROOT[c][1]}2/2")),
    ]),
    ending=Section([Part("p1", SOFT, "C5/1"), Part("tri", BASS, "C3/1")]),
)

# --- Underground Numbers: sparse A minor ----------------------------------
UG_CHORDS = ["Am", "Am", "F", "E", "Am", "Am", "Dm", "E"]
UG_ROOT = {"Am": ("A2", "E2"), "F": ("F2", "C2"), "E": ("E2", "B1"), "Dm": ("D2", "A1")}
UG_TONES = {"Am": "A4 C5 E5 A5", "F": "F4 A4 C5 F5", "E": "E4 G#4 B4 E5", "Dm": "D4 F4 A4 D5"}


def ug_bass(c):
    r, q = UG_ROOT[c]
    return f"{r}/8 r/8 {r}/8 r/8 r/4 {q}/4"


def ug_plink(c):
    a, b, d, e = UG_TONES[c].split()
    return f"{a}/16 {b}/16 {d}/16 {e}/16 r/4 r/2"


UG_TICK = "h/4 r/4 h/8 h/8 r/4"
underground = Cue(
    name="underground", title="Underground Numbers", bpm=100,
    intro=Section([
        Part("p1", LEAD, "r/1"), Part("p2", OCT, "r/1"),
        Part("tri", BASS, ug_bass("Am")), Part("noise", DRUM, UG_TICK),
    ]),
    loop=Section([
        Part("p1", LEAD, bars("r/2 E5/4 C5/4", "B4/2 r/2", "r/2 A4/4 C5/4", "B4/1",
                              "r/2 E5/4 G5/4", "F5/2 E5/4 r/4", "D5/4 F5/4 A5/4 G5/4", "G#5/2 r/2")),
        Part("p2", OCT, each(UG_CHORDS, ug_plink)),
        Part("tri", BASS, each(UG_CHORDS, ug_bass)),
        Part("noise", DRUM, bars(*[UG_TICK] * 8)),
    ]),
    ending=Section([
        Part("p1", LEAD, "A4/1"), Part("p2", OCT, "r/1"), Part("tri", BASS, "A2/1"), Part("noise", DRUM, "r/1"),
    ]),
)

# --- The Lab: plinky F major ----------------------------------------------
LAB_CHORDS = ["F", "C", "Dm", "Bb", "F", "C", "Bb", "F"]
LAB_ROOT = {"F": ("F2", "C3"), "C": ("C3", "G2"), "Dm": ("D3", "A2"), "Bb": ("Bb2", "F2")}
LAB_THIRD = {"F": "A4", "C": "E4", "Dm": "F4", "Bb": "D4"}
LAB_TICK = "h/4 h/4 h/4 h/4"
lab = Cue(
    name="lab", title="The Lab", bpm=112,
    intro=Section([
        Part("p1", PLINK, "C6/8 r/8 C6/8 r/8 r/2"), Part("p2", HARM, "r/1"),
        Part("tri", BASS, "F2/4 C3/4 F2/4 C3/4"), Part("noise", DRUM, LAB_TICK),
    ]),
    loop=Section([
        Part("p1", PLINK, bars(
            "F5/8 A5/8 C6/8 A5/8 F5/4 r/4", "E5/8 G5/8 C6/8 G5/8 E5/4 r/4",
            "D5/8 F5/8 A5/8 F5/8 D5/4 r/4", "D5/8 F5/8 Bb5/8 F5/8 D5/4 r/4",
            "C5/8 F5/8 A5/8 C6/8 A5/4 F5/4", "G5/8 E5/8 C5/8 E5/8 G5/2",
            "Bb5/8 A5/8 G5/8 F5/8 E5/8 F5/8 G5/4", "F5/2 r/2")),
        Part("p2", HARM, each(LAB_CHORDS, lambda c: f"r/4 {LAB_THIRD[c]}/4 r/4 {LAB_THIRD[c]}/4")),
        Part("tri", BASS, each(LAB_CHORDS, lambda c: "{0}/4 {1}/4 {0}/4 {1}/4".format(*LAB_ROOT[c]))),
        Part("noise", DRUM, bars(*[LAB_TICK] * 8)),
    ]),
    ending=Section([
        Part("p1", PLINK, "F5/4 C5/4 F4/2"), Part("p2", HARM, "r/1"),
        Part("tri", BASS, "F2/1"), Part("noise", DRUM, "r/1"),
    ]),
)

# --- Korobeiniki (traditional): A minor -----------------------------------
KORO_MELODY = bars(
    "E5/4 B4/8 C5/8 D5/4 C5/8 B4/8", "A4/4 A4/8 C5/8 E5/4 D5/8 C5/8",
    "B4/4. C5/8 D5/4 E5/4", "C5/4 A4/4 A4/2",
    "r/8 D5/4 F5/8 A5/4 G5/8 F5/8", "E5/4. C5/8 E5/4 D5/8 C5/8",
    "B4/4 B4/8 C5/8 D5/4 E5/4", "C5/4 A4/4 A4/2",
)
KORO_HARM = bars("C5/2 B4/2", "A4/2 C5/2", "G#4/2 B4/2", "A4/1", "F4/2 A4/2", "G4/2 E4/2", "G#4/2 B4/2", "A4/1")
KORO_CHORDS = ["Am", "Am", "E", "Am", "Dm", "Am", "E", "Am"]
KORO_ROOT = {"Am": "A", "E": "E", "Dm": "D"}


def octaves(c):
    r = KORO_ROOT[c]
    return " ".join([f"{r}2/8 {r}3/8"] * 4)


KORO_DRUMS = "k/8 h/8 s/8 h/8 k/8 h/8 s/8 h/8"


def korobeiniki_cue(name, title, bpm, drums):
    noise = [Part("noise", DRUM, bars(*[KORO_DRUMS] * 8))] if drums else []
    return Cue(
        name=name, title=title, bpm=bpm, credit="traditional",
        intro=Section([Part("p1", LEAD, "r/1"), Part("p2", HARM, "r/1"), Part("tri", BASS, octaves("Am"))]
                      + ([Part("noise", DRUM, KORO_DRUMS)] if drums else [])),
        loop=Section([Part("p1", LEAD, KORO_MELODY), Part("p2", HARM, KORO_HARM),
                      Part("tri", BASS, each(KORO_CHORDS, octaves))] + noise),
        ending=Section([Part("p1", LEAD, "A4/2. r/4"), Part("p2", HARM, "E4/2. r/4"), Part("tri", BASS, "A2/2. r/4")]
                       + ([Part("noise", DRUM, "k/4 k/4 o/2")] if drums else [])),
    )


korobeiniki = korobeiniki_cue("korobeiniki", "Korobeiniki", 140, drums=False)
korobeiniki_fast = korobeiniki_cue("korobeiniki_fast", "Korobeiniki (fast)", 176, drums=True)

# --- Fooled Again: sneaky staccato D minor --------------------------------
FA_CHORDS = ["Dm", "Dm", "Bb", "A", "Dm", "Gm", "A", "Dm"]
FA_ROOT = {"Dm": ("D", "A"), "Bb": ("Bb", "F"), "A": ("A", "E"), "Gm": ("G", "D")}
FA_THIRD = {"Dm": "F4", "Bb": "D4", "A": "C#4", "Gm": "Bb3"}
FA_TICK = "h/8 r/8 h/8 r/8 h/8 r/8 h/8 h/8"


def tiptoe(c):
    r, q = FA_ROOT[c]
    return f"{r}2/8 r/8 {r}3/8 r/8 {r}2/8 r/8 {q}2/8 r/8"


fooled = Cue(
    name="fooled", title="Fooled Again", bpm=120,
    intro=Section([Part("p1", STAC, "r/1"), Part("p2", HARM, "r/1"), Part("tri", BASS, tiptoe("Dm")),
                   Part("noise", DRUM, FA_TICK)]),
    loop=Section([
        Part("p1", STAC, bars(
            "D5/8 r/8 F5/8 r/8 A5/8 r/8 G5/8 F5/8", "E5/8 r/8 D5/8 r/8 r/2",
            "D5/8 r/8 F5/8 r/8 Bb5/8 r/8 A5/8 G5/8", "A5/8 r/8 C#5/8 r/8 r/2",
            "D5/8 F5/8 A5/8 D6/8 r/8 A5/8 F5/8 r/8", "G5/8 r/8 Bb5/8 r/8 D6/8 r/8 Bb5/8 r/8",
            "A5/8 G5/8 F5/8 E5/8 C#5/4 r/4", "D5/4 r/4 r/2")),
        Part("p2", HARM, each(FA_CHORDS, lambda c: f"r/4 {FA_THIRD[c]}/8 r/8 r/4 {FA_THIRD[c]}/8 r/8")),
        Part("tri", BASS, each(FA_CHORDS, tiptoe)),
        Part("noise", DRUM, bars(*[FA_TICK] * 8)),
    ]),
    ending=Section([Part("p1", STAC, "D5/8 r/8 A4/8 r/8 D4/4 r/4"), Part("p2", HARM, "r/1"),
                    Part("tri", BASS, "D2/8 r/8 A2/8 r/8 D2/4 r/4"), Part("noise", DRUM, "r/1")]),
)

# --- Minuet in G (Petzold), 3/4 -------------------------------------------
MINUET_A = [
    "D5/4 G4/8 A4/8 B4/8 C5/8", "D5/4 G4/4 G4/4", "E5/4 C5/8 D5/8 E5/8 F#5/8", "G5/4 G4/4 G4/4",
    "C5/4 D5/8 C5/8 B4/8 A4/8", "B4/4 C5/8 B4/8 A4/8 G4/8", "F#4/4 G4/8 A4/8 B4/8 G4/8", "A4/2.",
]
MINUET_B = MINUET_A[:6] + ["A4/4 B4/8 A4/8 G4/8 F#4/8", "G4/2."]
MINUET_BASS_A = ["G2/2 A2/4", "B2/2.", "C3/2.", "B2/2.", "A2/2.", "G2/2.", "D3/4 B2/4 G2/4", "D3/4 D2/4 C3/4"]
MINUET_BASS_B = MINUET_BASS_A[:6] + ["C3/4 D3/4 D2/4", "G2/2 G1/4"]
minuet = Cue(
    name="minuet", title="Minuet in G", bpm=120,
    credit="Christian Petzold, Minuet in G (BWV Anh. 114, long attributed to Bach)",
    loop=Section([Part("p1", SOFT, bars(*MINUET_A, *MINUET_B)),
                  Part("tri", BASS, bars(*MINUET_BASS_A, *MINUET_BASS_B))]),
    ending=Section([Part("p1", SOFT, "G4/2."), Part("tri", BASS, "G2/2.")]),
)

CUES = {c.name: c for c in [coin_run, underground, lab, korobeiniki, korobeiniki_fast, fooled, minuet, reprise]}

# --- Stings (all original) -------------------------------------------------
STINGS = {
    "blip": Cue(name="blip", title="Blip", bpm=150, loop=Section([Part("p1", PLINK, "C6/32 G6/32")])),
    "oneup": Cue(name="oneup", title="One up", bpm=150,
                 loop=Section([Part("p1", PLINK, "C6/16 E6/16 G6/16 C7/8")])),
    "wahwah": Cue(name="wahwah", title="Wah wah", bpm=90,
                  loop=Section([Part("p1", SWEEP, "Bb4/4 A4/4 Ab4/2"), Part("tri", BASS, "Bb2/4 A2/4 Ab2/2")])),
    "fanfare": Cue(name="fanfare", title="Fanfare", bpm=140, loop=Section([
        Part("p1", LEAD, "C5/8 E5/8 G5/8 C6/8 r/8 G5/8 C6/2"),
        Part("p2", HARM, "E4/8 G4/8 C5/8 E5/8 r/8 B4/8 E5/2"),
        Part("tri", BASS, "C3/2 r/8 G2/8 C3/2")])),
    "death": Cue(name="death", title="Death", bpm=120, loop=Section([
        Part("p1", STAC, "G4/8 F#4/8 F4/8 E4/2."), Part("tri", BASS, "G2/8 F#2/8 F2/8 E2/2.")])),
}

CHAPTER_CUES = {0: "coin_run", 1: "underground", 2: "lab", 3: "korobeiniki", 4: "lab",
                5: "fooled", 6: "korobeiniki_fast", 7: "minuet", 8: "minuet", 9: "reprise"}
```

`audition.py`:

```python
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
        x = render_cue(cue, 0 if name in STINGS else 2, cue.bpm, OUT / "work")
        write_wav(x, OUT / f"{name}.wav")
        spectrogram(x, OUT / f"{name}.png")
        peak = float(np.max(np.abs(x)))
        silent = float(np.mean(np.abs(x) < 1e-3))
        print(f"{name}: {len(x) / SAMPLE_RATE:.1f} s, peak {peak:.2f}, silent {silent:.0%}")


if __name__ == "__main__":
    main(sys.argv[1:])
```

- [ ] **Step 4: Run the tests and audition**

Run: `cd tools && python3 -m unittest discover -s reservoir_film/tests -t .`
Expected: all pass.

Run: `cd tools && python3 -m reservoir_film.music.audition`
Expected: one line per cue and sting, every peak between 0.2 and 0.95, no cue more than 40% silent. Read each spectrogram PNG: steady horizontal lines for held notes, a falling line for `wahwah`, broadband bursts for drums. A spectrogram with no lines, or with only noise, is a bug to fix before going on. If a cue sounds wrong to a reader of its spectrogram (a wrong-octave bass, a collision between p2 and p1), adjust its notes and say so in a ruling.

- [ ] **Step 5: Amend the spec and commit**

In `docs/superpowers/specs/2026-10-17-reservoir-film-design.md`, replace "Bach, Minuet from French Suite No. 3" with "Petzold, Minuet in G (BWV Anh. 114, long attributed to Bach)" in the cue table and the copyright line, and replace "triangle bass and kick (by a fast pitch drop), noise snare and hats" with "triangle bass, noise kick, snare and hats".

```bash
git add tools/reservoir_film/music/cues.py tools/reservoir_film/music/audition.py tools/reservoir_film/tests/test_cues.py docs/superpowers/specs/2026-10-17-reservoir-film-design.md
# message: "Compose the film's cues and stings for the APU."
git commit -F <message file>
```

---

## Phase B: Takes

### Task 6: The buggy builds and the rule's tripwires, as library options

**Files:**
- Modify: `src/nes/gym/reservoir/demo.rs` (`smb_window`, near line 93)
- Modify: `src/nes/gym/reservoir/search.rs` (`candidates`, `search_features`, near lines 23 and 205)
- Modify: `src/nes/gym/reservoir/teacher.rs` (`trigger`, near line 138)

**Interfaces:**
- Consumes: `SMB_INPUT_BYTES`, `POSITION_BYTES`, `candidates()`, `best_pick`, `trigger`.
- Produces:
  - `pub fn smb_window_leaky(emu: &mut NesEmulator) -> Vec<u8>` in `demo.rs`: the window with the controller bytes left in. `smb_window` is unchanged in behaviour.
  - `pub fn candidates_with_position() -> Vec<Feature>` in `search.rs`: every window byte, absolute and relative to Mario's x, including `POSITION_BYTES`, plus the enemy distance. The pool the search used before the fix.
  - `pub fn search_features_in(pool: &[Feature], windows, buttons, lambda, logistic, mask, log) -> [Feature; PIECES_PER_STEP]`; `search_features` becomes `search_features_in(&candidates(), ...)`.
  - `pub struct Tripwires { pub enemy: bool, pub pit: bool, pub wall: bool }` and `pub fn tripwires(rule: Rule, window: &[u8]) -> Tripwires` in `teacher.rs`, each flag computed as `trigger` computes it but all three reported, and only on the ground (all false in the air). `trigger` is rewritten on top of it with the same results.

- [ ] **Step 1: Write the failing tests**

In `demo.rs`'s test module:

```rust
  #[test]
  fn the_leaky_window_keeps_the_controller_bytes_and_the_default_hides_them() {
    let rom = match std::fs::read("roms2/super_mario_bros.nes") {
      Ok(r) => r,
      Err(_) => return,
    };
    let mut emu = NesEmulator::new(&rom);
    emu.load_state(&smb_start_state_at(&rom, Stage::ONE_ONE));
    for address in SMB_INPUT_BYTES {
      emu.ram_write(address, 0x5A);
    }
    let hidden = smb_window(&mut emu);
    let leaky = smb_window_leaky(&mut emu);
    for address in SMB_INPUT_BYTES {
      assert_eq!(hidden[address as usize], 0);
      assert_eq!(leaky[address as usize], 0x5A);
    }
    assert_eq!(hidden.len(), leaky.len());
  }
```

In `search.rs`'s test module:

```rust
  #[test]
  fn the_permissive_pool_admits_marios_position_and_the_default_does_not() {
    let wide = candidates_with_position();
    assert!(wide.iter().any(|f| f.address == 0x86 && !f.relative));
    assert!(wide.iter().any(|f| f.address == 0x6D));
    assert!(wide.iter().any(|f| f.address == 0x30 && f.relative), "relative reads of any byte");
    assert!(!candidates().iter().any(|f| f.address == 0x86 || f.address == 0x6D));
    assert!(wide.len() > candidates().len());
  }
```

In `teacher.rs`'s test module (reuse its existing window-building helpers if it has them; otherwise build the window by hand as below, `SMB_WINDOW_LEN` bytes with ground under Mario):

```rust
  #[test]
  fn tripwires_report_all_three_and_trigger_agrees() {
    let rule = Rule::parse("40,3,3,9").unwrap();
    let mut w = vec![0u8; SMB_WINDOW_LEN];
    // Ground everywhere ahead, then a pit two columns ahead.
    for d in 0..TERRAIN_COLUMNS {
      w[SMB_ZERO_PAGE_LEN as usize + d * BLOCK_ROWS + FOOT_ROW] = 0x54;
    }
    w[SMB_ZERO_PAGE_LEN as usize + 2 * BLOCK_ROWS + FOOT_ROW] = 0;
    let t = tripwires(rule, &w);
    assert!(t.pit && !t.wall && !t.enemy);
    assert_eq!(trigger(rule, &w), Some(Trigger::Obstacle));
    w[MARIO_AIRBORNE] = 1;
    let t = tripwires(rule, &w);
    assert!(!t.pit && !t.wall && !t.enemy, "nothing fires in the air");
    assert_eq!(trigger(rule, &w), None);
  }
```

- [ ] **Step 2: Run them to see them fail**

Run: `RUSTFLAGS=-Awarnings cargo test --lib reservoir -- leaky permissive tripwires`
Expected: compile errors for `smb_window_leaky`, `candidates_with_position`, `tripwires`.

- [ ] **Step 3: Implement**

`demo.rs`:

```rust
/// The zero page with the controller bytes hidden, then the terrain ahead of Mario.
pub fn smb_window(emu: &mut NesEmulator) -> Vec<u8> {
  window_with(emu, true)
}

/// The window with the controller bytes left in: the leak the first readouts
/// learned from. Only the film uses it, to show the bug.
pub fn smb_window_leaky(emu: &mut NesEmulator) -> Vec<u8> {
  window_with(emu, false)
}

fn window_with(emu: &mut NesEmulator, hide_inputs: bool) -> Vec<u8> {
  let mut window = emu.ram_read_range(SMB_WINDOW_START, SMB_ZERO_PAGE_LEN);
  if hide_inputs {
    for &address in &SMB_INPUT_BYTES {
      window[(address - SMB_WINDOW_START) as usize] = 0;
    }
  }
  let buffer_1 = emu.ram_read_range(SMB_BLOCK_BUFFER_1, SMB_BLOCK_BUFFER_LEN);
  let buffer_2 = emu.ram_read_range(SMB_BLOCK_BUFFER_2, SMB_BLOCK_BUFFER_LEN);
  window.extend(terrain_ahead(&window, &buffer_1, &buffer_2));
  window
}
```

`search.rs`:

```rust
/// Every window byte, absolute and relative to Mario's x, Mario's own
/// position bytes included, and the enemy distance: the pool before the
/// fix, where a byte minus Mario's x is his position in disguise. Only the
/// film uses it, to show the memoriser.
pub fn candidates_with_position() -> Vec<Feature> {
  let mut out = Vec::new();
  for address in 0..SMB_WINDOW_LEN as u16 {
    for relative in [false, true] {
      for shift in SHIFTS {
        out.push(Feature { address, relative, shift });
      }
    }
  }
  for shift in SHIFTS {
    out.push(Feature { address: ENEMY_DISTANCE, relative: false, shift });
  }
  out
}
```

Rename the body of `search_features` to `pub fn search_features_in(pool: &[Feature], windows, buttons, lambda, logistic, mask, log)` using `pool` instead of `let pool = candidates();`, and make `search_features` call `search_features_in(&candidates(), windows, buttons, lambda, logistic, mask, log)`.

`teacher.rs`: move the body of `trigger` into `tripwires`, computing `enemy`, `pit` and `wall` separately (pit: any of columns `1..=look` has an empty `FOOT_ROW`; wall: any of those columns has a tile in `WALL_ROWS`), returning all false when `window[MARIO_AIRBORNE] != 0`. Then:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Tripwires {
  pub enemy: bool,
  pub pit: bool,
  pub wall: bool,
}

/// What, if anything, the rule jumps for in this window. Only on the ground.
pub fn trigger(rule: Rule, window: &[u8]) -> Option<Trigger> {
  let t = tripwires(rule, window);
  if t.enemy {
    Some(Trigger::Enemy)
  } else if t.pit || t.wall {
    Some(Trigger::Obstacle)
  } else {
    None
  }
}
```

- [ ] **Step 4: Run the tests**

Run: `RUSTFLAGS=-Awarnings cargo test --lib reservoir`
Expected: all pass, including the existing teacher and search tests (the rewrite of `trigger` must not change them).

- [ ] **Step 5: Commit**

```bash
rustfmt --edition 2021 src/nes/gym/reservoir/demo.rs src/nes/gym/reservoir/search.rs src/nes/gym/reservoir/teacher.rs
git add src/nes/gym/reservoir/demo.rs src/nes/gym/reservoir/search.rs src/nes/gym/reservoir/teacher.rs
# message: "Expose the leaky window, the old search pool, and the rule's tripwires for the film."
git commit -F <message file>
```

### Task 7: The film module, and takes of Mario's world

**Files:**
- Create: `src/nes/gym/reservoir/film.rs`
- Modify: `src/nes/gym/reservoir/mod.rs` (add `pub mod film;`)
- Create: `examples/reservoir_film.rs`

**Interfaces:**
- Consumes: `NesEmulator` (`new`, `load_state`, `step_frame`, `step_frames`, `ram_read`, `ram_write`, `framebuffer`), `smb_start_state_at`, `smb_window`, `smb_x`, `smb_y`, `smb_dead`, `Stage`, `Player`, `Rule`, `tripwires` (Task 6), `trace::enemies`, `load_cached`, `smb1_ram::LIVES`.
- Produces (in `greenstone::nes::gym::reservoir::film`):
  - `pub struct FilmContext { pub smb_rom: Vec<u8>, pub tetris_rom: Vec<u8>, pub target: PathBuf }`
  - `impl FilmContext { pub fn load(smb: &Path, tetris: &Path, target: &Path) -> Result<FilmContext, String>; pub fn demos(&self, cache: &str, count: usize) -> Result<Vec<Demonstration>, String>; pub fn rule(&self, cache: &str) -> Result<Rule, String> }` — errors name the missing path and the command that makes it (`DEMO_CACHES` below).
  - `pub const DEMO_CACHES: &[(&str, &str)]`: cache directory name to the command that creates it:
    - `tetris-reservoir-demos` → `cargo run --release --example tetris_reservoir`
    - `tetris-reservoir-demos-rule` → `cargo run --release --example tetris_reservoir -- --rule`
    - `tetris-reservoir-demos-rule-1-1+6-1` → `cargo run --release --example tetris_reservoir -- --rule --stages=1-1,6-1`
  - `pub struct TakeWriter` with `pub fn new(dir: &Path) -> Result<TakeWriter, String>`, `pub fn frame(&mut self, stream: &str, rgb: &[u8]) -> Result<(), String>` (appends to `<dir>/<stream>.rgb`), `pub fn finish(self, meta: serde_json::Value) -> Result<(), String>` (writes `<dir>/take.json` as `meta` plus `"streams": {name: frame count}`).
  - `pub type TakeFn = fn(&FilmContext, &mut TakeWriter) -> Result<serde_json::Value, String>;`
  - `pub fn takes() -> Vec<(&'static str, TakeFn)>` in recording order.
  - `pub fn record(ctx: &FilmContext, name: &str, out: &Path) -> Result<(), String>`: records one take into `out/<name>/`, writing to `<name>.partial` and renaming at the end so a crash never leaves a take that looks finished.
  - `pub fn block_address(column: usize, row: usize) -> u16`: the block-buffer byte for an absolute level column and row (page parity picks `$0500` or `$05D0`, then `row * 16 + column % 16`), the same layout `terrain_ahead` reads.
  - Frames are 256x240 RGB24. JSON byte arrays (windows, boards) are lowercase hex strings.
  - CLI: `cargo run --release --example reservoir_film -- [--take NAME]... [--force]`. With no `--take`, records every take whose `take.json` is missing; `--force` re-records.
- Takes in this task:
  - `lives_glyphs`: for lives 0 to 255, the intermission screen ("MARIO × N") with `$075A` held at that value. Stream `screen` (256 frames). Meta: `{"digit_box": [x, y, w, h]}`, the bounding box of the pixels that differ between lives 20 and 21 (the ones digit).
  - `matrix_walk`: the rule (`--rule` cache's `rule.bin`, else 40,3,3,9) plays 1-1 for 300 steps. Stream `smb` (every frame). Meta: `steps: [{x, y, airborne, buttons, window, enemies, tripwires: {enemy, pit, wall}}]`, and `labels`: `[{"address": 0x86, "name": "Mario's x"}, {"address": 0x1D, "name": "on the ground, jumping, falling"}, {"address": 0x0F, "span": 5, "name": "enemy slots"}, {"address": 0x87, "span": 5, "name": "enemy x"}, {"address": 0x0100, "span": 104, "name": "the terrain ahead"}]` (window indices; the terrain starts at 256).
  - `poke_enemy`: the rule plays until an enemy slot is active within 200 px ahead of Mario; then Mario holds still (buttons 0) while every 30 frames, four times, the enemy's x pixel (`$87 + slot`) is written 48 px further right. Stream `smb` (60 frames before the first poke, 150 after). Meta: `pokes: [{frame, address, before, after}]`.
  - `poke_terrain`: two scenes in one take. `wall`: when Mario is on the ground at x ≥ 400, write `$51` into rows 8 to 10 of the column 3 ahead; Mario keeps running right and stops at nothing. `floor`: from a fresh start at the same x, write `$00` into rows 11 and 12 of the two columns 2 and 3 ahead; Mario runs and falls through ground that is still drawn. Streams `wall`, `floor` (180 frames each). Meta: `{"wall": {"pokes": [...], "x": [...]}, "floor": {"pokes": [...], "x": [...], "died": bool}}`.
  - `noise_bytes`: four seeds; from 1-1 at x ≥ 300 under the rule, every 10 frames write a random byte to 3 random addresses in `$0000-$00FF` and `$0200-$07FF` (never the stack page, never `$0770-$077F` which switch game modes). Streams `seed1` to `seed4` (240 frames each). Meta: `pokes` per seed.

- [ ] **Step 1: Write the failing tests**

In `film.rs`:

```rust
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_missing_cache_names_its_path_and_the_command_that_makes_it() {
    let ctx = FilmContext { smb_rom: vec![], tetris_rom: vec![], target: PathBuf::from("/nonexistent-target") };
    let err = ctx.demos("tetris-reservoir-demos-rule", 6).unwrap_err();
    assert!(err.contains("/nonexistent-target/tetris-reservoir-demos-rule"), "{}", err);
    assert!(err.contains("--example tetris_reservoir -- --rule"), "{}", err);
  }

  #[test]
  fn a_missing_rom_is_an_error_not_a_panic() {
    let err = FilmContext::load(Path::new("/nope/smb.nes"), Path::new("/nope/tetris.nes"), Path::new("target")).unwrap_err();
    assert!(err.contains("/nope/smb.nes"), "{}", err);
  }

  #[test]
  fn the_writer_counts_frames_per_stream_and_finishes_atomically() {
    let dir = std::env::temp_dir().join(format!("film-writer-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut w = TakeWriter::new(&dir).unwrap();
    let frame = vec![0u8; FRAME_BYTES];
    w.frame("smb", &frame).unwrap();
    w.frame("smb", &frame).unwrap();
    w.frame("tetris", &frame).unwrap();
    assert!(w.frame("smb", &[0u8; 3]).is_err(), "wrong frame size");
    w.finish(serde_json::json!({"hello": 1})).unwrap();
    let meta: serde_json::Value = serde_json::from_slice(&std::fs::read(dir.join("take.json")).unwrap()).unwrap();
    assert_eq!(meta["streams"]["smb"], 2);
    assert_eq!(meta["streams"]["tetris"], 1);
    assert_eq!(meta["hello"], 1);
    assert_eq!(std::fs::metadata(dir.join("smb.rgb")).unwrap().len(), 2 * FRAME_BYTES as u64);
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn block_address_follows_the_page_parity() {
    assert_eq!(block_address(0, 0), 0x0500);
    assert_eq!(block_address(17, 11), 0x05D0 + 11 * 16 + 1);
    assert_eq!(block_address(32 + 3, 2), 0x0500 + 2 * 16 + 3);
  }

  #[test]
  #[ignore = "needs roms2/super_mario_bros.nes and roms2/tetris.nes"]
  fn the_world_takes_record() {
    let ctx = FilmContext::load(Path::new("roms2/super_mario_bros.nes"), Path::new("roms2/tetris.nes"), Path::new("target")).unwrap();
    let out = std::env::temp_dir().join(format!("film-world-{}", std::process::id()));
    for name in ["lives_glyphs", "matrix_walk", "poke_enemy", "poke_terrain", "noise_bytes"] {
      record(&ctx, name, &out).unwrap();
      let meta: serde_json::Value = serde_json::from_slice(&std::fs::read(out.join(name).join("take.json")).unwrap()).unwrap();
      assert!(meta["streams"].as_object().unwrap().values().all(|n| n.as_u64().unwrap() > 0), "{}", name);
    }
    let glyphs: serde_json::Value = serde_json::from_slice(&std::fs::read(out.join("lives_glyphs/take.json")).unwrap()).unwrap();
    assert_eq!(glyphs["streams"]["screen"], 256);
    let _ = std::fs::remove_dir_all(&out);
  }
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `RUSTFLAGS=-Awarnings cargo test --lib reservoir::film`
Expected: compile errors, module and names missing.

- [ ] **Step 3: Implement the scaffolding**

`film.rs`, the part every take shares:

```rust
//! Takes for the reservoir film: each runs the emulators with one
//! intervention and writes frames plus a `take.json`. The compositor in
//! `tools/reservoir_film/` reads them. See
//! `docs/superpowers/specs/2026-10-17-reservoir-film-design.md`.

use super::demo::{load_cached, smb_start_state_at, smb_window, smb_x, smb_y, Demonstration, Stage};
use super::teacher::{tripwires, Player, Rule};
use super::trace::enemies;
use crate::nes::gym::reward::smb1_ram;
use crate::nes::gym::NesEmulator;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

pub const FRAME_BYTES: usize = 256 * 240 * 3;

pub const DEMO_CACHES: &[(&str, &str)] = &[
  ("tetris-reservoir-demos", "cargo run --release --example tetris_reservoir"),
  ("tetris-reservoir-demos-rule", "cargo run --release --example tetris_reservoir -- --rule"),
  (
    "tetris-reservoir-demos-rule-1-1+6-1",
    "cargo run --release --example tetris_reservoir -- --rule --stages=1-1,6-1",
  ),
];

/// The rule played when a cache has no `rule.bin`: run 30's.
pub const DEFAULT_RULE: &str = "40,3,3,9";

pub struct FilmContext {
  pub smb_rom: Vec<u8>,
  pub tetris_rom: Vec<u8>,
  pub target: PathBuf,
}

impl FilmContext {
  pub fn load(smb: &Path, tetris: &Path, target: &Path) -> Result<FilmContext, String> {
    let read = |p: &Path| std::fs::read(p).map_err(|e| format!("cannot read ROM {}: {}", p.display(), e));
    Ok(FilmContext {
      smb_rom: read(smb)?,
      tetris_rom: read(tetris)?,
      target: target.to_path_buf(),
    })
  }

  fn missing(&self, cache: &str) -> String {
    let dir = self.target.join(cache);
    let make = DEMO_CACHES.iter().find(|(c, _)| *c == cache).map_or("(no known command)", |(_, m)| m);
    format!("missing demonstrations in {}; make them with: {}", dir.display(), make)
  }

  pub fn demos(&self, cache: &str, count: usize) -> Result<Vec<Demonstration>, String> {
    load_cached(&self.target.join(cache), count).ok_or_else(|| self.missing(cache))
  }

  pub fn rule(&self, cache: &str) -> Result<Rule, String> {
    match std::fs::read(self.target.join(cache).join("rule.bin")) {
      Ok(bytes) => Rule::from_bytes(&bytes),
      Err(_) => Rule::parse(DEFAULT_RULE),
    }
  }
}

pub struct TakeWriter {
  dir: PathBuf,
  streams: BTreeMap<String, (BufWriter<File>, usize)>,
}

impl TakeWriter {
  pub fn new(dir: &Path) -> Result<TakeWriter, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("create {}: {}", dir.display(), e))?;
    Ok(TakeWriter { dir: dir.to_path_buf(), streams: BTreeMap::new() })
  }

  pub fn frame(&mut self, stream: &str, rgb: &[u8]) -> Result<(), String> {
    if rgb.len() != FRAME_BYTES {
      return Err(format!("frame for {} is {} bytes, not {}", stream, rgb.len(), FRAME_BYTES));
    }
    if !self.streams.contains_key(stream) {
      let path = self.dir.join(format!("{}.rgb", stream));
      let file = File::create(&path).map_err(|e| format!("create {}: {}", path.display(), e))?;
      self.streams.insert(stream.to_string(), (BufWriter::new(file), 0));
    }
    let (w, n) = self.streams.get_mut(stream).unwrap();
    w.write_all(rgb).map_err(|e| e.to_string())?;
    *n += 1;
    Ok(())
  }

  pub fn finish(self, mut meta: Value) -> Result<(), String> {
    let mut counts = serde_json::Map::new();
    for (name, (mut w, n)) in self.streams {
      w.flush().map_err(|e| e.to_string())?;
      counts.insert(name, json!(n));
    }
    meta["streams"] = Value::Object(counts);
    let tmp = self.dir.join("take.json.partial");
    std::fs::write(&tmp, serde_json::to_vec(&meta).unwrap()).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, self.dir.join("take.json")).map_err(|e| e.to_string())
  }
}

pub type TakeFn = fn(&FilmContext, &mut TakeWriter) -> Result<Value, String>;

pub fn takes() -> Vec<(&'static str, TakeFn)> {
  vec![
    ("lives_glyphs", lives_glyphs as TakeFn),
    ("matrix_walk", matrix_walk),
    ("poke_enemy", poke_enemy),
    ("poke_terrain", poke_terrain),
    ("noise_bytes", noise_bytes),
  ]
}

pub fn record(ctx: &FilmContext, name: &str, out: &Path) -> Result<(), String> {
  let (_, f) = takes().into_iter().find(|(n, _)| *n == name).ok_or_else(|| format!("no take named {}", name))?;
  let partial = out.join(format!("{}.partial", name));
  let _ = std::fs::remove_dir_all(&partial);
  let mut w = TakeWriter::new(&partial)?;
  let meta = f(ctx, &mut w)?;
  w.finish(meta)?;
  let done = out.join(name);
  let _ = std::fs::remove_dir_all(&done);
  std::fs::rename(&partial, &done).map_err(|e| e.to_string())
}

pub fn hex(bytes: &[u8]) -> String {
  bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn block_address(column: usize, row: usize) -> u16 {
  let base: u16 = if (column / 16) % 2 == 0 { 0x0500 } else { 0x05D0 };
  base + (row * 16 + column % 16) as u16
}

/// A fresh SMB at the start of `stage` after `noops` idle frames.
fn smb_at(ctx: &FilmContext, stage: Stage, noops: usize) -> NesEmulator {
  let mut smb = NesEmulator::new(&ctx.smb_rom);
  smb.load_state(&smb_start_state_at(&ctx.smb_rom, stage));
  smb.step_frames(0, noops);
  smb
}

/// Play the rule until `until` holds, recording nothing. Returns the step count, or an error after `limit` steps.
fn play_until(smb: &mut NesEmulator, player: &mut Player, limit: usize, until: &mut dyn FnMut(&mut NesEmulator) -> bool) -> Result<usize, String> {
  for step in 0..limit {
    if until(smb) {
      return Ok(step);
    }
    let window = smb_window(smb);
    let buttons = player.buttons(&window);
    smb.step_frames(buttons, super::demo::MACRO_FRAMES);
  }
  Err(format!("condition not reached in {} steps", limit))
}
```

`examples/reservoir_film.rs`:

```rust
//! Record the reservoir film's takes into target/reservoir-film/.
//!
//! Usage: cargo run --release --example reservoir_film -- [--take NAME]... [--force]

use greenstone::nes::gym::reservoir::film::{record, takes, FilmContext};
use std::path::Path;

fn main() {
  let args: Vec<String> = std::env::args().skip(1).collect();
  let force = args.iter().any(|a| a == "--force");
  let named: Vec<&str> = args.windows(2).filter(|w| w[0] == "--take").map(|w| w[1].as_str()).collect();
  let ctx = FilmContext::load(Path::new("roms2/super_mario_bros.nes"), Path::new("roms2/tetris.nes"), Path::new("target"))
    .unwrap_or_else(|e| {
      eprintln!("{}", e);
      std::process::exit(1);
    });
  let out = Path::new("target/reservoir-film");
  for (name, _) in takes() {
    let wanted = if named.is_empty() { force || !out.join(name).join("take.json").exists() } else { named.contains(&name) };
    if !wanted {
      continue;
    }
    eprintln!("recording {}", name);
    let start = std::time::Instant::now();
    if let Err(e) = record(&ctx, name, out) {
      eprintln!("{}: {}", name, e);
      std::process::exit(1);
    }
    eprintln!("{} done in {:.0?}", name, start.elapsed());
  }
}
```

- [ ] **Step 4: Implement the five world takes**

Each follows its description in the Interfaces block. The two that need care:

```rust
fn lives_glyphs(ctx: &FilmContext, w: &mut TakeWriter) -> Result<Value, String> {
  let mut frames = Vec::new();
  for lives in 0..=255u8 {
    let mut emu = NesEmulator::new(&ctx.smb_rom);
    emu.step_frames(0, 40);
    for f in 0..90 {
      emu.ram_write(smb1_ram::LIVES, lives);
      emu.step_frame(if f < 6 { crate::nes::bus::BUTTON_START } else { 0 });
    }
    w.frame("screen", emu.framebuffer())?;
    frames.push(emu.framebuffer().to_vec());
  }
  // The ones digit: where lives 20 and 21 differ.
  let (a, b) = (&frames[20], &frames[21]);
  let (mut x0, mut y0, mut x1, mut y1) = (256, 240, 0, 0);
  for y in 0..240 {
    for x in 0..256 {
      let i = (y * 256 + x) * 3;
      if a[i..i + 3] != b[i..i + 3] {
        x0 = x0.min(x);
        y0 = y0.min(y);
        x1 = x1.max(x);
        y1 = y1.max(y);
      }
    }
  }
  if x1 < x0 {
    return Err("lives 20 and 21 drew the same screen".into());
  }
  Ok(json!({"digit_box": [x0, y0, x1 - x0 + 1, y1 - y0 + 1]}))
}

fn poke_terrain(ctx: &FilmContext, w: &mut TakeWriter) -> Result<Value, String> {
  let rule = ctx.rule("tetris-reservoir-demos-rule")?;
  let mut scenes = serde_json::Map::new();
  for scene in ["wall", "floor"] {
    let mut smb = smb_at(ctx, Stage::ONE_ONE, 0);
    let mut player = Player::new(rule);
    play_until(&mut smb, &mut player, 400, &mut |s| smb_x(s) >= 400 && s.ram_read(smb1_ram::PLAYER_FLOAT_STATE) == 0)?;
    let column = smb_x(&mut smb) as usize / 16;
    let cells: Vec<(usize, usize, u8)> = match scene {
      "wall" => (8..=10).map(|r| (column + 3, r, 0x51)).collect(),
      _ => [2, 3].iter().flat_map(|&d| [(column + d, 11, 0), (column + d, 12, 0)]).collect(),
    };
    let mut pokes = Vec::new();
    for &(c, r, v) in &cells {
      let a = block_address(c, r);
      pokes.push(json!({"address": a, "before": smb.ram_read(a), "after": v, "column": c, "row": r}));
      smb.ram_write(a, v);
    }
    let mut xs = Vec::new();
    let mut died = false;
    for _ in 0..180 {
      // Run right, no jumping: the point is what the numbers do to him.
      smb.step_frame(crate::nes::bus::BUTTON_RIGHT);
      w.frame(scene, smb.framebuffer())?;
      xs.push(smb_x(&mut smb));
      died |= super::demo::smb_dead(&mut smb);
    }
    scenes.insert(scene.into(), json!({"pokes": pokes, "x": xs, "died": died}));
  }
  Ok(Value::Object(scenes))
}
```

`matrix_walk`, `poke_enemy` and `noise_bytes` follow the same pattern: `smb_at`, `Player::new(ctx.rule("tetris-reservoir-demos-rule")?)`, step frame by frame (four frames per step with the step's buttons), `w.frame("smb", ...)` each frame, and per-step records built with `hex(&window)`, `enemies(&window)` and `tripwires(rule, &window)`. `noise_bytes` uses `super::Lcg::new(seed)` for its random addresses and values (`next_f64`).

Add `pub mod film;` to `src/nes/gym/reservoir/mod.rs`.

- [ ] **Step 5: Run the tests, then record and look**

Run: `RUSTFLAGS=-Awarnings cargo test --lib reservoir::film`
Expected: the four unit tests pass.

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib reservoir::film::tests::the_world_takes_record -- --ignored --exact`
Expected: pass.

Run: `cargo run --release --example reservoir_film -- --take lives_glyphs --take matrix_walk --take poke_enemy --take poke_terrain --take noise_bytes`

Then look at the frames: write a short scratch script that saves frames 0, 60, 120 and the last of each stream as PNG (`PIL.Image.frombytes("RGB", (256, 240), ...)`) and read them. Check: the glyph screens show the crown and garbage digits; the Goomba visibly jumps in `poke_enemy`; Mario stops at nothing in `wall` and falls through drawn ground in `floor`; the noise seeds glitch without freezing the emulator. If `floor` does not kill him or `wall` does not stop him, the row indices are off by one: probe rows 7 to 12, fix, and record a ruling.

- [ ] **Step 6: Commit**

```bash
rustfmt --edition 2021 src/nes/gym/reservoir/film.rs src/nes/gym/reservoir/mod.rs examples/reservoir_film.rs
git add src/nes/gym/reservoir/film.rs src/nes/gym/reservoir/mod.rs examples/reservoir_film.rs
# message: "Record the film's takes of Mario's world: the lives glyphs, the walk, and three pokes."
git commit -F <message file>
```

### Task 8: Takes of the simple brain, the paste, the placements and the echo

**Files:**
- Modify: `src/nes/gym/reservoir/film.rs` (four take functions, `takes()` entries, one ignored test)

**Interfaces:**
- Consumes: `trace::fit_condition`, `trace::replay`, `trace::trace_episode`, `Condition::Raw`, `Reservoir::new`, `Reservoir::new_fast`, `Reservoir::step`, `Reservoir::place`, `Reservoir::playfield`, `Reservoir::framebuffer`, `Reservoir::in_play`, `Reservoir::reset`, `Drive`, `Encoder::Salient`, `encode`, `ExperimentConfig`, `episode_noops`, `MACRO_FRAMES`.
- Produces takes:
  - `simple_brain_drives`: fit `Condition::Raw` on demonstrations 1 to 5 of `tetris-reservoir-demos-rule` (ridge, `ExperimentConfig::default()` lambda); replay demonstration 0 with it (`replay`, stream `demo` of every SMB frame, a throwaway sink for Tetris); then one closed-loop episode at `episode_noops(0, 6, config.noop_max)` (stream `drive`). Meta: `weights_a` (the 361 weights to A, intercept first, as in the old `reservoir_dump.rs`'s `a_weights`), `demo_steps: [{x, buttons, a_out}]`, `drive_steps: [{x, buttons, dead}]`, `death_x` (x at the first dead step, or null).
  - `paste_board`: a fresh `Reservoir::new`, driven by `Drive::Playfield` from the rule's 1-1 windows for 120 steps. Streams `smb` (the frame at each step's start) and `tetris` (the Tetris screen after each step). Meta: `steps: [{board, window}]` (board = the 200 playfield bytes after the step). The screen and the RAM board disagree because Tetris redraws only on a lock; this take is where that shows.
  - `placements`: the same with `Drive::Placements(Encoder::Salient)` on `Reservoir::new_fast`. Meta adds `placements: [[column, rotation] x4]` and `features` (the four salient values: nearest enemy distance, next enemy distance, Mario's y, Mario's x speed) per step.
  - `echo_twins`: two `Reservoir::new_fast` boards fed the salient placements of the same 120 rule windows, piece by piece with `place`. At step 20 only, twin B's first piece goes to `(column + 5) % 10`. Meta: `steps: [{a, b, diff}]` (both boards, and the count of cells that differ), `perturbed_step: 20`, `half_life`: steps after 20 until `diff` first drops to half its peak, or null if it never does. Resets count as the difference ending only if both twins reset on the same step; otherwise the difference stays, and the take records `resets_a`, `resets_b`.

- [ ] **Step 1: Write the failing test**

```rust
  #[test]
  #[ignore = "needs the ROMs and target/tetris-reservoir-demos-rule"]
  fn the_brain_and_board_takes_record() {
    let ctx = FilmContext::load(Path::new("roms2/super_mario_bros.nes"), Path::new("roms2/tetris.nes"), Path::new("target")).unwrap();
    let out = std::env::temp_dir().join(format!("film-brain-{}", std::process::id()));
    for name in ["simple_brain_drives", "paste_board", "placements", "echo_twins"] {
      record(&ctx, name, &out).unwrap();
    }
    let read = |n: &str| -> serde_json::Value { serde_json::from_slice(&std::fs::read(out.join(n).join("take.json")).unwrap()).unwrap() };
    assert_eq!(read("simple_brain_drives")["weights_a"].as_array().unwrap().len(), 361);
    let echo = read("echo_twins");
    let steps = echo["steps"].as_array().unwrap();
    assert_eq!(steps[19]["diff"], 0, "identical before the nudge");
    assert!(steps[20]["diff"].as_u64().unwrap() > 0, "different after it");
    let _ = std::fs::remove_dir_all(&out);
  }
```

- [ ] **Step 2: Run it to see it fail**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib reservoir::film::tests::the_brain_and_board_takes_record -- --ignored --exact`
Expected: FAIL, "no take named simple_brain_drives".

- [ ] **Step 3: Implement the four takes**

The rule windows all four share:

```rust
/// The rule's windows on 1-1 from the start, one per step, `steps` of them.
fn rule_windows(ctx: &FilmContext, steps: usize) -> Result<Vec<Vec<u8>>, String> {
  let mut smb = smb_at(ctx, Stage::ONE_ONE, 0);
  let mut player = Player::new(ctx.rule("tetris-reservoir-demos-rule")?);
  let mut out = Vec::with_capacity(steps);
  for _ in 0..steps {
    let window = smb_window(&mut smb);
    let buttons = player.buttons(&window);
    out.push(window);
    smb.step_frames(buttons, MACRO_FRAMES);
  }
  Ok(out)
}
```

`echo_twins`:

```rust
fn echo_twins(ctx: &FilmContext, _w: &mut TakeWriter) -> Result<Value, String> {
  let windows = rule_windows(ctx, 120)?;
  let mut a = Reservoir::new_fast(&ctx.tetris_rom)?;
  let mut b = Reservoir::new_fast(&ctx.tetris_rom)?;
  let (mut resets_a, mut resets_b) = (0, 0);
  let mut steps = Vec::new();
  for (step, window) in windows.iter().enumerate() {
    for (i, p) in encode(Encoder::Salient, window).into_iter().enumerate() {
      let q = if step == PERTURBED_STEP && i == 0 { Placement { column: (p.column + 5) % 10, ..p } } else { p };
      resets_a += a.place(p).reset as usize;
      resets_b += b.place(q).reset as usize;
    }
    let (fa, fb) = (a.playfield(), b.playfield());
    let diff = fa.iter().zip(&fb).filter(|(x, y)| (**x >= PLAYFIELD_THRESHOLD) != (**y >= PLAYFIELD_THRESHOLD)).count();
    steps.push(json!({"a": hex(&fa), "b": hex(&fb), "diff": diff}));
  }
  let diffs: Vec<u64> = steps.iter().map(|s| s["diff"].as_u64().unwrap()).collect();
  let peak = *diffs[PERTURBED_STEP..].iter().max().unwrap();
  let half_life = diffs[PERTURBED_STEP..].iter().position(|&d| peak > 0 && d * 2 <= peak && d < peak);
  Ok(json!({"steps": steps, "perturbed_step": PERTURBED_STEP, "half_life": half_life, "resets_a": resets_a, "resets_b": resets_b}))
}

const PERTURBED_STEP: usize = 20;
```

`simple_brain_drives` reuses the pattern of the old `examples/reservoir_dump.rs` lines 52 to 108 (fit with `fit_condition`, `replay` with one readout, `trace_episode`), writing to `w` through a small adapter:

```rust
/// A `Write` that hands each full frame to the take writer as `stream`.
struct FrameSink<'a> {
  w: &'a mut TakeWriter,
  stream: &'static str,
  buf: Vec<u8>,
}

impl Write for FrameSink<'_> {
  fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
    self.buf.extend_from_slice(data);
    while self.buf.len() >= FRAME_BYTES {
      let frame: Vec<u8> = self.buf.drain(..FRAME_BYTES).collect();
      self.w.frame(self.stream, &frame).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    }
    Ok(data.len())
  }
  fn flush(&mut self) -> std::io::Result<()> {
    Ok(())
  }
}
```

`a_out` for each replayed step is `outputs[0][A_INDEX]` where `A_INDEX = BUTTON_A.trailing_zeros() as usize`.

`paste_board` and `placements`: loop over `rule_windows(ctx, 120)` with a separate SMB emulator stepping the same buttons for the `smb` stream (or record the `smb` frame inside `rule_windows` by giving it an optional writer; choose the latter and pass `Some(w)` / `None`).

Register all four in `takes()`.

- [ ] **Step 4: Run the test and look at the takes**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib reservoir::film::tests::the_brain_and_board_takes_record -- --ignored --exact`
Expected: pass.

Run: `cargo run --release --example reservoir_film -- --take simple_brain_drives --take paste_board --take placements --take echo_twins`
Then dump PNGs of a few frames from each stream and read them, and print `death_x` and `half_life`. Expect the raw readout to die early in 1-1 (the old video saw the first Goomba, around x 300 to 450). Whatever the numbers are, they are what the captions will say.

- [ ] **Step 5: Commit**

```bash
rustfmt --edition 2021 src/nes/gym/reservoir/film.rs
git add src/nes/gym/reservoir/film.rs
# message: "Record the simple brain, the paste, the placements, and the echo between twin boards."
git commit -F <message file>
```

### Task 9: Takes of the coin-flip teachers and the rule's tripwires

**Files:**
- Modify: `src/nes/gym/reservoir/film.rs`

**Interfaces:**
- Consumes: `FilmContext::demos("tetris-reservoir-demos", 6)` (the beam-search demonstrations), `tripwires`, `smb_window`, `smb_dead`, `smb_cleared_at`.
- Produces takes:
  - `coin_flip_teachers`: replay all six beam demonstrations without frames, recording per step `{x, a}` (a = A held); and demonstration 0's frames (stream `demo0`) for the backdrop. Meta: `demos: [{noops, steps: [{x, a}]}]`, `disagree`: over every pair of demonstrations, at every x bucket of 16 px both pass through on the ground, the fraction of buckets where one presses A and the other does not. This is the measured "no two examples agree" number.
  - `rule_tripwires`: the rule plays 1-1 to the flag (or 750 steps), stream `smb` every frame. Meta: `rule` (its four numbers and `describe()`), `steps: [{x, buttons, tripwires: {enemy, pit, wall}, window}]`, `cleared`.

- [ ] **Step 1: Write the failing test**

```rust
  #[test]
  #[ignore = "needs the ROMs and target/tetris-reservoir-demos"]
  fn the_teacher_takes_record() {
    let ctx = FilmContext::load(Path::new("roms2/super_mario_bros.nes"), Path::new("roms2/tetris.nes"), Path::new("target")).unwrap();
    let out = std::env::temp_dir().join(format!("film-teach-{}", std::process::id()));
    record(&ctx, "coin_flip_teachers", &out).unwrap();
    record(&ctx, "rule_tripwires", &out).unwrap();
    let read = |n: &str| -> serde_json::Value { serde_json::from_slice(&std::fs::read(out.join(n).join("take.json")).unwrap()).unwrap() };
    assert_eq!(read("coin_flip_teachers")["demos"].as_array().unwrap().len(), 6);
    let d = read("coin_flip_teachers")["disagree"].as_f64().unwrap();
    assert!((0.0..=1.0).contains(&d));
    assert_eq!(read("rule_tripwires")["cleared"], true, "the rule clears 1-1");
    let _ = std::fs::remove_dir_all(&out);
  }
```

- [ ] **Step 2: Run it to see it fail**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib reservoir::film::tests::the_teacher_takes_record -- --ignored --exact`
Expected: FAIL, "no take named coin_flip_teachers".

- [ ] **Step 3: Implement**

`disagree`:

```rust
/// Over every pair of demonstrations and every 16 px bucket both cross on
/// the ground, the fraction of buckets where exactly one presses A.
fn disagreement(demos: &[Vec<(i32, bool, bool)>]) -> f64 {
  // Each demo: (x, a, on_ground) per step, reduced to bucket -> pressed A anywhere in it.
  let buckets: Vec<std::collections::HashMap<i32, bool>> = demos
    .iter()
    .map(|steps| {
      let mut m = std::collections::HashMap::new();
      for &(x, a, ground) in steps {
        if ground {
          *m.entry(x / 16).or_insert(false) |= a;
        }
      }
      m
    })
    .collect();
  let (mut differ, mut total) = (0usize, 0usize);
  for i in 0..buckets.len() {
    for j in i + 1..buckets.len() {
      for (b, &a) in &buckets[i] {
        if let Some(&c) = buckets[j].get(b) {
          total += 1;
          differ += (a != c) as usize;
        }
      }
    }
  }
  differ as f64 / total.max(1) as f64
}
```

Replay each demonstration from `smb_at(ctx, demo.stage, demo.noops)` with its buttons, reading `x`, A and `ram_read(PLAYER_FLOAT_STATE) == 0` per step. Add a unit test for `disagreement` (not ignored): two demos that press A in the same bucket give 0.0; two that differ in one of two shared buckets give 0.5.

`rule_tripwires` plays like `matrix_walk` but to the flag (`smb_cleared_at(&mut smb, Stage::ONE_ONE)`) and records `tripwires`.

- [ ] **Step 4: Run the tests and record**

Run: `RUSTFLAGS=-Awarnings cargo test --lib reservoir::film` (the new unit test passes), then the ignored test, then `cargo run --release --example reservoir_film -- --take coin_flip_teachers --take rule_tripwires`. Print `disagree`.

- [ ] **Step 5: Commit**

```bash
rustfmt --edition 2021 src/nes/gym/reservoir/film.rs
git add src/nes/gym/reservoir/film.rs
# message: "Record the coin-flip teachers and the rule's tripwires."
git commit -F <message file>
```

### Task 10: Takes of the three ways we fooled ourselves

**Files:**
- Modify: `src/nes/gym/reservoir/film.rs`

**Interfaces:**
- Consumes: `smb_window_leaky`, `candidates_with_position`, `search_features_in`, `search::score`, `collect_windows` (Task 6 and existing), `features::feature_row`, `encode::onehot_row`, `fit_readout`, `buttons_to_targets`, `outputs_to_buttons`, `onset_hits`, `bit_accuracy`, `trace::fit_condition`, `trace::trace_episode`, `Condition::Clock`, `level_step`.
- Produces takes:
  - `controller_leak`: a raw readout fitted on demonstrations 1 to 5 of `tetris-reservoir-demos-rule` with leaky windows (`smb_window_leaky`; rows `feature_row(&window, None)`, targets `buttons_to_targets(action)`); scored on demonstration 0 (meta `held_out: {a_accuracy, onset_recall}`), then driving 1-1 closed loop with leaky windows from `episode_noops(0, 6, noop_max)` (stream `drive`, meta `drive_steps: [{x, buttons, dead}]`, `max_x`). For contrast, the same with honest windows: meta `honest: {a_accuracy, onset_recall, max_x}` (no frames).
  - `clock_late_start`: `Condition::Clock` fitted on all six rule demonstrations; one episode at `episode_noops(0, 6, noop_max)` (stream `on_time`) and one 120 frames later (stream `late`). Meta: `on_time: {noops, max_x, steps}`, `late: {noops, max_x, steps, death_x}`.
  - `memoriser`: the windows of the six rule demonstrations (`collect_windows`); `search_features_in(&candidates_with_position(), ...)` picks four features; `search::score` gives held-out onset recall for them; a one-hot readout on them (rows `onehot_row(&encode(Encoder::Searched(f), &window))`) fitted on all six drives 1-1 from `episode_noops(0, ...)` (stream `drive`). Meta: `features: [{describe, address, relative, position}]` where `position` is true for `$6D`, `$86` or any relative feature, `held_out_onsets`, `max_x`, `death_x`.

- [ ] **Step 1: Write the failing test**

```rust
  #[test]
  #[ignore = "needs the ROMs and target/tetris-reservoir-demos-rule; minutes"]
  fn the_fooled_takes_record() {
    let ctx = FilmContext::load(Path::new("roms2/super_mario_bros.nes"), Path::new("roms2/tetris.nes"), Path::new("target")).unwrap();
    let out = std::env::temp_dir().join(format!("film-fooled-{}", std::process::id()));
    for name in ["controller_leak", "clock_late_start", "memoriser"] {
      record(&ctx, name, &out).unwrap();
    }
    let read = |n: &str| -> serde_json::Value { serde_json::from_slice(&std::fs::read(out.join(n).join("take.json")).unwrap()).unwrap() };
    assert!(read("controller_leak")["held_out"]["a_accuracy"].as_f64().unwrap() > 0.0);
    assert_eq!(read("memoriser")["features"].as_array().unwrap().len(), 4);
    let _ = std::fs::remove_dir_all(&out);
  }
```

- [ ] **Step 2: Run it to see it fail**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib reservoir::film::tests::the_fooled_takes_record -- --ignored --exact`
Expected: FAIL, "no take named controller_leak".

- [ ] **Step 3: Implement**

A leaky-or-honest dataset helper:

```rust
/// Rows and A-targets from replaying demonstrations, with the window read by `window`.
fn raw_dataset(ctx: &FilmContext, demos: &[Demonstration], window: fn(&mut NesEmulator) -> Vec<u8>) -> (Vec<Vec<f32>>, Vec<Vec<f32>>, Vec<Vec<u8>>) {
  let (mut rows, mut targets, mut buttons) = (Vec::new(), Vec::new(), Vec::new());
  for demo in demos {
    let mut smb = smb_at(ctx, demo.stage, demo.noops);
    let mut b = Vec::new();
    for &action in &demo.buttons {
      if !super::demo::smb_in_cutscene(&mut smb) {
        rows.push(feature_row(&window(&mut smb), None));
        targets.push(buttons_to_targets(action));
        b.push(action);
      }
      smb.step_frames(action, MACRO_FRAMES);
    }
    buttons.push(b);
  }
  (rows, targets, buttons)
}
```

Score demonstration 0 by predicting each of its rows, `outputs_to_buttons`, then `bit_accuracy(&pred, &truth, BUTTON_A)` and `onset_hits`. The closed loop for the leak and the memoriser is a small loop like `trace_episode` but with the take's own row function; write it once:

```rust
/// Drive 1-1 closed loop with `row` turning a window into features, writing frames to `stream` if given.
fn drive(ctx: &FilmContext, readout: &Readout, noops: usize, row: &dyn Fn(&mut NesEmulator) -> Vec<f32>, mut sink: Option<(&mut TakeWriter, &str)>) -> Result<Vec<Value>, String> {
  let mut smb = smb_at(ctx, Stage::ONE_ONE, noops);
  let mut steps = Vec::new();
  for _ in 0..ExperimentConfig::default().max_steps {
    let buttons = outputs_to_buttons(&readout.predict(&row(&mut smb)));
    for _ in 0..MACRO_FRAMES {
      smb.step_frame(buttons);
      if let Some((w, stream)) = sink.as_mut() {
        w.frame(stream, smb.framebuffer())?;
      }
    }
    let dead = smb_dead(&mut smb);
    steps.push(json!({"x": smb_x(&mut smb), "buttons": buttons, "dead": dead}));
    if dead || smb_cleared_at(&mut smb, Stage::ONE_ONE) {
      break;
    }
  }
  Ok(steps)
}
```

The memoriser's search takes minutes (the pool is about three times the default's); log its picks to stderr with `eprintln!`.

- [ ] **Step 4: Run and look**

Run the ignored test, then `cargo run --release --example reservoir_film -- --take controller_leak --take clock_late_start --take memoriser`. Print the meta numbers. Check the stories hold: the leak scores well held out and drives badly; the clock does better on time than late; the memoriser picks at least one `position` feature, scores high held out and drives badly. If a story does not hold, the film says what did happen: record what was seen in a ruling, and Task 16's captions follow the numbers, not the plan.

- [ ] **Step 5: Commit**

```bash
rustfmt --edition 2021 src/nes/gym/reservoir/film.rs
git add src/nes/gym/reservoir/film.rs
# message: "Record the three buggy builds: the controller leak, the late clock, and the memoriser."
git commit -F <message file>
```

### Task 11: Takes of the bank, the unseen level and the two theories, and the long-run numbers

**Files:**
- Modify: `src/nes/gym/reservoir/film.rs`
- Create: `tools/reservoir_film/numbers.json`

**Interfaces:**
- Consumes: `trace::fit_condition`, `hidden_for`, `observe`, `Hidden::Bank` (crate-visible), `Condition::BankTetris(BankEncoder::Hand, true)`, `Condition::BankHeights(BankEncoder::Hand, true)`, `Condition::Clock`, `column_heights`, `bank_placements`, `Stage { world: 3, level: 0 }` (4-1), `Player`.
- Produces takes:
  - `bank_drive`: `BankTetris(Hand, tidy)` fitted on all six `tetris-reservoir-demos-rule` demonstrations; five closed-loop episodes at `episode_noops(i, 6, noop_max)` run without frames to find the best, then the best rerun with frames. Streams `smb` (every frame). Meta: `episodes: [{noops, max_x}]`, `best: {noops, steps: [{x, buttons, boards: [hex x4], placements: [[[c, r] x4] x4]}]}`, `best_x`, `mean_x`, `passed_goomba_pair` (best_x > 1850).
  - `bank_noise`: the same readout; for jitter 0, 8, 32 and 96, three episodes each where every window byte gets `+uniform(-j, j)` (clamped to 0..255) before the bank sees it. Meta: `levels: [{jitter, mean_x, xs}]`. Stream `jitter32` (the first episode at 32).
  - `unseen_4_1`: the rule of `tetris-reservoir-demos-rule-1-1+6-1` playing 4-1 for 400 steps (stream `teacher`), and `Condition::Clock` fitted on that cache's six demonstrations driving 4-1 (stream `clock`). Meta: `teacher_max_x`, `clock_max_x`.
  - `heights`: replay `tetris-reservoir-demos-rule` demonstration 0 through `BankHeights(Hand, true)`'s hidden layer for 200 steps. Meta: `steps: [{boards: [hex x4], heights: [[10] x4], a}]`.
  - `hold_bit`: demonstration 0's A per step and the bit "A held last step" beside it. Meta: `steps: [{a, held}]`, `onsets` (steps where A starts). No frames, no Tetris.
- `numbers.json`: the run 38 table, for chapters 7 and 8. Exact content in Step 3.

The spec's `cold_open_bank` take is dropped: the cold open uses `bank_drive`'s frames. Ruling, recorded here.

- [ ] **Step 1: Write the failing test**

```rust
  #[test]
  #[ignore = "needs the ROMs and both rule caches; tens of minutes"]
  fn the_bank_takes_record() {
    let ctx = FilmContext::load(Path::new("roms2/super_mario_bros.nes"), Path::new("roms2/tetris.nes"), Path::new("target")).unwrap();
    let out = std::env::temp_dir().join(format!("film-bank-{}", std::process::id()));
    for name in ["bank_drive", "bank_noise", "unseen_4_1", "heights", "hold_bit"] {
      record(&ctx, name, &out).unwrap();
    }
    let read = |n: &str| -> serde_json::Value { serde_json::from_slice(&std::fs::read(out.join(n).join("take.json")).unwrap()).unwrap() };
    let bank = read("bank_drive");
    assert_eq!(bank["episodes"].as_array().unwrap().len(), 5);
    assert_eq!(bank["best"]["steps"][0]["boards"].as_array().unwrap().len(), 4);
    assert_eq!(read("bank_noise")["levels"].as_array().unwrap().len(), 4);
    let hold = read("hold_bit");
    let steps = hold["steps"].as_array().unwrap();
    for i in 1..steps.len() {
      assert_eq!(steps[i]["held"], steps[i - 1]["a"]);
    }
    let _ = std::fs::remove_dir_all(&out);
  }
```

- [ ] **Step 2: Run it to see it fail**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib reservoir::film::tests::the_bank_takes_record -- --ignored --exact`
Expected: FAIL, "no take named bank_drive".

- [ ] **Step 3: Implement the takes and write the numbers**

The bank episode loop (crate-internal, so it can use `Hidden`):

```rust
/// One bank episode on `stage`; per step the four boards and placements when `detail`.
fn bank_episode(ctx: &FilmContext, condition: Condition, readout: &Readout, stage: Stage, noops: usize, jitter: u8, seed: u64, detail: bool, mut sink: Option<(&mut TakeWriter, &str)>) -> Result<(i32, Vec<Value>), String> {
  let mut smb = smb_at(ctx, stage, noops);
  let mut hidden = hidden_for(condition, &ctx.tetris_rom)?;
  let mut rng = super::Lcg::new(seed);
  let (mut max_x, mut steps) = (0, Vec::new());
  for step in 0..ExperimentConfig::default().max_steps {
    let mut window = smb_window(&mut smb);
    if jitter > 0 {
      for b in window.iter_mut() {
        let d = (rng.next_f64() * (2 * jitter as u32 + 1) as f64) as i32 - jitter as i32;
        *b = (*b as i32 + d).clamp(0, 255) as u8;
      }
    }
    let buttons = outputs_to_buttons(&readout.predict(&observe(condition, &window, &mut hidden, level_step(noops, step))));
    hidden.record(buttons);
    for _ in 0..MACRO_FRAMES {
      smb.step_frame(buttons);
      if let Some((w, s)) = sink.as_mut() {
        w.frame(s, smb.framebuffer())?;
      }
    }
    let x = smb_x(&mut smb);
    max_x = max_x.max(x);
    if detail {
      let boards: Vec<String> = match &mut hidden {
        Hidden::Bank(b) => b.iter_mut().map(|r| hex(&r.playfield())).collect(),
        _ => Vec::new(),
      };
      let placements: Vec<Vec<[u8; 2]>> = bank_placements(BankEncoder::Hand, &window)
        .iter()
        .map(|board| board.iter().map(|p| [p.column, p.rotation]).collect())
        .collect();
      steps.push(json!({"x": x, "buttons": buttons, "boards": boards, "placements": placements}));
    }
    if smb_dead(&mut smb) || smb_cleared_at(&mut smb, stage) {
      break;
    }
  }
  Ok((max_x, steps))
}
```

Fit once per take with `fit_condition(condition, &ctx.smb_rom, &ctx.tetris_rom, &demos, config.lambda, config.logistic, config.jump_horizon)`. Run `bank_noise`'s twelve episodes on threads (`std::thread::scope`, one per episode) as `run` does; they share the readout by reference.

`tools/reservoir_film/numbers.json` (from the run 38 table in `docs/ideas/brain-cartridge.md`, commit 8b1198d):

```json
{
  "source": "run 38, docs/ideas/brain-cartridge.md, commit 8b1198d: trained on 1-1 and 6-1 (three demonstrations each), 4-1 held out; 15 press onsets on 4-1, standard error about 0.12",
  "test_onsets": 15,
  "test_se": 0.12,
  "rows": [
    {"key": "searched-onehot", "label": "four searched features, no Tetris", "onset": 0.797, "cross": 0.669, "test": 0.800},
    {"key": "bank-searched-onehot", "label": "sixteen searched features, no Tetris", "onset": 0.917, "cross": 0.684, "test": 0.733},
    {"key": "echo-state", "label": "echo state network", "onset": 0.684, "cross": 0.391, "test": 0.667},
    {"key": "bank-searched-tidy", "label": "four Tetris games (searched, tidy)", "onset": 0.744, "cross": 0.459, "test": 0.667},
    {"key": "bank-tetris", "label": "four Tetris games (hand)", "onset": 0.707, "cross": 0.308, "test": 0.533},
    {"key": "random-tanh", "label": "random scrambler", "onset": 0.466, "cross": 0.609, "test": 0.467},
    {"key": "bank-tidy", "label": "four Tetris games (hand, tidy)", "onset": 0.805, "cross": 0.406, "test": 0.333},
    {"key": "clock", "label": "a clock", "onset": 0.233, "cross": 0.308, "test": 0.133},
    {"key": "bank-tidy-heights", "label": "stack heights only", "onset": 0.301, "cross": 0.323, "test": 0.133},
    {"key": "bank-heights", "label": "stack heights only (no tidy)", "onset": 0.165, "cross": 0.195, "test": 0.000},
    {"key": "raw", "label": "raw bytes", "onset": 0.135, "cross": 0.150, "test": 0.000}
  ],
  "hold": {"bank-hold": {"onset": 0.504, "mean_x": 324.6}, "bank-onehot": {"onset": 0.489, "mean_x": 324.2},
           "bank-searched-hold": {"onset": 0.301, "mean_x": 488.4}, "bank-searched-onehot": {"onset": 0.917, "mean_x": 602.8},
           "bank-searched-tetris": {"onset": 0.729, "mean_x": 1125.0}},
  "clock_test_x": 1016.3,
  "bank_tidy_1_1": {"source": "run 32, 1-1 only", "onset": 0.899, "mean_x": 2039, "max_x": 2470}
}
```

Before committing, check each number against the run 38 table and the run 32 line in `docs/ideas/brain-cartridge.md`; fix any that disagree with the doc and say so in a ruling.

- [ ] **Step 4: Run and look**

Run the ignored test, then `cargo run --release --example reservoir_film -- --take bank_drive --take bank_noise --take unseen_4_1 --take heights --take hold_bit`. Print `best_x`, `mean_x`, the noise levels' `mean_x`, and the 4-1 maxima. Dump PNGs from `bank_drive` near its best x and read them.

- [ ] **Step 5: Commit**

```bash
rustfmt --edition 2021 src/nes/gym/reservoir/film.rs
git add src/nes/gym/reservoir/film.rs tools/reservoir_film/numbers.json
# message: "Record the bank driving, the bank under noise, the unseen level, the two theories, and the run 38 numbers."
git commit -F <message file>
```

---

## Phase C: Compositor

### Task 12: Takes, captions and chapters: the compositor's model

**Files:**
- Create: `tools/reservoir_film/model.py`
- Create: `tools/reservoir_film/tests/test_model.py`

**Interfaces:**
- Consumes: the take layout of Task 7 (`target/reservoir-film/<take>/take.json`, `<stream>.rgb`), `tools/reservoir_film/numbers.json`.
- Produces (in `reservoir_film.model`):
  - `REPO`, `TAKES_DIR = REPO / "target/reservoir-film"`, `W, H, FPS = 1280, 720, 60`.
  - `class Takes(root=TAKES_DIR)`: `takes[name]` returns a `Take`; a missing take raises `FileNotFoundError` naming the path and `cargo run --release --example reservoir_film -- --take <name>`.
  - `class Take`: `.name`, `.meta` (dict), `.frames(stream) -> Frames`, `.mtime` (of its `take.json`).
  - `class Frames`: `len(frames)`; `frames[i]` returns a 240x256x3 `uint8` array, with `i` clamped to `[0, len - 1]` (a shot longer than its take holds the last frame); backed by `np.memmap`.
  - `load_numbers() -> dict`.
  - `reading_seconds(text) -> float`: `max(3.0, 4.0 * words / 15)`; 0 for empty text.
  - `fill(text, takes, numbers) -> str`: replaces `{take.path.to.value}` and `{numbers.path}` with values, an optional format spec after `:` (`{echo_twins.half_life:.0f}`, `{coin_flip_teachers.disagree:.0%}`); integer path parts index lists. A missing take, key or a null value raises `KeyError` naming the reference.
  - `@dataclass class Caption: at: float; text: str | Callable[[Context], str]` — a callable is evaluated against the context, for captions that depend on what a take measured.
  - `@dataclass class Shot: name: str; seconds: float | Callable[[Context], float]; draw: Callable[[Context, float, int], Image]; captions: list[Caption] = []; takes: tuple[str, ...] = (); stings: list[tuple[float, str]] | Callable[[Context], list] = []` — `draw(ctx, t, i)` gets seconds into the shot and the frame index in it and returns a 1280x720 RGB image without the caption (the build adds it).
  - `@dataclass class Chapter: number: int; title: str; subtitle: str; shots: list[Shot]`.
  - `class Context`: `.takes`, `.numbers`.
  - `shot_seconds(shot, ctx) -> float`, `chapter_seconds(chapter, ctx) -> float` (the title card included: `TITLE_SECONDS = 4.0`, added by the build for every chapter but 0).
  - `caption_spans(shot, ctx) -> list[tuple[float, float, str]]`: each caption's filled text and the span it shows for, until the next caption or the shot's end.
  - `check_captions(chapter, ctx) -> list[str]`: one message per caption shown for less than `reading_seconds`, naming chapter, shot and text. Empty when fine.

- [ ] **Step 1: Write the failing tests**

```python
import json
import tempfile
import unittest
from pathlib import Path

import numpy as np

from reservoir_film.model import (
    Caption, Chapter, Context, Shot, Takes, caption_spans, check_captions, fill, reading_seconds,
)


def make_take(root, name, meta, streams):
    d = Path(root) / name
    d.mkdir(parents=True)
    counts = {}
    for stream, n in streams.items():
        frames = np.zeros((n, 240, 256, 3), dtype=np.uint8)
        for i in range(n):
            frames[i, 0, 0, 0] = i
        frames.tofile(d / f"{stream}.rgb")
        counts[stream] = n
    (d / "take.json").write_text(json.dumps({**meta, "streams": counts}))


class TakesTest(unittest.TestCase):
    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        make_take(self.dir.name, "walk", {"best": {"x": [10, 20]}, "rate": 0.25, "none": None}, {"smb": 3})
        self.takes = Takes(self.dir.name)

    def tearDown(self):
        self.dir.cleanup()

    def test_frames_clamp_past_the_end(self):
        f = self.takes["walk"].frames("smb")
        self.assertEqual(len(f), 3)
        self.assertEqual(int(f[1][0, 0, 0]), 1)
        self.assertEqual(int(f[99][0, 0, 0]), 2, "holds the last frame")
        self.assertEqual(int(f[-5][0, 0, 0]), 0)

    def test_a_missing_take_says_how_to_record_it(self):
        with self.assertRaisesRegex(FileNotFoundError, "--take nope"):
            self.takes["nope"]

    def test_fill_reads_paths_indexes_and_formats(self):
        numbers = {"test_onsets": 15}
        self.assertEqual(fill("x {walk.best.x.1}, {walk.rate:.0%}, {numbers.test_onsets}", self.takes, numbers),
                         "x 20, 25%, 15")

    def test_fill_names_a_missing_or_null_reference(self):
        with self.assertRaisesRegex(KeyError, "walk.best.y"):
            fill("{walk.best.y}", self.takes, {})
        with self.assertRaisesRegex(KeyError, "walk.none"):
            fill("{walk.none}", self.takes, {})


class CaptionsTest(unittest.TestCase):
    def test_reading_time(self):
        self.assertEqual(reading_seconds(""), 0)
        self.assertEqual(reading_seconds("Three short words"), 3.0)
        self.assertAlmostEqual(reading_seconds(" ".join(["w"] * 30)), 8.0)

    def test_spans_and_the_timing_check(self):
        ctx = Context(takes=None, numbers={})
        shot = Shot(name="s", seconds=10, draw=None, captions=[
            Caption(0, "A caption of exactly six words here."), Caption(2, "Too soon for the first one."),
            Caption(6, lambda c: "Computed text."),
        ])
        spans = caption_spans(shot, ctx)
        self.assertEqual(spans, [(0, 2, "A caption of exactly six words here."), (2, 6, "Too soon for the first one."),
                                 (6, 10, "Computed text.")])
        problems = check_captions(Chapter(1, "T", "", [shot]), ctx)
        self.assertEqual(len(problems), 1)
        self.assertIn("A caption of exactly six words", problems[0])
```

- [ ] **Step 2: Run them to see them fail**

Run: `cd tools && python3 -m unittest discover -s reservoir_film/tests -t .`
Expected: ImportError for `reservoir_film.model`.

- [ ] **Step 3: Implement `model.py`**

```python
"""The film's model: takes recorded by examples/reservoir_film.rs, the
long-run numbers, and chapters made of shots with captions."""

import json
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable

import numpy as np

REPO = Path(__file__).resolve().parents[2]
TAKES_DIR = REPO / "target" / "reservoir-film"
NUMBERS = Path(__file__).resolve().parent / "numbers.json"
W, H, FPS = 1280, 720, 60
TITLE_SECONDS = 4.0
NES_W, NES_H = 256, 240
REF = re.compile(r"\{([a-z0-9_]+)\.([A-Za-z0-9_.]+)(?::([^}]+))?\}")


class Frames:
    def __init__(self, path, n):
        self.n = n
        self.data = np.memmap(path, dtype=np.uint8, mode="r", shape=(n, NES_H, NES_W, 3)) if n else None

    def __len__(self):
        return self.n

    def __getitem__(self, i):
        return np.asarray(self.data[min(max(int(i), 0), self.n - 1)])


class Take:
    def __init__(self, path):
        self.path = Path(path)
        self.name = self.path.name
        self.meta = json.loads((self.path / "take.json").read_text())
        self.mtime = (self.path / "take.json").stat().st_mtime

    def frames(self, stream):
        n = self.meta["streams"].get(stream)
        if n is None:
            raise KeyError(f"take {self.name} has no stream {stream}; it has {sorted(self.meta['streams'])}")
        return Frames(self.path / f"{stream}.rgb", n)


class Takes:
    def __init__(self, root=TAKES_DIR):
        self.root = Path(root)
        self.cache = {}

    def __getitem__(self, name):
        if name not in self.cache:
            path = self.root / name
            if not (path / "take.json").exists():
                raise FileNotFoundError(
                    f"no take at {path}; record it with: cargo run --release --example reservoir_film -- --take {name}")
            self.cache[name] = Take(path)
        return self.cache[name]


def load_numbers():
    return json.loads(NUMBERS.read_text())


def reading_seconds(text):
    words = len(text.split())
    return 0 if words == 0 else max(3.0, 4.0 * words / 15)


def _walk(value, path, ref):
    for part in path.split("."):
        try:
            value = value[int(part)] if isinstance(value, list) else value[part]
        except (KeyError, IndexError, ValueError, TypeError):
            raise KeyError(f"caption reference {{{ref}}} not found")
    if value is None:
        raise KeyError(f"caption reference {{{ref}}} is null")
    return value


def fill(text, takes, numbers):
    def one(m):
        head, path, spec = m.groups()
        ref = f"{head}.{path}"
        if head == "numbers":
            value = _walk(numbers, path, ref)
        else:
            try:
                meta = takes[head].meta
            except FileNotFoundError as e:
                raise KeyError(f"caption reference {{{ref}}}: {e}")
            value = _walk(meta, path, ref)
        return format(value, spec) if spec else str(value)
    return REF.sub(one, text)


@dataclass
class Context:
    takes: object
    numbers: dict


@dataclass
class Caption:
    at: float
    text: object


@dataclass
class Shot:
    name: str
    seconds: object
    draw: Callable
    captions: list = field(default_factory=list)
    takes: tuple = ()
    stings: object = field(default_factory=list)


@dataclass
class Chapter:
    number: int
    title: str
    subtitle: str
    shots: list


def shot_seconds(shot, ctx):
    return float(shot.seconds(ctx) if callable(shot.seconds) else shot.seconds)


def chapter_seconds(chapter, ctx):
    title = TITLE_SECONDS if chapter.number > 0 else 0.0
    return title + sum(shot_seconds(s, ctx) for s in chapter.shots)


def caption_text(caption, ctx):
    text = caption.text(ctx) if callable(caption.text) else caption.text
    return fill(text, ctx.takes, ctx.numbers)


def caption_spans(shot, ctx):
    end = shot_seconds(shot, ctx)
    caps = sorted(shot.captions, key=lambda c: c.at)
    out = []
    for i, c in enumerate(caps):
        stop = caps[i + 1].at if i + 1 < len(caps) else end
        out.append((c.at, stop, caption_text(c, ctx)))
    return out


def check_captions(chapter, ctx):
    problems = []
    for shot in chapter.shots:
        for start, stop, text in caption_spans(shot, ctx):
            need = reading_seconds(text)
            if stop - start + 1e-9 < need:
                problems.append(f"chapter {chapter.number}, shot {shot.name}: '{text}' shows {stop - start:.1f} s, needs {need:.1f} s")
    return problems


def shot_stings(shot, ctx):
    return shot.stings(ctx) if callable(shot.stings) else list(shot.stings)
```

- [ ] **Step 4: Run the tests**

Run: `cd tools && python3 -m unittest discover -s reservoir_film/tests -t .`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add tools/reservoir_film/model.py tools/reservoir_film/tests/test_model.py
# message: "Add the film's model: takes, numbers, captions and chapters."
git commit -F <message file>
```

### Task 13: Widgets

**Files:**
- Create: `tools/reservoir_film/draw.py`
- Create: `tools/reservoir_film/gallery.py`
- Create: `tools/reservoir_film/tests/test_draw.py`

**Interfaces:**
- Consumes: `W`, `H`, `NES_W`, `NES_H` (Task 12); takes' hex strings.
- Produces (in `reservoir_film.draw`), every function drawing onto a given `PIL.Image` (RGB, 1280x720) or returning one:
  - Palette constants: `BG = (12, 14, 20)`, `INK = (235, 235, 235)`, `DIM = (140, 145, 160)`, `GREEN = (90, 220, 120)`, `RED = (235, 90, 80)`, `GOLD = (245, 200, 80)`, `MATRIX = (60, 255, 120)`, and `REGION = {"mario": (240, 90, 80), "enemies": (250, 170, 60), "floor": (90, 200, 120), "wall": (90, 150, 250)}`.
  - `font(size, bold=False)`: a system TrueType font (try `/System/Library/Fonts/Supplemental/Arial.ttf`, `/System/Library/Fonts/Helvetica.ttc`, `DejaVuSans.ttf`), cached; `mono(size)` likewise from `Menlo.ttc` / `DejaVuSansMono.ttf`.
  - `blank() -> Image`
  - `nes(img, frame, box)`: an NES frame (240x256x3 array) scaled with nearest neighbour into `box = (x, y, w, h)`.
  - `caption(img, text)`: the bottom bar (96 px, translucent black) with the text wrapped to two lines at most, 30 px; text longer than two lines raises `ValueError` (captions must stay short).
  - `title_card(number, title, subtitle) -> Image`: chapter number small and gold, title large, subtitle dim.
  - `text_card(lines, kicker=None) -> Image`: one to four lines centred, optional small gold kicker above ("Mistake 1").
  - `byte_grid(img, data, box, cols, highlight=None, labels=None, changed=None)`: cells lit by value (black to `MATRIX`), `highlight` a dict index → colour drawing an outline, `changed` a set of indices drawn in `GOLD`, `labels` a list of (index, text) drawn with leader lines to the right of the grid.
  - `hex_grid(img, data, box, cols, alpha=1.0, highlight=None)`: the same bytes as two-digit hex in `MATRIX`, for the matrix shots.
  - `board(img, cells, box, diff=None, colour=INK, recent=None)`: a 10x20 Tetris playfield from 200 bytes (filled when ≥ `0xC0`, the code's `PLAYFIELD_THRESHOLD`), `diff` cells outlined `RED`, `recent` cells filled `GOLD`.
  - `heights_bars(img, heights, box)`: ten bars.
  - `gauge(img, history, truth, box, threshold=0.5)`: a scrolling strip of the readout's A output (a line) over true presses (blocks), with the threshold dashed.
  - `strip(img, box, lanes, x_max, marker=None)`: the level left to right, one lane per list of jump xs, ticks per jump, a marker at x.
  - `bar_chart(img, box, rows, value_key, label_key, shown, se=None, show_se=False)`: horizontal bars, the first `shown` rows only (for building one at a time), error bars of ±`se` when `show_se`.
  - `lamp(img, centre, on, colour, label)`: a round light with a label, for the tripwires.
  - `glyph_tiles(screens: Frames, box) -> list[Image]`: the ones-digit tile cut from each lives screen at `digit_box`, 8x8 scaled.
  - `rain(img, tiles, t, seed, speed=1.0, density=1.0)`: columns of falling tiles, deterministic for (t, seed).
  - `ripple(img, box, t, drops)`: a top-down tank with expanding rings from each (time, x, y) drop, and a ruler line whose height is the sum of the rings under it.
  - `fade(img, other, a) -> Image`: a new image, `img` blended toward `other` by `a` in 0..1.
- `gallery.py`: `python3 -m reservoir_film.gallery` writes `target/reservoir-film/gallery.png`, every widget drawn once from real takes where they exist and synthetic data where not, for eyeballing.

- [ ] **Step 1: Write the failing tests**

```python
import unittest

import numpy as np
from PIL import Image

from reservoir_film import draw
from reservoir_film.model import H, W


class Widgets(unittest.TestCase):
    def test_cards_are_frame_sized(self):
        for img in (draw.title_card(3, "Pour Mario into Tetris", "a subtitle"), draw.text_card(["one", "two"], kicker="k")):
            self.assertEqual(img.size, (W, H))
            self.assertEqual(img.mode, "RGB")

    def test_caption_refuses_three_lines(self):
        img = draw.blank()
        draw.caption(img, "A short caption.")
        with self.assertRaises(ValueError):
            draw.caption(img, " ".join(["overlong"] * 60))

    def test_board_fills_cells_at_the_threshold(self):
        img = draw.blank()
        cells = bytes([0xEF] * 199 + [0x7B]).hex()
        draw.board(img, cells, (0, 0, 100, 200))
        bottom_right = img.getpixel((95, 195))
        top_left = img.getpixel((5, 5))
        self.assertNotEqual(bottom_right, top_left)

    def test_rain_is_deterministic(self):
        tiles = [Image.new("RGB", (16, 16), (255, 255, 255))] * 4
        a, b = draw.blank(), draw.blank()
        draw.rain(a, tiles, 1.5, seed=7)
        draw.rain(b, tiles, 1.5, seed=7)
        self.assertTrue(np.array_equal(np.asarray(a), np.asarray(b)))

    def test_bar_chart_draws_only_the_shown_rows(self):
        rows = [{"label": f"r{i}", "v": 0.1 * (i + 1)} for i in range(5)]
        a, b = draw.blank(), draw.blank()
        draw.bar_chart(a, (100, 100, 800, 400), rows, "v", "label", shown=2)
        draw.bar_chart(b, (100, 100, 800, 400), rows, "v", "label", shown=5)
        self.assertFalse(np.array_equal(np.asarray(a), np.asarray(b)))
```

- [ ] **Step 2: Run them to see them fail**

Run: `cd tools && python3 -m unittest discover -s reservoir_film/tests -t .`
Expected: ImportError for `reservoir_film.draw`.

- [ ] **Step 3: Implement `draw.py` and `gallery.py`**

Port what still serves from `tools/reservoir_video.py` (its `caption`, `byte_grid`, `board`, `gauge`, `strip` and `panel` are a starting point at lines 127 to 290), restyle to the palette above, and add the new widgets. `rain`:

```python
def rain(img, tiles, t, seed, speed=1.0, density=1.0):
    """Columns of falling tiles. Each column has its own speed and phase from
    `seed`; a tile's glyph changes as it falls, like the film it parodies."""
    rng = np.random.default_rng(seed)
    size = tiles[0].size[0]
    cols = W // size
    speeds = rng.uniform(80, 260, cols) * speed
    phases = rng.uniform(0, H, cols)
    lengths = rng.integers(6, 22, cols)
    picks = rng.integers(0, len(tiles), (cols, 64))
    for c in range(cols):
        if rng.random() > density:
            continue
        head = (phases[c] + speeds[c] * t) % (H + lengths[c] * size)
        for k in range(lengths[c]):
            y = int(head - k * size)
            if 0 <= y < H:
                glyph = tiles[picks[c, (k + int(t * 8)) % 64]]
                shade = 1.0 - k / lengths[c]
                tinted = Image.eval(glyph, lambda v, s=shade: int(v * s))
                img.paste(tinted, (c * size, y))
```

The `rng` draws happen in a fixed order, so `rain` is deterministic for a seed. Tint glyph tiles toward `MATRIX` once, in `glyph_tiles`, by multiplying their luminance into the green colour.

- [ ] **Step 4: Run the tests and look at the gallery**

Run: `cd tools && python3 -m unittest discover -s reservoir_film/tests -t .` (all pass), then `cd tools && python3 -m reservoir_film.gallery` and read `target/reservoir-film/gallery.png`. Check: text legible at 1280x720, the caption bar does not cover the NES frame's bottom row of tiles when frames are placed in the upper 600 px, cards are centred, the rain reads as SMB tiles.

- [ ] **Step 5: Commit**

```bash
git add tools/reservoir_film/draw.py tools/reservoir_film/gallery.py tools/reservoir_film/tests/test_draw.py
# message: "Add the film's widgets: cards, grids, boards, gauges, charts, rain and ripples."
git commit -F <message file>
```

### Task 14: The build: segments, music, chapter markers

**Files:**
- Create: `tools/reservoir_film/build.py`
- Create: `tools/reservoir_film/chapters/__init__.py`
- Create: `tools/reservoir_film/tests/test_build.py`

**Interfaces:**
- Consumes: `model` (Task 12), `draw.title_card`, `draw.caption` (Task 13), `music.score.score`, `music.score.write_wav`, `music.cues.CUES`, `STINGS`, `CHAPTER_CUES` (Tasks 4 and 5).
- Produces:
  - `chapters/__init__.py`: `CHAPTERS: list[Chapter]`, imported from `c00_cold_open` … `c09_verdict` as they are written (Tasks 15 to 19); until then, the list holds what exists.
  - `build.py`, run from `tools/` as `python3 -m reservoir_film.build [--chapter N]... [--quick] [--no-music] [--contact]`:
    - Checks every chapter's captions with `check_captions`; any problem stops the build with the list.
    - Renders each chapter to `target/reservoir-film/segments/cNN.mp4` (video only, H.264 `-crf 20 -pix_fmt yuv420p`, 60 fps) by piping raw RGB frames to ffmpeg. Chapters after 0 open with `TITLE_SECONDS` of `title_card`. Captions are drawn on each frame from `caption_spans`. `--quick` renders every tenth frame at 6 fps into `segments/quick-cNN.mp4`.
    - Skips a chapter whose `segments/cNN.key` matches `segment_key(chapter, ctx)`.
    - Joins the segments with ffmpeg's concat demuxer.
    - Builds the score: `plan = [(CUES[CHAPTER_CUES[n]], chapter_seconds(ch, ctx)) for each chapter]`, `stings` from every shot's `stings` (shot-relative seconds turned into film seconds); writes `segments/score.wav`; muxes it with `-af loudnorm=I=-16:TP=-1.5:LRA=11 -c:a aac -b:a 192k`.
    - Writes chapter markers into the MP4 (an `ffmetadata` file with one `[CHAPTER]` per chapter, `TIMEBASE=1/1000`) and `target/reservoir-film/chapters.txt` in YouTube form (`0:00 Cold open`, `0:39 1. Mario is just numbers`, ...).
    - `--contact` writes `target/reservoir-film/contact/cNN.png`: a frame every two seconds, captions included, 8 across at 320x180.
  - Functions tested directly: `segment_key(chapter, ctx) -> str`, `chapter_starts(chapters, ctx) -> list[float]`, `youtube_chapters(chapters, ctx) -> str`, `ffmetadata(chapters, ctx) -> str`.

- [ ] **Step 1: Write the failing tests**

```python
import unittest

from reservoir_film import build
from reservoir_film.model import Caption, Chapter, Context, Shot

CTX = Context(takes=None, numbers={})


def chapter(n, seconds, title="T"):
    return Chapter(n, title, "", [Shot(name="s", seconds=seconds, draw=None, captions=[])])


class Build(unittest.TestCase):
    def test_chapter_starts_include_title_cards_after_the_cold_open(self):
        chs = [chapter(0, 39), chapter(1, 100), chapter(2, 60)]
        self.assertEqual(build.chapter_starts(chs, CTX), [0.0, 39.0, 143.0])

    def test_youtube_chapters(self):
        chs = [chapter(0, 39, "Cold open"), chapter(1, 100, "Mario is just numbers"), chapter(2, 600, "Long")]
        self.assertEqual(build.youtube_chapters(chs, CTX),
                         "0:00 Cold open\n0:39 1. Mario is just numbers\n2:23 2. Long\n")

    def test_ffmetadata_has_one_chapter_each(self):
        text = build.ffmetadata([chapter(0, 10), chapter(1, 20)], CTX)
        self.assertTrue(text.startswith(";FFMETADATA1"))
        self.assertEqual(text.count("[CHAPTER]"), 2)
        self.assertIn("START=10000", text)
        self.assertIn("END=34000", text)

    def test_segment_key_changes_with_the_chapter_and_its_takes(self):
        a = build.segment_key(chapter(1, 10), CTX, sources="code v1", take_times={"t": 1.0})
        self.assertEqual(a, build.segment_key(chapter(1, 10), CTX, sources="code v1", take_times={"t": 1.0}))
        self.assertNotEqual(a, build.segment_key(chapter(1, 11), CTX, sources="code v1", take_times={"t": 1.0}))
        self.assertNotEqual(a, build.segment_key(chapter(1, 10), CTX, sources="code v2", take_times={"t": 1.0}))
        self.assertNotEqual(a, build.segment_key(chapter(1, 10), CTX, sources="code v1", take_times={"t": 2.0}))
```

- [ ] **Step 2: Run them to see them fail**

Run: `cd tools && python3 -m unittest discover -s reservoir_film/tests -t .`
Expected: ImportError for `reservoir_film.build`.

- [ ] **Step 3: Implement `build.py`**

```python
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


def contact_sheet(chapter, ctx):
    out = TAKES_DIR / "contact"
    out.mkdir(parents=True, exist_ok=True)
    thumbs = [img.resize((320, 180)) for i, img in enumerate(frames_of(chapter, ctx, False)) if i % (2 * FPS) == 0]
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
```

- [ ] **Step 4: Run the tests**

Run: `cd tools && python3 -m unittest discover -s reservoir_film/tests -t .`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add tools/reservoir_film/build.py tools/reservoir_film/chapters/__init__.py tools/reservoir_film/tests/test_build.py
# message: "Build the film: cached chapter segments, the score, and chapter markers."
git commit -F <message file>
```

### How the chapter tasks work (read before Tasks 15 to 19)

Each chapter is a module `tools/reservoir_film/chapters/cNN_<slug>.py` defining `CHAPTER = Chapter(...)`, appended to `CHAPTERS` in `chapters/__init__.py` in number order. Each shot's `draw(ctx, t, i)` starts from `draw.blank()` and uses the Task 13 widgets. Durations are targets; adjust a shot by a second or two when the contact sheet shows it needs it, and never below what its captions need (`check_captions` enforces this).

Layout used throughout unless a shot says otherwise: an NES frame at 2x is 512x480 at `(64, 60)`; the right-hand panel is `(640, 60, 576, 480)`; captions sit in the bottom 96 px; nothing important below y = 600.

Playing a take "in real time" means frame `i` of the shot shows stream frame `i` (60 fps on both sides). Streams and records kept once per step (Tetris screens, `steps` lists) are indexed `i // 4`, since a step is four NES frames.

Every chapter task has the same test and check steps:

- [ ] **Test (write first, watch it fail):** add to `tools/reservoir_film/tests/test_chapters.py`:

```python
import unittest

from reservoir_film.model import TAKES_DIR, Context, Takes, check_captions, load_numbers, shot_seconds

HAVE_TAKES = (TAKES_DIR / "bank_drive" / "take.json").exists()


def chapter(n):
    from reservoir_film.chapters import CHAPTERS
    found = [c for c in CHAPTERS if c.number == n]
    if not found:
        raise AssertionError(f"chapter {n} is not in CHAPTERS")
    return found[0]


@unittest.skipUnless(HAVE_TAKES, "record the takes first: cargo run --release --example reservoir_film")
class Chapters(unittest.TestCase):
    def check(self, n):
        ctx = Context(takes=Takes(), numbers=load_numbers())
        ch = chapter(n)
        self.assertEqual(check_captions(ch, ctx), [])
        for shot in ch.shots:
            last = int(shot_seconds(shot, ctx) * 60) - 1
            for i in (0, last // 2, last):
                img = shot.draw(ctx, i / 60, i)
                self.assertEqual(img.size, (1280, 720), f"{shot.name} frame {i}")
```

and one method per chapter, `def test_chapter_N(self): self.check(N)`, added by that chapter's task. Run `cd tools && python3 -m unittest reservoir_film.tests.test_chapters`; it fails with "chapter N is not in CHAPTERS" before the module exists.

- [ ] **Contact sheet:** `cd tools && python3 -m reservoir_film.build --chapter N --contact`, then read `target/reservoir-film/contact/cNN.png`. Check that each caption matches what is on screen when it shows, that nothing overlaps the caption bar, that text is legible, and that every experiment is followed by a few seconds without a caption.
- [ ] **Quick render:** `cd tools && python3 -m reservoir_film.build --chapter N --quick`, and confirm it writes `segments/quick-cNN.mp4`.
- [ ] **Commit** the chapter module, `chapters/__init__.py` and the test, message "Add chapter N of the film: <title>."

### Task 15: Chapters 0 and 1 — cold open, and Mario is just numbers

**Files:** Create `tools/reservoir_film/chapters/c00_cold_open.py`, `c01_numbers.py`; modify `chapters/__init__.py`, `tests/test_chapters.py`.

**Interfaces:** Consumes takes `bank_drive`, `lives_glyphs`, `matrix_walk`, `poke_enemy`, `poke_terrain`, `noise_bytes`; widgets `nes`, `board`, `title_card`, `glyph_tiles`, `rain`, `hex_grid`, `byte_grid`, `fade`. Produces `CHAPTER` in each module.

Chapter 0, *Cold open*, about 39 s, cue `coin_run`:

| Shot | Seconds | What is drawn | Captions (at s: text) | Stings |
|---|---|---|---|---|
| `bank_open` | 12 | `bank_drive` stream `smb` in real time from frame 0 at 2x on the left; the four boards of `best.steps[i // 4]` on the right, 2x2, each outlined in its `REGION` colour | 6: "Four games of Tetris are deciding when Mario jumps." | |
| `title` | 5 | `title_card(0, "Can Tetris play Mario?", "A lab notebook in ten chapters")` with the number hidden for chapter 0 | (none) | |
| `lives` | 14 | `lives_glyphs` stream `screen` at 2x, centred: values 7 to 12 for a second each, then accelerating through 13 to 255 (frame index from an easing curve) | 0: "In Super Mario Bros., the lives counter is a single number." 5: "Past nine, the game runs out of digits and draws whatever tiles come next." 10: "We didn't earn these lives. We typed them in." | `blip` at each of the first twenty value changes |
| `rain_begins` | 8 | the last lives screen fading out while `rain(img, glyph_tiles(...), t, seed=1, speed=0.3 + t/4, density=t/8)` fades in | 2: "Everything Mario is, is numbers like these." | |

Chapter 1, *Mario is just numbers*, subtitle "Two thousand and forty-eight bytes", about 122 s plus the title, cue `underground`:

| Shot | Seconds | What is drawn | Captions | Stings |
|---|---|---|---|---|
| `rain_resolves` | 20 | rain slowing (`speed` from 1 to 0.1); from 6 s, `hex_grid` of `matrix_walk.steps[i//4].window` zero page (16x16) fading in over it | 0: "The NES has 2,048 bytes of memory, and Super Mario Bros. keeps its whole world in them." 8: "Most of them change every frame. It looks like noise." 14: "But some of them are things." | |
| `labels` | 24 | `matrix_walk` `smb` at 1x top-left (256x240 at (64, 60)); `byte_grid` of the window right, `changed` = bytes that differ from the previous step; at each caption one more label from `labels` lights (`highlight` in `GOLD`, leader line and text) | 0: "This byte is how far Mario has walked." 5: "This one says whether he's on the ground, jumping, or falling." 11: "These five are the enemies on screen, and these say where they are." 17: "And this block is a map of the ground ahead: pits, pipes and bricks." | |
| `matrix` | 8 | the `smb` frame fades out; the grid redrawn as `hex_grid` in `MATRIX`, labels kept | 1: "After a while you don't see the code. You see Goomba, pipe, pit." | |
| `poke_enemy` | 16 | `poke_enemy` `smb` at 2x left; right, the poked byte as a big hex number that flips at each poke frame, with "before → after" | 0: "If the numbers are the world, changing a number should change the world." 6: "So we moved a Goomba by writing a new x into its byte." 11: "" | |
| `wall` | 12 | `poke_terrain` stream `wall` at 2x left; right, the 13-row terrain map around Mario with the poked cells in `GOLD` | 0: "Here we wrote bricks into the map, but not onto the screen." 6: "Mario runs into a wall nobody can see." | |
| `floor` | 12 | stream `floor` likewise, the erased cells outlined `RED` | 0: "And here we erased the ground from the map, but not from the picture." 6: (callable) if `poke_terrain.floor.died`: "He falls through. The picture is a drawing; the numbers are what's real." else "The picture is a drawing; the numbers are what's real." | `death` at the frame he dies, if he does |
| `noise` | 14 | `noise_bytes` streams `seed1` to `seed4` in a 2x2 grid at 1x | 0: "Spray random numbers into memory and the world comes apart." 7: "" | |
| `brain_view` | 16 | `matrix_walk` `smb` at 2x left fading to black from 6 s to 10 s; right, the whole 360-byte window: zero page 16x16 above, terrain as an 8-column by 13-row map below | 0: "A brain playing Mario never sees the picture. It gets these 360 numbers every step." 8: "Could you tell when to jump from this?" | |

`c00_cold_open.py` in full, as the model for the others:

```python
"""Chapter 0: four boards driving Mario, the title, the lives counter running past nine into garbage tiles, and the rain."""

import numpy as np

from .. import draw
from ..model import Caption, Chapter, Shot

REGIONS = ["mario", "enemies", "floor", "wall"]


def bank_open(ctx, t, i):
    img = draw.blank()
    take = ctx.takes["bank_drive"]
    draw.nes(img, take.frames("smb")[i], (64, 60, 512, 480))
    steps = take.meta["best"]["steps"]
    boards = steps[min(i // 4, len(steps) - 1)]["boards"]
    for k, (cells, region) in enumerate(zip(boards, REGIONS)):
        x, y = 700 + (k % 2) * 260, 70 + (k // 2) * 240
        draw.board(img, cells, (x, y, 110, 220), colour=draw.REGION[region])
    return img


def title(ctx, t, i):
    return draw.title_card(None, "Can Tetris play Mario?", "A lab notebook in ten chapters")


def lives_value(t):
    """7 to 12 one a second, then accelerating to 255 by 14 s."""
    if t < 6:
        return 7 + int(t)
    return min(255, 13 + int((t - 6) ** 2.6 * 1.2))


def lives(ctx, t, i):
    img = draw.blank()
    draw.nes(img, ctx.takes["lives_glyphs"].frames("screen")[lives_value(t)], (256, 0, 768, 720))
    return img


def lives_stings(ctx):
    out, last = [], None
    for k in range(14 * 60):
        v = lives_value(k / 60)
        if v != last and len(out) < 20:
            out.append((k / 60, "blip"))
        last = v
    return out


_tiles = {}


def tiles(ctx):
    if "t" not in _tiles:
        take = ctx.takes["lives_glyphs"]
        _tiles["t"] = draw.glyph_tiles(take.frames("screen"), take.meta["digit_box"])
    return _tiles["t"]


def rain_begins(ctx, t, i):
    under = lives(ctx, 14.0, 0)
    img = draw.blank()
    draw.rain(img, tiles(ctx), t, seed=1, speed=0.3 + t / 4, density=min(1.0, t / 8))
    return draw.fade(under, img, min(1.0, t / 3))


CHAPTER = Chapter(0, "Cold open", "", [
    Shot("bank_open", 12, bank_open, [Caption(6, "Four games of Tetris are deciding when Mario jumps.")], takes=("bank_drive",)),
    Shot("title", 5, title),
    Shot("lives", 14, lives, [
        Caption(0, "In Super Mario Bros., the lives counter is a single number."),
        Caption(5, "Past nine, the game runs out of digits and draws whatever tiles come next."),
        Caption(10, "We didn't earn these lives. We typed them in."),
    ], takes=("lives_glyphs",), stings=lives_stings),
    Shot("rain_begins", 8, rain_begins, [Caption(2, "Everything Mario is, is numbers like these.")], takes=("lives_glyphs",)),
])
```

`title_card(None, ...)` hides the number; make `draw.title_card` accept `None` for that.

### Task 16: Chapters 2 and 3 — the simplest brain, and pour Mario into Tetris

**Files:** Create `chapters/c02_simplest_brain.py`, `c03_pour.py`; modify `chapters/__init__.py`, `tests/test_chapters.py`.

**Interfaces:** Consumes takes `simple_brain_drives`, `paste_board`, `placements`, `echo_twins`; widgets `byte_grid`, `gauge`, `text_card`, `ripple`, `board`, `nes`.

Chapter 2, *The simplest brain*, subtitle "One weight per number", cue `lab`:

| Shot | Seconds | What is drawn | Captions | Stings |
|---|---|---|---|---|
| `weights` | 18 | the window grid with each cell coloured by its weight to A (`weights_a[1:]`, green positive, red negative, intensity by size); from 10 s, a sum bar fills to the threshold | 0: "The simplest brain gives every number a weight." 5: "Multiply each number by its weight and add them all up." 10: "If the total is over one half, press jump." | |
| `training` | 10 | `text_card` | 0: "Training means picking the weights that best copy a teacher's examples." 5: "Green weights push toward jumping. Red ones push away." | |
| `gauge` | 20 | `simple_brain_drives` stream `demo` at 2x left; right, `gauge` of `demo_steps[..k].a_out` against true A presses, scrolling | 0: "Here it watches a run it wasn't trained on. The line is how much it wants to jump." 9: "It rises near the jumps, but rarely crosses the line." 15: "A weighted sum can't say 'jump when the Goomba is close'." | |
| `drives` | 14 | stream `drive` at 2x centred, holding the last frame after it ends | 0: "Now let it play." 6: (callable) if `death_x` is set: "It dies at x {simple_brain_drives.death_x}, every time." else "It never gets anywhere." | `death` at the frame of the first dead step |

Chapter 3, *Pour Mario into Tetris*, subtitle "Reservoir computing", cue `korobeiniki`:

| Shot | Seconds | What is drawn | Captions | Stings |
|---|---|---|---|---|
| `ripple` | 20 | `ripple` with drops at 1, 4, 5, 9, 12 s at varied points; a stone glyph falls before each | 0: "Scientists have computed with buckets of water." 5: "Drop numbers in as stones. The ripples mix them up in complicated ways." 12: "A simple brain reads the ripples instead of the stones. That's reservoir computing." | |
| `why_tetris` | 10 | `text_card` | 0: "Tetris is a messy system that lives in 2,048 bytes too." 5: "What if Tetris were the bucket?" | |
| `paste` | 18 | `paste_board` `smb` at 1x top-left; arrow; the board from `steps[i // 4].board` at 2x right; the window bytes ≥ 0xC0 lit in the grid between | 0: "First try: paste Mario's numbers straight onto the Tetris board." 7: "Big numbers become blocks, and Tetris drops a piece onto them." 13: "Mostly the board fills with junk. Tetris has nothing to do." | |
| `screen_lies` | 12 | `paste_board` stream `tetris` at 2x left beside the RAM board at 2x right, labelled "the screen" and "the memory" | 0: "And the Tetris screen lies: it only redraws when a piece lands." 6: "The board in memory is the real one, so that's the one we draw." | |
| `placements` | 24 | `placements` `smb` at 1x; four labelled bars for `features` (nearest enemy, next enemy, height, speed); the board, with the four pieces' columns outlined as they land | 0: "Second try: pick four numbers about Mario." 5: "How far the nearest enemy is. The next one. Mario's height. His speed." 12: "Each number picks where a piece lands and which way it turns." 18: "" | |
| `echo` | 22 | `echo_twins` boards A and B side by side at 2x, differing cells outlined `RED`, a step counter, a sparkline of `diff` below | 0: "How long does Tetris remember? Feed two boards the same pieces but one." 8: "Red shows where they differ." 13: (callable) if `half_life` is set: "The difference halves in about {echo_twins.half_life} steps." else "The difference doesn't fade on its own; only a game over wipes it." | |

### Task 17: Chapters 4 and 5 — the coin-flip teacher, and three ways we fooled ourselves

**Files:** Create `chapters/c04_teacher.py`, `c05_fooled.py`; modify `chapters/__init__.py`, `tests/test_chapters.py`.

**Interfaces:** Consumes takes `coin_flip_teachers`, `rule_tripwires`, `controller_leak`, `clock_late_start`, `memoriser`; widgets `strip`, `lamp`, `text_card`, `nes`, `byte_grid`.

Chapter 4, *The teacher was flipping coins*, subtitle "Where the examples came from", cue `lab`:

| Shot | Seconds | What is drawn | Captions | Stings |
|---|---|---|---|---|
| `six` | 20 | `coin_flip_teachers` `demo0` at 1x top; below, `strip` with six lanes, one per demonstration, ticks at its A presses, a marker at demo 0's x | 0: "To learn, a brain needs examples. Ours came from a search that found ways through the level." 9: "Here are six. Each tick is a jump." 14: "At the same spot, two of them disagree {coin_flip_teachers.disagree:.0%} of the time." | |
| `copy` | 8 | `text_card` | 0: "Copy that, if you can. No brain could: the timing was a coin flip." | |
| `rule` | 26 | `rule_tripwires` `smb` at 2x left; right, three `lamp`s (enemy close, pit ahead, wall ahead) lit from `steps[i // 4].tripwires`, and a fourth "JUMP" lamp from `buttons & 1` | 0: "So we wrote a teacher that jumps for reasons." 6: "Jump when an enemy is close." 10: "Or a pit is ahead." 14: "Or a wall." 18: "It clears the level, the same way every time." | |
| `score` | 12 | a timeline: teacher's jump starts as gold ticks; a brain's presses as blocks; hits ringed green, misses ringed red (illustrative, drawn from `rule_tripwires` steps with a made-up brain that misses one in three: label it "an example brain") | 0: "The score that matters: when the teacher started a jump, did the brain start one too?" | |

Chapter 5, *Three ways we fooled ourselves*, subtitle "Lab notes", cue `fooled`:

| Shot | Seconds | What is drawn | Captions | Stings |
|---|---|---|---|---|
| `leak_card` | 6 | `text_card(["The brain could see its own hands."], kicker="Mistake 1")` | | |
| `leak_score` | 10 | a big number: `held_out.a_accuracy` as a percentage, with "jumps it got right on a run it never saw" | 0: "Our first brain got {controller_leak.held_out.a_accuracy:.0%} of the jumps right on a run it had never seen." 5: "Then we let it play." | |
| `leak_drive` | 14 | stream `drive` at 2x, holding its last frame; at 6 s, the window grid beside it with `$0A` to `$0D` highlighted `RED` | 0: "" 6: "It went nowhere. Four of its numbers were the controller: it was reading the button it had just pressed." | `wahwah` at 5 s |
| `leak_fix` | 6 | `text_card(["The fix: hide the controller from the brain."], kicker="Fixed")` | | |
| `clock_card` | 6 | `text_card(["A brain that only knows the time."], kicker="Mistake 2")` | | |
| `clock_on_time` | 14 | stream `on_time` at 2x, a clock face counting steps | 0: "This brain knows nothing but how long the level has been running." 7: "It still gets to x {clock_late_start.on_time.max_x}. It memorised when the teacher jumps." | |
| `clock_late` | 12 | stream `late` at 2x | 0: "Start Mario two seconds late, and it jumps at nothing." | `wahwah` at the frame of `late.death_x` if set, else at 8 s |
| `clock_fix` | 6 | `text_card(["The fix: start the examples at many different times."], kicker="Fixed")` | | |
| `memo_card` | 6 | `text_card(["The search that found a shortcut."], kicker="Mistake 3")` | | |
| `memo_score` | 12 | the four picked features listed, the `position` ones in `RED` with "Mario's position, in disguise" | 0: "We let a search pick the four best numbers. It scored {memoriser.held_out_onsets:.0%}." 6: (callable) if any feature is `position`: "Look at what it picked: Mario's position, in disguise." else "It picked numbers that change with where Mario is." | |
| `memo_drive` | 12 | stream `drive` at 2x | 0: (callable) if `death_x` is set: "It learned where the jumps are in this level, not why. It dies at x {memoriser.death_x}." else "It learned where the jumps are in this level, not why." | `wahwah` at the death frame if any |
| `memo_fix` | 8 | `text_card(["The fix: never tell the brain where Mario is.", "Make it read what's around him."], kicker="Fixed")` | | |

If Task 10 recorded a story that differs from these captions (see its Step 4), rewrite the affected captions to say what the take shows, and record the ruling.

### Task 18: Chapters 6 and 7 — four Tetris games, and a level it never saw

**Files:** Create `chapters/c06_bank.py`, `c07_unseen.py`; modify `chapters/__init__.py`, `tests/test_chapters.py`.

**Interfaces:** Consumes takes `bank_drive`, `bank_noise`, `unseen_4_1`; `numbers.json`; widgets `byte_grid`, `board`, `bar_chart`, `nes`, `text_card`.

Chapter 6, *Four Tetris games*, subtitle "A bank of boards", cue `korobeiniki_fast`:

| Shot | Seconds | What is drawn | Captions | Stings |
|---|---|---|---|---|
| `regions` | 18 | the window map with four regions tinted in `REGION` colours (Mario's bytes, the enemy slots, the floor row of the terrain ahead, the wall rows), an arrow from each to its board | 0: "One Tetris game wasn't enough, so we gave Mario four." 6: "One watches Mario. One the enemies. One the floor ahead. One the wall ahead." 13: "Each picks four pieces for its own board, every step." | |
| `tidy` | 12 | one board animating `best.steps` with the fifth (tidy) piece drawn in `GOLD` as it lands in the lowest column | 0: "A fifth piece tidies each board, filling its lowest gap, so the games last longer." | |
| `drive` | (callable) `min(48, len(bank_drive smb) / 60)` | `bank_drive` `smb` at 2x left in real time; four boards right; an x meter at the bottom of the panel with a mark at 1850 ("where the echo state died") | 0: "Now the four games drive." 8: "" 22: (callable) if `passed_goomba_pair`: "Past the two Goombas that stopped every brain before it." else "" | `fanfare` at the first frame where x passes 1850, if it does |
| `noise` | 16 | `bar_chart` of `bank_noise.levels` mean x by jitter, plus `jitter32` at 1x | 0: "Shake its numbers a little, and it still plays." 8: (callable) chosen from the data: if the largest jitter's mean x is under half the clean one, "Shake them hard, and it falls apart." else "Even shaken hard, it keeps going." | |
| `best` | 8 | `text_card` | 0: "Its best run on 1-1 reached x {numbers.bank_tidy_1_1.max_x}." | |

Chapter 7, *A level it never saw*, subtitle "World 4-1", cue `minuet`:

| Shot | Seconds | What is drawn | Captions | Stings |
|---|---|---|---|---|
| `why` | 12 | `text_card` | 0: "Doing well on a level you trained on proves little. You might have memorised it." 6: "So we trained on two levels and tested on a third: World 4-1." | |
| `teacher_4_1` | 14 | `unseen_4_1` `teacher` at 2x | 0: "The teacher plays 4-1. Every brain has to guess when it would jump." | |
| `bars` | 30 | `bar_chart` of `numbers.rows` by `test`, labels from `label`, one more bar every 2 s from the top; error bars (`numbers.test_se`) from 20 s | 0: "How often each brain started a jump when the teacher did, on the level it never saw:" 10: "" 20: "With only {numbers.test_onsets} jumps to go on, the top four are a tie." | |
| `clock_joke` | 12 | `unseen_4_1` `clock` at 2x | 0: "And the clock? It reached x {unseen_4_1.clock_max_x} by jumping at the wrong times. 4-1 is forgiving." | |

### Task 19: Chapters 8 and 9 — so what is Tetris doing, and the verdict

**Files:** Create `chapters/c08_what.py`, `c09_verdict.py`; modify `chapters/__init__.py`, `tests/test_chapters.py`.

**Interfaces:** Consumes takes `heights`, `hold_bit`, `echo_twins`; `numbers.json`; widgets `board`, `heights_bars`, `bar_chart`, `text_card`.

Chapter 8, *So what is Tetris doing?*, subtitle "Two theories", cue `minuet`:

| Shot | Seconds | What is drawn | Captions | Stings |
|---|---|---|---|---|
| `theory1` | 20 | `heights` boards with `heights_bars` beneath each; from 14 s, two bars: "the boards" (`bank-tidy` test) and "only the heights" (`bank-tidy-heights` test) | 0: "Theory one: Tetris remembers how much happened lately, in how tall the stacks are." 8: "Give a brain only the heights, and test it." 14: "It almost never jumps. The heights aren't it." | |
| `theory2` | 20 | `hold_bit` as two rows of blocks (A, and A last step); from 14 s, bars of onsets for `bank-searched-onehot`, `bank-searched-hold` and closed-loop mean x for those and `bank-searched-tetris` from `numbers.hold` | 0: "Theory two: Tetris just remembers 'I was jumping a moment ago'." 8: "Give a brain without Tetris that one bit of memory instead." 14: "It doesn't come close. Tetris isn't one bit." | |
| `answer` | 24 | `echo_twins` board A, with the cells that changed this step filled `GOLD` (`recent`) and older cells dim | 0: "What the brain really reads is where this step's pieces just landed." 8: "Tetris is a scrambler, not a memory." 15: "And the scramble is only as good as the four numbers we feed it." | |

Chapter 9, *Verdict*, subtitle "And what we'd do next", cue `reprise`:

| Shot | Seconds | What is drawn | Captions | Stings |
|---|---|---|---|---|
| `verdict` | 12 | `text_card(["Can Tetris play Mario? Sort of.", "Four games of Tetris learn to make Mario jump about as well as a 2,048-unit echo state network.", "But they don't remember anything."])` | | |
| `next` | 14 | `text_card(["With more hardware, we'd try:", "1. A task that needs memory.", "2. Hundreds of Tetris games at once.", "3. More levels to test on."])` | | |
| `credits` | 20 | credits rolling up: "Made with Greenstone, an NES emulator in Rust." / "The music is played on Greenstone's own sound chip." / "Writing it found a bug: one of the chip's two square-wave channels had been silent." / "Korobeiniki: traditional. Minuet in G: Christian Petzold." / "All other music original." / "Super Mario Bros. and Tetris belong to Nintendo. We only poked their memory." | | |

### Task 20: The whole film, checked

**Files:**
- Delete: `examples/reservoir_dump.rs`, `tools/reservoir_video.py`
- Modify: `src/nes/gym/reservoir/trace.rs` only if deleting the dump leaves code unused that nothing else needs (check with `cargo build --release --examples`; leave `replay` and `trace_episode`, which the takes use)
- Modify: `docs/superpowers/specs/2026-10-14-reservoir-video-design.md` (one line at the top: superseded by the 2026-10-17 film design)
- Modify: `docs/ideas/brain-cartridge.md` (the paragraph about the explainer video, near "A captioned explainer video")

**Interfaces:** Consumes everything above.

- [ ] **Step 1: Record every take fresh**

Run: `cargo run --release --example reservoir_film -- --force`
Expected: every take recorded; note the wall-clock time.

- [ ] **Step 2: Run every test**

Run: `RUSTFLAGS=-Awarnings cargo test` and `cd tools && python3 -m unittest discover -s reservoir_film/tests -t .`
Expected: all pass (the Rust suite was 416 passed before this plan, plus the new tests).

- [ ] **Step 3: Build the film**

Run: `cd tools && python3 -m reservoir_film.build`
Expected: `target/reservoir-film/tetris-reservoir.mp4` and `chapters.txt`.

- [ ] **Step 4: Check the result**

- `ffprobe -v error -show_chapters -show_entries format=duration target/reservoir-film/tetris-reservoir.mp4`: ten chapters whose starts match `chapters.txt`, duration within a second of the sum of chapter lengths.
- Loudness: `ffmpeg -i target/reservoir-film/tetris-reservoir.mp4 -af ebur128 -f null - 2>&1 | tail -12`: integrated loudness between -17 and -15 LUFS, true peak under -1 dBTP.
- A spectrogram of `segments/score.wav` (reuse `audition.spectrogram`): no silent gaps longer than a second except where a chapter ends.
- Contact sheets for all chapters (`--contact`), read in order, checking continuity across chapter joins.
- Ten sampled frames from the final MP4 (`ffmpeg -ss <t> -i ... -frames:v 1`) at chapter starts plus 5 s: captions legible after compression.

- [ ] **Step 5: Remove the old video and update the docs**

Delete `examples/reservoir_dump.rs` and `tools/reservoir_video.py`. In `docs/ideas/brain-cartridge.md`, replace the explainer-video paragraph with one that points at `examples/reservoir_film.rs`, `tools/reservoir_film/` and the 2026-10-17 spec, and keeps its two facts (only the first 160 window bytes fit the board; the Tetris screen lies). Mark the 2026-10-14 video spec superseded.

- [ ] **Step 6: Commit and hand over**

```bash
git add -A examples/reservoir_dump.rs tools/reservoir_video.py docs/superpowers/specs/2026-10-14-reservoir-video-design.md docs/ideas/brain-cartridge.md
# message: "Replace the old explainer with the reservoir film."
git commit -F <message file>
```

Then send the user the video (SendUserFile) and `chapters.txt`, with the running time, and ask them to watch it at normal speed: pacing and whether it is funny are theirs to judge.
