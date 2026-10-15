//! A teacher that is a function of the window the readouts see.
//!
//! The beam-search demonstrations jump whenever a jump survives, so when
//! they jump is a coin toss that no readout of the state can predict, and
//! every readout trained on them misses every press onset. This teacher
//! jumps by a rule over the same window the readouts are given: an enemy
//! within a distance ahead, or a pit or wall within a few columns, while
//! Mario is on the ground. Its onsets are a function of the state by
//! construction, so a readout that misses them is failing to represent
//! the rule, not guessing a coin.

use super::demo::{smb_cleared, smb_dead, smb_window, smb_x, Demonstration, MACRO_FRAMES};
use super::features::{BLOCK_ROWS, SMB_ZERO_PAGE_LEN, TERRAIN_COLUMNS};
use super::trace::enemies;
use crate::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_RIGHT};
use crate::nes::gym::reward::smb1_ram;
use crate::nes::gym::NesEmulator;

/// SMB zero page: Mario's screen y, and the airborne byte (0 on the ground,
/// 1 in a jump, 2 in a fall, 3 on the flagpole).
const MARIO_Y: usize = 0x00CE;
const MARIO_AIRBORNE: usize = 0x001D;
/// Terrain row Mario stands on: the ground is rows 11 and 12 of 13.
pub const FOOT_ROW: usize = 11;
/// Terrain rows a tile would have to be in to block Mario's body. Row 7 holds
/// the floating blocks small Mario walks under.
const WALL_ROWS: std::ops::RangeInclusive<usize> = 8..=10;
/// An enemy counts as on Mario's floor if its y is within this many pixels of his.
const ENEMY_DY: i32 = 48;

const RUN: u8 = BUTTON_RIGHT | BUTTON_B;

/// The rule's parameters. Every one is searched over `GRID`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule {
  /// Jump when an enemy is this close ahead, in pixels.
  pub enemy_px: i32,
  /// Jump when a pit or wall is within this many columns ahead.
  pub look: usize,
  /// Steps to hold A for an enemy.
  pub hold_enemy: usize,
  /// Steps to hold A for a pit or wall.
  pub hold_obstacle: usize,
}

/// The grid the rule is searched over, one axis per field.
pub const GRID: ([i32; 4], [usize; 3], [usize; 2], [usize; 3]) = ([24, 32, 40, 48], [1, 2, 3], [2, 3], [5, 7, 9]);

impl Rule {
  /// `[enemy_px u8][look u8][hold_enemy u8][hold_obstacle u8]`.
  pub fn to_bytes(self) -> [u8; 4] {
    [self.enemy_px as u8, self.look as u8, self.hold_enemy as u8, self.hold_obstacle as u8]
  }

  pub fn from_bytes(bytes: &[u8]) -> Result<Rule, String> {
    if bytes.len() != 4 {
      return Err(format!("a rule is 4 bytes, got {}", bytes.len()));
    }
    Ok(Rule {
      enemy_px: bytes[0] as i32,
      look: bytes[1] as usize,
      hold_enemy: bytes[2] as usize,
      hold_obstacle: bytes[3] as usize,
    })
  }

  /// Every rule on the grid.
  pub fn grid() -> Vec<Rule> {
    let (px, look, he, ho) = GRID;
    let mut rules = Vec::new();
    for &enemy_px in &px {
      for &look in &look {
        for &hold_enemy in &he {
          for &hold_obstacle in &ho {
            rules.push(Rule {
              enemy_px,
              look,
              hold_enemy,
              hold_obstacle,
            });
          }
        }
      }
    }
    rules
  }

  pub fn describe(self) -> String {
    format!(
      "jump at an enemy within {} px (hold {} steps) or a pit or wall within {} columns (hold {} steps)",
      self.enemy_px, self.hold_enemy, self.look, self.hold_obstacle
    )
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
  Enemy,
  Obstacle,
}

fn mario_x(window: &[u8]) -> i32 {
  window[smb1_ram::PLAYER_X_PAGE as usize] as i32 * 256 + window[smb1_ram::PLAYER_X_PIXEL as usize] as i32
}

/// The terrain byte `columns` ahead of Mario at `row`.
fn terrain(window: &[u8], columns: usize, row: usize) -> u8 {
  window[SMB_ZERO_PAGE_LEN as usize + columns * BLOCK_ROWS + row]
}

/// What, if anything, the rule jumps for in this window. Only on the ground.
pub fn trigger(rule: Rule, window: &[u8]) -> Option<Trigger> {
  if window[MARIO_AIRBORNE] != 0 {
    return None;
  }
  let x = mario_x(window);
  let y = window[MARIO_Y] as i32;
  let enemy_close = enemies(window)
    .iter()
    .any(|e| (0..rule.enemy_px).contains(&(e.x - x)) && (e.y as i32 - y).abs() <= ENEMY_DY);
  if enemy_close {
    return Some(Trigger::Enemy);
  }
  let obstacle = (1..=rule.look.min(TERRAIN_COLUMNS - 1)).any(|d| {
    let pit = terrain(window, d, FOOT_ROW) == 0;
    let wall = WALL_ROWS.clone().any(|row| terrain(window, d, row) != 0);
    pit || wall
  });
  if obstacle {
    return Some(Trigger::Obstacle);
  }
  None
}

/// The rule with its one piece of state: how many more steps A is held.
#[derive(Debug, Clone)]
pub struct Player {
  pub rule: Rule,
  holding: usize,
}

impl Player {
  pub fn new(rule: Rule) -> Player {
    Player { rule, holding: 0 }
  }

  /// Buttons for this step: always run right; A while a jump is held.
  pub fn buttons(&mut self, window: &[u8]) -> u8 {
    if self.holding == 0 {
      self.holding = match trigger(self.rule, window) {
        Some(Trigger::Enemy) => self.rule.hold_enemy,
        Some(Trigger::Obstacle) => self.rule.hold_obstacle,
        None => 0,
      };
    }
    if self.holding > 0 {
      self.holding -= 1;
      RUN | BUTTON_A
    } else {
      RUN
    }
  }
}

/// Play the rule from `noops` idle frames after `start` until the level is
/// cleared, Mario dies, or `max_steps` steps pass.
pub fn rollout(rom: &[u8], start: &[u8], noops: usize, rule: Rule, max_steps: usize) -> Demonstration {
  let mut emu = NesEmulator::new(rom);
  emu.load_state(start);
  emu.step_frames(0, noops);
  let mut player = Player::new(rule);
  let mut buttons = Vec::new();
  let mut x_reached = 0;
  let mut cleared = false;
  for _ in 0..max_steps {
    let window = smb_window(&mut emu);
    let b = player.buttons(&window);
    emu.step_frames(b, MACRO_FRAMES);
    buttons.push(b);
    x_reached = x_reached.max(smb_x(&mut emu));
    cleared = smb_cleared(&mut emu);
    if cleared || smb_dead(&mut emu) {
      break;
    }
  }
  Demonstration {
    buttons,
    x_reached,
    cleared,
    noops,
  }
}

/// Score of a rule over the offsets: clears first, then the sum of x reached.
pub fn score(demos: &[Demonstration]) -> (usize, i32) {
  (demos.iter().filter(|d| d.cleared).count(), demos.iter().map(|d| d.x_reached).sum())
}

/// Play every rule on the grid from every offset, across threads, and return
/// the best rule with its demonstrations.
pub fn search_rule(rom: &[u8], start: &[u8], noops: &[usize], max_steps: usize, log: &mut dyn FnMut(&str)) -> (Rule, Vec<Demonstration>) {
  let rules = Rule::grid();
  let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
  let chunk = rules.len().div_ceil(threads);
  let mut played: Vec<(Rule, Vec<Demonstration>)> = std::thread::scope(|s| {
    let handles: Vec<_> = rules
      .chunks(chunk)
      .map(|rules| {
        s.spawn(move || {
          rules
            .iter()
            .map(|&rule| (rule, noops.iter().map(|&n| rollout(rom, start, n, rule, max_steps)).collect::<Vec<_>>()))
            .collect::<Vec<_>>()
        })
      })
      .collect();
    handles.into_iter().flat_map(|h| h.join().expect("rule worker panicked")).collect()
  });
  played.sort_by_key(|(_, demos)| std::cmp::Reverse(score(demos)));
  for (rule, demos) in played.iter().take(5) {
    let (clears, x) = score(demos);
    log(&format!("rule {}: {} of {} clear, x sum {}", rule.describe(), clears, demos.len(), x));
  }
  played.swap_remove(0)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::reservoir::features::SMB_WINDOW_LEN;

  const RULE: Rule = Rule {
    enemy_px: 32,
    look: 2,
    hold_enemy: 2,
    hold_obstacle: 5,
  };

  /// Mario on flat ground at x 400, y 176, with ground in every column.
  fn grounded() -> Vec<u8> {
    let mut w = vec![0u8; SMB_WINDOW_LEN];
    w[smb1_ram::PLAYER_X_PAGE as usize] = 1;
    w[smb1_ram::PLAYER_X_PIXEL as usize] = 144;
    w[MARIO_Y] = 176;
    for d in 0..TERRAIN_COLUMNS {
      w[SMB_ZERO_PAGE_LEN as usize + d * BLOCK_ROWS + 11] = 0x54;
      w[SMB_ZERO_PAGE_LEN as usize + d * BLOCK_ROWS + 12] = 0x54;
    }
    w
  }

  fn add_goomba(w: &mut [u8], x: i32, y: u8) {
    w[0x000F] = 1;
    w[0x0016] = 0x06;
    w[0x006E] = (x / 256) as u8;
    w[0x0087] = (x % 256) as u8;
    w[0x00CF] = y;
  }

  fn set_terrain(w: &mut [u8], columns: usize, row: usize, tile: u8) {
    w[SMB_ZERO_PAGE_LEN as usize + columns * BLOCK_ROWS + row] = tile;
  }

  #[test]
  fn flat_ground_with_nothing_ahead_is_no_trigger() {
    assert_eq!(trigger(RULE, &grounded()), None);
  }

  #[test]
  fn an_enemy_within_range_on_the_floor_triggers() {
    let mut w = grounded();
    add_goomba(&mut w, 400 + 24, 184);
    assert_eq!(trigger(RULE, &w), Some(Trigger::Enemy));
  }

  #[test]
  fn an_enemy_out_of_range_or_behind_or_above_does_not_trigger() {
    let mut far = grounded();
    add_goomba(&mut far, 400 + 32, 184);
    assert_eq!(trigger(RULE, &far), None);
    let mut behind = grounded();
    add_goomba(&mut behind, 400 - 8, 184);
    assert_eq!(trigger(RULE, &behind), None);
    let mut above = grounded();
    add_goomba(&mut above, 400 + 16, 100);
    assert_eq!(trigger(RULE, &above), None);
  }

  #[test]
  fn a_pit_within_look_triggers_and_beyond_it_does_not() {
    let mut near = grounded();
    set_terrain(&mut near, 2, FOOT_ROW, 0);
    assert_eq!(trigger(RULE, &near), Some(Trigger::Obstacle));
    let mut far = grounded();
    set_terrain(&mut far, 3, FOOT_ROW, 0);
    assert_eq!(trigger(RULE, &far), None);
  }

  #[test]
  fn a_wall_at_body_height_triggers_but_a_floating_block_does_not() {
    let mut pipe = grounded();
    set_terrain(&mut pipe, 1, 9, 0x12);
    set_terrain(&mut pipe, 1, 10, 0x14);
    assert_eq!(trigger(RULE, &pipe), Some(Trigger::Obstacle));
    let mut block = grounded();
    set_terrain(&mut block, 1, 7, 0xC0);
    assert_eq!(trigger(RULE, &block), None);
  }

  #[test]
  fn airborne_mario_never_triggers() {
    let mut w = grounded();
    add_goomba(&mut w, 400 + 8, 184);
    w[MARIO_AIRBORNE] = 1;
    assert_eq!(trigger(RULE, &w), None);
  }

  #[test]
  fn the_player_holds_a_for_the_trigger_length_then_runs() {
    let mut p = Player::new(RULE);
    let mut w = grounded();
    add_goomba(&mut w, 400 + 8, 184);
    assert_eq!(p.buttons(&w), RUN | BUTTON_A);
    w[MARIO_AIRBORNE] = 1;
    assert_eq!(p.buttons(&w), RUN | BUTTON_A);
    assert_eq!(p.buttons(&w), RUN);
    w[MARIO_AIRBORNE] = 0;
    assert_eq!(p.buttons(&w), RUN | BUTTON_A);
  }

  #[test]
  fn rules_round_trip_through_bytes_and_the_grid_is_full() {
    assert_eq!(Rule::from_bytes(&RULE.to_bytes()).unwrap(), RULE);
    assert!(Rule::from_bytes(&[1, 2]).is_err());
    assert_eq!(Rule::grid().len(), 4 * 3 * 2 * 3);
  }

  #[test]
  fn score_prefers_clears_then_distance() {
    let d = |x, cleared| Demonstration {
      buttons: vec![],
      x_reached: x,
      cleared,
      noops: 0,
    };
    assert!(score(&[d(100, true), d(50, false)]) > score(&[d(3000, false), d(3000, false)]));
    assert!(score(&[d(200, false)]) > score(&[d(100, false)]));
  }
}

#[cfg(test)]
mod rom_tests {
  use super::*;
  use crate::nes::gym::reservoir::demo::smb_start_state;
  use crate::nes::gym::reservoir::rom_or_skip;

  /// The searched rule clears 1-1 from the first offset.
  #[test]
  #[ignore]
  fn the_searched_rule_clears_1_1() {
    let rom = match rom_or_skip("roms2/super_mario_bros.nes") {
      Some(r) => r,
      None => return,
    };
    let start = smb_start_state(&rom);
    let mut log = |line: &str| println!("{}", line);
    let (rule, demos) = search_rule(&rom, &start, &[0, 15], 750, &mut log);
    println!("best: {}", rule.describe());
    for d in &demos {
      println!("offset {}: {} steps, x {}, cleared {}", d.noops, d.buttons.len(), d.x_reached, d.cleared);
    }
    assert!(demos[0].cleared, "{} did not clear 1-1", rule.describe());
  }
}
