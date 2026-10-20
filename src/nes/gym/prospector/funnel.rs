//! The cheap stages: does the level load, which level is it, and can a
//! dumb walker get anywhere.

use crate::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_RIGHT};
use crate::nes::gym::reservoir::demo::{beam_search_until, smb_dead, smb_stage, smb_start_state_at, smb_x, Clear, RoundStats, Stage, MACRO_FRAMES};
use crate::nes::gym::reservoir::features::{SMB_BLOCK_BUFFER_1, SMB_BLOCK_BUFFER_2, SMB_BLOCK_BUFFER_LEN};
use crate::nes::gym::reward::smb1_ram;
use crate::nes::gym::NesEmulator;

/// Frames of standing still after the level starts.
pub const IDLE_FRAMES: usize = 180;
/// Macro steps the walker takes (4 frames each).
pub const WALK_STEPS: usize = 100;
/// Less progress than this, in pixels, and the walker is stuck.
pub const MOVED_ENOUGH: i32 = 64;
/// SMB RAM: frame counter, ticked by every NMI.
const FRAME_COUNTER: u16 = 0x0009;
/// SMB RAM: the five enemy slots' active flags.
const ENEMY_FLAGS: u16 = 0x000F;

const RUN: u8 = BUTTON_RIGHT | BUTTON_B;
const JUMP: u8 = BUTTON_RIGHT | BUTTON_B | BUTTON_A;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadClass {
  /// The game isn't running a level: wrong engine state, or a frozen frame counter.
  Jammed,
  /// The first frame is one colour.
  Blank,
  /// Mario dies standing still.
  InstantDeath,
  Alive,
}

pub fn classify_load(engine_state: u8, ticking: bool, blank: bool, died: bool) -> LoadClass {
  if engine_state != 1 || !ticking {
    LoadClass::Jammed
  } else if blank {
    LoadClass::Blank
  } else if died {
    LoadClass::InstantDeath
  } else {
    LoadClass::Alive
  }
}

pub fn one_colour(rgb: &[u8]) -> bool {
  rgb.chunks(3).all(|p| p == &rgb[..3])
}

pub struct Loaded {
  pub class: LoadClass,
  /// The state the level starts in; every later stage starts here.
  pub state: Vec<u8>,
  /// The first frame of the level, 256x240 RGB.
  pub thumb: Vec<u8>,
}

/// Frames allowed between the boot state and Mario taking control.
const START_LIMIT: usize = 300;

/// Boot into `stage` and step past the black "WORLD 1-1" card until the
/// level is running: the player state leaves 0 for the entrance (7) or
/// normal play (8). Garbage headers often start in the entrance state and
/// stay there or fall, so waiting for 8 alone would file them as jammed.
/// Returns that state and its frame (a save state does not carry the
/// framebuffer), or None if play never starts.
pub fn level_start_at(rom: &[u8], stage: Stage) -> Option<(Vec<u8>, Vec<u8>)> {
  let mut emu = NesEmulator::new(rom);
  emu.load_state(&smb_start_state_at(rom, stage));
  for _ in 0..START_LIMIT {
    if emu.ram_read(smb1_ram::PLAYER_STATE) != 0 && emu.ram_read(smb1_ram::GAME_ENGINE_STATE) == 1 {
      return Some((emu.save_state(), emu.framebuffer().to_vec()));
    }
    emu.step_frame(0);
  }
  None
}

/// Boot a patched ROM into 1-1, then stand still for `IDLE_FRAMES`.
pub fn load(rom: &[u8]) -> Loaded {
  let Some((state, thumb)) = level_start_at(rom, Stage::ONE_ONE) else {
    return Loaded {
      class: LoadClass::Jammed,
      state: Vec::new(),
      thumb: Vec::new(),
    };
  };
  let mut emu = NesEmulator::new(rom);
  emu.load_state(&state);
  let engine = emu.ram_read(smb1_ram::GAME_ENGINE_STATE);
  let ticks = emu.ram_read(FRAME_COUNTER);
  let lives = emu.ram_read(smb1_ram::LIVES);
  emu.step_frames(0, IDLE_FRAMES);
  let ticking = emu.ram_read(FRAME_COUNTER) != ticks;
  let died = smb_dead(&mut emu) || emu.ram_read(smb1_ram::LIVES) < lives;
  Loaded {
    class: classify_load(engine, ticking, one_colour(&thumb), died),
    state,
    thumb,
  }
}

/// Hash of both terrain block buffers as the level starts: two offsets with
/// the same fingerprint built the same opening screens.
pub fn fingerprint(rom: &[u8], state: &[u8]) -> u64 {
  let mut emu = NesEmulator::new(rom);
  emu.load_state(state);
  let mut h = 0xCBF2_9CE4_8422_2325u64;
  for base in [SMB_BLOCK_BUFFER_1, SMB_BLOCK_BUFFER_2] {
    for a in base..base + SMB_BLOCK_BUFFER_LEN {
      h = (h ^ emu.ram_read(a) as u64).wrapping_mul(0x100_0000_01B3);
    }
  }
  h
}

#[derive(Debug, Clone, Copy)]
pub struct Walk {
  /// Furthest x reached.
  pub x: i32,
  /// Furthest x minus the starting x.
  pub moved: i32,
  pub died: bool,
}

/// Run right holding B, starting a jump at random: hold A for five steps,
/// release for one. Seeded, so the same seed walks the same way.
pub fn walk(rom: &[u8], state: &[u8], seed: u64) -> Walk {
  let mut emu = NesEmulator::new(rom);
  emu.load_state(state);
  let start = smb_x(&mut emu);
  let lives = emu.ram_read(smb1_ram::LIVES);
  let mut rng = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
  let mut jump: u32 = 0;
  let mut best = start;
  for _ in 0..WALK_STEPS {
    rng ^= rng << 13;
    rng ^= rng >> 7;
    rng ^= rng << 17;
    if jump == 0 && rng % 4 == 0 {
      jump = 6;
    }
    let buttons = if jump > 1 { JUMP } else { RUN };
    jump = jump.saturating_sub(1);
    emu.step_frames(buttons, MACRO_FRAMES);
    best = best.max(smb_x(&mut emu));
    if smb_dead(&mut emu) || emu.ram_read(smb1_ram::LIVES) < lives {
      return Walk {
        x: best,
        moved: best - start,
        died: true,
      };
    }
  }
  Walk {
    x: best,
    moved: best - start,
    died: false,
  }
}

/// Alive at this x counts as cleared: about the length of 1-1.
pub const GOAL_X: i32 = 3072;
pub const SOLVE_BEAM: usize = 16;
pub const SOLVE_ROUNDS: usize = 150;

/// Cleared when SMB leaves `stage`, or when Mario is alive at `GOAL_X`.
pub fn goal_for(stage: Stage) -> impl Fn(&mut NesEmulator) -> Option<Clear> + Sync {
  move |emu: &mut NesEmulator| {
    if smb_stage(emu) != stage {
      Some(Clear::Flag)
    } else if smb_x(emu) >= GOAL_X && !smb_dead(emu) {
      Some(Clear::Distance)
    } else {
      None
    }
  }
}

#[derive(Debug, Clone)]
pub struct Solved {
  pub clear: Option<Clear>,
  pub x_reached: i32,
  pub panics: usize,
  pub expansions: usize,
  /// One button byte per macro step, from the state solved from.
  pub path: Vec<u8>,
  pub stats: Vec<RoundStats>,
}

pub fn solve(rom: &[u8], state: &[u8], stage: Stage) -> Solved {
  let goal = goal_for(stage);
  let mut stats = Vec::new();
  let out = beam_search_until(rom, state, 0, SOLVE_BEAM, SOLVE_ROUNDS, 0, stage, &goal, &mut |s| stats.push(s));
  Solved {
    clear: out.clear,
    x_reached: out.demo.x_reached,
    panics: out.panics,
    expansions: out.expansions,
    path: out.demo.buttons,
    stats,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::prospector::patch::{patch_area, patch_rom, ONE_ONE_AREA};
  use crate::nes::gym::reservoir::rom_or_skip;

  #[test]
  fn classify_load_checks_running_then_blank_then_death() {
    assert_eq!(classify_load(0, true, false, false), LoadClass::Jammed);
    assert_eq!(classify_load(1, false, false, false), LoadClass::Jammed);
    assert_eq!(classify_load(1, true, true, true), LoadClass::Blank);
    assert_eq!(classify_load(1, true, false, true), LoadClass::InstantDeath);
    assert_eq!(classify_load(1, true, false, false), LoadClass::Alive);
  }

  #[test]
  fn one_colour_spots_a_flat_frame() {
    assert!(one_colour(&[1, 2, 3, 1, 2, 3]));
    assert!(!one_colour(&[1, 2, 3, 1, 2, 4]));
  }

  #[test]
  fn real_1_1_without_enemies_loads_alive() {
    let Some(rom) = rom_or_skip("roms2/super_mario_bros.nes") else { return };
    let loaded = load(&patch_rom(&rom, ONE_ONE_AREA));
    assert_eq!(loaded.class, LoadClass::Alive);
    assert_eq!(loaded.thumb.len(), 256 * 240 * 3);
  }

  #[test]
  fn the_patch_removes_1_1s_goombas() {
    let Some(rom) = rom_or_skip("roms2/super_mario_bros.nes") else { return };
    let enemies_after_a_walk = |rom: &[u8]| {
      let mut emu = NesEmulator::new(rom);
      emu.load_state(&smb_start_state_at(rom, Stage::ONE_ONE));
      emu.step_frames(RUN, 200);
      (0..5).filter(|&i| emu.ram_read(ENEMY_FLAGS + i) != 0).count()
    };
    assert!(
      enemies_after_a_walk(&patch_area(&rom, ONE_ONE_AREA)) > 0,
      "control: real 1-1 has a Goomba on screen"
    );
    assert_eq!(enemies_after_a_walk(&patch_rom(&rom, ONE_ONE_AREA)), 0);
  }

  #[test]
  fn the_level_starts_after_the_world_card_with_a_picture() {
    let Some(rom) = rom_or_skip("roms2/super_mario_bros.nes") else { return };
    let (state, frame) = level_start_at(&rom, Stage::ONE_ONE).unwrap();
    assert!(!one_colour(&frame), "the level's first frame should not be the black card");
    let mut emu = NesEmulator::new(&rom);
    emu.load_state(&state);
    assert_ne!(emu.ram_read(smb1_ram::PLAYER_STATE), 0);
  }

  #[test]
  fn the_goal_needs_a_stage_change_or_distance_alive() {
    let Some(rom) = rom_or_skip("roms2/super_mario_bros.nes") else { return };
    let mut emu = NesEmulator::new(&rom);
    emu.load_state(&level_start_at(&rom, Stage::ONE_ONE).unwrap().0);
    let goal = goal_for(Stage::ONE_ONE);
    assert_eq!(goal(&mut emu), None);
    emu.ram_write(smb1_ram::PLAYER_X_PAGE, (GOAL_X / 256) as u8);
    emu.ram_write(smb1_ram::PLAYER_X_PIXEL, 0);
    assert_eq!(goal(&mut emu), Some(Clear::Distance));
    emu.ram_write(smb1_ram::LEVEL, 1);
    assert_eq!(goal(&mut emu), Some(Clear::Flag));
  }

  #[test]
  fn a_level_that_starts_in_the_entrance_state_is_not_jammed() {
    let Some(rom) = rom_or_skip("roms2/super_mario_bros.nes") else { return };
    assert_eq!(
      load(&patch_rom(&rom, 0xA690)).class,
      LoadClass::InstantDeath,
      "$A690 drops Mario into a pit"
    );
    assert_ne!(
      load(&patch_rom(&rom, 0xA68F)).class,
      LoadClass::Jammed,
      "$A68F starts, in the entrance state"
    );
  }

  #[test]
  fn fingerprint_is_stable_and_tells_levels_apart() {
    let Some(rom) = rom_or_skip("roms2/super_mario_bros.nes") else { return };
    let a = level_start_at(&rom, Stage::ONE_ONE).unwrap().0;
    let b = level_start_at(&rom, Stage::ONE_ONE).unwrap().0;
    let c = level_start_at(&rom, Stage { world: 3, level: 0 }).unwrap().0;
    assert_eq!(fingerprint(&rom, &a), fingerprint(&rom, &b));
    assert_ne!(fingerprint(&rom, &a), fingerprint(&rom, &c));
  }

  #[test]
  fn walk_is_deterministic_and_moves_on_1_1() {
    let Some(rom) = rom_or_skip("roms2/super_mario_bros.nes") else { return };
    let rom = patch_rom(&rom, ONE_ONE_AREA);
    let state = level_start_at(&rom, Stage::ONE_ONE).unwrap().0;
    let one = walk(&rom, &state, 7);
    let two = walk(&rom, &state, 7);
    assert_eq!((one.x, one.moved, one.died), (two.x, two.moved, two.died));
    assert!(one.moved >= MOVED_ENOUGH, "walker should get going on 1-1, moved {}", one.moved);
  }
}
