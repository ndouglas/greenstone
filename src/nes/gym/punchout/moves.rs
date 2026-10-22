//! What Mac can do, phase by phase, and what one move does to the fight.

use serde::{Deserialize, Serialize};

use crate::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT, BUTTON_START, BUTTON_UP};
use crate::nes::gym::punchout::ram::{mac_health, opponent_health};
use crate::nes::gym::NesEmulator;

/// The frames one move holds its buttons for.
pub const MOVE_FRAMES: usize = 4;

/// The three phases of the experiment: dodge only; dodge and jab; everything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
  Defence,
  Counter,
  Fight,
}

impl Phase {
  pub fn number(self) -> u8 {
    match self {
      Phase::Defence => 1,
      Phase::Counter => 2,
      Phase::Fight => 3,
    }
  }

  pub fn from_number(n: u8) -> Option<Phase> {
    match n {
      1 => Some(Phase::Defence),
      2 => Some(Phase::Counter),
      3 => Some(Phase::Fight),
      _ => None,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Move {
  Idle,
  DodgeLeft,
  DodgeRight,
  Duck,
  JabLeft,
  JabRight,
  Uppercut,
}

impl Move {
  pub fn buttons(self) -> u8 {
    match self {
      Move::Idle => 0,
      Move::DodgeLeft => BUTTON_LEFT,
      Move::DodgeRight => BUTTON_RIGHT,
      // Held, Down is the block; the game ducks on a double tap.
      Move::Duck => BUTTON_DOWN,
      Move::JabLeft => BUTTON_UP | BUTTON_B,
      Move::JabRight => BUTTON_UP | BUTTON_A,
      Move::Uppercut => BUTTON_START,
    }
  }

  /// This move's index among `moves(phase)`.
  pub fn index_in(self, phase: Phase) -> usize {
    moves(phase)
      .iter()
      .position(|&m| m == self)
      .unwrap_or_else(|| panic!("{:?} is not a {:?} move", self, phase))
  }
}

/// The moves a phase allows; each phase's list starts with the previous one's.
pub fn moves(phase: Phase) -> &'static [Move] {
  let n = match phase {
    Phase::Defence => 4,
    Phase::Counter => 6,
    Phase::Fight => 7,
  };
  &ALL_MOVES[..n]
}

const ALL_MOVES: [Move; 7] = [
  Move::Idle,
  Move::DodgeLeft,
  Move::DodgeRight,
  Move::Duck,
  Move::JabLeft,
  Move::JabRight,
  Move::Uppercut,
];

/// What one move did, measured across its frames.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Outcome {
  pub hit: bool,
  pub damage_taken: u8,
  pub damage_dealt: u8,
}

/// Hold `mv`'s buttons for `MOVE_FRAMES` frames.
pub fn play(emu: &mut NesEmulator, mv: Move) -> Outcome {
  let (mac, opponent) = (mac_health(emu), opponent_health(emu));
  emu.step_frames(mv.buttons(), MOVE_FRAMES);
  // Health refills when a fighter gets up, so only falls count.
  let damage_taken = mac.saturating_sub(mac_health(emu));
  let damage_dealt = opponent.saturating_sub(opponent_health(emu));
  Outcome {
    hit: damage_taken > 0,
    damage_taken,
    damage_dealt,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn moves_grow_by_phase_and_keep_their_indices() {
    assert_eq!(moves(Phase::Defence).len(), 4);
    assert_eq!(moves(Phase::Counter).len(), 6);
    assert_eq!(moves(Phase::Fight).len(), 7);
    for p in [Phase::Defence, Phase::Counter, Phase::Fight] {
      for (i, m) in moves(p).iter().enumerate() {
        assert_eq!(m.index_in(p), i);
      }
    }
    assert_eq!(&moves(Phase::Fight)[..4], moves(Phase::Defence));
  }

  #[test]
  fn phases_round_trip_through_their_numbers() {
    for p in [Phase::Defence, Phase::Counter, Phase::Fight] {
      assert_eq!(Phase::from_number(p.number()), Some(p));
    }
    assert_eq!(Phase::from_number(0), None);
    assert_eq!(Phase::from_number(4), None);
  }
}
