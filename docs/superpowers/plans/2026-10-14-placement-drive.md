# Placement Drive Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Drive Tetris by steering real tetromino placements derived from Mario's state, and score the resulting Tetris RAM as a readout input against a one-hot control, a hash control, raw and clock.

**Architecture:** A pure encoder (`encode.rs`) turns the 256-byte SMB window into four placements (column, rotation). The reservoir (`drive.rs`) boots Tetris to level 19 through its menu and steers each placement by controller taps, resetting on game over. Three new `Condition`s route placements either straight to the readout as one-hot features or through Tetris; the experiment, dump and video pick them up.

**Tech Stack:** Rust 2021 (the greenstone crate, no new dependencies), Python 3 with Pillow and numpy for the video chapter, ffmpeg.

**Spec:** `docs/superpowers/specs/2026-10-14-placement-drive-design.md`

## Global Constraints

- Indentation is two spaces (`rustfmt.toml`); format touched files with `rustfmt --edition 2021 <files>`, never `cargo fmt`.
- Commit messages end with `Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e`.
- Never stage `src/main.rs` or `src/nes/bus/mod.rs` (the user's uncommitted debug edits).
- `PIECES_PER_STEP = 4`, `PIECE_FRAME_LIMIT = 120`, Tetris level 19 selected through the menu (nine Right taps at the level menu, then Start with A held).
- Steering is by controller only; the only RAM writes to Tetris are none (the menu script replaces the level byte write).
- ROM-backed tests are `#[ignore]` and skip with a notice when `roms2/tetris.nes` is absent; run them with `cargo test --release --lib <name> -- --ignored`.

## Probed facts (2026-10-14, Tetris US)

- Start taps every 30 frames: mode `$00C0` goes 0 → 1 (title, frame 271) → 2 (game type, 301) → 3 (level menu, 331) → 4 (play, 361).
- In play: `$0040` piece x (spawn 5, one column per tap, press one frame then release one frame), `$0041` piece y (0 at spawn, 18 at the floor), `$0042` orientation id, `$0048` play state (1 falling, 2 lock, 3 row check, 5 update, 8 spawn, 10 game over), `$0044` level, `$00BF` next piece.
- At the level menu, nine Right taps six frames apart then Start with A held for four frames starts at level 19. Gravity at 19 is one row per two frames; a spawned piece reaches the floor in about 36 frames and the next spawns about 20 frames later.
- A-rotation cycles from spawn: T `02 03 00 01`, J `07 04 05 06`, Z `08 09`, O `0a`, S `0b 0c`, L `0e 0f 10 0d`, I `12 11`.
- Ten pieces dropped in one column reach game over (play state 10).

## Review Focus

1. A window with no enemy ahead of Mario must encode "none" (column 9), not panic or pick column 0. Pinned in Task 1.
2. A target column a piece cannot reach (an I piece asked for column 9) must stop steering when a tap no longer moves it, within the frame limit, not loop. Pinned in Task 3.
3. A game over in the middle of a step must reset and continue with the remaining placements, counting one reset. Pinned in Task 3.
4. `Condition::ALL` changes; every match on `Condition` must be exhaustive, and `row_len` for the one-hot condition must equal the row the encoder produces. Pinned in Task 4.
5. The dump and video must not break when the playfield condition is no longer in the default set: the dump names its conditions explicitly. Pinned in Task 6.

---

### Task 1: The encoder

**Files:**
- Create: `src/nes/gym/reservoir/encode.rs`
- Modify: `src/nes/gym/reservoir/mod.rs:5-9` (add `pub mod encode;`)
- Modify: `src/nes/gym/reservoir/trace.rs:7-30` (add `y` to `Enemy`, read from `$00CF + slot`)

**Interfaces:**
- Consumes: `trace::enemies(window) -> Vec<Enemy>`.
- Produces: `pub struct Placement { pub column: u8, pub rotation: u8 }`, `pub const PIECES_PER_STEP: usize = 4`, `pub enum Encoder { Salient, Hash }`, `pub fn encode(encoder: Encoder, window: &[u8]) -> [Placement; PIECES_PER_STEP]`, `pub fn onehot_row(placements: &[Placement]) -> Vec<f32>` (length `1 + PIECES_PER_STEP * 14`), `pub const ONEHOT_ROW_LEN: usize = 57`, `pub const ENEMY_BIN_PX: i32 = 32`.

- [ ] **Step 1: Add `y` to `Enemy` and extend its test**

In `trace.rs`, change the struct and reader:

```rust
pub struct Enemy {
  pub slot: u8,
  pub kind: u8,
  pub x: i32,
  /// Screen y, smaller is higher.
  pub y: u8,
}

const ENEMY_Y: usize = 0x00CF;
```

and in `enemies`, add `y: window[ENEMY_Y + i],` to the constructor. Update the existing test's expected value to `Enemy { slot: 1, kind: 0x06, x: 528, y: 0 }` and add `window[ENEMY_Y + 1] = 0xB0;` before the assertion with `y: 0xB0`.

Run: `cargo test --lib reservoir::trace`
Expected: PASS after the edit (the test fails to compile until the field exists; that is the RED).

- [ ] **Step 2: Write the failing encoder tests**

Create `src/nes/gym/reservoir/encode.rs`:

```rust
//! Mario's window as tetromino placements.

use super::trace::enemies;

/// Placements sent to Tetris per SMB step.
pub const PIECES_PER_STEP: usize = 4;
/// Enemy distance bin width in pixels. Nine bins cover 288 pixels; the tenth column means "none".
pub const ENEMY_BIN_PX: i32 = 32;
/// Features per placement in the one-hot row: ten columns and four rotations.
pub const PLACEMENT_ONEHOT: usize = 14;
/// Intercept plus one-hot of every placement.
pub const ONEHOT_ROW_LEN: usize = 1 + PIECES_PER_STEP * PLACEMENT_ONEHOT;

const MARIO_X_PAGE: usize = 0x006D;
const MARIO_X_PIXEL: usize = 0x0086;
const MARIO_Y: usize = 0x00CE;
const MARIO_X_SPEED: usize = 0x0057;
const MARIO_Y_SPEED: usize = 0x009F;
const MARIO_AIRBORNE: usize = 0x001D;
const MARIO_STATE: usize = 0x000E;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
  /// Target column for the piece's x register, 0..=9.
  pub column: u8,
  /// Target rotation index into the piece's A-rotation cycle, 0..=3.
  pub rotation: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoder {
  /// Four hand-chosen features: the two nearest enemies ahead, Mario's height, Mario's speed.
  Salient,
  /// A hash of the whole window. Control: shows that locality in the encoding matters.
  Hash,
}

#[cfg(test)]
mod tests {
  use super::*;

  fn window_with_mario(x: i32, y: u8) -> Vec<u8> {
    let mut w = vec![0u8; 256];
    w[MARIO_X_PAGE] = (x / 256) as u8;
    w[MARIO_X_PIXEL] = (x % 256) as u8;
    w[MARIO_Y] = y;
    w
  }

  fn add_enemy(w: &mut [u8], slot: usize, x: i32, y: u8) {
    w[0x000F + slot] = 1;
    w[0x0016 + slot] = 0x06;
    w[0x006E + slot] = (x / 256) as u8;
    w[0x0087 + slot] = (x % 256) as u8;
    w[0x00CF + slot] = y;
  }

  #[test]
  fn nearest_enemy_ahead_sets_the_first_column_in_32_pixel_bins() {
    let mut w = window_with_mario(1000, 176);
    add_enemy(&mut w, 0, 1000 + 70, 0x80);
    add_enemy(&mut w, 1, 1000 + 200, 0x40);
    add_enemy(&mut w, 2, 900, 0x80); // behind Mario: ignored
    let p = encode(Encoder::Salient, &w);
    assert_eq!(p[0].column, 2, "70 px ahead is bin 2");
    assert_eq!(p[0].rotation, 2, "y 0x80 is band 2 of four");
    assert_eq!(p[1].column, 6, "200 px ahead is bin 6");
    assert_eq!(p[1].rotation, 1);
  }

  #[test]
  fn no_enemy_ahead_means_column_nine() {
    let mut w = window_with_mario(1000, 176);
    add_enemy(&mut w, 0, 900, 0x80);
    let p = encode(Encoder::Salient, &w);
    assert_eq!(p[0].column, 9);
    assert_eq!(p[1].column, 9);
    assert_eq!(p[0].rotation, 0);
  }

  #[test]
  fn far_enemies_clamp_to_the_last_distance_bin() {
    let mut w = window_with_mario(0, 176);
    add_enemy(&mut w, 0, 5000, 0);
    assert_eq!(encode(Encoder::Salient, &w)[0].column, 8);
  }

  #[test]
  fn height_speed_and_state_fill_the_last_two_placements() {
    let mut w = window_with_mario(0, 96);
    w[MARIO_AIRBORNE] = 1;
    w[MARIO_Y_SPEED] = 0xFC; // rising
    w[MARIO_X_SPEED] = 0x28; // 40, full run speed
    w[MARIO_STATE] = 0x08;
    let p = encode(Encoder::Salient, &w);
    assert_eq!(p[2].column, 4, "y 96 in 24-pixel bins");
    assert_eq!(p[2].rotation, 3, "airborne and rising");
    assert_eq!(p[3].column, 9, "speed 40 is the top bin");
    assert_eq!(p[3].rotation, 0, "state 8 & 3");
    w[MARIO_X_SPEED] = 0xD8; // -40
    w[MARIO_Y] = 255;
    assert_eq!(encode(Encoder::Salient, &w)[3].column, 0);
    assert_eq!(encode(Encoder::Salient, &w)[2].column, 9, "y clamps to the last bin");
  }

  #[test]
  fn hash_encoder_is_deterministic_in_range_and_sensitive_to_every_byte() {
    let w = vec![7u8; 256];
    let a = encode(Encoder::Hash, &w);
    assert_eq!(a, encode(Encoder::Hash, &w));
    for p in a {
      assert!(p.column <= 9 && p.rotation <= 3);
    }
    let mut w2 = w.clone();
    w2[200] = 8;
    assert_ne!(a, encode(Encoder::Hash, &w2));
  }

  #[test]
  fn onehot_row_has_an_intercept_and_two_hot_bits_per_placement() {
    let p = [Placement { column: 3, rotation: 1 }; PIECES_PER_STEP];
    let row = onehot_row(&p);
    assert_eq!(row.len(), ONEHOT_ROW_LEN);
    assert_eq!(row[0], 1.0);
    assert_eq!(row.iter().filter(|&&v| v == 1.0).count(), 1 + 2 * PIECES_PER_STEP);
    assert_eq!(row[1 + 3], 1.0);
    assert_eq!(row[1 + 10 + 1], 1.0);
    assert_eq!(row[1 + PLACEMENT_ONEHOT + 3], 1.0);
  }
}
```

Add `pub mod encode;` to `mod.rs` after `pub mod drive;`.

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test --lib reservoir::encode`
Expected: compile errors naming `encode` and `onehot_row` as not found.

- [ ] **Step 4: Implement the encoder**

Add above the tests in `encode.rs`:

```rust
fn mario_x(window: &[u8]) -> i32 {
  window[MARIO_X_PAGE] as i32 * 256 + window[MARIO_X_PIXEL] as i32
}

/// Distance bin for an enemy `ahead` pixels in front of Mario, or 9 for none.
fn distance_column(ahead: Option<i32>) -> u8 {
  match ahead {
    Some(d) => ((d / ENEMY_BIN_PX).clamp(0, 8)) as u8,
    None => 9,
  }
}

fn y_band(y: u8) -> u8 {
  y / 64
}

/// The two nearest enemies ahead of Mario, nearest first, as (distance, y).
fn enemies_ahead(window: &[u8]) -> Vec<(i32, u8)> {
  let mx = mario_x(window);
  let mut ahead: Vec<(i32, u8)> = enemies(window)
    .into_iter()
    .filter(|e| e.x > mx)
    .map(|e| (e.x - mx, e.y))
    .collect();
  ahead.sort();
  ahead
}

fn salient(window: &[u8]) -> [Placement; PIECES_PER_STEP] {
  let ahead = enemies_ahead(window);
  let enemy = |i: usize| match ahead.get(i) {
    Some(&(d, y)) => Placement {
      column: distance_column(Some(d)),
      rotation: y_band(y),
    },
    None => Placement { column: 9, rotation: 0 },
  };
  let airborne = (window[MARIO_AIRBORNE] != 0) as u8;
  let rising = ((window[MARIO_Y_SPEED] as i8) < 0) as u8;
  let height = Placement {
    column: (window[MARIO_Y] / 24).min(9),
    rotation: airborne * 2 + rising,
  };
  let speed = window[MARIO_X_SPEED] as i8 as i32;
  let motion = Placement {
    column: ((speed + 40) / 8).clamp(0, 9) as u8,
    rotation: window[MARIO_STATE] & 3,
  };
  [enemy(0), enemy(1), height, motion]
}

fn hash(window: &[u8]) -> [Placement; PIECES_PER_STEP] {
  let mut out = [Placement { column: 0, rotation: 0 }; PIECES_PER_STEP];
  for (i, slot) in out.iter_mut().enumerate() {
    let mut h = 0xCBF2_9CE4_8422_2325u64 ^ (i as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for &b in window {
      h = (h ^ b as u64).wrapping_mul(0x100_0000_01B3);
    }
    *slot = Placement {
      column: (h % 10) as u8,
      rotation: ((h >> 8) % 4) as u8,
    };
  }
  out
}

/// Four placements for one SMB window.
pub fn encode(encoder: Encoder, window: &[u8]) -> [Placement; PIECES_PER_STEP] {
  match encoder {
    Encoder::Salient => salient(window),
    Encoder::Hash => hash(window),
  }
}

/// `[1.0, then per placement: ten column bits, four rotation bits]`.
pub fn onehot_row(placements: &[Placement]) -> Vec<f32> {
  let mut row = vec![0.0; 1 + placements.len() * PLACEMENT_ONEHOT];
  row[0] = 1.0;
  for (i, p) in placements.iter().enumerate() {
    let base = 1 + i * PLACEMENT_ONEHOT;
    row[base + p.column as usize] = 1.0;
    row[base + 10 + p.rotation as usize] = 1.0;
  }
  row
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test --lib reservoir::encode`
Expected: 6 passed. Then `cargo test --lib reservoir` all green.

- [ ] **Step 6: Commit**

```bash
rustfmt --edition 2021 src/nes/gym/reservoir/encode.rs src/nes/gym/reservoir/trace.rs src/nes/gym/reservoir/mod.rs
git add src/nes/gym/reservoir/encode.rs src/nes/gym/reservoir/trace.rs src/nes/gym/reservoir/mod.rs
git commit -m "Encode Mario's window as four tetromino placements.

Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e"
```

---

### Task 2: Tetris at level 19

**Files:**
- Modify: `src/nes/gym/reservoir/drive.rs:56-90` (boot), tests at the end of the file

**Interfaces:**
- Produces: `pub const TETRIS_LEVEL: u16 = 0x0044`, `pub const TETRIS_MODE_LEVEL_MENU: u8 = 0x03`, `pub const FAST_LEVEL: u8 = 19`, `Reservoir::new_fast(rom: &[u8]) -> Result<Reservoir, String>` (play mode at level 19, snapshot taken there, so `reset` returns to level 19).

- [ ] **Step 1: Write the failing ROM-backed test**

Append to the `tests` module in `drive.rs` (it already imports `rom_or_skip`; if not, add `use crate::nes::gym::reservoir::rom_or_skip;`):

```rust
  #[test]
  #[ignore]
  fn new_fast_starts_at_level_19_and_resets_there() {
    let rom = match rom_or_skip("roms2/tetris.nes") {
      Some(r) => r,
      None => return,
    };
    let mut r = Reservoir::new_fast(&rom).unwrap();
    assert!(r.in_play());
    assert_eq!(r.emu.ram_read(TETRIS_LEVEL), FAST_LEVEL);
    // A piece reaches the floor in well under 60 frames at this level.
    let y0 = r.emu.ram_read(TETRIS_PIECE_Y);
    r.emu.step_frames(0, 30);
    assert!(r.emu.ram_read(TETRIS_PIECE_Y) >= y0 + 10, "gravity should be two frames per row");
    r.emu.step_frames(0, 600);
    r.reset();
    assert_eq!(r.emu.ram_read(TETRIS_LEVEL), FAST_LEVEL);
    assert!(r.in_play());
  }
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test --release --lib new_fast_starts -- --ignored`
Expected: compile error, `new_fast` not found.

- [ ] **Step 3: Implement the menu script**

In `drive.rs`, add constants near the others:

```rust
/// Tetris RAM: current level.
pub const TETRIS_LEVEL: u16 = 0x0044;
pub const TETRIS_MODE_LEVEL_MENU: u8 = 0x03;
/// Level the placement drive plays at: one row per two frames.
pub const FAST_LEVEL: u8 = 19;
```

Refactor `new` so both constructors share the boot walk, and add `new_fast`:

```rust
  /// Boot Tetris, tap Start through the menus to gameplay at level 0, snapshot.
  pub fn new(rom: &[u8]) -> Result<Reservoir, String> {
    let mut emu = NesEmulator::new(rom);
    boot_until(&mut emu, TETRIS_MODE_PLAY)?;
    Self::settle(emu)
  }

  /// Boot Tetris to the level menu, pick level 9 and start with A held, which
  /// the game reads as level 19. Snapshot there, so resets return to level 19.
  pub fn new_fast(rom: &[u8]) -> Result<Reservoir, String> {
    let mut emu = NesEmulator::new(rom);
    boot_until(&mut emu, TETRIS_MODE_LEVEL_MENU)?;
    emu.step_frames(0, 30);
    for _ in 0..9 {
      emu.step_frame(BUTTON_RIGHT);
      emu.step_frame(0);
      emu.step_frames(0, 6);
    }
    emu.step_frames(BUTTON_A, 4);
    emu.step_frames(BUTTON_A | BUTTON_START, 4);
    emu.step_frames(BUTTON_A, 30);
    if emu.ram_read(TETRIS_GAME_MODE) != TETRIS_MODE_PLAY {
      return Err("Tetris did not start from the level menu".to_string());
    }
    if emu.ram_read(TETRIS_LEVEL) != FAST_LEVEL {
      return Err(format!("expected level {}, got {}", FAST_LEVEL, emu.ram_read(TETRIS_LEVEL)));
    }
    Self::settle(emu)
  }

  /// Confirm a piece is falling (un-pausing once if not) and snapshot.
  fn settle(mut emu: NesEmulator) -> Result<Reservoir, String> {
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
```

and a free function:

```rust
/// Tap Start every 30 frames until `mode` is reached.
fn boot_until(emu: &mut NesEmulator, mode: u8) -> Result<(), String> {
  for frame in 0..BOOT_FRAME_LIMIT {
    let buttons = if frame % 30 < 4 { BUTTON_START } else { 0 };
    emu.step_frame(buttons);
    if emu.ram_read(TETRIS_GAME_MODE) == mode {
      return Ok(());
    }
  }
  Err(format!("Tetris did not reach mode {} within {} frames", mode, BOOT_FRAME_LIMIT))
}
```

Note `settle` at level 19 waits up to 120 frames for a piece to move; at level 19 it moves within two frames, so this is quick.

- [ ] **Step 4: Run the test to verify it passes, and the existing boot test**

Run: `cargo test --release --lib drive -- --ignored` and `cargo test --lib reservoir`
Expected: all PASS (the existing `new` behaviour is unchanged).

- [ ] **Step 5: Commit**

```bash
rustfmt --edition 2021 src/nes/gym/reservoir/drive.rs
git add src/nes/gym/reservoir/drive.rs
git commit -m "Boot Tetris to level 19 through its own menu.

Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e"
```

---

### Task 3: Steering a placement

**Files:**
- Modify: `src/nes/gym/reservoir/drive.rs` (new constants, `Drive::Placements`, `Reservoir::place`, `step` handling)

**Interfaces:**
- Consumes: `encode::{encode, Encoder, Placement, PIECES_PER_STEP}`.
- Produces: `pub const PIECE_FRAME_LIMIT: usize = 120`, `pub const TETRIS_PIECE_X: u16 = 0x0040`, `pub const TETRIS_PIECE_ORIENTATION: u16 = 0x0042`, `pub const TETRIS_PLAY_STATE_FALLING: u8 = 0x01`, `pub fn rotation_cycle(orientation: u8) -> &'static [u8]`, `pub struct PlaceOutcome { pub frames: usize, pub locked: bool, pub reset: bool }`, `Reservoir::place(&mut self, target: Placement) -> PlaceOutcome`, `Drive::Placements(Encoder)`, and `Reservoir::step` runs the four placements for that drive (ignoring `frames`).

- [ ] **Step 1: Write the failing unit test for the rotation cycles**

```rust
  #[test]
  fn rotation_cycles_follow_the_a_button_from_spawn() {
    assert_eq!(rotation_cycle(0x0E), &[0x0E, 0x0F, 0x10, 0x0D]);
    assert_eq!(rotation_cycle(0x10), &[0x0E, 0x0F, 0x10, 0x0D], "any member maps to its cycle");
    assert_eq!(rotation_cycle(0x02), &[0x02, 0x03, 0x00, 0x01]);
    assert_eq!(rotation_cycle(0x07), &[0x07, 0x04, 0x05, 0x06]);
    assert_eq!(rotation_cycle(0x08), &[0x08, 0x09]);
    assert_eq!(rotation_cycle(0x0A), &[0x0A]);
    assert_eq!(rotation_cycle(0x0B), &[0x0B, 0x0C]);
    assert_eq!(rotation_cycle(0x12), &[0x12, 0x11]);
    assert_eq!(rotation_cycle(0x13), &[0x13], "no piece: a one-element cycle, never a panic");
  }
```

- [ ] **Step 2: Write the failing ROM-backed steering tests**

```rust
  #[test]
  #[ignore]
  fn place_steers_to_the_column_and_rotation_and_locks() {
    let rom = match rom_or_skip("roms2/tetris.nes") {
      Some(r) => r,
      None => return,
    };
    let mut r = Reservoir::new_fast(&rom).unwrap();
    let orientation = r.emu.ram_read(TETRIS_PIECE_ORIENTATION);
    let cycle = rotation_cycle(orientation);
    let target = Placement { column: 1, rotation: 1 };
    let out = r.place(target);
    assert!(out.locked, "piece should lock within {} frames", PIECE_FRAME_LIMIT);
    assert!(out.frames < PIECE_FRAME_LIMIT);
    assert!(!out.reset);
    // The locked cells sit in the low columns of the board.
    let board = r.emu.ram_read_range(TETRIS_PLAYFIELD, TETRIS_PLAYFIELD_LEN);
    let filled: Vec<usize> = board.iter().enumerate().filter(|(_, &c)| c != PLAYFIELD_EMPTY).map(|(i, _)| i % 10).collect();
    assert!(!filled.is_empty());
    assert!(filled.iter().all(|&c| c <= 3), "cells in columns {:?}", filled);
    let _ = cycle;
  }

  #[test]
  #[ignore]
  fn place_gives_up_on_an_unreachable_column_without_looping() {
    let rom = match rom_or_skip("roms2/tetris.nes") {
      Some(r) => r,
      None => return,
    };
    let mut r = Reservoir::new_fast(&rom).unwrap();
    // Column 9 is unreachable for wide pieces; steering must stop at the wall and still lock.
    let out = r.place(Placement { column: 9, rotation: 0 });
    assert!(out.locked);
    assert!(out.frames < PIECE_FRAME_LIMIT);
  }

  #[test]
  #[ignore]
  fn placements_in_one_column_reach_game_over_and_reset() {
    let rom = match rom_or_skip("roms2/tetris.nes") {
      Some(r) => r,
      None => return,
    };
    let mut r = Reservoir::new_fast(&rom).unwrap();
    let mut resets = 0;
    for _ in 0..16 {
      let out = r.place(Placement { column: 5, rotation: 0 });
      resets += out.reset as u32;
    }
    assert!(resets >= 1, "sixteen pieces in one column must top out at least once");
    assert_eq!(r.resets, resets);
    assert!(r.in_play());
    assert_eq!(r.emu.ram_read(TETRIS_LEVEL), FAST_LEVEL);
  }

  #[test]
  #[ignore]
  fn step_with_placements_runs_four_pieces() {
    let rom = match rom_or_skip("roms2/tetris.nes") {
      Some(r) => r,
      None => return,
    };
    let mut r = Reservoir::new_fast(&rom).unwrap();
    let window = vec![0u8; 256];
    let before = r.emu.ram_read_range(TETRIS_PLAYFIELD, TETRIS_PLAYFIELD_LEN);
    r.step(Some(Drive::Placements(Encoder::Salient)), &window, 4);
    let after = r.emu.ram_read_range(TETRIS_PLAYFIELD, TETRIS_PLAYFIELD_LEN);
    let cells = |b: &[u8]| b.iter().filter(|&&c| c != PLAYFIELD_EMPTY).count();
    assert!(cells(&after) >= cells(&before) + 12, "four locked pieces add at least sixteen cells minus any cleared line");
  }
```

Add `use super::encode::{Encoder, Placement};` to the test module imports if the file's `use super::*` does not already bring them in (it will once Step 4 imports them at the top).

- [ ] **Step 3: Run to verify they fail**

Run: `cargo test --lib rotation_cycles`
Expected: compile error, `rotation_cycle` not found.

- [ ] **Step 4: Implement the cycles, `place`, and the drive**

At the top of `drive.rs`, add `use super::encode::{encode, Encoder, Placement};`. Add constants:

```rust
/// Tetris RAM: falling piece x and orientation id.
pub const TETRIS_PIECE_X: u16 = 0x0040;
pub const TETRIS_PIECE_ORIENTATION: u16 = 0x0042;
pub const TETRIS_PLAY_STATE_FALLING: u8 = 0x01;
/// Frames one placement may take before the drive moves on.
pub const PIECE_FRAME_LIMIT: usize = 120;

const CYCLES: [&[u8]; 7] = [
  &[0x02, 0x03, 0x00, 0x01], // T
  &[0x07, 0x04, 0x05, 0x06], // J
  &[0x08, 0x09],             // Z
  &[0x0A],                   // O
  &[0x0B, 0x0C],             // S
  &[0x0E, 0x0F, 0x10, 0x0D], // L
  &[0x12, 0x11],             // I
];

/// The orientation ids a piece passes through under A presses, starting at spawn.
pub fn rotation_cycle(orientation: u8) -> &'static [u8] {
  for cycle in CYCLES {
    if cycle.contains(&orientation) {
      return cycle;
    }
  }
  &[0x13]
}
```

Extend `Drive`:

```rust
pub enum Drive {
  Controller,
  Playfield,
  /// Steer four real placements per step, chosen by the encoder.
  Placements(Encoder),
}
```

Add to `impl Reservoir`:

```rust
  fn tap(&mut self, buttons: u8) -> usize {
    self.emu.step_frame(buttons);
    self.emu.step_frame(0);
    2
  }

  fn piece(&mut self) -> (u8, u8, u8) {
    (
      self.emu.ram_read(TETRIS_PIECE_X),
      self.emu.ram_read(TETRIS_PIECE_ORIENTATION),
      self.emu.ram_read(TETRIS_PLAY_STATE),
    )
  }

  /// Steer the falling piece to `target` by controller taps and let it lock,
  /// then wait for the next piece to spawn. Resets on game over.
  pub fn place(&mut self, target: Placement) -> PlaceOutcome {
    let mut frames = 0;
    let mut reset = false;
    // Wait for a falling piece.
    while frames < PIECE_FRAME_LIMIT && self.emu.ram_read(TETRIS_PLAY_STATE) != TETRIS_PLAY_STATE_FALLING {
      if !self.in_play() {
        self.reset();
        self.resets += 1;
        reset = true;
      }
      self.emu.step_frame(0);
      frames += 1;
    }
    let (_, orientation, _) = self.piece();
    let cycle = rotation_cycle(orientation);
    let want = cycle[target.rotation as usize % cycle.len()];
    let mut taps = 0;
    while frames < PIECE_FRAME_LIMIT && self.piece().1 != want && taps < 4 {
      frames += self.tap(BUTTON_A);
      taps += 1;
    }
    let mut stuck = false;
    loop {
      let (x, _, state) = self.piece();
      if frames >= PIECE_FRAME_LIMIT || state != TETRIS_PLAY_STATE_FALLING || stuck || x == target.column {
        break;
      }
      let button = if x < target.column { BUTTON_RIGHT } else { BUTTON_LEFT };
      frames += self.tap(button);
      stuck = self.piece().0 == x;
    }
    // Let it fall and lock.
    let mut locked = false;
    while frames < PIECE_FRAME_LIMIT {
      let (_, _, state) = self.piece();
      if state != TETRIS_PLAY_STATE_FALLING {
        locked = true;
        break;
      }
      self.emu.step_frame(0);
      frames += 1;
    }
    if !self.in_play() {
      self.reset();
      self.resets += 1;
      reset = true;
    }
    PlaceOutcome { frames, locked, reset }
  }
```

and the outcome type:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaceOutcome {
  pub frames: usize,
  pub locked: bool,
  pub reset: bool,
}
```

Change `step`:

```rust
  /// Apply the drive, advance `frames`, and restore the snapshot if the game ended.
  /// For the placement drive, `frames` is ignored: the step is four placements.
  pub fn step(&mut self, drive: Option<Drive>, window: &[u8], frames: usize) {
    let buttons = match drive {
      Some(Drive::Controller) => fold_to_controller(window),
      Some(Drive::Playfield) => {
        self.write_playfield(window);
        0
      }
      Some(Drive::Placements(encoder)) => {
        for target in encode(encoder, window) {
          self.place(target);
        }
        return;
      }
      None => 0,
    };
    self.emu.step_frames(buttons, frames);
    if !self.in_play() {
      self.reset();
      self.resets += 1;
    }
  }
```

- [ ] **Step 5: Run the unit test, then the ROM tests**

Run: `cargo test --lib rotation_cycles` then `cargo test --release --lib drive -- --ignored`
Expected: all PASS. If `place_steers_to_the_column` fails on the column assertion, print `filled` and check whether `$0040` is the piece's left or centre cell; adjust the assertion bound (`<= 3`) with a comment, not the steering.

- [ ] **Step 6: Commit**

```bash
rustfmt --edition 2021 src/nes/gym/reservoir/drive.rs
git add src/nes/gym/reservoir/drive.rs
git commit -m "Steer tetromino placements by controller taps.

Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e"
```

---

### Task 4: The three placement conditions

**Files:**
- Modify: `src/nes/gym/reservoir/features.rs` (variants, `ALL`, `LEGACY`, `name`, `drive`, `uses_reservoir`, `row_len`, `fast_tetris`)
- Modify: `src/nes/gym/reservoir/mod.rs` (`observe`, a `reservoir_for` helper used by `collect`, `episode`)
- Modify: `src/nes/gym/reservoir/trace.rs` (use `reservoir_for` in `replay` and `trace_episode`)

**Interfaces:**
- Consumes: `encode::{encode, onehot_row, Encoder, ONEHOT_ROW_LEN}`, `drive::{Drive, Reservoir::new, Reservoir::new_fast}`.
- Produces: `Condition::{PlacementOnehot, PlacementTetris, PlacementHash}`, `Condition::ALL` = `[Clock, Raw, PlacementOnehot, PlacementTetris, PlacementHash]`, `Condition::LEGACY` = `[Undriven, DrivenController, DrivenPlayfield]`, `Condition::fast_tetris(self) -> bool`, `pub(crate) fn reservoir_for(condition: Condition, tetris_rom: &[u8]) -> Result<Option<Reservoir>, String>`.

- [ ] **Step 1: Write the failing feature tests**

In `features.rs` tests, replace `row_lengths_match_conditions`, `only_clock_and_raw_skip_the_reservoir` and `names_are_distinct` with:

```rust
  #[test]
  fn row_lengths_match_conditions() {
    assert_eq!(Condition::Clock.row_len(), 1 + CLOCK_CAPACITY);
    assert_eq!(Condition::Raw.row_len(), 1 + 256);
    assert_eq!(Condition::Undriven.row_len(), 1 + 2048);
    assert_eq!(Condition::DrivenController.row_len(), 1 + 2048 + 8);
    assert_eq!(Condition::DrivenPlayfield.row_len(), 1 + 2048);
    assert_eq!(Condition::PlacementOnehot.row_len(), super::super::encode::ONEHOT_ROW_LEN);
    assert_eq!(Condition::PlacementTetris.row_len(), 1 + 2048);
    assert_eq!(Condition::PlacementHash.row_len(), 1 + 2048);
  }

  #[test]
  fn reservoir_use_and_speed_follow_the_drive() {
    for c in [Condition::Clock, Condition::Raw, Condition::PlacementOnehot] {
      assert!(!c.uses_reservoir(), "{}", c.name());
      assert!(c.drive().is_none());
    }
    for c in [Condition::PlacementTetris, Condition::PlacementHash] {
      assert!(c.uses_reservoir(), "{}", c.name());
      assert!(c.fast_tetris(), "{}", c.name());
    }
    for c in Condition::LEGACY {
      assert!(c.uses_reservoir());
      assert!(!c.fast_tetris());
    }
    assert_eq!(Condition::PlacementTetris.drive(), Some(super::super::drive::Drive::Placements(super::super::encode::Encoder::Salient)));
    assert_eq!(Condition::PlacementHash.drive(), Some(super::super::drive::Drive::Placements(super::super::encode::Encoder::Hash)));
  }

  #[test]
  fn names_are_distinct_across_default_and_legacy() {
    let mut names: Vec<&str> = Condition::ALL.iter().chain(Condition::LEGACY.iter()).map(|c| c.name()).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), Condition::ALL.len() + Condition::LEGACY.len());
    assert_eq!(Condition::ALL.len(), 5);
    assert_eq!(Condition::LEGACY.len(), 3);
  }
```

And in `mod.rs` tests add:

```rust
  #[test]
  fn observe_onehot_needs_no_reservoir() {
    let window = vec![0u8; 256];
    let row = observe(Condition::PlacementOnehot, &window, None, 0);
    assert_eq!(row.len(), Condition::PlacementOnehot.row_len());
    assert_eq!(row[0], 1.0);
  }
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test --lib reservoir::features`
Expected: compile errors on the new variants.

- [ ] **Step 3: Implement the variants**

In `features.rs`:

```rust
use super::drive::Drive;
use super::encode::{Encoder, ONEHOT_ROW_LEN};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
  Clock,
  Raw,
  Undriven,
  DrivenController,
  DrivenPlayfield,
  /// The four placements as one-hot features, no Tetris. The control for the two below.
  PlacementOnehot,
  /// Tetris RAM after steering the four salient placements.
  PlacementTetris,
  /// Tetris RAM after steering placements hashed from the whole window.
  PlacementHash,
}

impl Condition {
  /// The default table.
  pub const ALL: [Condition; 5] = [
    Condition::Clock,
    Condition::Raw,
    Condition::PlacementOnehot,
    Condition::PlacementTetris,
    Condition::PlacementHash,
  ];
  /// The earlier drives, still runnable.
  pub const LEGACY: [Condition; 3] = [Condition::Undriven, Condition::DrivenController, Condition::DrivenPlayfield];

  pub fn name(self) -> &'static str {
    match self {
      Condition::Clock => "clock",
      Condition::Raw => "raw",
      Condition::Undriven => "undriven",
      Condition::DrivenController => "driven-controller",
      Condition::DrivenPlayfield => "driven-playfield",
      Condition::PlacementOnehot => "placement-onehot",
      Condition::PlacementTetris => "placement-tetris",
      Condition::PlacementHash => "placement-hash",
    }
  }

  pub fn drive(self) -> Option<Drive> {
    match self {
      Condition::DrivenController => Some(Drive::Controller),
      Condition::DrivenPlayfield => Some(Drive::Playfield),
      Condition::PlacementTetris => Some(Drive::Placements(Encoder::Salient)),
      Condition::PlacementHash => Some(Drive::Placements(Encoder::Hash)),
      _ => None,
    }
  }

  pub fn uses_reservoir(self) -> bool {
    !matches!(self, Condition::Raw | Condition::Clock | Condition::PlacementOnehot)
  }

  /// Placement drives play at level 19 so a piece drops in a few dozen frames.
  pub fn fast_tetris(self) -> bool {
    matches!(self.drive(), Some(Drive::Placements(_)))
  }

  pub fn appends_drive_bits(self) -> bool {
    matches!(self, Condition::DrivenController)
  }

  pub fn row_len(self) -> usize {
    match self {
      Condition::Clock => 1 + CLOCK_CAPACITY,
      Condition::PlacementOnehot => ONEHOT_ROW_LEN,
      _ => {
        let body = if self.uses_reservoir() { TETRIS_RAM_LEN } else { SMB_WINDOW_LEN } as usize;
        1 + body + if self.appends_drive_bits() { DRIVE_BITS } else { 0 }
      }
    }
  }
}
```

Remove the old `drive()` that referenced `super::drive::Drive` by path; `Drive` and `Encoder` must derive `PartialEq, Eq, Debug, Clone, Copy` (they do).

In `mod.rs`, add the helper and use it in `collect` and `episode`:

```rust
/// The reservoir a condition needs, booted the way its drive needs it, or None.
pub(crate) fn reservoir_for(condition: Condition, tetris_rom: &[u8]) -> Result<Option<Reservoir>, String> {
  if !condition.uses_reservoir() {
    return Ok(None);
  }
  if condition.fast_tetris() {
    Reservoir::new_fast(tetris_rom).map(Some)
  } else {
    Reservoir::new(tetris_rom).map(Some)
  }
}
```

Replace the two `let mut reservoir = if condition.uses_reservoir() { Some(Reservoir::new(tetris_rom)?) } else { None };` blocks with `let mut reservoir = reservoir_for(condition, tetris_rom)?;`. In `observe`, add an arm before the reservoir arm:

```rust
    (Condition::PlacementOnehot, _) => onehot_row(&encode(Encoder::Salient, window)),
```

with `use encode::{encode, onehot_row, Encoder};` at the top. In `trace.rs`, replace the per-condition `Reservoir::new` calls in `replay` and `trace_episode` with `reservoir_for(*c, tetris_rom)` / `reservoir_for(condition, tetris_rom)?` and import it from `super`.

- [ ] **Step 4: Run all reservoir tests**

Run: `cargo test --lib reservoir` and `cargo build --release --examples`
Expected: all PASS, examples compile (the example prints `Condition::ALL`, which now is the new table).

- [ ] **Step 5: Commit**

```bash
rustfmt --edition 2021 src/nes/gym/reservoir/features.rs src/nes/gym/reservoir/mod.rs src/nes/gym/reservoir/trace.rs
git add src/nes/gym/reservoir/features.rs src/nes/gym/reservoir/mod.rs src/nes/gym/reservoir/trace.rs
git commit -m "Add the placement conditions and their one-hot and hash controls.

Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e"
```

---

### Task 5: The table

**Files:**
- Modify: `examples/tetris_reservoir.rs` (header lines, `--legacy` flag)
- Modify: `docs/ideas/brain-cartridge.md` (new result section after the run)

**Interfaces:**
- Consumes: `Condition::{ALL, LEGACY}`, `run`, `encode::{PIECES_PER_STEP, ENEMY_BIN_PX}`, `drive::{FAST_LEVEL, PIECE_FRAME_LIMIT}`.

- [ ] **Step 1: Add the flag and header**

In `examples/tetris_reservoir.rs`, parse `--legacy` from `std::env::args()` (filter it out before positional args, as `tools/reservoir_video.py` does for `--quick`). Pass the condition list to `run`: change `run`'s signature in `mod.rs` to take `conditions: &[Condition]` after `demos`, and iterate it instead of `Condition::ALL`; with `--legacy` the example passes `Condition::ALL.iter().chain(Condition::LEGACY.iter())` collected. Update the dump example's call accordingly (it calls `fit_condition`, not `run`, so no change there). Print before the table:

```rust
  println!(
    "placements: {} per step, enemy bins {} px, Tetris level {} via the menu, {} frames per piece at most",
    PIECES_PER_STEP, ENEMY_BIN_PX, FAST_LEVEL, PIECE_FRAME_LIMIT
  );
```

- [ ] **Step 2: Run the table on the cached demonstrations**

Run: `cargo run --release --example tetris_reservoir > /private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/reservoir-run11.txt 2>&1`
Expected: a five-row table within about twenty minutes; `resets` large for the two Tetris conditions.

- [ ] **Step 3: Record the result**

Append to the Tetris reservoir section of `docs/ideas/brain-cartridge.md`, before "Earlier tables", a paragraph headed `Result (2026-10-14, placement drive):` describing the encoder in the spec's terms, the table verbatim, and an interpretation that answers: did placement-tetris beat placement-onehot, did placement-hash sit on the clock floor, did either beat raw, and what the closed-loop column did.

- [ ] **Step 4: Commit**

```bash
rustfmt --edition 2021 examples/tetris_reservoir.rs src/nes/gym/reservoir/mod.rs
git add examples/tetris_reservoir.rs src/nes/gym/reservoir/mod.rs docs/ideas/brain-cartridge.md
git commit -m "Run and record the placement drive table.

Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e"
```

---

### Task 6: The video chapter

**Files:**
- Modify: `examples/reservoir_dump.rs` (conditions `[Raw, PlacementOnehot, PlacementTetris, Clock]`, placements per step in the JSON)
- Modify: `src/nes/gym/reservoir/trace.rs` (`StepRecord.placements: Vec<(u8, u8)>` from `encode(Encoder::Salient, &window)`)
- Modify: `tools/reservoir_video.py` (chapter 4b "A placement", realtime gauges use `placement-tetris`, verdict rows)

**Interfaces:**
- Consumes: everything above; the dump's JSON gains `"placements": [[col, rot], ...]` per step and `"conditions"` lists the four names.

- [ ] **Step 1: Record placements in the dump**

In `trace.rs`, add `pub placements: Vec<(u8, u8)>` to `StepRecord` and fill it with `encode(Encoder::Salient, &window).iter().map(|p| (p.column, p.rotation)).collect()`. In the dump example, change the condition list to `[Condition::Raw, Condition::PlacementOnehot, Condition::PlacementTetris, Condition::Clock]`, serialise `"placements": s.placements`, and move the clock episode index to 3. The `weights` block keeps `raw_a` and replaces `playfield_a` with `tetris_a` from readout 2.

Run: `cargo build --release --example reservoir_dump && ./target/release/examples/reservoir_dump`
Expected: writes the dump in under ten minutes (placements make the Tetris replay slower).

- [ ] **Step 2: Add the chapter**

In `tools/reservoir_video.py`, add `chapter_placement(dump, n)` after `chapter_reservoir`, with these captions:

```python
  schedule = [
    (0, "Second try. Instead of pasting bytes, we read four numbers off Mario and play them as tetrominoes."),
    (8, "Piece one: how far ahead the nearest enemy is, 32 pixels per column. Piece two: the next enemy. Three: Mario's height. Four: his speed."),
    (18, "Tetris plays them at level 19 by tapping the controller: rotate, slide, drop. It tops out often; a game over just clears the board."),
    (28, "Now the board is a picture of what is around Mario, and a cell in it is the kind of feature a weighted sum could never build from bytes."),
  ]
```

Layout: Mario's screen at 1x left, the four placements as four small labelled columns (`enemy 1`, `enemy 2`, `height`, `speed`) each a 10-wide bar with the chosen column lit and the rotation printed, the board from RAM at cell 16, the Tetris readout's A gauge, and both the `placement-onehot` and `placement-tetris` gauges below. Use `s["placements"]` and `dump.a_output(step, "placement-tetris")`. Register it in `chapters()` after `chapter_reservoir` with 38 seconds. In `chapter_realtime`, replace the third gauge's condition with `placement-tetris` and its label with "Tetris readout"; in `trace_panel`, the second trace likewise. Update `chapter_verdict` rows from the run-11 table.

- [ ] **Step 3: Render and check**

Run: `python3 tools/reservoir_video.py target/tetris-reservoir-dump <scratch>/quick.mp4 --quick`, extract a frame from the new chapter with ffmpeg, view it, then render the full video to `<scratch>/tetris-reservoir.mp4` and send it to the user.

- [ ] **Step 4: Commit**

```bash
rustfmt --edition 2021 src/nes/gym/reservoir/trace.rs examples/reservoir_dump.rs
git add src/nes/gym/reservoir/trace.rs examples/reservoir_dump.rs tools/reservoir_video.py
git commit -m "Add the placement chapter to the reservoir video.

Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e"
```
