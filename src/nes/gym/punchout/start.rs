//! Starting a fight against any opponent.

use serde::{Deserialize, Serialize};

use crate::nes::bus::BUTTON_START;
use crate::nes::gym::punchout::ram::{mac_can_act, FIGHT_NUMBER};
use crate::nes::gym::{NesEmulator, NesState};

/// Boot taps Start for four frames in every 240, as the gate does.
const START_EVERY: usize = 240;
/// The frame, counted from power-on, at which the fight number is written:
/// after the title's own use of it and before the "vs" screen reads it at 948.
const PICK_FRAME: usize = 940;
/// How long a boot may take to reach a frame where Mac can act.
const MAX_BOOT_FRAMES: usize = 6000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Opponent {
  GlassJoe,
  VonKaiser,
  PistonHonda,
  DonFlamenco,
  BaldBull,
}

/// The opponents the brains train on.
pub const TRAIN: [Opponent; 3] = [Opponent::GlassJoe, Opponent::VonKaiser, Opponent::PistonHonda];
/// The opponents the brains never see until the test.
pub const HELD_OUT: [Opponent; 2] = [Opponent::DonFlamenco, Opponent::BaldBull];

impl Opponent {
  pub fn name(self) -> &'static str {
    match self {
      Opponent::GlassJoe => "glass-joe",
      Opponent::VonKaiser => "von-kaiser",
      Opponent::PistonHonda => "piston-honda",
      Opponent::DonFlamenco => "don-flamenco",
      Opponent::BaldBull => "bald-bull",
    }
  }

  /// The game's fight number for this opponent's first fight.
  fn fight_number(self) -> u8 {
    match self {
      Opponent::GlassJoe => 0,
      Opponent::VonKaiser => 1,
      Opponent::PistonHonda => 2,
      Opponent::DonFlamenco => 3,
      Opponent::BaldBull => 6,
    }
  }
}

/// The state at the first frame Mac can act against `opponent`, and that
/// frame's number counted from power-on.
pub fn fight_start(rom: &[u8], opponent: Opponent) -> Result<(NesState, usize), String> {
  let mut emu = NesEmulator::new(rom);
  for frame in 1..=MAX_BOOT_FRAMES {
    if frame == PICK_FRAME {
      emu.ram_write(FIGHT_NUMBER, opponent.fight_number());
    }
    emu.step_frames(if frame % START_EVERY < 4 { BUTTON_START } else { 0 }, 1);
    if frame > PICK_FRAME && mac_can_act(&mut emu) {
      return Ok((emu.save_state(), frame));
    }
  }
  Err(format!("{} never reached a fight", opponent.name()))
}
