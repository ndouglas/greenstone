//! How SMB state enters Tetris, and the Tetris lifecycle as a reservoir.

use super::encode::{encode, Encoder, Placement};
use super::features::TETRIS_RAM_LEN;
use crate::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT, BUTTON_START, BUTTON_UP};
use crate::nes::gym::NesEmulator;

/// Tetris RAM: current game mode. 4 is gameplay.
pub const TETRIS_GAME_MODE: u16 = 0x00C0;
pub const TETRIS_MODE_PLAY: u8 = 0x04;
/// Tetris RAM: play state within game mode 4. Game over is a play state, not a mode change.
pub const TETRIS_PLAY_STATE: u16 = 0x0048;
pub const TETRIS_PLAY_STATE_GAME_OVER: u8 = 0x0A;
/// Playfield geometry and cell values. Any cell other than `PLAYFIELD_EMPTY` is solid.
pub const PLAYFIELD_COLS: usize = 10;
pub const PLAYFIELD_ROWS: usize = 20;
/// Top rows kept clear so a new piece can always spawn.
pub const PLAYFIELD_SPAWN_ROWS: usize = 4;
pub const PLAYFIELD_EMPTY: u8 = 0xEF;
pub const PLAYFIELD_BLOCK: u8 = 0x7B;
/// A window byte at or above this becomes a block cell.
pub const PLAYFIELD_THRESHOLD: u8 = 0xC0;
/// Tetris RAM: falling piece Y, used as a liveness check.
pub const TETRIS_PIECE_Y: u16 = 0x0041;
/// Tetris RAM: current level.
pub const TETRIS_LEVEL: u16 = 0x0044;
pub const TETRIS_MODE_LEVEL_MENU: u8 = 0x03;
/// Level the placement drive plays at: one row per two frames.
pub const FAST_LEVEL: u8 = 19;
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

/// What one placement did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaceOutcome {
  pub frames: usize,
  pub locked: bool,
  pub reset: bool,
}
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
  /// Steer four real placements per step, chosen by the encoder.
  Placements(Encoder),
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
  /// Boot Tetris, tap Start through the menus to gameplay at level 0, snapshot.
  pub fn new(rom: &[u8]) -> Result<Reservoir, String> {
    let mut emu = NesEmulator::new(rom);
    boot_until(&mut emu, TETRIS_MODE_PLAY)?;
    Self::settle(emu, 120)
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
    // The first piece of a game waits about 96 frames before it drops.
    Self::settle(emu, 200)
  }

  /// Step until the falling piece first moves, within `limit` frames (un-pausing
  /// once if it does not), and snapshot at that moment, before anything locks.
  fn settle(mut emu: NesEmulator, limit: usize) -> Result<Reservoir, String> {
    if !wait_for_piece_motion(&mut emu, limit) {
      emu.step_frames(BUTTON_START, 4);
      emu.step_frames(0, 4);
      if !wait_for_piece_motion(&mut emu, limit) {
        return Err("Tetris reached play mode but no piece is falling".to_string());
      }
    }
    let snapshot = emu.save_state();
    Ok(Reservoir { emu, snapshot, resets: 0 })
  }

  /// Gameplay, and not sitting on the game-over screen inside gameplay.
  pub fn in_play(&mut self) -> bool {
    self.emu.ram_read(TETRIS_GAME_MODE) == TETRIS_MODE_PLAY && self.emu.ram_read(TETRIS_PLAY_STATE) != TETRIS_PLAY_STATE_GAME_OVER
  }

  pub fn reset(&mut self) {
    self.emu.load_state(&self.snapshot);
  }

  /// Clear the spawn rows, then fill the rest of the field with one cell per
  /// window byte: a block where the byte is at or above the threshold, empty
  /// otherwise. Raw bytes would read as solid cells and top the game out.
  pub(crate) fn write_playfield(&mut self, window: &[u8]) {
    let spawn_cells = PLAYFIELD_SPAWN_ROWS * PLAYFIELD_COLS;
    let body_cells = (PLAYFIELD_ROWS - PLAYFIELD_SPAWN_ROWS) * PLAYFIELD_COLS;
    for i in 0..spawn_cells {
      self.emu.ram_write(TETRIS_PLAYFIELD + i as u16, PLAYFIELD_EMPTY);
    }
    for i in 0..body_cells {
      let cell = match window.get(i) {
        Some(&b) if b >= PLAYFIELD_THRESHOLD => PLAYFIELD_BLOCK,
        _ => PLAYFIELD_EMPTY,
      };
      self.emu.ram_write(TETRIS_PLAYFIELD + (spawn_cells + i) as u16, cell);
    }
  }

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

  fn reset_if_over(&mut self) -> bool {
    if self.in_play() {
      return false;
    }
    self.reset();
    self.resets += 1;
    true
  }

  /// Steer the falling piece to `target` by controller taps and let it lock.
  /// Resets on game over, before or after the piece.
  pub fn place(&mut self, target: Placement) -> PlaceOutcome {
    let mut frames = 0;
    let mut reset = false;
    // Wait for a falling piece.
    while frames < PIECE_FRAME_LIMIT && self.emu.ram_read(TETRIS_PLAY_STATE) != TETRIS_PLAY_STATE_FALLING {
      reset |= self.reset_if_over();
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
      if self.piece().2 != TETRIS_PLAY_STATE_FALLING {
        locked = true;
        break;
      }
      self.emu.step_frame(0);
      frames += 1;
    }
    // Wait through the lock sequence for the next spawn, so the board holds this piece.
    while frames < PIECE_FRAME_LIMIT && self.piece().2 != TETRIS_PLAY_STATE_FALLING {
      if self.reset_if_over() {
        reset = true;
        break;
      }
      self.emu.step_frame(0);
      frames += 1;
    }
    reset |= self.reset_if_over();
    PlaceOutcome { frames, locked, reset }
  }

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

  /// The whole of Tetris's work RAM.
  /// The Tetris screen, 256x240 RGB.
  pub fn framebuffer(&self) -> &[u8] {
    self.emu.framebuffer()
  }

  pub fn ram(&mut self) -> Vec<u8> {
    self.emu.ram_read_range(0, TETRIS_RAM_LEN)
  }
}

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

/// Step until the falling piece's Y changes, up to `limit` frames. True if it moved.
fn wait_for_piece_motion(emu: &mut NesEmulator, limit: usize) -> bool {
  let y0 = emu.ram_read(TETRIS_PIECE_Y);
  for _ in 0..limit {
    emu.step_frame(0);
    if emu.ram_read(TETRIS_PIECE_Y) != y0 {
      return true;
    }
  }
  false
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::bus::{BUTTON_SELECT, BUTTON_START};
  use crate::nes::gym::reservoir::rom_or_skip;

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

  #[test]
  #[ignore]
  fn place_steers_to_the_column_and_rotation_and_locks() {
    let rom = match rom_or_skip("roms2/tetris.nes") {
      Some(r) => r,
      None => return,
    };
    let mut r = Reservoir::new_fast(&rom).unwrap();
    let out = r.place(Placement { column: 1, rotation: 1 });
    assert!(out.locked, "piece should lock within {} frames", PIECE_FRAME_LIMIT);
    assert!(out.frames < PIECE_FRAME_LIMIT);
    assert!(!out.reset);
    // The locked cells sit in the low columns of the board.
    let board = r.emu.ram_read_range(TETRIS_PLAYFIELD, TETRIS_PLAYFIELD_LEN);
    let filled: Vec<usize> = board
      .iter()
      .enumerate()
      .filter(|(_, &c)| c != PLAYFIELD_EMPTY)
      .map(|(i, _)| i % 10)
      .collect();
    assert!(!filled.is_empty());
    assert!(filled.iter().all(|&c| c <= 3), "cells in columns {:?}", filled);
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
    assert!(
      cells(&after) >= cells(&before) + 12,
      "four locked pieces add at least sixteen cells minus any cleared line"
    );
  }

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
  fn game_over_inside_play_mode_counts_as_not_in_play() {
    let rom = match rom_or_skip("roms2/tetris.nes") {
      Some(r) => r,
      None => return,
    };
    let mut reservoir = Reservoir::new(&rom).expect("tetris should reach play");
    assert!(reservoir.in_play());
    // Fill the whole field, spawn rows included, so the next piece cannot spawn.
    for i in 0..TETRIS_PLAYFIELD_LEN {
      reservoir.emu.ram_write(TETRIS_PLAYFIELD + i, PLAYFIELD_BLOCK);
    }
    let mut saw_game_over = false;
    for _ in 0..300 {
      reservoir.emu.step_frames(0, 4);
      if reservoir.emu.ram_read(TETRIS_GAME_MODE) == TETRIS_MODE_PLAY && reservoir.emu.ram_read(TETRIS_PLAY_STATE) == TETRIS_PLAY_STATE_GAME_OVER {
        saw_game_over = true;
        break;
      }
    }
    assert!(saw_game_over, "a solid field should top the game out while game mode stays 4");
    assert!(!reservoir.in_play(), "game over is a play state inside game mode 4");
    reservoir.step(None, &[0; 256], 4);
    assert_eq!(reservoir.resets, 1);
    assert!(reservoir.in_play());
  }

  #[test]
  #[ignore]
  fn playfield_drive_writes_empty_or_block_cells_below_the_spawn_rows() {
    let rom = match rom_or_skip("roms2/tetris.nes") {
      Some(r) => r,
      None => return,
    };
    let mut reservoir = Reservoir::new(&rom).expect("tetris should reach play");
    let window: Vec<u8> = (0..=255).collect();
    reservoir.write_playfield(&window);
    let field = reservoir.emu.ram_read_range(TETRIS_PLAYFIELD, TETRIS_PLAYFIELD_LEN);
    let spawn_cells = PLAYFIELD_SPAWN_ROWS * PLAYFIELD_COLS;
    assert!(field[..spawn_cells].iter().all(|&b| b == PLAYFIELD_EMPTY), "spawn rows must stay clear");
    let body = &field[spawn_cells..];
    assert!(body.iter().all(|&b| b == PLAYFIELD_EMPTY || b == PLAYFIELD_BLOCK));
    let expected_blocks = window[..body.len()].iter().filter(|&&b| b >= PLAYFIELD_THRESHOLD).count();
    let blocks = body.iter().filter(|&&b| b == PLAYFIELD_BLOCK).count();
    assert_eq!(blocks, expected_blocks);
    assert_ne!(reservoir.emu.ram_read(TETRIS_PLAYFIELD + TETRIS_PLAYFIELD_LEN), PLAYFIELD_BLOCK);
  }

  #[test]
  #[ignore]
  fn playfield_drive_with_real_smb_windows_mostly_keeps_tetris_alive() {
    let (tetris, smb) = match (rom_or_skip("roms2/tetris.nes"), rom_or_skip("roms2/super_mario_bros.nes")) {
      (Some(t), Some(s)) => (t, s),
      _ => return,
    };
    use crate::nes::gym::reservoir::demo::{smb_start_state, smb_window};
    let mut mario = NesEmulator::new(&smb);
    mario.load_state(&smb_start_state(&smb));
    let mut reservoir = Reservoir::new(&tetris).expect("tetris should reach play");
    let steps = 200;
    for _ in 0..steps {
      let window = smb_window(&mut mario);
      reservoir.step(Some(Drive::Playfield), &window, 4);
      assert!(reservoir.in_play());
      assert_ne!(reservoir.emu.ram_read(TETRIS_PLAY_STATE), TETRIS_PLAY_STATE_GAME_OVER);
      mario.step_frames(crate::nes::bus::BUTTON_RIGHT, 4);
    }
    assert!(reservoir.resets < steps / 10, "reset {} times in {} steps", reservoir.resets, steps);
  }
}
