//! A demonstration for SMB 1-1 found by beam search over save states.

use super::features::{SMB_WINDOW_LEN, SMB_WINDOW_START};
use crate::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_RIGHT, BUTTON_START};
use crate::nes::gym::reward::smb1_ram;
use crate::nes::gym::NesEmulator;

/// Frames per step, everywhere in the experiment.
pub const MACRO_FRAMES: usize = 4;

/// Actions the demonstration may take.
pub const ACTIONS: [u8; 6] = [
  BUTTON_RIGHT,
  BUTTON_RIGHT | BUTTON_A,
  BUTTON_RIGHT | BUTTON_B,
  BUTTON_RIGHT | BUTTON_A | BUTTON_B,
  BUTTON_A,
  0,
];

pub fn smb_x(emu: &mut NesEmulator) -> i32 {
  emu.ram_read(smb1_ram::PLAYER_X_PAGE) as i32 * 256 + emu.ram_read(smb1_ram::PLAYER_X_PIXEL) as i32
}

/// Player state `$000E` is `0x0B` while dying and `0x06` on the final death frame.
/// This fires about 200 frames before the lives counter drops.
pub fn smb_dead(emu: &mut NesEmulator) -> bool {
  matches!(emu.ram_read(smb1_ram::PLAYER_STATE), 0x06 | 0x0B)
}

/// True once the level counter has advanced past 1-1.
pub fn smb_cleared(emu: &mut NesEmulator) -> bool {
  emu.ram_read(smb1_ram::LEVEL) != 0 || emu.ram_read(smb1_ram::WORLD) != 0
}

pub fn smb_window(emu: &mut NesEmulator) -> Vec<u8> {
  emu.ram_read_range(SMB_WINDOW_START, SMB_WINDOW_LEN)
}

/// Boot SMB, tap Start through the title, wait for 1-1 to begin, and snapshot.
pub fn smb_start_state(rom: &[u8]) -> Vec<u8> {
  let mut emu = NesEmulator::new(rom);
  for frame in 0..120 {
    emu.step_frame(if frame % 20 < 10 { BUTTON_START } else { 0 });
  }
  emu.step_frames(0, 80);
  emu.save_state()
}

/// The best path the search found.
#[derive(Debug, Clone)]
pub struct Demonstration {
  /// One button byte per macro step.
  pub buttons: Vec<u8>,
  pub x_reached: i32,
  pub cleared: bool,
}

#[derive(Debug, Clone)]
pub struct Candidate {
  pub state: Vec<u8>,
  pub path: Vec<u8>,
  pub x: i32,
  pub dead: bool,
  pub cleared: bool,
}

/// Drop dead candidates, order by x descending, keep `beam`.
pub fn prune_and_rank(mut cands: Vec<Candidate>, beam: usize) -> Vec<Candidate> {
  cands.retain(|c| !c.dead);
  cands.sort_by_key(|c| std::cmp::Reverse(c.x));
  cands.truncate(beam);
  cands
}

/// Beam search over save states. Each step expands every frontier candidate by
/// every action across threads, one emulator per thread.
pub fn beam_search(rom: &[u8], start: &[u8], beam: usize, max_steps: usize) -> Demonstration {
  let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
  let lives_at_start = {
    let mut emu = NesEmulator::new(rom);
    emu.load_state(start);
    emu.ram_read(smb1_ram::LIVES)
  };
  let mut frontier = vec![Candidate {
    state: start.to_vec(),
    path: Vec::new(),
    x: 0,
    dead: false,
    cleared: false,
  }];
  let mut best = frontier[0].clone();
  for _ in 0..max_steps {
    let jobs: Vec<(usize, u8)> = (0..frontier.len()).flat_map(|p| ACTIONS.iter().map(move |&a| (p, a))).collect();
    let chunk = jobs.len().div_ceil(threads);
    let expanded: Vec<Candidate> = std::thread::scope(|s| {
      let handles: Vec<_> = jobs
        .chunks(chunk)
        .map(|jobs| {
          let frontier = &frontier;
          s.spawn(move || {
            let mut emu = NesEmulator::new(rom);
            jobs
              .iter()
              .map(|&(parent, action)| {
                emu.load_state(&frontier[parent].state);
                emu.step_frames(action, MACRO_FRAMES);
                let mut path = frontier[parent].path.clone();
                path.push(action);
                let dead = smb_dead(&mut emu) || emu.ram_read(smb1_ram::LIVES) < lives_at_start;
                Candidate {
                  x: smb_x(&mut emu),
                  dead,
                  cleared: smb_cleared(&mut emu),
                  state: emu.save_state(),
                  path,
                }
              })
              .collect::<Vec<_>>()
          })
        })
        .collect();
      handles.into_iter().flat_map(|h| h.join().expect("beam worker panicked")).collect()
    });
    let survivors = prune_and_rank(expanded, beam);
    if survivors.is_empty() {
      break;
    }
    if survivors[0].cleared || survivors[0].x > best.x {
      best = survivors[0].clone();
    }
    if best.cleared {
      break;
    }
    frontier = survivors;
  }
  Demonstration {
    buttons: best.path,
    x_reached: best.x,
    cleared: best.cleared,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::reservoir::rom_or_skip;

  fn cand(x: i32, dead: bool) -> Candidate {
    Candidate {
      state: vec![],
      path: vec![],
      x,
      dead,
      cleared: false,
    }
  }

  #[test]
  fn prune_drops_dead_and_keeps_best_by_x() {
    let cands = vec![cand(10, false), cand(50, true), cand(30, false), cand(20, false)];
    let kept = prune_and_rank(cands, 2);
    let xs: Vec<i32> = kept.iter().map(|c| c.x).collect();
    assert_eq!(xs, vec![30, 20]);
  }

  #[test]
  fn prune_returns_empty_when_everything_is_dead() {
    let cands = vec![cand(10, true), cand(50, true)];
    assert!(prune_and_rank(cands, 4).is_empty());
  }

  #[test]
  fn actions_never_include_start_or_select() {
    use crate::nes::bus::{BUTTON_SELECT, BUTTON_START};
    for a in ACTIONS {
      assert_eq!(a & (BUTTON_START | BUTTON_SELECT), 0);
    }
  }

  #[test]
  #[ignore]
  fn start_state_is_at_the_beginning_of_1_1() {
    let rom = match rom_or_skip("roms2/super_mario_bros.nes") {
      Some(r) => r,
      None => return,
    };
    let start = smb_start_state(&rom);
    let mut emu = NesEmulator::new(&rom);
    emu.load_state(&start);
    assert_eq!(smb_x(&mut emu), 0);
    assert!(!smb_dead(&mut emu));
    assert!(!smb_cleared(&mut emu));
    emu.step_frames(crate::nes::bus::BUTTON_RIGHT, 60);
    assert!(smb_x(&mut emu) > 20, "Mario should move right from the start state");
  }

  #[test]
  #[ignore]
  fn beam_search_makes_progress_in_1_1() {
    let rom = match rom_or_skip("roms2/super_mario_bros.nes") {
      Some(r) => r,
      None => return,
    };
    let start = smb_start_state(&rom);
    let demo = beam_search(&rom, &start, 8, 150);
    assert!(demo.buttons.len() <= 150);
    assert!(demo.x_reached > 500, "reached only x={}", demo.x_reached);
  }

  #[test]
  #[ignore]
  fn death_is_detected_well_before_the_lives_counter_drops() {
    let rom = match rom_or_skip("roms2/super_mario_bros.nes") {
      Some(r) => r,
      None => return,
    };
    let mut emu = NesEmulator::new(&rom);
    emu.load_state(&smb_start_state(&rom));
    let lives_at_start = emu.ram_read(smb1_ram::LIVES);
    let mut dead_frame = None;
    let mut lives_frame = None;
    for frame in 0..700 {
      emu.step_frame(crate::nes::bus::BUTTON_RIGHT);
      if dead_frame.is_none() && smb_dead(&mut emu) {
        dead_frame = Some(frame);
      }
      if lives_frame.is_none() && emu.ram_read(smb1_ram::LIVES) < lives_at_start {
        lives_frame = Some(frame);
      }
    }
    let lives_frame = lives_frame.expect("walking right into the first Goomba should cost a life");
    let dead_frame = dead_frame.expect("smb_dead should fire during the death animation");
    assert!(
      dead_frame + 60 < lives_frame,
      "dead at frame {}, lives dropped at frame {}",
      dead_frame,
      lives_frame
    );
  }
}
