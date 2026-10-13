# Tetris Reservoir Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** An example binary that runs Tetris as a reservoir computer driven by Super Mario Bros., fits a ridge-regression readout to a beam-searched demonstration, and prints one table comparing it against the baselines.

**Architecture:** A new `src/nes/gym/reservoir/` module with four focused files (ridge solver, demonstration search, Tetris drive, feature rows) and an experiment runner in `mod.rs`. Emulator instances are not `Send` (the bus holds an `Rc`), so every thread builds its own instance from the ROM bytes and only save states cross thread boundaries. No new dependencies.

**Tech Stack:** Rust 2021, `std::thread::scope`, the existing `NesEmulator` in `src/nes/gym/mod.rs`, hand-written Cholesky in f64.

**Spec:** `docs/superpowers/specs/2026-10-13-tetris-reservoir-design.md`

## Global Constraints

- No new crate dependencies.
- Both emulators advance `MACRO_FRAMES = 4` frames per step everywhere: demonstration, dataset collection, closed loop.
- Commercial ROMs live in `roms2/` and are untracked. Tests that need them are `#[ignore]` and return early if the file is missing.
- Ridge `lambda` must be positive; default `1.0`.
- Train/test split is by time, first 80% train.
- The closed-loop readout must never press Start or Select (Start pauses SMB and the episode would stall until the step budget).
- The Tetris controller drive must never press Start or Select (Start pauses Tetris and kills the reservoir dynamics). This is a correction to the spec's "8 bits": the fold produces 6 live bits in the A, B, Up, Down, Left, Right positions, and the two others are always zero.
- Formatting follows `rustfmt.toml` (2-space indent). Run `cargo fmt` before each commit.
- Commit messages end with `Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e`.

## Review Focus

1. A demonstration shorter than 10 steps makes the test split empty and accuracy undefined. `split_index` must return an error below 10 rows. Pinned in Task 5.
2. The readout predicting Start or Select in closed loop. `outputs_to_buttons` must mask both bits. Pinned in Task 5.
3. A feature row whose length differs from what the readout was fitted on. `Readout::predict` must panic with a message naming both lengths rather than index silently. Pinned in Task 1.
4. Every beam candidate dead at some step. `prune_and_rank` returns empty and `beam_search` must return the best path so far, not panic. Pinned in Task 4.
5. The Tetris controller fold producing Start or Select. `fold_to_controller` must leave those bits clear for every input. Pinned in Task 3.

---

### Task 1: Ridge regression solver

**Files:**
- Create: `src/nes/gym/reservoir/mod.rs`
- Create: `src/nes/gym/reservoir/ridge.rs`
- Modify: `src/nes/gym/mod.rs` (add `pub mod reservoir;` after `pub mod reward;`)

**Interfaces:**
- Consumes: nothing.
- Produces: `pub fn fit(rows: &[Vec<f32>], targets: &[Vec<f32>], lambda: f64) -> Result<Readout, String>`; `pub struct Readout { pub d: usize, pub k: usize, pub weights: Vec<f64> }` with `pub fn predict(&self, row: &[f32]) -> Vec<f64>`.

- [ ] **Step 1: Register the module and write the failing tests**

Add to `src/nes/gym/mod.rs` next to the other `pub mod` lines:

```rust
pub mod reservoir;
```

Create `src/nes/gym/reservoir/mod.rs`:

```rust
//! Tetris as a reservoir computer, driven by Super Mario Bros.
//!
//! See `docs/superpowers/specs/2026-10-13-tetris-reservoir-design.md`.

pub mod ridge;

/// Read a ROM for an integration test, or print a skip notice and return None.
#[cfg(test)]
pub(crate) fn rom_or_skip(path: &str) -> Option<Vec<u8>> {
  match std::fs::read(path) {
    Ok(bytes) => Some(bytes),
    Err(_) => {
      eprintln!("skipping: {} is not present", path);
      None
    },
  }
}
```

Create `src/nes/gym/reservoir/ridge.rs` with only the tests:

```rust
//! Ridge regression by normal equations, solved with Cholesky in f64.

#[cfg(test)]
mod tests {
  use super::*;

  fn lcg(seed: &mut u64) -> f32 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((*seed >> 33) as f32 / (1u64 << 31) as f32) - 0.5
  }

  /// Rows with a leading 1.0, random true weights, exact targets.
  fn synthetic(n: usize, d: usize, k: usize) -> (Vec<Vec<f32>>, Vec<Vec<f32>>, Vec<f64>) {
    let mut seed = 7u64;
    let weights: Vec<f64> = (0..d * k).map(|_| lcg(&mut seed) as f64).collect();
    let mut rows = Vec::new();
    let mut targets = Vec::new();
    for _ in 0..n {
      let mut row = vec![1.0f32];
      row.extend((1..d).map(|_| lcg(&mut seed)));
      let target: Vec<f32> = (0..k)
        .map(|j| row.iter().enumerate().map(|(i, &x)| x as f64 * weights[i * k + j]).sum::<f64>() as f32)
        .collect();
      rows.push(row);
      targets.push(target);
    }
    (rows, targets, weights)
  }

  #[test]
  fn recovers_exact_weights_with_tiny_lambda() {
    let (rows, targets, truth) = synthetic(200, 5, 2);
    let readout = fit(&rows, &targets, 1e-9).unwrap();
    assert_eq!(readout.d, 5);
    assert_eq!(readout.k, 2);
    for (got, want) in readout.weights.iter().zip(&truth) {
      assert!((got - want).abs() < 1e-4, "got {} want {}", got, want);
    }
  }

  #[test]
  fn large_lambda_shrinks_weights_toward_zero() {
    let (rows, targets, _) = synthetic(200, 5, 2);
    let readout = fit(&rows, &targets, 1e6).unwrap();
    for w in &readout.weights {
      assert!(w.abs() < 1e-3, "weight {} not shrunk", w);
    }
  }

  #[test]
  fn rejects_nonpositive_lambda() {
    let (rows, targets, _) = synthetic(20, 3, 1);
    assert!(fit(&rows, &targets, 0.0).is_err());
    assert!(fit(&rows, &targets, -1.0).is_err());
  }

  #[test]
  fn rejects_mismatched_row_counts() {
    let (rows, targets, _) = synthetic(20, 3, 1);
    assert!(fit(&rows[..10], &targets, 1.0).is_err());
  }

  #[test]
  fn predict_reproduces_training_targets() {
    let (rows, targets, _) = synthetic(50, 4, 3);
    let readout = fit(&rows, &targets, 1e-9).unwrap();
    for (row, target) in rows.iter().zip(&targets) {
      let out = readout.predict(row);
      for (o, t) in out.iter().zip(target) {
        assert!((o - *t as f64).abs() < 1e-4);
      }
    }
  }

  #[test]
  #[should_panic(expected = "readout expects 4 features")]
  fn predict_rejects_wrong_row_length() {
    let (rows, targets, _) = synthetic(20, 4, 1);
    let readout = fit(&rows, &targets, 1.0).unwrap();
    readout.predict(&[1.0, 2.0]);
  }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test reservoir::ridge 2>&1 | tail -20`
Expected: compile error, `fit` and `Readout` not found.

- [ ] **Step 3: Implement the solver**

Insert above the `#[cfg(test)]` block in `ridge.rs`:

```rust
/// A fitted linear readout. `weights` is `d x k`, row-major: `weights[i * k + j]`
/// is the weight from feature `i` to output `j`.
#[derive(Debug, Clone)]
pub struct Readout {
  pub d: usize,
  pub k: usize,
  pub weights: Vec<f64>,
}

impl Readout {
  /// Outputs for one feature row.
  pub fn predict(&self, row: &[f32]) -> Vec<f64> {
    assert_eq!(row.len(), self.d, "readout expects {} features, got {}", self.d, row.len());
    let mut out = vec![0.0; self.k];
    for (i, &x) in row.iter().enumerate() {
      if x == 0.0 {
        continue;
      }
      let x = x as f64;
      let w = &self.weights[i * self.k..(i + 1) * self.k];
      for j in 0..self.k {
        out[j] += x * w[j];
      }
    }
    out
  }
}

/// Fit `W = (X^T X + lambda I)^-1 X^T Y`.
pub fn fit(rows: &[Vec<f32>], targets: &[Vec<f32>], lambda: f64) -> Result<Readout, String> {
  if lambda <= 0.0 {
    return Err(format!("lambda must be positive, got {}", lambda));
  }
  if rows.is_empty() {
    return Err("no rows to fit".to_string());
  }
  if rows.len() != targets.len() {
    return Err(format!("{} rows but {} targets", rows.len(), targets.len()));
  }
  let d = rows[0].len();
  let k = targets[0].len();
  let mut gram = vec![0.0f64; d * d];
  let mut xty = vec![0.0f64; d * k];
  for (row, target) in rows.iter().zip(targets) {
    if row.len() != d || target.len() != k {
      return Err(format!("ragged input: row {} target {}", row.len(), target.len()));
    }
    for i in 0..d {
      let xi = row[i] as f64;
      if xi == 0.0 {
        continue;
      }
      let g = &mut gram[i * d..(i + 1) * d];
      for j in i..d {
        g[j] += xi * row[j] as f64;
      }
      let b = &mut xty[i * k..(i + 1) * k];
      for j in 0..k {
        b[j] += xi * target[j] as f64;
      }
    }
  }
  for i in 0..d {
    gram[i * d + i] += lambda;
    for j in 0..i {
      gram[i * d + j] = gram[j * d + i];
    }
  }
  let lower = cholesky(&gram, d)?;
  let mut weights = vec![0.0; d * k];
  let mut rhs = vec![0.0; d];
  for j in 0..k {
    for i in 0..d {
      rhs[i] = xty[i * k + j];
    }
    let solution = cholesky_solve(&lower, d, &rhs);
    for i in 0..d {
      weights[i * k + j] = solution[i];
    }
  }
  Ok(Readout { d, k, weights })
}

/// Lower-triangular `L` with `a = L L^T`. `a` is `d x d` row-major and symmetric.
fn cholesky(a: &[f64], d: usize) -> Result<Vec<f64>, String> {
  let mut l = vec![0.0; d * d];
  for i in 0..d {
    for j in 0..=i {
      let mut sum = a[i * d + j];
      for p in 0..j {
        sum -= l[i * d + p] * l[j * d + p];
      }
      if i == j {
        if sum <= 0.0 {
          return Err(format!("matrix is not positive definite at row {}", i));
        }
        l[i * d + i] = sum.sqrt();
      } else {
        l[i * d + j] = sum / l[j * d + j];
      }
    }
  }
  Ok(l)
}

/// Solve `L L^T x = b` by forward then back substitution.
fn cholesky_solve(l: &[f64], d: usize, b: &[f64]) -> Vec<f64> {
  let mut y = vec![0.0; d];
  for i in 0..d {
    let mut sum = b[i];
    for p in 0..i {
      sum -= l[i * d + p] * y[p];
    }
    y[i] = sum / l[i * d + i];
  }
  let mut x = vec![0.0; d];
  for i in (0..d).rev() {
    let mut sum = y[i];
    for p in i + 1..d {
      sum -= l[p * d + i] * x[p];
    }
    x[i] = sum / l[i * d + i];
  }
  x
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test reservoir::ridge 2>&1 | tail -20`
Expected: 6 passed.

- [ ] **Step 5: Commit**

```bash
cargo fmt
git add src/nes/gym/mod.rs src/nes/gym/reservoir/
git commit -m "Add ridge regression solver for the reservoir readout.

Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e"
```

---

### Task 2: Feature rows and conditions

**Files:**
- Create: `src/nes/gym/reservoir/features.rs`
- Modify: `src/nes/gym/reservoir/mod.rs` (add `pub mod features;`)

**Interfaces:**
- Consumes: nothing from earlier tasks. Refers to `Drive` from Task 3; to keep this task compiling on its own, define `Drive` here in Task 2 is NOT done. Instead `Condition::drive` is added in Task 3. This task only produces the enum, names, lengths, and `feature_row`.
- Produces: `pub enum Condition { Raw, Undriven, DrivenController, DrivenPlayfield }` with `Condition::ALL`, `name()`, `uses_reservoir()`, `appends_drive_bits()`, `row_len()`; `pub fn feature_row(body: &[u8], drive_bits: Option<u8>) -> Vec<f32>`; constants `SMB_WINDOW_START`, `SMB_WINDOW_LEN`, `TETRIS_RAM_LEN`, `DRIVE_BITS`.

- [ ] **Step 1: Write the failing tests**

Create `src/nes/gym/reservoir/features.rs`:

```rust
//! Feature rows for each experimental condition.

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn row_starts_with_bias_and_scales_bytes() {
    let row = feature_row(&[0, 255, 51], None);
    assert_eq!(row.len(), 4);
    assert_eq!(row[0], 1.0);
    assert_eq!(row[1], 0.0);
    assert_eq!(row[2], 1.0);
    assert!((row[3] - 0.2).abs() < 1e-6);
  }

  #[test]
  fn drive_bits_are_appended_low_bit_first() {
    let row = feature_row(&[9], Some(0b1000_0001));
    assert_eq!(row.len(), 1 + 1 + DRIVE_BITS);
    assert_eq!(row[2], 1.0);
    assert_eq!(row[3], 0.0);
    assert_eq!(row[9], 1.0);
  }

  #[test]
  fn row_lengths_match_conditions() {
    assert_eq!(Condition::Raw.row_len(), 1 + 256);
    assert_eq!(Condition::Undriven.row_len(), 1 + 2048);
    assert_eq!(Condition::DrivenController.row_len(), 1 + 2048 + 8);
    assert_eq!(Condition::DrivenPlayfield.row_len(), 1 + 2048);
  }

  #[test]
  fn only_raw_skips_the_reservoir() {
    assert!(!Condition::Raw.uses_reservoir());
    for c in Condition::ALL.iter().filter(|c| **c != Condition::Raw) {
      assert!(c.uses_reservoir(), "{}", c.name());
    }
  }

  #[test]
  fn names_are_distinct() {
    let mut names: Vec<&str> = Condition::ALL.iter().map(|c| c.name()).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), 4);
  }
}
```

Add `pub mod features;` to `reservoir/mod.rs`.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test reservoir::features 2>&1 | tail -20`
Expected: compile error, `feature_row` and `Condition` not found.

- [ ] **Step 3: Implement**

Insert above the test block in `features.rs`:

```rust
/// Start of the SMB state window handed to the reservoir: zero page.
pub const SMB_WINDOW_START: u16 = 0x0000;
/// Length of the SMB state window.
pub const SMB_WINDOW_LEN: u16 = 256;
/// Tetris work RAM read as the reservoir state.
pub const TETRIS_RAM_LEN: u16 = 2048;
/// Number of drive bits appended for the controller-driven condition.
pub const DRIVE_BITS: usize = 8;

/// Which features the readout sees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
  /// The SMB window itself. The real baseline.
  Raw,
  /// Tetris RAM with no input. The control.
  Undriven,
  /// Tetris RAM, Tetris driven through its controller by a fold of the window.
  DrivenController,
  /// Tetris RAM, the window copied into Tetris's playfield.
  DrivenPlayfield,
}

impl Condition {
  pub const ALL: [Condition; 4] = [Condition::Raw, Condition::Undriven, Condition::DrivenController, Condition::DrivenPlayfield];

  pub fn name(self) -> &'static str {
    match self {
      Condition::Raw => "raw",
      Condition::Undriven => "undriven",
      Condition::DrivenController => "driven-controller",
      Condition::DrivenPlayfield => "driven-playfield",
    }
  }

  pub fn uses_reservoir(self) -> bool {
    !matches!(self, Condition::Raw)
  }

  pub fn appends_drive_bits(self) -> bool {
    matches!(self, Condition::DrivenController)
  }

  pub fn row_len(self) -> usize {
    let body = if self.uses_reservoir() { TETRIS_RAM_LEN } else { SMB_WINDOW_LEN } as usize;
    1 + body + if self.appends_drive_bits() { DRIVE_BITS } else { 0 }
  }
}

/// `[1.0, body / 255 ..., drive bit 0 .. drive bit 7]`.
pub fn feature_row(body: &[u8], drive_bits: Option<u8>) -> Vec<f32> {
  let mut row = Vec::with_capacity(1 + body.len() + DRIVE_BITS);
  row.push(1.0);
  row.extend(body.iter().map(|&b| b as f32 / 255.0));
  if let Some(bits) = drive_bits {
    for i in 0..DRIVE_BITS {
      row.push(((bits >> i) & 1) as f32);
    }
  }
  row
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test reservoir::features 2>&1 | tail -20`
Expected: 5 passed.

- [ ] **Step 5: Commit**

```bash
cargo fmt
git add src/nes/gym/reservoir/
git commit -m "Add feature rows and conditions for the reservoir experiment.

Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e"
```

---

### Task 3: Tetris drive and reservoir lifecycle

**Files:**
- Create: `src/nes/gym/reservoir/drive.rs`
- Modify: `src/nes/gym/reservoir/mod.rs` (add `pub mod drive;`)
- Modify: `src/nes/gym/reservoir/features.rs` (add `Condition::drive`)

**Interfaces:**
- Consumes: `NesEmulator` (`new`, `step_frame`, `step_frames`, `ram_read`, `ram_write`, `ram_read_range`, `save_state`, `load_state`); `TETRIS_RAM_LEN` from Task 2; `BUTTON_*` from `crate::nes::bus`.
- Produces: `pub enum Drive { Controller, Playfield }`; `pub fn fold_to_controller(window: &[u8]) -> u8`; `pub struct Reservoir { pub resets: u32, .. }` with `pub fn new(rom: &[u8]) -> Result<Reservoir, String>`, `pub fn reset(&mut self)`, `pub fn step(&mut self, drive: Option<Drive>, window: &[u8], frames: usize)`, `pub fn ram(&mut self) -> Vec<u8>`, `pub fn in_play(&mut self) -> bool`; `Condition::drive(self) -> Option<Drive>`.

- [ ] **Step 1: Write the failing tests**

Create `src/nes/gym/reservoir/drive.rs`:

```rust
//! How SMB state enters Tetris, and the Tetris lifecycle as a reservoir.

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::bus::{BUTTON_SELECT, BUTTON_START};
  use crate::nes::gym::reservoir::rom_or_skip;

  #[test]
  fn fold_is_deterministic() {
    let window: Vec<u8> = (0..=255).collect();
    assert_eq!(fold_to_controller(&window), fold_to_controller(&window));
  }

  #[test]
  fn fold_never_presses_start_or_select() {
    let mut seed = 1u64;
    for _ in 0..500 {
      let window: Vec<u8> = (0..256)
        .map(|_| {
          seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
          (seed >> 56) as u8
        })
        .collect();
      let bits = fold_to_controller(&window);
      assert_eq!(bits & (BUTTON_START | BUTTON_SELECT), 0, "window folded to {:08b}", bits);
    }
  }

  #[test]
  fn fold_is_sensitive_to_every_live_block() {
    let base: Vec<u8> = vec![0; 256];
    let base_bits = fold_to_controller(&base);
    for (block, button) in LIVE_BUTTONS.iter().enumerate() {
      let mut window = base.clone();
      window[block * 32 + 5] ^= 0x10;
      let bits = fold_to_controller(&window);
      assert_eq!(bits ^ base_bits, *button, "block {} should flip exactly its button", block);
    }
  }

  #[test]
  #[ignore]
  fn tetris_boots_into_play_and_snapshot_restores_it() {
    let rom = match rom_or_skip("roms2/tetris.nes") {
      Some(r) => r,
      None => return,
    };
    let mut reservoir = Reservoir::new(&rom).expect("tetris should reach play");
    assert!(reservoir.in_play());
    let window: Vec<u8> = (0..=255).collect();
    for _ in 0..50 {
      reservoir.step(Some(Drive::Controller), &window, 4);
    }
    assert!(reservoir.in_play());
    assert_eq!(reservoir.ram().len(), 2048);
    reservoir.reset();
    assert!(reservoir.in_play());
  }

  #[test]
  #[ignore]
  fn playfield_drive_touches_exactly_200_bytes() {
    let rom = match rom_or_skip("roms2/tetris.nes") {
      Some(r) => r,
      None => return,
    };
    let mut reservoir = Reservoir::new(&rom).expect("tetris should reach play");
    let window: Vec<u8> = vec![0xAB; 256];
    reservoir.write_playfield(&window);
    let field = reservoir.emu.ram_read_range(TETRIS_PLAYFIELD, TETRIS_PLAYFIELD_LEN);
    assert!(field.iter().all(|&b| b == 0xAB));
    assert_ne!(reservoir.emu.ram_read(TETRIS_PLAYFIELD + TETRIS_PLAYFIELD_LEN), 0xAB);
  }
}
```

Add `pub mod drive;` to `reservoir/mod.rs`. Add to `features.rs`, inside `impl Condition`:

```rust
  pub fn drive(self) -> Option<super::drive::Drive> {
    match self {
      Condition::DrivenController => Some(super::drive::Drive::Controller),
      Condition::DrivenPlayfield => Some(super::drive::Drive::Playfield),
      _ => None,
    }
  }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test reservoir::drive 2>&1 | tail -20`
Expected: compile error, `fold_to_controller`, `LIVE_BUTTONS`, `Reservoir` not found.

- [ ] **Step 3: Implement**

Insert above the test block in `drive.rs`:

```rust
use super::features::TETRIS_RAM_LEN;
use crate::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT, BUTTON_START, BUTTON_UP};
use crate::nes::gym::NesEmulator;

/// Tetris RAM: current game mode. 4 is gameplay.
pub const TETRIS_GAME_MODE: u16 = 0x00C0;
pub const TETRIS_MODE_PLAY: u8 = 0x04;
/// Tetris RAM: falling piece Y, used as a liveness check.
pub const TETRIS_PIECE_Y: u16 = 0x0041;
/// Tetris RAM: the 10x20 playfield.
pub const TETRIS_PLAYFIELD: u16 = 0x0400;
pub const TETRIS_PLAYFIELD_LEN: u16 = 200;
const BOOT_FRAME_LIMIT: usize = 600;

/// Buttons the controller drive may press, one per 32-byte block of the window.
/// Start and Select are excluded because Start pauses Tetris.
pub const LIVE_BUTTONS: [u8; 6] = [BUTTON_A, BUTTON_B, BUTTON_UP, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT];

/// How the SMB window enters Tetris.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Drive {
  /// Fold the window to button bits and hold them on Tetris's controller.
  Controller,
  /// Copy the window into Tetris's playfield.
  Playfield,
}

/// Fold a window to controller bits: block `i` of 32 bytes sets `LIVE_BUTTONS[i]`
/// when the parity of all its bits is odd. Blocks beyond the sixth are ignored.
pub fn fold_to_controller(window: &[u8]) -> u8 {
  let mut out = 0u8;
  for (chunk, button) in window.chunks(32).zip(LIVE_BUTTONS.iter()) {
    let xor = chunk.iter().fold(0u8, |acc, &b| acc ^ b);
    if xor.count_ones() % 2 == 1 {
      out |= button;
    }
  }
  out
}

/// A Tetris instance held in gameplay, with a snapshot to return to.
pub struct Reservoir {
  pub(crate) emu: NesEmulator,
  snapshot: Vec<u8>,
  /// How many times the game left play mode and was restored from the snapshot.
  pub resets: u32,
}

impl Reservoir {
  /// Boot Tetris, tap Start until gameplay, confirm a piece is falling, snapshot.
  pub fn new(rom: &[u8]) -> Result<Reservoir, String> {
    let mut emu = NesEmulator::new(rom);
    let mut reached = false;
    for frame in 0..BOOT_FRAME_LIMIT {
      let buttons = if frame % 30 < 4 { BUTTON_START } else { 0 };
      emu.step_frame(buttons);
      if emu.ram_read(TETRIS_GAME_MODE) == TETRIS_MODE_PLAY {
        reached = true;
        break;
      }
    }
    if !reached {
      return Err(format!("Tetris did not reach play mode within {} frames", BOOT_FRAME_LIMIT));
    }
    // A piece should fall during 120 idle frames. If not, the game is paused: tap Start once.
    if !piece_moves(&mut emu, 120) {
      emu.step_frames(BUTTON_START, 4);
      emu.step_frames(0, 4);
      if !piece_moves(&mut emu, 120) {
        return Err("Tetris reached play mode but no piece is falling".to_string());
      }
    }
    let snapshot = emu.save_state();
    Ok(Reservoir { emu, snapshot, resets: 0 })
  }

  pub fn in_play(&mut self) -> bool {
    self.emu.ram_read(TETRIS_GAME_MODE) == TETRIS_MODE_PLAY
  }

  pub fn reset(&mut self) {
    self.emu.load_state(&self.snapshot);
  }

  pub(crate) fn write_playfield(&mut self, window: &[u8]) {
    for (i, &b) in window.iter().take(TETRIS_PLAYFIELD_LEN as usize).enumerate() {
      self.emu.ram_write(TETRIS_PLAYFIELD + i as u16, b);
    }
  }

  /// Apply the drive, advance `frames`, and restore the snapshot if the game ended.
  pub fn step(&mut self, drive: Option<Drive>, window: &[u8], frames: usize) {
    let buttons = match drive {
      Some(Drive::Controller) => fold_to_controller(window),
      Some(Drive::Playfield) => {
        self.write_playfield(window);
        0
      },
      None => 0,
    };
    self.emu.step_frames(buttons, frames);
    if !self.in_play() {
      self.reset();
      self.resets += 1;
    }
  }

  /// The whole of Tetris's work RAM.
  pub fn ram(&mut self) -> Vec<u8> {
    self.emu.ram_read_range(0, TETRIS_RAM_LEN)
  }
}

fn piece_moves(emu: &mut NesEmulator, frames: usize) -> bool {
  let before = emu.ram_read(TETRIS_PIECE_Y);
  emu.step_frames(0, frames);
  emu.ram_read(TETRIS_PIECE_Y) != before
}
```

- [ ] **Step 4: Run the unit tests, then the ignored integration tests**

Run: `cargo test reservoir::drive 2>&1 | tail -20`
Expected: 3 passed, 2 ignored.

Run: `cargo test --release reservoir::drive -- --ignored 2>&1 | tail -20`
Expected: 2 passed. If `tetris_boots_into_play` fails on the liveness check, print `TETRIS_PIECE_Y` over 200 frames after mode 4 to confirm the address, and adjust the constant.

- [ ] **Step 5: Commit**

```bash
cargo fmt
git add src/nes/gym/reservoir/
git commit -m "Add Tetris drive and reservoir lifecycle.

Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e"
```

---

### Task 4: Demonstration by beam search

**Files:**
- Create: `src/nes/gym/reservoir/demo.rs`
- Modify: `src/nes/gym/reservoir/mod.rs` (add `pub mod demo;`)
- Possibly modify: `src/nes/gym/reward.rs` (make `smb1_ram` a `pub mod` if it is not already)

**Interfaces:**
- Consumes: `NesEmulator`; `smb1_ram::{PLAYER_X_PAGE, PLAYER_X_PIXEL, GAME_ENGINE_STATE, PLAYER_FLOAT_STATE, LIVES, LEVEL, WORLD}` from `reward.rs`; `SMB_WINDOW_START`, `SMB_WINDOW_LEN` from Task 2.
- Produces: `pub const MACRO_FRAMES: usize = 4`; `pub const ACTIONS: [u8; 6]`; `pub fn smb_x`, `smb_dead`, `smb_cleared`, `smb_window(&mut NesEmulator)`; `pub fn smb_start_state(rom: &[u8]) -> Vec<u8>`; `pub struct Demonstration { pub buttons: Vec<u8>, pub x_reached: i32, pub cleared: bool }`; `pub struct Candidate`; `pub fn prune_and_rank(cands: Vec<Candidate>, beam: usize) -> Vec<Candidate>`; `pub fn beam_search(rom: &[u8], start: &[u8], beam: usize, max_steps: usize) -> Demonstration`.

- [ ] **Step 1: Check `smb1_ram` visibility**

Run: `grep -n "mod smb1_ram" src/nes/gym/reward.rs`
If it prints `mod smb1_ram {` without `pub`, change it to `pub mod smb1_ram {`.

- [ ] **Step 2: Write the failing tests**

Create `src/nes/gym/reservoir/demo.rs`:

```rust
//! A demonstration for SMB 1-1 found by beam search over save states.

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::reservoir::rom_or_skip;

  fn cand(x: i32, dead: bool) -> Candidate {
    Candidate { state: vec![], path: vec![], x, dead, cleared: false }
  }

  #[test]
  fn prune_drops_dead_and_keeps_best_by_x() {
    let cands = vec![cand(10, false), cand(50, true), cand(30, false), cand(20, false)];
    let kept = prune_and_rank(cands, 2);
    let xs: Vec<i32> = kept.iter().map(|c| c.x).collect();
    assert_eq!(xs, vec![30, 20]);
  }

  #[test]
  fn prune_returns_empty_when_everything_is_dead() {
    let cands = vec![cand(10, true), cand(50, true)];
    assert!(prune_and_rank(cands, 4).is_empty());
  }

  #[test]
  fn actions_never_include_start_or_select() {
    use crate::nes::bus::{BUTTON_SELECT, BUTTON_START};
    for a in ACTIONS {
      assert_eq!(a & (BUTTON_START | BUTTON_SELECT), 0);
    }
  }

  #[test]
  #[ignore]
  fn start_state_is_at_the_beginning_of_1_1() {
    let rom = match rom_or_skip("roms2/super_mario_bros.nes") {
      Some(r) => r,
      None => return,
    };
    let start = smb_start_state(&rom);
    let mut emu = NesEmulator::new(&rom);
    emu.load_state(&start);
    assert_eq!(smb_x(&mut emu), 0);
    assert!(!smb_dead(&mut emu));
    assert!(!smb_cleared(&mut emu));
    emu.step_frames(crate::nes::bus::BUTTON_RIGHT, 60);
    assert!(smb_x(&mut emu) > 20, "Mario should move right from the start state");
  }

  #[test]
  #[ignore]
  fn beam_search_makes_progress_in_1_1() {
    let rom = match rom_or_skip("roms2/super_mario_bros.nes") {
      Some(r) => r,
      None => return,
    };
    let start = smb_start_state(&rom);
    let demo = beam_search(&rom, &start, 8, 150);
    assert_eq!(demo.buttons.len() <= 150, true);
    assert!(demo.x_reached > 500, "reached only x={}", demo.x_reached);
  }
}
```

Add `pub mod demo;` to `reservoir/mod.rs`.

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test reservoir::demo 2>&1 | tail -20`
Expected: compile error, `Candidate`, `prune_and_rank`, `ACTIONS` not found.

- [ ] **Step 4: Implement**

Insert above the test block in `demo.rs`:

```rust
use super::features::{SMB_WINDOW_LEN, SMB_WINDOW_START};
use crate::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_RIGHT, BUTTON_START};
use crate::nes::gym::reward::smb1_ram;
use crate::nes::gym::NesEmulator;

/// Frames per step, everywhere in the experiment.
pub const MACRO_FRAMES: usize = 4;

/// Actions the demonstration may take.
pub const ACTIONS: [u8; 6] = [
  BUTTON_RIGHT,
  BUTTON_RIGHT | BUTTON_A,
  BUTTON_RIGHT | BUTTON_B,
  BUTTON_RIGHT | BUTTON_A | BUTTON_B,
  BUTTON_A,
  0,
];

pub fn smb_x(emu: &mut NesEmulator) -> i32 {
  emu.ram_read(smb1_ram::PLAYER_X_PAGE) as i32 * 256 + emu.ram_read(smb1_ram::PLAYER_X_PIXEL) as i32
}

pub fn smb_dead(emu: &mut NesEmulator) -> bool {
  emu.ram_read(smb1_ram::GAME_ENGINE_STATE) == 0x0B || emu.ram_read(smb1_ram::PLAYER_FLOAT_STATE) == 0x03
}

/// True once the level counter has advanced past 1-1.
pub fn smb_cleared(emu: &mut NesEmulator) -> bool {
  emu.ram_read(smb1_ram::LEVEL) != 0 || emu.ram_read(smb1_ram::WORLD) != 0
}

pub fn smb_window(emu: &mut NesEmulator) -> Vec<u8> {
  emu.ram_read_range(SMB_WINDOW_START, SMB_WINDOW_LEN)
}

/// Boot SMB, tap Start through the title, wait for 1-1 to begin, and snapshot.
pub fn smb_start_state(rom: &[u8]) -> Vec<u8> {
  let mut emu = NesEmulator::new(rom);
  for frame in 0..120 {
    emu.step_frame(if frame % 20 < 10 { BUTTON_START } else { 0 });
  }
  emu.step_frames(0, 80);
  emu.save_state()
}

/// The best path the search found.
#[derive(Debug, Clone)]
pub struct Demonstration {
  /// One button byte per macro step.
  pub buttons: Vec<u8>,
  pub x_reached: i32,
  pub cleared: bool,
}

#[derive(Debug, Clone)]
pub struct Candidate {
  pub state: Vec<u8>,
  pub path: Vec<u8>,
  pub x: i32,
  pub dead: bool,
  pub cleared: bool,
}

/// Drop dead candidates, order by x descending, keep `beam`.
pub fn prune_and_rank(mut cands: Vec<Candidate>, beam: usize) -> Vec<Candidate> {
  cands.retain(|c| !c.dead);
  cands.sort_by(|a, b| b.x.cmp(&a.x));
  cands.truncate(beam);
  cands
}

/// Beam search over save states. Each step expands every frontier candidate by
/// every action across threads, one emulator per thread.
pub fn beam_search(rom: &[u8], start: &[u8], beam: usize, max_steps: usize) -> Demonstration {
  let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
  let lives_at_start = {
    let mut emu = NesEmulator::new(rom);
    emu.load_state(start);
    emu.ram_read(smb1_ram::LIVES)
  };
  let mut frontier = vec![Candidate { state: start.to_vec(), path: Vec::new(), x: 0, dead: false, cleared: false }];
  let mut best = frontier[0].clone();
  for _ in 0..max_steps {
    let jobs: Vec<(usize, u8)> = (0..frontier.len()).flat_map(|p| ACTIONS.iter().map(move |&a| (p, a))).collect();
    let chunk = (jobs.len() + threads - 1) / threads;
    let expanded: Vec<Candidate> = std::thread::scope(|s| {
      let handles: Vec<_> = jobs
        .chunks(chunk)
        .map(|jobs| {
          let frontier = &frontier;
          s.spawn(move || {
            let mut emu = NesEmulator::new(rom);
            jobs
              .iter()
              .map(|&(parent, action)| {
                emu.load_state(&frontier[parent].state);
                emu.step_frames(action, MACRO_FRAMES);
                let mut path = frontier[parent].path.clone();
                path.push(action);
                let dead = smb_dead(&mut emu) || emu.ram_read(smb1_ram::LIVES) < lives_at_start;
                Candidate { x: smb_x(&mut emu), dead, cleared: smb_cleared(&mut emu), state: emu.save_state(), path }
              })
              .collect::<Vec<_>>()
          })
        })
        .collect();
      handles.into_iter().flat_map(|h| h.join().expect("beam worker panicked")).collect()
    });
    let survivors = prune_and_rank(expanded, beam);
    if survivors.is_empty() {
      break;
    }
    if survivors[0].cleared || survivors[0].x > best.x {
      best = survivors[0].clone();
    }
    if best.cleared {
      break;
    }
    frontier = survivors;
  }
  Demonstration { buttons: best.path, x_reached: best.x, cleared: best.cleared }
}
```

- [ ] **Step 5: Run the unit tests, then the ignored integration tests**

Run: `cargo test reservoir::demo 2>&1 | tail -20`
Expected: 3 passed, 2 ignored.

Run: `cargo test --release reservoir::demo -- --ignored 2>&1 | tail -20`
Expected: 2 passed. The beam test takes under a minute on 10 cores. If `x_reached` stalls below 500, print the x of `survivors[0]` each step to see where it sticks; the first pipe is near x=450 and needs `Right+A` held across several consecutive steps, which beam width 8 should keep alive.

- [ ] **Step 6: Commit**

```bash
cargo fmt
git add src/nes/gym/reservoir/ src/nes/gym/reward.rs
git commit -m "Add beam-search demonstration for SMB 1-1.

Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e"
```

---

### Task 5: Experiment runner and example binary

**Files:**
- Modify: `src/nes/gym/reservoir/mod.rs` (runner, config, report, helpers)
- Create: `examples/tetris_reservoir.rs`

**Interfaces:**
- Consumes: everything above.
- Produces: `pub struct ExperimentConfig`, `pub struct ConditionResult`, `pub struct ExperimentReport`, `pub fn run(smb_rom: &[u8], tetris_rom: &[u8], config: &ExperimentConfig, log: &mut dyn FnMut(&str)) -> Result<ExperimentReport, String>`, plus pure helpers `split_index`, `buttons_to_targets`, `outputs_to_buttons`, `active_bits`, `bit_accuracy`.

- [ ] **Step 1: Write the failing tests**

Append to `src/nes/gym/reservoir/mod.rs`:

```rust
#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_RIGHT, BUTTON_SELECT, BUTTON_START};

  #[test]
  fn split_rejects_fewer_than_ten_rows() {
    assert!(split_index(9, 0.8).is_err());
    assert_eq!(split_index(10, 0.8).unwrap(), 8);
    assert_eq!(split_index(100, 0.8).unwrap(), 80);
  }

  #[test]
  fn targets_and_buttons_round_trip() {
    let b = BUTTON_A | BUTTON_RIGHT;
    let t = buttons_to_targets(b);
    assert_eq!(t.len(), 8);
    let out: Vec<f64> = t.iter().map(|&v| v as f64).collect();
    assert_eq!(outputs_to_buttons(&out), b);
  }

  #[test]
  fn outputs_never_press_start_or_select() {
    let out = vec![1.0; 8];
    let b = outputs_to_buttons(&out);
    assert_eq!(b & (BUTTON_START | BUTTON_SELECT), 0);
    assert_eq!(b & BUTTON_A, BUTTON_A);
  }

  #[test]
  fn active_bits_is_the_union() {
    assert_eq!(active_bits(&[BUTTON_A, BUTTON_RIGHT, 0]), BUTTON_A | BUTTON_RIGHT);
  }

  #[test]
  fn bit_accuracy_counts_only_masked_bits() {
    let truth = [BUTTON_A | BUTTON_RIGHT, BUTTON_B];
    let pred = [BUTTON_A, BUTTON_B | BUTTON_RIGHT];
    let mask = BUTTON_A | BUTTON_B | BUTTON_RIGHT;
    // Row 0: A right, B right, Right wrong. Row 1: A right, B right, Right wrong. 4 of 6.
    assert!((bit_accuracy(&pred, &truth, mask) - 4.0 / 6.0).abs() < 1e-9);
    assert_eq!(bit_accuracy(&pred, &truth, BUTTON_A), 1.0);
  }

  #[test]
  fn noop_frames_are_deterministic_and_bounded() {
    for e in 0..100 {
      let n = noop_frames(e, 30);
      assert!(n < 30);
      assert_eq!(n, noop_frames(e, 30));
    }
    assert_eq!(noop_frames(5, 0), 0);
  }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test reservoir::tests 2>&1 | tail -20`
Expected: compile error, helpers not found.

- [ ] **Step 3: Implement the runner**

Replace the top of `reservoir/mod.rs` (keep `rom_or_skip` and the test block) with:

```rust
//! Tetris as a reservoir computer, driven by Super Mario Bros.
//!
//! See `docs/superpowers/specs/2026-10-13-tetris-reservoir-design.md`.

pub mod demo;
pub mod drive;
pub mod features;
pub mod ridge;

use crate::nes::bus::{BUTTON_SELECT, BUTTON_START};
use crate::nes::gym::NesEmulator;
use demo::{beam_search, smb_cleared, smb_dead, smb_start_state, smb_window, smb_x, Demonstration, MACRO_FRAMES};
use drive::{fold_to_controller, Reservoir};
use features::{feature_row, Condition};
use ridge::{fit, Readout};

pub struct ExperimentConfig {
  pub lambda: f64,
  pub beam: usize,
  pub demo_max_steps: usize,
  pub episodes: usize,
  pub max_steps: usize,
  pub noop_max: usize,
  pub train_fraction: f64,
}

impl Default for ExperimentConfig {
  fn default() -> Self {
    ExperimentConfig { lambda: 1.0, beam: 16, demo_max_steps: 600, episodes: 5, max_steps: 750, noop_max: 30, train_fraction: 0.8 }
  }
}

#[derive(Debug, Clone)]
pub struct ConditionResult {
  pub condition: Condition,
  pub rows: usize,
  pub accuracy: f64,
  pub mean_x: f64,
  pub max_x: i32,
  pub resets: u32,
}

#[derive(Debug, Clone)]
pub struct ExperimentReport {
  pub demo_steps: usize,
  pub demo_x: i32,
  pub demo_cleared: bool,
  pub active_bits: u8,
  pub results: Vec<ConditionResult>,
}

/// First index of the test split. Needs at least ten rows so the test split is non-empty.
pub fn split_index(n: usize, train_fraction: f64) -> Result<usize, String> {
  if n < 10 {
    return Err(format!("demonstration has only {} steps; need at least 10", n));
  }
  Ok(((n as f64) * train_fraction).floor() as usize)
}

pub fn buttons_to_targets(buttons: u8) -> Vec<f32> {
  (0..8).map(|i| ((buttons >> i) & 1) as f32).collect()
}

/// Threshold each output at 0.5. Start and Select are never pressed.
pub fn outputs_to_buttons(outputs: &[f64]) -> u8 {
  let mut buttons = 0u8;
  for (i, &o) in outputs.iter().enumerate().take(8) {
    if o > 0.5 {
      buttons |= 1 << i;
    }
  }
  buttons & !(BUTTON_START | BUTTON_SELECT)
}

pub fn active_bits(buttons: &[u8]) -> u8 {
  buttons.iter().fold(0, |acc, &b| acc | b)
}

/// Fraction of masked bits on which prediction and truth agree.
pub fn bit_accuracy(pred: &[u8], truth: &[u8], mask: u8) -> f64 {
  let per_row = mask.count_ones() as usize;
  if per_row == 0 || pred.is_empty() {
    return 0.0;
  }
  let correct: usize = pred.iter().zip(truth).map(|(&p, &t)| (!(p ^ t) & mask).count_ones() as usize).sum();
  correct as f64 / (per_row * pred.len()) as f64
}

/// Deterministic per-episode no-op count in `[0, noop_max)`.
pub fn noop_frames(episode: usize, noop_max: usize) -> usize {
  if noop_max == 0 {
    return 0;
  }
  let h = (episode as u64 + 1).wrapping_mul(2654435761).wrapping_mul(0x9E3779B97F4A7C15);
  ((h >> 33) as usize) % noop_max
}

struct Dataset {
  rows: Vec<Vec<f32>>,
  targets: Vec<Vec<f32>>,
  buttons: Vec<u8>,
  resets: u32,
}

/// Step the reservoir with the current window (if the condition has one) and build the row.
fn observe(condition: Condition, window: &[u8], reservoir: Option<&mut Reservoir>) -> Vec<f32> {
  match (condition, reservoir) {
    (Condition::Raw, _) => feature_row(window, None),
    (c, Some(r)) => {
      r.step(c.drive(), window, MACRO_FRAMES);
      let bits = if c.appends_drive_bits() { Some(fold_to_controller(window)) } else { None };
      feature_row(&r.ram(), bits)
    },
    (c, None) => panic!("condition {} needs a reservoir", c.name()),
  }
}

fn collect(condition: Condition, smb_rom: &[u8], tetris_rom: &[u8], start: &[u8], demo: &Demonstration) -> Result<Dataset, String> {
  let mut smb = NesEmulator::new(smb_rom);
  smb.load_state(start);
  let mut reservoir = if condition.uses_reservoir() { Some(Reservoir::new(tetris_rom)?) } else { None };
  let mut data = Dataset { rows: Vec::new(), targets: Vec::new(), buttons: Vec::new(), resets: 0 };
  for &action in &demo.buttons {
    let window = smb_window(&mut smb);
    data.rows.push(observe(condition, &window, reservoir.as_mut()));
    data.targets.push(buttons_to_targets(action));
    data.buttons.push(action);
    smb.step_frames(action, MACRO_FRAMES);
  }
  data.resets = reservoir.map(|r| r.resets).unwrap_or(0);
  Ok(data)
}

/// One closed-loop episode. Returns (max x reached, reservoir resets).
fn episode(
  condition: Condition,
  readout: &Readout,
  smb_rom: &[u8],
  tetris_rom: &[u8],
  start: &[u8],
  noops: usize,
  max_steps: usize,
) -> Result<(i32, u32), String> {
  let mut smb = NesEmulator::new(smb_rom);
  smb.load_state(start);
  smb.step_frames(0, noops);
  let mut reservoir = if condition.uses_reservoir() { Some(Reservoir::new(tetris_rom)?) } else { None };
  let mut max_x = 0;
  for _ in 0..max_steps {
    let window = smb_window(&mut smb);
    let row = observe(condition, &window, reservoir.as_mut());
    let buttons = outputs_to_buttons(&readout.predict(&row));
    smb.step_frames(buttons, MACRO_FRAMES);
    max_x = max_x.max(smb_x(&mut smb));
    if smb_dead(&mut smb) || smb_cleared(&mut smb) {
      break;
    }
  }
  Ok((max_x, reservoir.map(|r| r.resets).unwrap_or(0)))
}

/// Run every condition and return the report. `log` receives progress lines.
pub fn run(smb_rom: &[u8], tetris_rom: &[u8], config: &ExperimentConfig, log: &mut dyn FnMut(&str)) -> Result<ExperimentReport, String> {
  let start = smb_start_state(smb_rom);
  log("searching for a demonstration");
  let demo = beam_search(smb_rom, &start, config.beam, config.demo_max_steps);
  log(&format!("demonstration: {} steps, x={}, cleared={}", demo.buttons.len(), demo.x_reached, demo.cleared));
  let split = split_index(demo.buttons.len(), config.train_fraction)?;
  let mask = active_bits(&demo.buttons);
  let mut results = Vec::new();
  for condition in Condition::ALL {
    log(&format!("{}: collecting features", condition.name()));
    let data = collect(condition, smb_rom, tetris_rom, &start, &demo)?;
    log(&format!("{}: fitting readout on {} rows of {}", condition.name(), split, data.rows[0].len()));
    let readout = fit(&data.rows[..split], &data.targets[..split], config.lambda)?;
    let preds: Vec<u8> = data.rows[split..].iter().map(|r| outputs_to_buttons(&readout.predict(r))).collect();
    let accuracy = bit_accuracy(&preds, &data.buttons[split..], mask);
    log(&format!("{}: running {} episodes", condition.name(), config.episodes));
    let outcomes: Vec<(i32, u32)> = std::thread::scope(|s| {
      let handles: Vec<_> = (0..config.episodes)
        .map(|e| {
          let readout = &readout;
          let start = &start;
          let noops = noop_frames(e, config.noop_max);
          let max_steps = config.max_steps;
          s.spawn(move || episode(condition, readout, smb_rom, tetris_rom, start, noops, max_steps))
        })
        .collect();
      handles.into_iter().map(|h| h.join().expect("episode worker panicked")).collect::<Result<Vec<_>, _>>()
    })?;
    let mean_x = outcomes.iter().map(|o| o.0 as f64).sum::<f64>() / outcomes.len().max(1) as f64;
    let max_x = outcomes.iter().map(|o| o.0).max().unwrap_or(0);
    let resets = data.resets + outcomes.iter().map(|o| o.1).sum::<u32>();
    results.push(ConditionResult { condition, rows: data.rows.len(), accuracy, mean_x, max_x, resets });
  }
  Ok(ExperimentReport { demo_steps: demo.buttons.len(), demo_x: demo.x_reached, demo_cleared: demo.cleared, active_bits: mask, results })
}
```

Keep the existing `rom_or_skip` and the test block below this.

- [ ] **Step 4: Run the unit tests**

Run: `cargo test reservoir 2>&1 | tail -20`
Expected: all unit tests pass; the ignored ones stay ignored.

- [ ] **Step 5: Write the example binary**

Create `examples/tetris_reservoir.rs`:

```rust
//! Tetris as a reservoir computer for Super Mario Bros.
//!
//! Usage: cargo run --release --example tetris_reservoir -- [smb.nes] [tetris.nes]

use greenstone::nes::gym::reservoir::{run, ExperimentConfig};
use std::path::Path;

fn main() {
  let args: Vec<String> = std::env::args().collect();
  let smb_path = args.get(1).map(String::as_str).unwrap_or("roms2/super_mario_bros.nes");
  let tetris_path = args.get(2).map(String::as_str).unwrap_or("roms2/tetris.nes");
  for path in [smb_path, tetris_path] {
    if !Path::new(path).exists() {
      println!("missing ROM: {}", path);
      return;
    }
  }
  let smb_rom = std::fs::read(smb_path).expect("read smb");
  let tetris_rom = std::fs::read(tetris_path).expect("read tetris");
  let config = ExperimentConfig::default();
  let report = match run(&smb_rom, &tetris_rom, &config, &mut |line| eprintln!("{}", line)) {
    Ok(r) => r,
    Err(e) => {
      eprintln!("error: {}", e);
      std::process::exit(1);
    },
  };

  println!();
  println!("demonstration: {} steps, x reached {}, cleared {}", report.demo_steps, report.demo_x, report.demo_cleared);
  println!("accuracy is over the buttons the demonstration uses: {:08b}", report.active_bits);
  println!();
  println!("{:<18} | {:>5} | {:>17} | {:>7} | {:>5} | {:>16}", "condition", "rows", "held-out accuracy", "mean x", "max x", "reservoir resets");
  println!("{}", "-".repeat(84));
  for r in &report.results {
    println!("{:<18} | {:>5} | {:>17.3} | {:>7.1} | {:>5} | {:>16}", r.condition.name(), r.rows, r.accuracy, r.mean_x, r.max_x, r.resets);
  }
}
```

- [ ] **Step 6: Build and run the experiment**

Run: `cargo build --release --example tetris_reservoir 2>&1 | grep -E "^error" -A8`
Expected: no errors.

Run: `./target/release/examples/tetris_reservoir 2>&1 | tee /private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/reservoir-run.txt`
Expected: progress lines on stderr, then the table. Budget roughly ten minutes: the demonstration is about a minute, each condition's collection is a few seconds, each fit a few seconds, and five parallel episodes of up to 3000 frames each at 150 frames/sec are about 20 seconds per condition.

- [ ] **Step 7: Commit**

```bash
cargo fmt
git add src/nes/gym/reservoir/mod.rs examples/tetris_reservoir.rs
git commit -m "Add the Tetris reservoir experiment runner and example.

Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e"
```

---

### Task 6: Full test run, lint, and record the result

**Files:**
- Modify: `docs/ideas/brain-cartridge.md` (add a "Result" paragraph under item 1)

- [ ] **Step 1: Run the whole suite and clippy on the new module**

Run: `cargo test 2>&1 | tail -5`
Expected: all passing, ignored tests listed as ignored.

Run: `cargo clippy --release --example tetris_reservoir 2>&1 | grep -A6 "reservoir/" | head -60`
Expected: no warnings in `src/nes/gym/reservoir/`. Fix any that appear.

- [ ] **Step 2: Record the result**

Under "### 1. Tetris reservoir" in `docs/ideas/brain-cartridge.md`, after the bullet list, add a paragraph headed `Result (2026-10-13):` containing the printed table inside a fenced code block and two or three sentences saying whether the driven conditions beat `raw` on held-out accuracy and on closed-loop distance, and what the reset counts say about the playfield drive. Do not editorialize beyond what the table shows.

- [ ] **Step 3: Commit**

```bash
git add docs/ideas/brain-cartridge.md
git commit -m "Record the first Tetris reservoir result.

Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e"
```
