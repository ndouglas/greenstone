//! Film takes: one fight by a fitted brain, every screen frame kept.

use std::cell::RefCell;
use std::path::Path;

use serde_json::{json, Value};

use crate::nes::gym::punchout::brains::{Brain, BrainKind, Projection};
use crate::nes::gym::punchout::dagger::choose;
use crate::nes::gym::punchout::fight::{run_fight_watched, Step, Watched, MAX_STEPS};
use crate::nes::gym::punchout::moves::{moves, Move, Phase};
use crate::nes::gym::punchout::start::Opponent;
use crate::nes::gym::reservoir::film::TakeWriter;
use crate::nes::gym::reservoir::ridge::Readout;

/// The stream a take writes the fight's screen to.
pub const STREAM: &str = "punchout";

/// Play one fight with `kind` and `readout` against `opponent` for at most
/// `max_steps` steps, writing the screen at every frame and a metadata line
/// per step to `out`: the move, health, and the brain's insides (the
/// readout's score for each move, the projection, and the bank's boards).
/// Returns the number of steps.
#[allow(clippy::too_many_arguments)]
pub fn record_take(
  rom: &[u8],
  tetris: &[u8],
  phase: Phase,
  kind: BrainKind,
  opponent: Opponent,
  readout: &Readout,
  max_steps: usize,
  out: &Path,
) -> Result<usize, String> {
  let projection = Projection::new();
  let mut brain = Brain::new(kind, tetris)?;
  let mut policy = |s: &mut Step| {
    let proj = projection.project(&s.emu.framebuffer_grayscale_84x84());
    let ram = s.emu.ram_read_range(0, 2048);
    let row = brain.row(&proj, &ram);
    let scores = readout.predict(&row);
    let mut insides = json!({
      "scores": scores,
      "proj": proj.iter().map(|&p| (p * 1000.0).round() / 1000.0).collect::<Vec<f32>>(),
    });
    if let Some(boards) = brain.boards() {
      insides["boards"] = json!(boards);
    }
    (choose(readout, &row, phase), insides)
  };
  take(rom, phase, &kind.name(), opponent, max_steps, &mut policy, out)
}

/// A take of Mac standing still: the cold open.
pub fn record_idle_take(rom: &[u8], phase: Phase, opponent: Opponent, out: &Path) -> Result<usize, String> {
  take(rom, phase, "idle", opponent, MAX_STEPS, &mut |_| (Move::Idle, Value::Null), out)
}

/// One fight with `policy`, which answers each step with a move and any
/// insides to keep with the step (an object, or null for none).
fn take(
  rom: &[u8],
  phase: Phase,
  driver: &str,
  opponent: Opponent,
  max_steps: usize,
  policy: &mut dyn FnMut(&mut Step) -> (Move, Value),
  out: &Path,
) -> Result<usize, String> {
  let mut writer = TakeWriter::new(out)?;
  let (mut steps, mut error) = (vec![], None);
  // The policy runs before its step is watched, so its insides wait here.
  let pending = RefCell::new(Value::Null);
  let mut chosen = |s: &mut Step| {
    let (m, insides) = policy(s);
    *pending.borrow_mut() = insides;
    m
  };
  let score = run_fight_watched(rom, opponent, max_steps, &mut chosen, &mut |w| match w {
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
    } => {
      let mut step = json!({
        "step": index,
        "move": played,
        "can_act": can_act,
        "mac_health": mac_health,
        "opponent_health": opponent_health,
        "hit": outcome.hit,
      });
      if let Value::Object(insides) = pending.replace(Value::Null) {
        step.as_object_mut().unwrap().extend(insides);
      }
      steps.push(step);
    }
  })?;
  if let Some(e) = error {
    return Err(e);
  }
  let n = steps.len();
  writer.finish(json!({
    "kind": driver,
    "opponent": opponent.name(),
    "phase": phase.number(),
    "moves": moves(phase),
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
    let steps = record_take(
      &rom,
      &[],
      Phase::Defence,
      BrainKind::Stack(1),
      Opponent::GlassJoe,
      &readout,
      MAX_STEPS,
      &dir,
    )
    .unwrap();
    let meta: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("take.json")).unwrap()).unwrap();
    assert_eq!(meta["steps"].as_array().unwrap().len(), steps);
    assert_eq!(meta["streams"][STREAM].as_u64().unwrap() as usize, steps * MOVE_FRAMES);
    assert_eq!(meta["score"]["steps"].as_u64().unwrap() as usize, steps);
    let _ = std::fs::remove_dir_all(&dir);
  }

  fn zero_readout(d: usize, phase: Phase) -> Readout {
    let k = moves(phase).len();
    Readout {
      d,
      k,
      weights: vec![0.0; d * k],
      logistic: false,
    }
  }

  fn read_meta(dir: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(dir.join("take.json")).unwrap()).unwrap()
  }

  #[test]
  fn a_take_records_scores_and_projection_per_step() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let dir = std::env::temp_dir().join(format!("punchout-take-insides-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let readout = zero_readout(1 + PROJ, Phase::Defence);
    record_take(&rom, &[], Phase::Defence, BrainKind::Stack(1), Opponent::GlassJoe, &readout, 3, &dir).unwrap();
    let meta = read_meta(&dir);
    for step in meta["steps"].as_array().unwrap() {
      assert_eq!(step["scores"].as_array().unwrap().len(), moves(Phase::Defence).len());
      assert_eq!(step["proj"].as_array().unwrap().len(), PROJ);
      assert!(step.get("boards").is_none());
    }
    assert_eq!(meta["moves"][0], "Idle");
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn a_bank_take_records_four_boards_per_step() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let Some(tetris) = rom_or_skip("roms2/tetris.nes") else { return };
    let dir = std::env::temp_dir().join(format!("punchout-take-bank-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let d = Brain::new(BrainKind::Bank, &tetris).unwrap().row(&vec![0.0; PROJ], &[]).len();
    let readout = zero_readout(d, Phase::Defence);
    record_take(&rom, &tetris, Phase::Defence, BrainKind::Bank, Opponent::GlassJoe, &readout, 3, &dir).unwrap();
    let meta = read_meta(&dir);
    let boards = meta["steps"][0]["boards"].as_array().unwrap();
    assert_eq!(boards.len(), 4);
    assert!(boards.iter().all(|b| b.as_array().unwrap().len() == 200));
    let _ = std::fs::remove_dir_all(&dir);
  }
}
