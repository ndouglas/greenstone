//! How SMB state enters Tetris, and the Tetris lifecycle as a reservoir.

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

fn piece_moves(emu: &mut NesEmulator, frames: usize) -> bool {
  let before = emu.ram_read(TETRIS_PIECE_Y);
  emu.step_frames(0, frames);
  emu.ram_read(TETRIS_PIECE_Y) != before
}

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
