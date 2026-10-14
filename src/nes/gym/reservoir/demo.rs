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

/// True while the game is driving Mario rather than the player: the flagpole
/// slide, the walk into the castle, pipes. Progress stalls here without the
/// candidate being stuck.
pub fn smb_in_cutscene(emu: &mut NesEmulator) -> bool {
  emu.ram_read(smb1_ram::PLAYER_FLOAT_STATE) == 0x03 || emu.ram_read(smb1_ram::PLAYER_STATE) != 0x08
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
  /// Highest x seen along the path. The level reset after a clear does not lower it.
  pub x_reached: i32,
  pub cleared: bool,
  /// Idle frames after the level starts before the first button.
  pub noops: usize,
}

const DEMO_HEADER: usize = 7;

impl Demonstration {
  /// `[noops u16 LE][x_reached i32 LE][cleared u8][buttons...]`.
  pub fn to_bytes(&self) -> Vec<u8> {
    let mut out = Vec::with_capacity(DEMO_HEADER + self.buttons.len());
    out.extend_from_slice(&(self.noops as u16).to_le_bytes());
    out.extend_from_slice(&self.x_reached.to_le_bytes());
    out.push(self.cleared as u8);
    out.extend_from_slice(&self.buttons);
    out
  }

  pub fn from_bytes(bytes: &[u8]) -> Result<Demonstration, String> {
    if bytes.len() < DEMO_HEADER {
      return Err(format!("demonstration needs at least {} bytes, got {}", DEMO_HEADER, bytes.len()));
    }
    Ok(Demonstration {
      noops: u16::from_le_bytes([bytes[0], bytes[1]]) as usize,
      x_reached: i32::from_le_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]),
      cleared: bytes[6] != 0,
      buttons: bytes[DEMO_HEADER..].to_vec(),
    })
  }
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

/// Continuations tried when asking whether a candidate can survive the next
/// round: run on, or jump now.
const PEEK_MACROS: [usize; 2] = [0, 1];

/// True if every peek continuation from `state` dies within one round. A
/// candidate in this position is committed to a collision the x ranking
/// cannot see, and would otherwise crowd survivors out of the beam.
pub fn doomed_next_round(emu: &mut NesEmulator, state: &[u8], lives_at_start: u8) -> bool {
  for &m in &PEEK_MACROS {
    emu.load_state(state);
    let mut died = false;
    for &buttons in &MACROS[m] {
      emu.step_frames(buttons, MACRO_FRAMES);
      if smb_dead(emu) || emu.ram_read(smb1_ram::LIVES) < lives_at_start {
        died = true;
        break;
      }
    }
    if !died {
      return false;
    }
  }
  true
}

/// Next stuck count: reset on progress, hold during a cutscene, otherwise grow.
pub fn next_stuck(parent: u32, improved: bool, in_cutscene: bool) -> u32 {
  if improved {
    0
  } else if in_cutscene {
    parent
  } else {
    parent + 1
  }
}

/// Seeded hash of a path, so exact ties in the ranking break differently per seed.
fn tie_break(path: &[u8], seed: u64) -> u64 {
  let mut h = seed ^ 0x9E37_79B9_7F4A_7C15;
  for &b in path {
    h = (h ^ b as u64).wrapping_mul(0x100_0000_01B3);
  }
  h
}

/// Drop dead and stuck candidates, order by x descending then height, keep `beam`.
///
/// The height tie-break keeps lineages that started a jump a few steps ago
/// alive at a wall, where ranking by x alone would let ground-huggers crowd
/// them out of the beam. Exact ties break by a seeded hash of the path, so
/// searches with different seeds find different demonstrations.
pub fn prune_and_rank(mut cands: Vec<Candidate>, beam: usize, seed: u64) -> Vec<Candidate> {
  cands.retain(|c| !c.dead && c.stuck < STUCK_LIMIT);
  cands.sort_by_key(|c| (std::cmp::Reverse(c.x), c.y, tie_break(&c.path, seed)));
  cands.truncate(beam);
  cands
}

/// Beam search over save states from `noops` idle frames after `start`. Each
/// round expands every frontier candidate by every macro across threads, one
/// emulator per thread. `seed` only breaks exact ranking ties.
pub fn beam_search(rom: &[u8], start: &[u8], noops: usize, beam: usize, max_rounds: usize, seed: u64) -> Demonstration {
  let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
  let (first, lives_at_start) = {
    let mut emu = NesEmulator::new(rom);
    emu.load_state(start);
    emu.step_frames(0, noops);
    (emu.save_state(), emu.ram_read(smb1_ram::LIVES))
  };
  let mut frontier = vec![Candidate {
    state: first,
    path: Vec::new(),
    x: 0,
    y: 0,
    dead: false,
    cleared: false,
    stuck: 0,
  }];
  let mut best = frontier[0].clone();
  let mut max_x = 0;
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
                let stuck = next_stuck(frontier[parent].stuck, x > frontier[parent].x, smb_in_cutscene(&mut emu));
                let cleared = smb_cleared(&mut emu);
                let y = smb_y(&mut emu);
                let state = emu.save_state();
                let dead = dead || (!cleared && doomed_next_round(&mut emu, &state, lives_at_start));
                Candidate {
                  x,
                  y,
                  dead,
                  cleared,
                  state,
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
    let survivors = prune_and_rank(expanded, beam, seed);
    if survivors.is_empty() {
      break;
    }
    if survivors[0].cleared || survivors[0].x > best.x {
      best = survivors[0].clone();
    }
    max_x = max_x.max(survivors[0].x);
    if best.cleared {
      break;
    }
    frontier = survivors;
  }
  Demonstration {
    buttons: best.path,
    x_reached: max_x,
    cleared: best.cleared,
    noops,
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
    let kept = prune_and_rank(vec![on_ground, in_air], 1, 0);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].y, 0x60);
  }

  #[test]
  fn prune_breaks_exact_ties_by_seed() {
    let mut a = cand(100, false);
    a.path = vec![1, 2, 3];
    let mut b = cand(100, false);
    b.path = vec![4, 5, 6];
    let firsts: std::collections::HashSet<Vec<u8>> = (0..32)
      .map(|seed| prune_and_rank(vec![a.clone(), b.clone()], 1, seed)[0].path.clone())
      .collect();
    assert_eq!(firsts.len(), 2, "some seed must prefer each of two tied candidates");
    assert_eq!(
      prune_and_rank(vec![a.clone(), b.clone()], 1, 7)[0].path,
      prune_and_rank(vec![b, a], 1, 7)[0].path
    );
  }

  #[test]
  fn prune_drops_candidates_stuck_for_too_long() {
    let mut stuck = cand(100, false);
    stuck.stuck = STUCK_LIMIT;
    let mut moving = cand(90, false);
    moving.stuck = STUCK_LIMIT - 1;
    let kept = prune_and_rank(vec![stuck, moving], 4, 0);
    let xs: Vec<i32> = kept.iter().map(|c| c.x).collect();
    assert_eq!(xs, vec![90]);
  }

  #[test]
  #[ignore]
  fn peek_sees_the_first_goomba_coming_but_not_from_far_away() {
    let rom = match rom_or_skip("roms2/super_mario_bros.nes") {
      Some(r) => r,
      None => return,
    };
    let mut emu = NesEmulator::new(&rom);
    emu.load_state(&smb_start_state(&rom));
    let lives = emu.ram_read(smb1_ram::LIVES);
    emu.step_frames(crate::nes::bus::BUTTON_RIGHT, 100);
    let far = emu.save_state();
    assert!(!doomed_next_round(&mut emu, &far, lives), "a jump still saves Mario 100 frames in");
    // A jump still clears the Goomba up to two frames before contact at frame 205.
    emu.load_state(&far);
    emu.step_frames(crate::nes::bus::BUTTON_RIGHT, 103);
    let near = emu.save_state();
    assert!(!smb_dead(&mut emu), "not dead yet at frame 203");
    assert!(doomed_next_round(&mut emu, &near, lives), "two frames from the Goomba nothing saves him");
  }

  /// Takes about six minutes; runs only with `GREENSTONE_SLOW_TESTS` set.
  #[test]
  #[ignore]
  fn beam_search_clears_1_1_within_a_hundred_rounds() {
    if std::env::var_os("GREENSTONE_SLOW_TESTS").is_none() {
      eprintln!("skipping: set GREENSTONE_SLOW_TESTS to run the full clear");
      return;
    }
    let rom = match rom_or_skip("roms2/super_mario_bros.nes") {
      Some(r) => r,
      None => return,
    };
    let start = smb_start_state(&rom);
    let demo = beam_search(&rom, &start, 0, 16, 100, 0);
    assert!(demo.cleared, "did not clear: x={} in {} steps", demo.x_reached, demo.buttons.len());
    assert!(demo.x_reached > 3000, "x_reached should be the highest x seen, got {}", demo.x_reached);
  }

  #[test]
  #[ignore]
  fn beam_search_passes_the_goombas_before_the_stairs_in_1_1() {
    let rom = match rom_or_skip("roms2/super_mario_bros.nes") {
      Some(r) => r,
      None => return,
    };
    let start = smb_start_state(&rom);
    let demo = beam_search(&rom, &start, 0, 16, 60, 0);
    assert!(demo.x_reached > 2200, "reached only x={} in {} steps", demo.x_reached, demo.buttons.len());
  }

  #[test]
  #[ignore]
  fn beam_search_clears_the_pipes_in_1_1() {
    let rom = match rom_or_skip("roms2/super_mario_bros.nes") {
      Some(r) => r,
      None => return,
    };
    let start = smb_start_state(&rom);
    let demo = beam_search(&rom, &start, 0, 16, 60, 0);
    assert!(demo.x_reached > 1000, "reached only x={} in {} steps", demo.x_reached, demo.buttons.len());
  }

  #[test]
  fn demonstration_round_trips_through_bytes() {
    let demo = Demonstration {
      buttons: vec![0, RUN, RUN_JUMP, 0xFF],
      x_reached: 70000,
      cleared: true,
      noops: 17,
    };
    let back = Demonstration::from_bytes(&demo.to_bytes()).unwrap();
    assert_eq!(back.buttons, demo.buttons);
    assert_eq!(back.x_reached, 70000);
    assert!(back.cleared);
    assert_eq!(back.noops, 17);
    assert!(Demonstration::from_bytes(&[1, 2, 3]).is_err());
  }

  #[test]
  fn stuck_count_pauses_during_cutscenes() {
    assert_eq!(next_stuck(4, true, false), 0);
    assert_eq!(next_stuck(4, false, false), 5);
    assert_eq!(next_stuck(4, false, true), 4);
    assert_eq!(next_stuck(4, true, true), 0);
  }

  #[test]
  fn prune_drops_dead_and_keeps_best_by_x() {
    let cands = vec![cand(10, false), cand(50, true), cand(30, false), cand(20, false)];
    let kept = prune_and_rank(cands, 2, 0);
    let xs: Vec<i32> = kept.iter().map(|c| c.x).collect();
    assert_eq!(xs, vec![30, 20]);
  }

  #[test]
  fn prune_returns_empty_when_everything_is_dead() {
    let cands = vec![cand(10, true), cand(50, true)];
    assert!(prune_and_rank(cands, 4, 0).is_empty());
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
    let demo = beam_search(&rom, &start, 0, 8, 30, 0);
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
