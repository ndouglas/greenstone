//! A demonstration for SMB 1-1 found by beam search over save states.

use super::features::{SMB_WINDOW_LEN, SMB_WINDOW_START};
use crate::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_RIGHT, BUTTON_START};
use crate::nes::gym::reward::smb1_ram;
use crate::nes::gym::NesEmulator;

/// Frames per step, everywhere in the experiment.
pub const MACRO_FRAMES: usize = 4;

/// A candidate whose x has not improved for this many rounds is dropped.
pub const STUCK_LIMIT: u32 = 3;

/// SMB RAM: player Y on screen. Smaller is higher.
const PLAYER_Y: u16 = 0x00CE;

/// Macro steps per search round. Each round commits to one macro below.
pub const ROUND_STEPS: usize = 6;

const RUN: u8 = BUTTON_RIGHT | BUTTON_B;
const RUN_JUMP: u8 = BUTTON_RIGHT | BUTTON_B | BUTTON_A;

/// Macros the demonstration may take, one per round, as per-step button bytes.
/// A full jump is one decision, with three phase offsets so the beam holds
/// jumps started at different distances from an obstacle.
pub const MACROS: [[u8; ROUND_STEPS]; 6] = [
  [RUN; ROUND_STEPS],
  [RUN_JUMP; ROUND_STEPS],
  [RUN, RUN, RUN_JUMP, RUN_JUMP, RUN_JUMP, RUN_JUMP],
  [RUN, RUN, RUN, RUN, RUN_JUMP, RUN_JUMP],
  [BUTTON_RIGHT; ROUND_STEPS],
  [0; ROUND_STEPS],
];

pub fn smb_x(emu: &mut NesEmulator) -> i32 {
  emu.ram_read(smb1_ram::PLAYER_X_PAGE) as i32 * 256 + emu.ram_read(smb1_ram::PLAYER_X_PIXEL) as i32
}

pub fn smb_dead(emu: &mut NesEmulator) -> bool {
  smb1_ram::player_is_dead(emu)
}

/// True once the level counter has advanced past 1-1.
pub fn smb_cleared(emu: &mut NesEmulator) -> bool {
  emu.ram_read(smb1_ram::LEVEL) != 0 || emu.ram_read(smb1_ram::WORLD) != 0
}

/// Player Y on screen. Smaller is higher.
pub fn smb_y(emu: &mut NesEmulator) -> u8 {
  emu.ram_read(PLAYER_Y)
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
  /// Player Y on screen; smaller is higher.
  pub y: u8,
  pub dead: bool,
  pub cleared: bool,
  /// Consecutive steps without an x improvement.
  pub stuck: u32,
}

/// Drop dead and stuck candidates, order by x descending then height, keep `beam`.
///
/// The height tie-break keeps lineages that started a jump a few steps ago
/// alive at a wall, where ranking by x alone would let ground-huggers crowd
/// them out of the beam.
pub fn prune_and_rank(mut cands: Vec<Candidate>, beam: usize) -> Vec<Candidate> {
  cands.retain(|c| !c.dead && c.stuck < STUCK_LIMIT);
  cands.sort_by_key(|c| (std::cmp::Reverse(c.x), c.y));
  cands.truncate(beam);
  cands
}

/// Beam search over save states. Each round expands every frontier candidate by
/// every macro across threads, one emulator per thread.
pub fn beam_search(rom: &[u8], start: &[u8], beam: usize, max_rounds: usize) -> Demonstration {
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
    y: 0,
    dead: false,
    cleared: false,
    stuck: 0,
  }];
  let mut best = frontier[0].clone();
  for _ in 0..max_rounds {
    let jobs: Vec<(usize, usize)> = (0..frontier.len()).flat_map(|p| (0..MACROS.len()).map(move |m| (p, m))).collect();
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
              .map(|&(parent, macro_index)| {
                emu.load_state(&frontier[parent].state);
                let mut path = frontier[parent].path.clone();
                let mut dead = false;
                for &buttons in &MACROS[macro_index] {
                  emu.step_frames(buttons, MACRO_FRAMES);
                  path.push(buttons);
                  if smb_dead(&mut emu) || emu.ram_read(smb1_ram::LIVES) < lives_at_start {
                    dead = true;
                    break;
                  }
                }
                let x = smb_x(&mut emu);
                let stuck = if x > frontier[parent].x { 0 } else { frontier[parent].stuck + 1 };
                Candidate {
                  x,
                  y: smb_y(&mut emu),
                  dead,
                  cleared: smb_cleared(&mut emu),
                  state: emu.save_state(),
                  path,
                  stuck,
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
      y: 0xAB,
      dead,
      cleared: false,
      stuck: 0,
    }
  }

  #[test]
  fn prune_prefers_the_higher_candidate_at_equal_x() {
    let mut on_ground = cand(100, false);
    on_ground.y = 0xAB;
    let mut in_air = cand(100, false);
    in_air.y = 0x60;
    let kept = prune_and_rank(vec![on_ground, in_air], 1);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].y, 0x60);
  }

  #[test]
  fn prune_drops_candidates_stuck_for_too_long() {
    let mut stuck = cand(100, false);
    stuck.stuck = STUCK_LIMIT;
    let mut moving = cand(90, false);
    moving.stuck = STUCK_LIMIT - 1;
    let kept = prune_and_rank(vec![stuck, moving], 4);
    let xs: Vec<i32> = kept.iter().map(|c| c.x).collect();
    assert_eq!(xs, vec![90]);
  }

  #[test]
  #[ignore]
  fn beam_search_clears_the_pipes_in_1_1() {
    let rom = match rom_or_skip("roms2/super_mario_bros.nes") {
      Some(r) => r,
      None => return,
    };
    let start = smb_start_state(&rom);
    let demo = beam_search(&rom, &start, 16, 60);
    assert!(demo.x_reached > 1000, "reached only x={} in {} steps", demo.x_reached, demo.buttons.len());
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
  fn macros_never_include_start_or_select() {
    use crate::nes::bus::{BUTTON_SELECT, BUTTON_START};
    for m in MACROS {
      for a in m {
        assert_eq!(a & (BUTTON_START | BUTTON_SELECT), 0);
      }
    }
  }

  #[test]
  fn macros_include_jumps_at_three_phases() {
    let jump_starts: Vec<usize> = MACROS.iter().filter_map(|m| m.iter().position(|&b| b & BUTTON_A != 0)).collect();
    assert_eq!(jump_starts, vec![0, 2, 4]);
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
    let demo = beam_search(&rom, &start, 8, 30);
    assert!(demo.buttons.len() <= 30 * ROUND_STEPS);
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
