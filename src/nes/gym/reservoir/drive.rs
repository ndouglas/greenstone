//! How SMB state enters Tetris, and the Tetris lifecycle as a reservoir.

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

  /// Apply the drive, advance `frames`, and restore the snapshot if the game ended.
  pub fn step(&mut self, drive: Option<Drive>, window: &[u8], frames: usize) {
    let buttons = match drive {
      Some(Drive::Controller) => fold_to_controller(window),
      Some(Drive::Playfield) => {
        self.write_playfield(window);
        0
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
  pub fn ram(&mut self) -> Vec<u8> {
    self.emu.ram_read_range(0, TETRIS_RAM_LEN)
  }
}

/// True if any consecutive pair of samples differs.
fn changed_at_any_frame(samples: &[u8]) -> bool {
  samples.windows(2).any(|w| w[0] != w[1])
}

/// Sample the falling piece's Y every frame for `frames` frames.
fn piece_moves(emu: &mut NesEmulator, frames: usize) -> bool {
  let mut samples = Vec::with_capacity(frames + 1);
  samples.push(emu.ram_read(TETRIS_PIECE_Y));
  for _ in 0..frames {
    emu.step_frame(0);
    samples.push(emu.ram_read(TETRIS_PIECE_Y));
  }
  changed_at_any_frame(&samples)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::bus::{BUTTON_SELECT, BUTTON_START};
  use crate::nes::gym::reservoir::rom_or_skip;

  #[test]
  fn liveness_sees_a_change_that_returns_to_the_starting_value() {
    assert!(!changed_at_any_frame(&[5, 5, 5, 5]));
    assert!(changed_at_any_frame(&[5, 6, 7, 5]));
    assert!(!changed_at_any_frame(&[]));
    assert!(!changed_at_any_frame(&[9]));
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
