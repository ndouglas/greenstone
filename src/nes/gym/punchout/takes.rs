//! Film takes: one fight by a fitted brain, every screen frame kept.

use std::path::Path;

use serde_json::json;

use crate::nes::gym::punchout::brains::{Brain, BrainKind, Projection};
use crate::nes::gym::punchout::dagger::choose;
use crate::nes::gym::punchout::fight::{run_fight_watched, Step, Watched, MAX_STEPS};
use crate::nes::gym::punchout::moves::{Move, Phase};
use crate::nes::gym::punchout::start::Opponent;
use crate::nes::gym::reservoir::film::TakeWriter;
use crate::nes::gym::reservoir::ridge::Readout;

/// The stream a take writes the fight's screen to.
pub const STREAM: &str = "punchout";

/// Play one fight with `kind` and `readout` against `opponent`, writing the
/// screen at every frame and a metadata line per step to `out`. Returns the
/// number of steps.
pub fn record_take(
  rom: &[u8],
  tetris: &[u8],
  phase: Phase,
  kind: BrainKind,
  opponent: Opponent,
  readout: &Readout,
  out: &Path,
) -> Result<usize, String> {
  let projection = Projection::new();
  let mut brain = Brain::new(kind, tetris)?;
  let mut policy = |s: &mut Step| {
    let proj = projection.project(&s.emu.framebuffer_grayscale_84x84());
    let ram = s.emu.ram_read_range(0, 2048);
    choose(readout, &brain.row(&proj, &ram), phase)
  };
  take(rom, phase, &kind.name(), opponent, &mut policy, out)
}

/// A take of Mac standing still: the cold open.
pub fn record_idle_take(rom: &[u8], phase: Phase, opponent: Opponent, out: &Path) -> Result<usize, String> {
  take(rom, phase, "idle", opponent, &mut |_| Move::Idle, out)
}

fn take(rom: &[u8], phase: Phase, driver: &str, opponent: Opponent, policy: &mut dyn FnMut(&mut Step) -> Move, out: &Path) -> Result<usize, String> {
  let mut writer = TakeWriter::new(out)?;
  let (mut steps, mut error) = (vec![], None);
  let score = run_fight_watched(rom, opponent, MAX_STEPS, policy, &mut |w| match w {
    Watched::Frame(rgb) => {
      if let Err(e) = writer.frame(STREAM, rgb) {
        error.get_or_insert(e);
      }
    }
    Watched::Step {
      index,
      played,
      can_act,
      outcome,
      mac_health,
      opponent_health,
    } => steps.push(json!({
      "step": index,
      "move": played,
      "can_act": can_act,
      "mac_health": mac_health,
      "opponent_health": opponent_health,
      "hit": outcome.hit,
    })),
  })?;
  if let Some(e) = error {
    return Err(e);
  }
  let n = steps.len();
  writer.finish(json!({
    "kind": driver,
    "opponent": opponent.name(),
    "phase": phase.number(),
    "score": score,
    "hits_per_minute": score.hits_per_minute(),
    "survival_seconds": score.survival_seconds(),
    "steps": steps,
  }))?;
  Ok(n)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::punchout::brains::PROJ;
  use crate::nes::gym::punchout::moves::MOVE_FRAMES;
  use crate::nes::gym::punchout::PUNCH_OUT;
  use crate::nes::gym::reservoir::rom_or_skip;

  #[test]
  fn a_take_has_one_metadata_step_per_screen_frame_group() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let dir = std::env::temp_dir().join(format!("punchout-take-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let readout = Readout {
      d: 1 + PROJ,
      k: 4,
      weights: vec![0.0; (1 + PROJ) * 4],
      logistic: false,
    };
    let steps = record_take(&rom, &[], Phase::Defence, BrainKind::Stack(1), Opponent::GlassJoe, &readout, &dir).unwrap();
    let meta: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("take.json")).unwrap()).unwrap();
    assert_eq!(meta["steps"].as_array().unwrap().len(), steps);
    assert_eq!(meta["streams"][STREAM].as_u64().unwrap() as usize, steps * MOVE_FRAMES);
    assert_eq!(meta["score"]["steps"].as_u64().unwrap() as usize, steps);
    let _ = std::fs::remove_dir_all(&dir);
  }
}
