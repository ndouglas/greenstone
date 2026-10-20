//! Replay a cleared level's path into a take the Python clip maker reads.

use crate::nes::gym::prospector::catalog::Entry;
use crate::nes::gym::prospector::funnel::level_start_at;
use crate::nes::gym::prospector::patch::patch_rom;
use crate::nes::gym::reservoir::demo::{Stage, MACRO_FRAMES};
use crate::nes::gym::reservoir::film::TakeWriter;
use crate::nes::gym::NesEmulator;
use serde_json::Value;
use std::path::Path;

/// Frames the level's first screen is held before Mario moves.
pub const INTRO_FRAMES: usize = 120;

/// Replay `entry`'s solved path from the level start the solver used, after
/// holding the first frame for `INTRO_FRAMES`. The take goes in
/// `out/<hex without $>/`, with `row` as its metadata.
pub fn record_clip(rom: &[u8], entry: &Entry, row: &Value, out: &Path) -> Result<usize, String> {
  let solve = entry.solve.as_ref().ok_or_else(|| format!("{} was never solved", entry.hex))?;
  let patched = patch_rom(rom, entry.offset);
  let (state, first) = level_start_at(&patched, Stage::ONE_ONE).ok_or_else(|| format!("{} never started", entry.hex))?;
  let mut emu = NesEmulator::new(&patched);
  emu.load_state(&state);
  let mut w = TakeWriter::new(&out.join(entry.hex.trim_start_matches('$')))?;
  for _ in 0..INTRO_FRAMES {
    w.frame("smb", &first)?;
  }
  let mut n = INTRO_FRAMES;
  for &buttons in &solve.path {
    for _ in 0..MACRO_FRAMES {
      emu.step_frame(buttons);
      w.frame("smb", emu.framebuffer())?;
      n += 1;
    }
  }
  let mut meta = row.clone();
  meta["offset"] = entry.offset.into();
  meta["hex"] = entry.hex.clone().into();
  meta["intro_frames"] = INTRO_FRAMES.into();
  w.finish(meta)?;
  Ok(n)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::prospector::catalog::{Class, SolveRecord};
  use crate::nes::gym::prospector::patch::ONE_ONE_AREA;
  use crate::nes::gym::reservoir::rom_or_skip;

  #[test]
  fn a_clip_is_the_intro_then_the_path() {
    let Some(rom) = rom_or_skip("roms2/super_mario_bros.nes") else { return };
    let mut e = Entry::new(ONE_ONE_AREA, Class::ClearedDistance);
    e.solve = Some(SolveRecord {
      clear: None,
      x_reached: 0,
      rounds: 1,
      panics: 0,
      bottleneck: 1,
      kill_rate: 0.0,
      path: vec![0x81; 10],
    });
    let dir = std::env::temp_dir().join(format!("prospector-clip-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let n = record_clip(&rom, &e, &serde_json::json!({"rank": 1}), &dir).unwrap();
    assert_eq!(n, INTRO_FRAMES + 10 * MACRO_FRAMES);
    let meta: Value = serde_json::from_slice(&std::fs::read(dir.join("A68E").join("take.json")).unwrap()).unwrap();
    assert_eq!(meta["streams"]["smb"], n);
    assert_eq!(meta["intro_frames"], INTRO_FRAMES);
    assert_eq!(meta["rank"], 1);
  }
}
