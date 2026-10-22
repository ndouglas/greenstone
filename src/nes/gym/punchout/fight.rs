//! One fight, start to finish, played by a policy, and its score.

use serde::{Deserialize, Serialize};

use crate::nes::bus::{BUTTON_A, BUTTON_START};
use crate::nes::gym::punchout::moves::{play, Move, MOVE_FRAMES};
use crate::nes::gym::punchout::ram::{fight_over, mac_can_act, mac_down, opponent_down, round_over};
use crate::nes::gym::punchout::start::{fight_start, Opponent};
use crate::nes::gym::reservoir::demo::guarded;
use crate::nes::gym::NesEmulator;

/// The most moves a fight lasts: 9,600 frames, under three minutes.
pub const MAX_STEPS: usize = 2400;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Score {
  pub steps: usize,
  pub active_steps: usize,
  pub hits: usize,
  pub damage_taken: u32,
  pub damage_dealt: u32,
  pub knockdowns_dealt: u32,
  pub mac_down_at: Option<usize>,
  pub won: bool,
}

impl Score {
  /// Hits Mac took per minute of play in which he could act.
  pub fn hits_per_minute(&self) -> f64 {
    let minutes = (self.active_steps * MOVE_FRAMES) as f64 / 3600.0;
    if minutes > 0.0 {
      self.hits as f64 / minutes
    } else {
      0.0
    }
  }

  /// Seconds until Mac first went down, or the whole fight if he never did.
  pub fn survival_seconds(&self) -> f64 {
    (self.mac_down_at.unwrap_or(self.steps) * MOVE_FRAMES) as f64 / 60.0
  }
}

/// What the policy sees at each step.
pub struct Step<'a> {
  pub index: usize,
  pub emu: &'a mut NesEmulator,
  pub can_act: bool,
}

/// Fight `opponent` from the first frame Mac can act until the fight is over
/// or `MAX_STEPS` moves have passed. The policy is asked at every step, so a
/// brain with memory sees every frame; when Mac cannot act, Idle is played
/// whatever it answers (or the loop's own taps, between rounds and on the
/// canvas).
pub fn run_fight(rom: &[u8], opponent: Opponent, policy: &mut dyn FnMut(&mut Step) -> Move) -> Result<Score, String> {
  guarded(|| {
    let (state, _) = fight_start(rom, opponent)?;
    let mut emu = NesEmulator::new(rom);
    emu.load_state(&state);
    let mut score = Score::default();
    let mut was_down = false;
    for index in 0..MAX_STEPS {
      if fight_over(&mut emu) {
        break;
      }
      let can_act = mac_can_act(&mut emu);
      let chosen = policy(&mut Step {
        index,
        emu: &mut emu,
        can_act,
      });
      // Between rounds the game waits for Start, and a downed Mac gets up by
      // mashing A: neither is a skill the brains learn, so the loop taps for
      // them, releasing on alternate steps.
      let tap = if can_act {
        None
      } else if round_over(&mut emu) {
        Some(BUTTON_START)
      } else if mac_down(&mut emu) {
        Some(BUTTON_A)
      } else {
        None
      };
      let outcome = if let Some(button) = tap {
        emu.step_frames(if index % 2 == 0 { button } else { 0 }, MOVE_FRAMES);
        Default::default()
      } else {
        play(&mut emu, if can_act { chosen } else { Move::Idle })
      };
      score.steps += 1;
      score.active_steps += can_act as usize;
      score.hits += outcome.hit as usize;
      score.damage_taken += outcome.damage_taken as u32;
      score.damage_dealt += outcome.damage_dealt as u32;
      let down = opponent_down(&mut emu);
      score.knockdowns_dealt += (down && !was_down) as u32;
      was_down = down;
      if score.mac_down_at.is_none() && mac_down(&mut emu) {
        score.mac_down_at = Some(index);
      }
    }
    score.won = fight_over(&mut emu) && opponent_down(&mut emu);
    Ok(score)
  })?
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::punchout::PUNCH_OUT;
  use crate::nes::gym::reservoir::rom_or_skip;

  #[test]
  fn always_idle_gets_hit_and_scores_count_only_active_steps() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let mut asked_inactive = 0;
    let score = run_fight(&rom, Opponent::GlassJoe, &mut |s| {
      if !s.can_act {
        asked_inactive += 1;
      }
      Move::Idle
    })
    .unwrap();
    assert!(score.hits > 0);
    assert!(score.active_steps < score.steps, "knockdowns and intros are not active");
    assert!(asked_inactive > 0, "the policy still sees inactive steps");
    assert!(score.hits_per_minute() > 0.0);
  }

  #[test]
  fn holding_a_move_changes_the_fight() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let idle = run_fight(&rom, Opponent::GlassJoe, &mut |_| Move::Idle).unwrap();
    let block = run_fight(&rom, Opponent::GlassJoe, &mut |_| Move::Duck).unwrap();
    assert_ne!(idle.hits_per_minute(), block.hits_per_minute(), "moves change the fight");
  }

  #[test]
  fn idle_mac_goes_down_and_loses() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let score = run_fight(&rom, Opponent::GlassJoe, &mut |_| Move::Idle).unwrap();
    assert!(score.mac_down_at.is_some() && !score.won);
    assert_eq!(score.knockdowns_dealt, 0);
    assert!(score.survival_seconds() < (score.steps * MOVE_FRAMES) as f64 / 60.0);
  }

  /// Don Flamenco never attacks a Mac who never punches, so idle against him
  /// lasts past round one's bell; the loop must press on into round two.
  #[test]
  fn the_loop_carries_on_past_the_bell() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let score = run_fight(&rom, Opponent::DonFlamenco, &mut |_| Move::Idle).unwrap();
    // The game's clock ticks a second every 20 frames: three minutes is 3,600.
    let round = 3 * 60 * 20 / MOVE_FRAMES;
    assert!(score.active_steps > round + 100, "active {} of {}", score.active_steps, score.steps);
  }

  /// Getting up is mashing, not a skill the brains learn: the loop mashes for
  /// them, so a knockdown need not end the fight.
  #[test]
  fn mac_gets_up_after_a_knockdown() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let (mut was_down, mut got_up) = (false, false);
    run_fight(&rom, Opponent::GlassJoe, &mut |s| {
      was_down |= mac_down(s.emu);
      got_up |= was_down && s.can_act;
      Move::Idle
    })
    .unwrap();
    assert!(was_down && got_up, "down {} up {}", was_down, got_up);
  }

  #[test]
  fn a_policy_that_panics_is_an_error_not_a_crash() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let result = run_fight(&rom, Opponent::GlassJoe, &mut |s| {
      if s.index == 10 {
        panic!("brain fell over");
      }
      Move::Idle
    });
    assert!(result.unwrap_err().contains("brain fell over"));
  }
}
