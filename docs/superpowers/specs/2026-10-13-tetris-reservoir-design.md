# Tetris reservoir: design

Date: 2026-10-13. Status: written autonomously after the user said
"proceed" on the in-chat design; open for review.

## Purpose

Test whether a running commercial NES game can serve as a reservoir
computer. Tetris is run in a second emulator instance, driven by the state
of Super Mario Bros., and a linear readout from Tetris's 2KB of RAM is
trained to reproduce a demonstration's button presses. The question is
whether that readout beats a linear readout on the SMB state alone.

This is item 1 in `docs/ideas/brain-cartridge.md`.

## Success criteria

- An example binary runs the whole experiment from two ROM files and
  prints one table: per condition, held-out button accuracy and
  closed-loop distance reached in 1-1.
- The experiment includes the baselines needed to tell whether Tetris
  contributes anything. A result where it does not is a success for the
  experiment and a failure for the joke, and the table must make that
  visible.
- Everything that does not need a ROM is unit tested. Nothing that needs a
  commercial ROM runs in `cargo test` unless the ROM is present.

## Measured constraints

From `examples/headless_bench.rs` on this machine, release build, one core:

```
frames/sec, SMB:        ~300   (5x realtime)
frames/sec, Tetris:     ~314
instructions/frame:     ~9800
save state size:        238 KB
save + load round trip: ~0.5 ms
cores:                  10
```

Two instances per step halve throughput to ~150 frames/sec. A 1-1 run is
roughly 2000 frames, so one closed-loop episode is about 13 seconds.
Anything that needs many episodes must run across threads. Emulator
instances share nothing, so this is `std::thread` with one emulator per
thread and no new dependencies.

## Architecture

New module `src/nes/gym/reservoir/` with four files and one example.

```
src/nes/gym/reservoir/
  mod.rs        public surface, condition enum, experiment config
  ridge.rs      ridge regression by normal equations and Cholesky, f64
  demo.rs       beam search over save states that produces a demonstration
  drive.rs      how SMB state enters Tetris, and Tetris lifecycle
  features.rs   feature vectors for each condition
examples/
  tetris_reservoir.rs   runs the experiment, prints the table
```

### Data flow

```
SMB emulator ----state window----> drive ----> Tetris emulator
     |                                              |
     | demonstration buttons (targets)              | RAM (features)
     v                                              v
  ridge.fit(features, targets) -> readout weights W
                                        |
  closed loop:  features(t) . W -> buttons(t) -> SMB.step_frames(buttons, 4)
```

### Macro step

Both emulators advance 4 frames per step. The demonstration is recorded at
the same granularity, so one row of the training set is one macro step.

### Demonstration (`demo.rs`)

Beam search over SMB save states. Actions at each macro step are the six
combinations `{Right, Right+A, Right+B, Right+A+B, A, none}`. Beam width
16. Score is x position, with death scoring negative infinity. The search
runs each macro step's expansions across threads, one emulator per thread,
loading the parent save state into the thread's emulator before stepping.
Output is the button byte per macro step along the best surviving path,
plus the x reached. It stops at a frame budget or at the level-clear flag.

This replaces a human recording. A human recording can be substituted
later by anything that produces the same `Vec<u8>` of per-step buttons.

### Drive (`drive.rs`)

Tetris is brought to gameplay by tapping Start until RAM `$00C0` reads 4,
then a save state is taken. Every episode reset reloads that snapshot. If
the game mode ever leaves 4 during a run (game over), the snapshot is
reloaded and the event is counted.

The SMB state window is 256 bytes: zero page `$0000..$00FF`, which holds
player position, speed, animation state, and the enemy slots. Two drives:

- `Controller`: fold the window to 8 bits by XOR over bytes, one bit per
  32-byte block, and hold it on Tetris's controller for the macro step.
- `Playfield`: copy the first 200 bytes of the window into Tetris's
  playfield at `$0400..$04C7` at the start of each macro step.

### Features (`features.rs`)

A feature row is `[1.0, body..., drive_bits...]` with bytes scaled to
`[0, 1]`. The body depends on the condition:

```
condition         | body                          | drive bits appended
------------------+-------------------------------+---------------------
Raw               | SMB window (256)              | no
Undriven          | Tetris RAM (2048), no drive   | no
DrivenController  | Tetris RAM (2048)             | yes (8)
DrivenPlayfield   | Tetris RAM (2048)             | no
```

`Raw` is the real baseline: a linear policy on the signal the reservoir
receives. `Undriven` is the control. `DrivenPlayfield` contains the window
verbatim inside the playfield, so it is a superset of `Raw` and the
comparison asks whether the rest of Tetris adds anything.

### Readout (`ridge.rs`)

Targets are the 8 button bits of the demonstration as 0/1. Fit
`W = (X^T X + lambda I)^-1 X^T Y` by accumulating the Gram matrix and
solving with Cholesky in f64. Dimension is at most 2057, so the solve is
under a few seconds. `lambda` is a config value, default 1.0. Prediction
is `x . W > 0.5` per button. Button accuracy is per-bit agreement on the
held-out rows.

Train/test split is by time: the first 80% of demonstration steps train,
the last 20% test.

### Closed loop

For each condition, run 5 episodes of SMB with the fitted readout from the
start of 1-1, with a random number of no-op frames in `[0, 30)` first,
until death, level clear, or 750 macro steps. Report the mean and max x
reached. Episodes run across threads.

### Output

One table, written to stdout, plus the demonstration's x reached and
length so the reader can tell how far the targets go.

```
condition         | held-out accuracy | mean x | max x | reservoir resets
```

## Error handling

- Missing ROM: the example prints which file is missing and exits 0.
- Tetris never reaches mode 4 within 600 frames: the example reports it
  and exits 1. This is a bug in the drive, not an experimental result.
- Singular Gram matrix: ridge with `lambda > 0` is always positive
  definite, so Cholesky cannot fail unless `lambda <= 0`, which is
  rejected with an error.

## Testing

Unit tests, no ROM needed:

- `ridge`: recovers the exact weights of a synthetic linear system with
  `lambda` near zero; shrinks weights toward zero as `lambda` grows;
  rejects `lambda <= 0`.
- `features`: row length is `1 + body + drive` for each condition; the
  first element is 1.0; bytes scale to `[0, 1]`.
- `drive`: the controller fold is deterministic and sensitive to every
  block; the playfield copy touches exactly 200 bytes.

Integration tests, marked `#[ignore]` and skipped when `roms2/` is absent:

- Tetris reaches mode 4 and the snapshot reloads to mode 4.
- The demonstration beam search reaches x > 500 in 1-1 within budget.

## Out of scope

- Ranking the whole ROM library as reservoirs. Follow-up once this works.
- Any training of the readout other than ridge regression.
- Pixel observations.
- Emulator throughput work, even though 300 frames/sec is poor.
