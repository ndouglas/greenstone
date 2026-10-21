//! The mapper gate: does Punch-Out!! boot and draw?

use std::path::Path;

use crate::nes::bus::BUTTON_START;
use crate::nes::gym::NesEmulator;

/// Write a 256x240 RGB frame as a binary PPM.
pub fn write_ppm(path: &Path, rgb: &[u8]) -> Result<(), String> {
  let mut bytes = b"P6\n256 240\n255\n".to_vec();
  bytes.extend_from_slice(rgb);
  std::fs::write(path, bytes).map_err(|e| e.to_string())
}

/// Boot the ROM and keep the frames numbered in `frames`, tapping Start for
/// four frames every `press_start_every` frames so the game leaves its title
/// and intro screens.
pub fn boot_strip(rom: &[u8], frames: &[usize], press_start_every: usize) -> Vec<(usize, Vec<u8>)> {
  let mut emu = NesEmulator::new(rom);
  let last = frames.iter().copied().max().unwrap_or(0);
  let mut out = Vec::new();
  for f in 1..=last {
    let buttons = if press_start_every > 0 && f % press_start_every < 4 {
      BUTTON_START
    } else {
      0
    };
    emu.step_frames(buttons, 1);
    if frames.contains(&f) {
      out.push((f, emu.framebuffer().to_vec()));
    }
  }
  out
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::reservoir::rom_or_skip;

  #[test]
  fn punch_out_boots_and_draws_more_than_one_colour() {
    let Some(rom) = rom_or_skip(super::super::PUNCH_OUT) else { return };
    let strip = boot_strip(&rom, &[300, 1800], 240);
    assert_eq!(strip.len(), 2);
    for (frame, rgb) in &strip {
      assert_eq!(rgb.len(), 256 * 240 * 3);
      let first = &rgb[0..3];
      assert!(rgb.chunks(3).any(|p| p != first), "frame {} is one flat colour", frame);
    }
  }
}
