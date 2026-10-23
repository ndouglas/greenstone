//! The teacher: from a save state, try every move and see what follows.

use crate::nes::gym::punchout::moves::{moves, play, Move, Phase, MOVE_FRAMES};
use crate::nes::gym::punchout::ram::{mac_can_act, mac_down, stars};
use crate::nes::gym::NesEmulator;

/// Frames the teacher looks ahead: the move, then a continuation.
pub const HORIZON: usize = 60;
/// Moves in a continuation: the rest of the horizon after the first move.
const CONTINUATION: usize = (HORIZON - MOVE_FRAMES) / MOVE_FRAMES;

/// The lines the teacher tries after each first move.
pub fn continuations(phase: Phase, stars: u8) -> Vec<Vec<Move>> {
  let alternate = |first: Move| (0..CONTINUATION).map(|i| if i % 2 == 0 { first } else { Move::Idle }).collect();
  let mut lines = vec![vec![Move::Idle; CONTINUATION]];
  if phase != Phase::Defence {
    lines.push(alternate(Move::JabRight));
  }
  if phase == Phase::Fight && stars > 0 {
    lines.push(alternate(Move::Uppercut));
  }
  lines
}

/// How good a line was. Defence counts hits taken; the others weigh health
/// lost twice as much as health dealt.
pub fn score(phase: Phase, taken: u32, dealt: u32) -> i64 {
  match phase {
    Phase::Defence => -(taken as i64),
    Phase::Counter | Phase::Fight => dealt as i64 - 2 * taken as i64,
  }
}

/// Play `first` then `line` from the current state and score the result.
/// A line ends early when Mac can no longer act; in Defence a knockdown that
/// ends it counts as one more hit.
fn rollout(emu: &mut NesEmulator, phase: Phase, first: Move, line: &[Move]) -> i64 {
  let (mut hits, mut taken, mut dealt) = (0u32, 0u32, 0u32);
  for &mv in std::iter::once(&first).chain(line) {
    let outcome = play(emu, mv);
    hits += outcome.hit as u32;
    taken += outcome.damage_taken as u32;
    dealt += outcome.damage_dealt as u32;
    if !mac_can_act(emu) {
      hits += mac_down(emu) as u32;
      break;
    }
  }
  match phase {
    Phase::Defence => score(phase, hits, dealt),
    _ => score(phase, taken, dealt),
  }
}

/// The best move from here, leaving the emulator as it was.
pub fn label(emu: &mut NesEmulator, phase: Phase) -> Move {
  search(emu, phase).0
}

/// `label`, and how many rollouts it took.
fn search(emu: &mut NesEmulator, phase: Phase) -> (Move, usize) {
  let start = emu.save_state();
  let lines = continuations(phase, stars(emu));
  let mut best = (i64::MIN, Move::Idle);
  let mut rollouts = 0;
  for &first in moves(phase) {
    for line in &lines {
      emu.load_state(&start);
      let s = rollout(emu, phase, first, line);
      rollouts += 1;
      // Strictly better only: ties keep the earlier move, Idle first.
      if s > best.0 {
        best = (s, first);
      }
      // Defence scores are never above 0, so nothing later can beat a 0.
      if phase == Phase::Defence && best.0 == 0 {
        emu.load_state(&start);
        return (best.1, rollouts);
      }
    }
  }
  emu.load_state(&start);
  (best.1, rollouts)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::punchout::start::{fight_start, Opponent};
  use crate::nes::gym::punchout::PUNCH_OUT;
  use crate::nes::gym::reservoir::rom_or_skip;

  #[test]
  fn the_teacher_leaves_the_emulator_as_it_found_it() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let (state, _) = fight_start(&rom, Opponent::GlassJoe).unwrap();
    let mut emu = NesEmulator::new(&rom);
    emu.load_state(&state);
    emu.step_frames(0, 200);
    let before = emu.save_state();
    label(&mut emu, Phase::Defence);
    emu.step_frames(0, 30);
    let after_label = emu.framebuffer().to_vec();
    emu.load_state(&before);
    emu.step_frames(0, 30);
    assert_eq!(emu.framebuffer(), &after_label[..]);
  }

  #[test]
  fn the_teacher_dodges_before_a_hit_and_idles_otherwise() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let (state, _) = fight_start(&rom, Opponent::GlassJoe).unwrap();
    let mut emu = NesEmulator::new(&rom);
    emu.load_state(&state);
    // Walk an idle fight; remember the state a few steps before the first hit.
    let mut history = Vec::new();
    loop {
      history.push(emu.save_state());
      if play(&mut emu, Move::Idle).hit {
        break;
      }
      assert!(history.len() < 2000, "no hit in 2000 steps");
    }
    let before_hit = &history[history.len() - 3];
    emu.load_state(before_hit);
    assert_ne!(label(&mut emu, Phase::Defence), Move::Idle, "a punch is coming");
    emu.load_state(&history[0]);
    assert_eq!(label(&mut emu, Phase::Defence), Move::Idle, "nothing is coming at the bell");
  }

  #[test]
  fn defence_stops_searching_once_idle_takes_no_hit() {
    let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
    let (state, _) = fight_start(&rom, Opponent::GlassJoe).unwrap();
    let mut emu = NesEmulator::new(&rom);
    emu.load_state(&state);
    // Nothing is coming at the bell: Idle scores the best Defence can, 0.
    assert_eq!(search(&mut emu, Phase::Defence), (Move::Idle, 1));
    // Counter has no such ceiling, so it tries every move on every line.
    let full = moves(Phase::Counter).len() * continuations(Phase::Counter, stars(&mut emu)).len();
    assert_eq!(search(&mut emu, Phase::Counter).1, full);
  }

  #[test]
  fn scores_weigh_damage_taken_twice() {
    assert_eq!(score(Phase::Defence, 2, 50), -2);
    assert_eq!(score(Phase::Counter, 3, 10), 4);
    assert!(continuations(Phase::Fight, 1).len() > continuations(Phase::Fight, 0).len());
  }

  #[test]
  fn continuations_fill_the_horizon() {
    for phase in [Phase::Defence, Phase::Counter, Phase::Fight] {
      for line in continuations(phase, 1) {
        assert_eq!((1 + line.len()) * MOVE_FRAMES, HORIZON - HORIZON % MOVE_FRAMES);
        assert!(line.iter().all(|m| moves(phase).contains(m)), "{:?} uses a move it lacks", phase);
      }
    }
    assert_eq!(continuations(Phase::Defence, 1).len(), 1);
    assert_eq!(continuations(Phase::Counter, 1).len(), 2);
  }
}
