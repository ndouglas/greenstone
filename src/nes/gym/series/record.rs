//! Record simulated play: every frame's RAM and buttons, and optionally the
//! screen, for the series' experiments. See `docs/series/games/001.md`.

use std::io::Write;
use std::path::{Path, PathBuf};

use serde_json::json;

use crate::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_LEFT, BUTTON_RIGHT};
use crate::nes::gym::reservoir::demo::{smb_window, MACRO_FRAMES};
use crate::nes::gym::reservoir::film::TakeWriter;
use crate::nes::gym::reservoir::teacher::{Player as RulePlayer, Rule};
use crate::nes::gym::reservoir::Lcg;
use crate::nes::gym::series::ram::Game;
use crate::nes::gym::NesEmulator;

/// Who presses the buttons.
pub trait Player {
  fn name(&self) -> String;
  fn buttons(&mut self, emu: &mut NesEmulator, frame: usize) -> u8;
}

/// Presses nothing.
pub struct Idle;

impl Player for Idle {
  fn name(&self) -> String {
    "idle".into()
  }
  fn buttons(&mut self, _: &mut NesEmulator, _: usize) -> u8 {
    0
  }
}

/// A rough stand-in for a person: picks a button combination, holds it for
/// a while, picks again. Combinations and their weights are per game.
pub struct HumanLike {
  seed: u64,
  rng: Lcg,
  choices: &'static [(u8, f64)],
  held: u8,
  left: usize,
}

/// Mario games: mostly running right, often jumping, sometimes idling or
/// turning back. Tuned so a few minutes of play jump often but not always.
const MARIO: &[(u8, f64)] = &[
  (BUTTON_RIGHT, 0.30),
  (BUTTON_RIGHT | BUTTON_A, 0.25),
  (BUTTON_RIGHT | BUTTON_B, 0.10),
  (BUTTON_RIGHT | BUTTON_A | BUTTON_B, 0.10),
  (0, 0.10),
  (BUTTON_LEFT, 0.05),
  (BUTTON_A, 0.10),
];

/// Castlevania: walking, jumping and whipping (B).
const CASTLEVANIA: &[(u8, f64)] = &[
  (BUTTON_RIGHT, 0.30),
  (BUTTON_RIGHT | BUTTON_A, 0.15),
  (BUTTON_A, 0.15),
  (BUTTON_B, 0.15),
  (0, 0.15),
  (BUTTON_LEFT, 0.10),
];

/// Hold lengths are 4 + a geometric number of frames with this mean.
const MEAN_EXTRA_HOLD: f64 = 16.0;

impl HumanLike {
  pub fn new(game: &Game, seed: u64) -> HumanLike {
    let choices = if game.name.contains("Mario") { MARIO } else { CASTLEVANIA };
    HumanLike {
      seed,
      rng: Lcg::new(seed),
      choices,
      held: 0,
      left: 0,
    }
  }
}

impl Player for HumanLike {
  fn name(&self) -> String {
    format!("human-like seed {}", self.seed)
  }

  fn buttons(&mut self, _: &mut NesEmulator, _: usize) -> u8 {
    if self.left == 0 {
      let total: f64 = self.choices.iter().map(|c| c.1).sum();
      let mut pick = self.rng.next_f64() * total;
      self.held = self.choices.last().unwrap().0;
      for &(b, w) in self.choices {
        if pick < w {
          self.held = b;
          break;
        }
        pick -= w;
      }
      let p = 1.0 / (MEAN_EXTRA_HOLD + 1.0);
      let extra = (self.rng.next_f64().max(1e-12).ln() / (1.0 - p).ln()).floor() as usize;
      self.left = 4 + extra;
    }
    self.left -= 1;
    self.held
  }
}

/// Super Mario Bros.' rule teacher from the reservoir experiments: runs
/// right and jumps at enemies, pits and walls. Decides every macro step.
pub struct RuleTeacher {
  player: RulePlayer,
  held: u8,
}

/// A rule from the reservoir experiments that clears 1-1.
pub const RULE: &str = "40,3,3,9";

impl RuleTeacher {
  pub fn new() -> RuleTeacher {
    RuleTeacher {
      player: RulePlayer::new(Rule::parse(RULE).unwrap()),
      held: 0,
    }
  }
}

impl Default for RuleTeacher {
  fn default() -> Self {
    Self::new()
  }
}

impl Player for RuleTeacher {
  fn name(&self) -> String {
    format!("rule teacher {}", RULE)
  }

  fn buttons(&mut self, emu: &mut NesEmulator, frame: usize) -> u8 {
    if frame % MACRO_FRAMES == 0 {
      self.held = self.player.buttons(&smb_window(emu));
    }
    self.held
  }
}

pub struct RecordSummary {
  pub frames: usize,
  /// The fraction of frames the answer key says the player is in the air.
  pub label_rate: f64,
  /// How many times play started over after a death.
  pub restarts: usize,
}

/// Play `frames` frames of `game` with `player`, writing `ram.u8` (2048 bytes
/// a frame), `buttons.u8` (one a frame), `meta.json` and, with `footage`, a
/// take of the screen. A death starts play over from the first frame of play.
pub fn record(game: &Game, rom: &[u8], player: &mut dyn Player, frames: usize, out: &Path, footage: bool) -> Result<RecordSummary, String> {
  std::fs::create_dir_all(out).map_err(|e| format!("create {}: {}", out.display(), e))?;
  let mut emu = game.boot(rom)?;
  let start = emu.save_state();
  let file = |name: &str| {
    std::fs::File::create(out.join(name))
      .map(std::io::BufWriter::new)
      .map_err(|e| e.to_string())
  };
  let (mut ram, mut buttons) = (file("ram.u8")?, file("buttons.u8")?);
  let mut take = if footage { Some(TakeWriter::new(out)?) } else { None };
  let (mut airborne, mut restarts) = (0usize, 0usize);
  let mut restart_frames = Vec::new();
  for f in 0..frames {
    let b = player.buttons(&mut emu, f);
    emu.step_frame(b);
    ram.write_all(&emu.ram_read_range(0x0000, 0x0800)).map_err(|e| e.to_string())?;
    buttons.write_all(&[b]).map_err(|e| e.to_string())?;
    airborne += game.airborne(&mut emu) as usize;
    if let Some(t) = take.as_mut() {
      t.frame("game", emu.framebuffer())?;
    }
    if game.died(&mut emu) {
      emu.load_state(&start);
      restarts += 1;
      restart_frames.push(f);
    }
  }
  ram.flush().map_err(|e| e.to_string())?;
  buttons.flush().map_err(|e| e.to_string())?;
  let summary = RecordSummary {
    frames,
    label_rate: airborne as f64 / frames.max(1) as f64,
    restarts,
  };
  let meta = json!({
    "game": game.name,
    "rom": game.rom,
    "sha1": game.sha1,
    "player": player.name(),
    "frames": frames,
    "airborne_byte": game.airborne_byte,
    "label_rate": summary.label_rate,
    "restarts": restarts,
    // The frames after which play started over (the next frame is a fresh start).
    "restart_frames": restart_frames,
  });
  std::fs::write(out.join("meta.json"), serde_json::to_vec_pretty(&meta).unwrap()).map_err(|e| e.to_string())?;
  if let Some(t) = take {
    t.finish(meta)?;
  }
  Ok(summary)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::reservoir::rom_or_skip;
  use crate::nes::gym::series::ram::{SMB1, SMB3};

  fn tmp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("series-record-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
  }

  #[test]
  fn the_same_seed_records_the_same_frames() {
    let Some(rom) = rom_or_skip(SMB1.rom) else { return };
    let (a, b, c) = (tmp("seed-a"), tmp("seed-b"), tmp("seed-c"));
    record(&SMB1, &rom, &mut HumanLike::new(&SMB1, 1), 600, &a, false).unwrap();
    record(&SMB1, &rom, &mut HumanLike::new(&SMB1, 1), 600, &b, false).unwrap();
    record(&SMB1, &rom, &mut HumanLike::new(&SMB1, 2), 600, &c, false).unwrap();
    let read = |d: &Path| std::fs::read(d.join("ram.u8")).unwrap();
    assert_eq!(read(&a), read(&b));
    assert_ne!(read(&a), read(&c));
  }

  #[test]
  fn ram_and_buttons_line_up() {
    let Some(rom) = rom_or_skip(SMB1.rom) else { return };
    let d = tmp("sizes");
    record(&SMB1, &rom, &mut HumanLike::new(&SMB1, 1), 300, &d, false).unwrap();
    assert_eq!(std::fs::metadata(d.join("ram.u8")).unwrap().len(), 300 * 2048);
    assert_eq!(std::fs::metadata(d.join("buttons.u8")).unwrap().len(), 300);
    let meta: serde_json::Value = serde_json::from_slice(&std::fs::read(d.join("meta.json")).unwrap()).unwrap();
    assert_eq!(meta["frames"], 300);
    assert_eq!(meta["sha1"], SMB1.sha1);
  }

  #[test]
  fn human_like_holds_buttons_for_a_while() {
    let Some(rom) = rom_or_skip(SMB1.rom) else { return };
    let mut emu = SMB1.boot(&rom).unwrap();
    let mut player = HumanLike::new(&SMB1, 7);
    let presses: Vec<u8> = (0..10_000).map(|f| player.buttons(&mut emu, f)).collect();
    let changes = presses.windows(2).filter(|w| w[0] != w[1]).count();
    let mean_hold = 10_000.0 / (changes + 1) as f64;
    assert!((5.0..=60.0).contains(&mean_hold), "mean hold {:.1} frames", mean_hold);
  }

  #[test]
  fn mario_jumps_often_enough() {
    let Some(rom) = rom_or_skip(SMB1.rom) else { return };
    let summary = record(&SMB1, &rom, &mut HumanLike::new(&SMB1, 3), 3600, &tmp("rate"), false).unwrap();
    assert!(
      (0.05..=0.95).contains(&summary.label_rate),
      "airborne {:.3} of frames",
      summary.label_rate
    );
  }

  #[test]
  fn a_death_restarts_play() {
    // In SMB3 the first Goomba walks into a Mario who stands still. (In SMB1
    // it never appears: a still Mario never scrolls the screen to it.)
    let Some(rom) = rom_or_skip(SMB3.rom) else { return };
    let d = tmp("death");
    let summary = record(&SMB3, &rom, &mut Idle, 1200, &d, false).unwrap();
    assert!(summary.restarts >= 1, "no restart in a minute of standing still");
    let meta: serde_json::Value = serde_json::from_slice(&std::fs::read(d.join("meta.json")).unwrap()).unwrap();
    assert_eq!(meta["restart_frames"].as_array().unwrap().len(), summary.restarts);
  }

  #[test]
  fn smb3_restarts_as_the_death_begins() {
    // $058C turns on the frame Mario 3's death begins and stays on through
    // the death animation; a restart should cut it off at once.
    let Some(rom) = rom_or_skip(SMB3.rom) else { return };
    let d = tmp("smb3-death");
    let summary = record(&SMB3, &rom, &mut Idle, 1200, &d, false).unwrap();
    let ram = std::fs::read(d.join("ram.u8")).unwrap();
    let dying = ram.chunks(2048).filter(|f| f[0x058C] != 0).count();
    assert!(summary.restarts >= 1);
    assert!(
      dying <= summary.restarts,
      "{} dying frames recorded for {} deaths",
      dying,
      summary.restarts
    );
  }

  #[test]
  fn the_rule_teacher_plays_mario() {
    let Some(rom) = rom_or_skip(SMB1.rom) else { return };
    let summary = record(&SMB1, &rom, &mut RuleTeacher::new(), 1800, &tmp("rule"), false).unwrap();
    assert!(
      summary.label_rate > 0.05,
      "the rule teacher jumped on {:.3} of frames",
      summary.label_rate
    );
  }
}
