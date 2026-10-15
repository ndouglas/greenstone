//! A demonstration for SMB 1-1 found by beam search over save states.

#[cfg(test)]
use super::features::SMB_WINDOW_LEN;
use super::features::{
  BLOCK_COLUMNS, BLOCK_ROWS, SMB_BLOCK_BUFFER_1, SMB_BLOCK_BUFFER_2, SMB_BLOCK_BUFFER_LEN, SMB_WINDOW_START, SMB_ZERO_PAGE_LEN, TERRAIN_COLUMNS,
};
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
  smb_cleared_at(emu, Stage::ONE_ONE)
}

/// True once the game has moved on from `stage`.
pub fn smb_cleared_at(emu: &mut NesEmulator, stage: Stage) -> bool {
  smb_stage(emu) != stage
}

/// Player Y on screen. Smaller is higher.
pub fn smb_y(emu: &mut NesEmulator) -> u8 {
  emu.ram_read(PLAYER_Y)
}

/// SMB zero-page bytes that hold the controller's own input: A/B buttons,
/// up/down, left/right, and last frame's A/B. Zeroed in the window so no
/// condition can read the previous action back; the window is state, not input.
pub const SMB_INPUT_BYTES: [u16; 4] = [0x000A, 0x000B, 0x000C, 0x000D];

/// The terrain from Mario's column rightwards, `TERRAIN_COLUMNS` columns of
/// `BLOCK_ROWS` bytes each, read from whichever block buffer holds each page.
pub fn terrain_ahead(zero_page: &[u8], buffer_1: &[u8], buffer_2: &[u8]) -> Vec<u8> {
  let x = zero_page[smb1_ram::PLAYER_X_PAGE as usize] as usize * 256 + zero_page[smb1_ram::PLAYER_X_PIXEL as usize] as usize;
  let first = x / 16;
  let mut out = Vec::with_capacity(TERRAIN_COLUMNS * BLOCK_ROWS);
  for d in 0..TERRAIN_COLUMNS {
    let column = first + d;
    let page = column / BLOCK_COLUMNS;
    let within = column % BLOCK_COLUMNS;
    let buffer = if page % 2 == 0 { buffer_1 } else { buffer_2 };
    for row in 0..BLOCK_ROWS {
      out.push(buffer[row * BLOCK_COLUMNS + within]);
    }
  }
  out
}

/// The zero page with the controller bytes hidden, then the terrain ahead of Mario.
pub fn smb_window(emu: &mut NesEmulator) -> Vec<u8> {
  let mut window = emu.ram_read_range(SMB_WINDOW_START, SMB_ZERO_PAGE_LEN);
  for &address in &SMB_INPUT_BYTES {
    window[(address - SMB_WINDOW_START) as usize] = 0;
  }
  let buffer_1 = emu.ram_read_range(SMB_BLOCK_BUFFER_1, SMB_BLOCK_BUFFER_LEN);
  let buffer_2 = emu.ram_read_range(SMB_BLOCK_BUFFER_2, SMB_BLOCK_BUFFER_LEN);
  window.extend(terrain_ahead(&window, &buffer_1, &buffer_2));
  window
}

/// A level of SMB, zero-based as the game stores it: 1-1 is world 0, level 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Stage {
  pub world: u8,
  pub level: u8,
}

/// SMB RAM: the level within the world that the loader reads. `smb1_ram::LEVEL`
/// is the copy the status bar shows; both are set so they agree.
const LOADER_LEVEL: u16 = 0x075C;

impl Stage {
  pub const ONE_ONE: Stage = Stage { world: 0, level: 0 };

  /// `1-1`, as the game prints it.
  pub fn name(self) -> String {
    format!("{}-{}", self.world + 1, self.level + 1)
  }

  /// The inverse of `name`: `6-1` is world 5, level 0.
  pub fn parse(text: &str) -> Result<Stage, String> {
    let (w, l) = text.split_once('-').ok_or_else(|| format!("a stage is W-L, got {:?}", text))?;
    let parse = |s: &str| {
      s.trim()
        .parse::<u8>()
        .ok()
        .filter(|n| (1..=8).contains(n))
        .ok_or_else(|| format!("a stage is W-L with 1..=8, got {:?}", text))
    };
    Ok(Stage {
      world: parse(w)? - 1,
      level: parse(l)? - 1,
    })
  }
}

/// Boot SMB, tap Start through the title, wait for 1-1 to begin, and snapshot.
pub fn smb_start_state(rom: &[u8]) -> Vec<u8> {
  smb_start_state_at(rom, Stage::ONE_ONE)
}

/// Boot SMB, write the stage into the world and level bytes while tapping
/// Start through the title so the loader starts there, wait for the level
/// to begin, and snapshot.
pub fn smb_start_state_at(rom: &[u8], stage: Stage) -> Vec<u8> {
  let mut emu = NesEmulator::new(rom);
  for frame in 0..120 {
    emu.ram_write(smb1_ram::WORLD, stage.world);
    emu.ram_write(LOADER_LEVEL, stage.level);
    emu.ram_write(smb1_ram::LEVEL, stage.level);
    emu.step_frame(if frame % 20 < 10 { BUTTON_START } else { 0 });
  }
  emu.step_frames(0, 80);
  emu.save_state()
}

/// The stage the emulator is in now.
pub fn smb_stage(emu: &mut NesEmulator) -> Stage {
  Stage {
    world: emu.ram_read(smb1_ram::WORLD),
    level: emu.ram_read(smb1_ram::LEVEL),
  }
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
  /// The level it was played on.
  pub stage: Stage,
}

const DEMO_HEADER: usize = 9;

/// Path of cached demonstration `index` under `dir`.
pub fn cached_demo_path(dir: &std::path::Path, index: usize) -> std::path::PathBuf {
  dir.join(format!("demo-{}.bin", index))
}

/// Load `count` cached demonstrations from `dir`, or None if any is missing or malformed.
pub fn load_cached(dir: &std::path::Path, count: usize) -> Option<Vec<Demonstration>> {
  (0..count)
    .map(|i| {
      std::fs::read(cached_demo_path(dir, i))
        .ok()
        .and_then(|b| Demonstration::from_bytes(&b).ok())
    })
    .collect()
}

/// Write demonstrations to `dir` as `demo-N.bin`.
pub fn save_cached(dir: &std::path::Path, demos: &[Demonstration]) -> std::io::Result<()> {
  std::fs::create_dir_all(dir)?;
  for (i, demo) in demos.iter().enumerate() {
    std::fs::write(cached_demo_path(dir, i), demo.to_bytes())?;
  }
  Ok(())
}

impl Demonstration {
  /// `[noops u16 LE][x_reached i32 LE][cleared u8][world u8][level u8][buttons...]`.
  pub fn to_bytes(&self) -> Vec<u8> {
    let mut out = Vec::with_capacity(DEMO_HEADER + self.buttons.len());
    out.extend_from_slice(&(self.noops as u16).to_le_bytes());
    out.extend_from_slice(&self.x_reached.to_le_bytes());
    out.push(self.cleared as u8);
    out.push(self.stage.world);
    out.push(self.stage.level);
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
      stage: Stage {
        world: bytes[7],
        level: bytes[8],
      },
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
  /// Hash of the zero page. Two candidates with the same key are the same
  /// game state reached by different paths.
  pub key: u64,
}

/// Hash of the SMB zero page, the beam's notion of state identity.
pub fn state_key(emu: &mut NesEmulator) -> u64 {
  smb_window(emu)
    .iter()
    .fold(0xCBF2_9CE4_8422_2325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x100_0000_01B3))
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

/// Progress is compared in buckets this wide, so that among candidates within
/// a few pixels of each other the one that jumped least wins.
pub const X_BUCKET: usize = 8;

/// Steps on a path with A held.
pub fn a_presses(path: &[u8]) -> usize {
  path.iter().filter(|&&b| b & BUTTON_A != 0).count()
}

/// Drop dead and stuck candidates, order by progress in `X_BUCKET` buckets,
/// then fewest A presses, then height, keep one candidate per game state,
/// and keep `beam` of them.
///
/// Fewest A presses makes every demonstration jump as late and as little
/// as it can, so when it jumps is a function of what is in front of Mario
/// rather than of a coin, and a state readout can learn it. The height
/// tie-break keeps lineages that started a jump a few steps ago alive at a
/// wall. Exact ties break by a seeded hash of the path, so searches with
/// different seeds find different demonstrations. Without the dedupe, the
/// beam fills with copies of the single best state and dies together at
/// the first obstacle that state cannot pass.
pub fn prune_and_rank(mut cands: Vec<Candidate>, beam: usize, seed: u64) -> Vec<Candidate> {
  cands.retain(|c| !c.dead && c.stuck < STUCK_LIMIT);
  cands.sort_by_key(|c| {
    (
      std::cmp::Reverse(c.x.max(0) as usize / X_BUCKET),
      a_presses(&c.path),
      c.y,
      tie_break(&c.path, seed),
    )
  });
  let mut seen = std::collections::HashSet::new();
  cands.retain(|c| seen.insert(c.key));
  cands.truncate(beam);
  cands
}

/// Beam search over save states from `noops` idle frames after `start`. Each
/// round expands every frontier candidate by every macro across threads, one
/// emulator per thread. `seed` only breaks exact ranking ties.
pub fn beam_search(rom: &[u8], start: &[u8], noops: usize, beam: usize, max_rounds: usize, seed: u64, stage: Stage) -> Demonstration {
  let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
  let (first, lives_at_start, key) = {
    let mut emu = NesEmulator::new(rom);
    emu.load_state(start);
    emu.step_frames(0, noops);
    (emu.save_state(), emu.ram_read(smb1_ram::LIVES), state_key(&mut emu))
  };
  let mut frontier = vec![Candidate {
    state: first,
    path: Vec::new(),
    x: 0,
    y: 0,
    dead: false,
    cleared: false,
    stuck: 0,
    key,
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
                let cleared = smb_cleared_at(&mut emu, stage);
                let y = smb_y(&mut emu);
                let key = state_key(&mut emu);
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
                  key,
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
    stage,
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
      key: NEXT_KEY.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
    }
  }

  /// Every test candidate is a distinct state unless a test says otherwise.
  static NEXT_KEY: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1000);

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
  fn prune_prefers_fewer_jumps_at_similar_progress() {
    let mut jumpy = cand(103, false);
    jumpy.path = vec![RUN_JUMP, RUN_JUMP, RUN_JUMP];
    let mut calm = cand(100, false);
    calm.path = vec![RUN, RUN, RUN];
    let kept = prune_and_rank(vec![jumpy.clone(), calm.clone()], 1, 0);
    assert_eq!(kept[0].path, calm.path, "within one bucket the path with fewer A presses wins");
    let mut far = cand(100 + X_BUCKET as i32, false);
    far.path = vec![RUN_JUMP; 3];
    let kept = prune_and_rank(vec![far.clone(), calm], 1, 0);
    assert_eq!(kept[0].path, far.path, "a full bucket of progress still beats fewer jumps");
    assert_eq!(a_presses(&[RUN, RUN_JUMP, 0, RUN_JUMP]), 2);
  }

  #[test]
  fn prune_keeps_one_candidate_per_state() {
    let mut a = cand(100, false);
    a.key = 7;
    a.path = vec![1];
    let mut same = cand(100, false);
    same.key = 7;
    same.path = vec![2];
    let mut other = cand(90, false);
    other.key = 8;
    let kept = prune_and_rank(vec![a, same, other], 3, 0);
    let keys: Vec<u64> = kept.iter().map(|c| c.key).collect();
    assert_eq!(keys, vec![7, 8], "two paths to the same state are one candidate");
  }

  #[test]
  fn prune_breaks_exact_ties_by_seed() {
    let mut a = cand(100, false);
    a.path = vec![2, 4, 6];
    let mut b = cand(100, false);
    b.path = vec![8, 10, 12];
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
    let demo = beam_search(&rom, &start, 0, 16, 100, 0, Stage::ONE_ONE);
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
    let demo = beam_search(&rom, &start, 0, 16, 60, 0, Stage::ONE_ONE);
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
    let demo = beam_search(&rom, &start, 0, 16, 60, 0, Stage::ONE_ONE);
    assert!(demo.x_reached > 1000, "reached only x={} in {} steps", demo.x_reached, demo.buttons.len());
  }

  #[test]
  fn demonstration_round_trips_through_bytes() {
    let demo = Demonstration {
      buttons: vec![0, RUN, RUN_JUMP, 0xFF],
      x_reached: 70000,
      cleared: true,
      noops: 17,
      stage: Stage { world: 5, level: 0 },
    };
    let back = Demonstration::from_bytes(&demo.to_bytes()).unwrap();
    assert_eq!(back.buttons, demo.buttons);
    assert_eq!(back.x_reached, 70000);
    assert!(back.cleared);
    assert_eq!(back.noops, 17);
    assert_eq!(back.stage, Stage { world: 5, level: 0 });
    assert!(Demonstration::from_bytes(&[1, 2, 3]).is_err());
  }

  #[test]
  fn window_hides_the_controller_bytes() {
    let rom = std::fs::read("roms/snake.nes").expect("tracked test ROM");
    let mut emu = NesEmulator::new(&rom);
    for &address in &SMB_INPUT_BYTES {
      emu.ram_write(address, 0xFF);
    }
    emu.ram_write(0x0009, 0x42);
    emu.ram_write(0x000E, 0x08);
    // Mario at x 0, page 0: his column is buffer 1 column 0.
    emu.ram_write(SMB_BLOCK_BUFFER_1 + 11 * 16, 0x54);
    let window = smb_window(&mut emu);
    assert_eq!(window.len(), SMB_WINDOW_LEN);
    assert_eq!(window.len(), 256 + 8 * 13);
    for &address in &SMB_INPUT_BYTES {
      assert_eq!(window[address as usize], 0, "input byte ${:02X} must be hidden", address);
    }
    assert_eq!(window[0x09], 0x42, "neighbouring bytes pass through");
    assert_eq!(window[0x0E], 0x08);
    assert_eq!(window[256 + 11], 0x54, "the terrain ahead follows the zero page, column 0 row 11");
  }

  #[test]
  fn terrain_ahead_reads_columns_from_mario_across_the_page_boundary() {
    // Mario at x = 256 + 14 * 16 + 3: page 1, column 14. Page 1 is buffer 2.
    let mut zero_page = vec![0u8; 256];
    zero_page[0x6D] = 1;
    zero_page[0x86] = 14 * 16 + 3;
    let mut buf1 = vec![0u8; 208];
    let mut buf2 = vec![0u8; 208];
    // Page 1 column 14 row 11 is ground, column 15 row 11 is a pit (empty).
    buf2[11 * 16 + 14] = 0x54;
    buf2[11 * 16 + 15] = 0x00;
    // Page 2 (buffer 1 again) column 0 row 11 is ground, column 1 row 9 is a pipe top.
    buf1[11 * 16] = 0x54;
    buf1[9 * 16 + 1] = 0x12;
    let t = terrain_ahead(&zero_page, &buf1, &buf2);
    assert_eq!(t.len(), TERRAIN_COLUMNS * BLOCK_ROWS);
    let cell = |col: usize, row: usize| t[col * BLOCK_ROWS + row];
    assert_eq!(cell(0, 11), 0x54, "Mario's own column");
    assert_eq!(cell(1, 11), 0x00, "the pit one column ahead");
    assert_eq!(cell(2, 11), 0x54, "two ahead wraps into the next page's buffer");
    assert_eq!(cell(3, 9), 0x12, "three ahead, the pipe top");
    assert_eq!(cell(7, 11), 0x00);
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
  fn stages_are_named_as_the_game_prints_them() {
    assert_eq!(Stage::ONE_ONE.name(), "1-1");
    assert_eq!(Stage { world: 3, level: 1 }.name(), "4-2");
    assert_eq!(Stage::parse("6-1").unwrap(), Stage { world: 5, level: 0 });
    assert_eq!(Stage::parse(" 1-1 ").unwrap(), Stage::ONE_ONE);
    assert!(Stage::parse("0-1").is_err());
    assert!(Stage::parse("9-1").is_err());
    assert!(Stage::parse("61").is_err());
  }

  #[test]
  #[ignore]
  fn a_start_state_at_another_stage_loads_that_stage_and_its_own_terrain() {
    let rom = match rom_or_skip("roms2/super_mario_bros.nes") {
      Some(r) => r,
      None => return,
    };
    let two_one = Stage { world: 1, level: 0 };
    let mut emu = NesEmulator::new(&rom);
    emu.load_state(&smb_start_state_at(&rom, two_one));
    assert_eq!(smb_stage(&mut emu), two_one);
    assert!(!smb_cleared_at(&mut emu, two_one));
    assert!(smb_cleared_at(&mut emu, Stage::ONE_ONE), "from 1-1's point of view the game has moved on");
    assert!(!smb_dead(&mut emu));
    // Both levels open on flat ground; a few seconds in, the terrain differs.
    let mut home = NesEmulator::new(&rom);
    home.load_state(&smb_start_state(&rom));
    emu.step_frames(crate::nes::bus::BUTTON_RIGHT, 300);
    home.step_frames(crate::nes::bus::BUTTON_RIGHT, 300);
    assert!(smb_x(&mut emu) > 200, "Mario should move right from the start state");
    assert_eq!(smb_stage(&mut emu), two_one, "still 2-1 after running");
    let there = smb_window(&mut emu);
    let here = smb_window(&mut home);
    assert_ne!(there[256..], here[256..], "2-1 and 1-1 have different terrain ahead");
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
    let demo = beam_search(&rom, &start, 0, 8, 30, 0, Stage::ONE_ONE);
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
