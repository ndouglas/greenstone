//! Reward extractors for NES games.
//!
//! Each game needs a custom reward function that reads game-specific RAM
//! addresses to derive a scalar reward signal. The [`RewardExtractor`] trait
//! provides the interface, and concrete implementations are provided for
//! well-known games.
//!
//! # Adding a new game
//!
//! 1. Create a struct implementing `RewardExtractor`.
//! 2. In `extract()`, read the relevant RAM addresses from the emulator
//!    and return a reward delta (change since last call).
//! 3. Register it as a variant in the `GameReward` enum for convenience.

use super::NesEmulator;

/// Trait for extracting reward signals from NES game state.
///
/// Implementations read game-specific RAM addresses and return a scalar
/// reward (positive = good, negative = bad, zero = neutral).
pub trait RewardExtractor {
  /// Extract reward from the current emulator state.
  /// Should return the *delta* reward since the last call.
  fn extract(&mut self, emu: &mut NesEmulator) -> f32;

  /// Reset internal state (called on environment reset).
  fn reset(&mut self);

  /// Human-readable name of this reward function.
  fn name(&self) -> &str;
}

/// A reward extractor that always returns zero.
/// Useful as a baseline or for unsupervised exploration.
pub struct NullReward;

impl RewardExtractor for NullReward {
  fn extract(&mut self, _emu: &mut NesEmulator) -> f32 {
    0.0
  }

  fn reset(&mut self) {}

  fn name(&self) -> &str {
    "null"
  }
}

/// Custom reward extractor that reads a single RAM address and
/// tracks the delta of its value.
///
/// Good for games where score or progress is stored in a single byte.
pub struct SingleByteReward {
  name: &'static str,
  address: u16,
  prev_value: u8,
  scale: f32,
}

impl SingleByteReward {
  pub fn new(name: &'static str, address: u16, scale: f32) -> Self {
    SingleByteReward {
      name,
      address,
      prev_value: 0,
      scale,
    }
  }
}

impl RewardExtractor for SingleByteReward {
  fn extract(&mut self, emu: &mut NesEmulator) -> f32 {
    let current = emu.ram_read(self.address);
    let delta = current as i16 - self.prev_value as i16;
    self.prev_value = current;
    delta as f32 * self.scale
  }

  fn reset(&mut self) {
    self.prev_value = 0;
  }

  fn name(&self) -> &str {
    self.name
  }
}

// =============================================================================
// Super Mario Bros. reward extractor
// =============================================================================

/// RAM addresses for Super Mario Bros. (US, PRG0/PRG1).
///
/// Sources:
/// - <https://datacrystal.tcrf.net/wiki/Super_Mario_Bros.:RAM_map>
/// - <https://www.nesdev.org/wiki/Super_Mario_Bros.>
mod smb1_ram {
  #![allow(dead_code)]
  /// Player X position within the current screen (0-255 pixels).
  pub const PLAYER_X_PIXEL: u16 = 0x0086;
  /// Player X position in pages (each page = 256 pixels).
  pub const PLAYER_X_PAGE: u16 = 0x006D;
  /// Player state: 0=dead, else alive.
  pub const PLAYER_STATE: u16 = 0x000E;
  /// Current world (0-indexed).
  pub const WORLD: u16 = 0x075F;
  /// Current level within world (0-indexed).
  pub const LEVEL: u16 = 0x0760;
  /// Score digits (BCD, 6 bytes at $07DD-$07E2).
  pub const SCORE_START: u16 = 0x07DD;
  /// Number of lives remaining.
  pub const LIVES: u16 = 0x075A;
  /// Timer hundreds digit.
  pub const TIMER_HUNDREDS: u16 = 0x07F8;
  /// Timer tens digit.
  pub const TIMER_TENS: u16 = 0x07F9;
  /// Timer ones digit.
  pub const TIMER_ONES: u16 = 0x07FA;
  /// Player float state (0x03 = dying animation).
  pub const PLAYER_FLOAT_STATE: u16 = 0x001D;
  /// Game engine state (e.g., 0x0B = dying, 0x06 = playing).
  pub const GAME_ENGINE_STATE: u16 = 0x0770;
  /// Flag area offset (increments on level clear).
  pub const LEVEL_CLEAR_FLAG: u16 = 0x0001;
}

/// Reward extractor for Super Mario Bros. (US).
///
/// Reward signal components:
/// - **Horizontal progress** (primary): delta of x-position in world coordinates.
///   Scaled so a full screen (256 pixels) ≈ 1.0 reward.
/// - **Death penalty**: -15.0 when player dies.
/// - **Time penalty**: small negative reward per frame to encourage speed.
/// - **Score bonus**: small reward for collecting coins and killing enemies.
pub struct SuperMarioBrosReward {
  prev_x: i32,
  prev_score: u32,
  prev_lives: u8,
  prev_world: u8,
  prev_level: u8,
  was_dead: bool,
}

impl SuperMarioBrosReward {
  pub fn new() -> Self {
    SuperMarioBrosReward {
      prev_x: 0,
      prev_score: 0,
      prev_lives: 0,
      prev_world: 0,
      prev_level: 0,
      was_dead: false,
    }
  }

  fn read_x_position(emu: &mut NesEmulator) -> i32 {
    let page = emu.ram_read(smb1_ram::PLAYER_X_PAGE) as i32;
    let pixel = emu.ram_read(smb1_ram::PLAYER_X_PIXEL) as i32;
    page * 256 + pixel
  }

  fn read_score(emu: &mut NesEmulator) -> u32 {
    // Score is stored as 6 BCD digits at $07DD-$07E2
    let mut score: u32 = 0;
    for i in 0..6 {
      let digit = emu.ram_read(smb1_ram::SCORE_START + i) as u32;
      score = score * 10 + (digit & 0x0F);
    }
    score
  }

  fn is_dead(emu: &mut NesEmulator) -> bool {
    let engine_state = emu.ram_read(smb1_ram::GAME_ENGINE_STATE);
    let float_state = emu.ram_read(smb1_ram::PLAYER_FLOAT_STATE);
    engine_state == 0x0B || float_state == 0x03
  }
}

impl Default for SuperMarioBrosReward {
  fn default() -> Self {
    Self::new()
  }
}

impl RewardExtractor for SuperMarioBrosReward {
  fn extract(&mut self, emu: &mut NesEmulator) -> f32 {
    let mut reward: f32 = 0.0;

    // Horizontal progress reward
    let x = Self::read_x_position(emu);
    let x_delta = x - self.prev_x;

    // Check for level transition (x resets to 0)
    let world = emu.ram_read(smb1_ram::WORLD);
    let level = emu.ram_read(smb1_ram::LEVEL);
    if world != self.prev_world || level != self.prev_level {
      // Level changed — don't penalize the x reset
      // Give a bonus for completing a level
      reward += 10.0;
      self.prev_world = world;
      self.prev_level = level;
    } else if x_delta.abs() < 128 {
      // Normal movement (ignore teleports > half screen)
      reward += x_delta as f32 / 256.0; // ~1.0 per screen
    }
    self.prev_x = x;

    // Score bonus (small)
    let score = Self::read_score(emu);
    if score > self.prev_score {
      reward += (score - self.prev_score) as f32 / 1000.0;
    }
    self.prev_score = score;

    // Death penalty
    let dead = Self::is_dead(emu);
    if dead && !self.was_dead {
      reward -= 15.0;
    }
    self.was_dead = dead;

    // Time penalty (encourage speed)
    reward -= 0.001;

    reward
  }

  fn reset(&mut self) {
    self.prev_x = 0;
    self.prev_score = 0;
    self.prev_lives = 0;
    self.prev_world = 0;
    self.prev_level = 0;
    self.was_dead = false;
  }

  fn name(&self) -> &str {
    "super_mario_bros"
  }
}

// =============================================================================
// Composite reward: combine multiple extractors
// =============================================================================

/// Combines multiple reward extractors with weights.
pub struct CompositeReward {
  name: String,
  extractors: Vec<(f32, Box<dyn RewardExtractor>)>,
}

impl CompositeReward {
  pub fn new(name: &str) -> Self {
    CompositeReward {
      name: name.to_string(),
      extractors: Vec::new(),
    }
  }

  /// Add a weighted reward extractor.
  pub fn add(mut self, weight: f32, extractor: Box<dyn RewardExtractor>) -> Self {
    self.extractors.push((weight, extractor));
    self
  }
}

impl RewardExtractor for CompositeReward {
  fn extract(&mut self, emu: &mut NesEmulator) -> f32 {
    self.extractors.iter_mut().map(|(w, e)| *w * e.extract(emu)).sum()
  }

  fn reset(&mut self) {
    for (_, e) in &mut self.extractors {
      e.reset();
    }
  }

  fn name(&self) -> &str {
    &self.name
  }
}

// =============================================================================
// Clipped reward wrapper
// =============================================================================

/// Clips reward to [-1, 1] range (standard DQN practice).
pub struct ClippedReward {
  inner: Box<dyn RewardExtractor>,
}

impl ClippedReward {
  pub fn new(inner: Box<dyn RewardExtractor>) -> Self {
    ClippedReward { inner }
  }
}

impl RewardExtractor for ClippedReward {
  fn extract(&mut self, emu: &mut NesEmulator) -> f32 {
    self.inner.extract(emu).clamp(-1.0, 1.0)
  }

  fn reset(&mut self) {
    self.inner.reset();
  }

  fn name(&self) -> &str {
    self.inner.name()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test::init;

  fn build_test_rom() -> Vec<u8> {
    let mut data = vec![
      0x4e, 0x45, 0x53, 0x1a, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    data.resize(16 + 0x8000, 0x00);
    data[16] = 0x4C;
    data[17] = 0x00;
    data[18] = 0x80;
    data[16 + 0x7FFC] = 0x00;
    data[16 + 0x7FFD] = 0x80;
    data.resize(16 + 0x8000 + 0x2000, 0x00);
    data
  }

  #[test]
  fn test_null_reward() {
    init();
    let rom = build_test_rom();
    let mut emu = NesEmulator::new(&rom);
    let mut reward = NullReward;
    assert_eq!(reward.extract(&mut emu), 0.0);
    assert_eq!(reward.name(), "null");
  }

  #[test]
  fn test_single_byte_reward() {
    init();
    let rom = build_test_rom();
    let mut emu = NesEmulator::new(&rom);
    let mut reward = SingleByteReward::new("test", 0x0200, 1.0);

    // Initial read - delta from 0
    emu.ram_write(0x0200, 10);
    let r = reward.extract(&mut emu);
    assert_eq!(r, 10.0);

    // Delta = 5
    emu.ram_write(0x0200, 15);
    let r = reward.extract(&mut emu);
    assert_eq!(r, 5.0);

    // No change
    let r = reward.extract(&mut emu);
    assert_eq!(r, 0.0);

    // Decrease
    emu.ram_write(0x0200, 10);
    let r = reward.extract(&mut emu);
    assert_eq!(r, -5.0);
  }

  #[test]
  fn test_composite_reward() {
    init();
    let rom = build_test_rom();
    let mut emu = NesEmulator::new(&rom);
    let mut reward = CompositeReward::new("combo")
      .add(2.0, Box::new(NullReward))
      .add(1.0, Box::new(NullReward));

    let r = reward.extract(&mut emu);
    assert_eq!(r, 0.0);
    assert_eq!(reward.name(), "combo");
  }

  #[test]
  fn test_clipped_reward() {
    init();
    let rom = build_test_rom();
    let mut emu = NesEmulator::new(&rom);

    // Use a SingleByteReward with big scale to test clipping
    let inner = SingleByteReward::new("big", 0x0200, 100.0);
    let mut reward = ClippedReward::new(Box::new(inner));

    emu.ram_write(0x0200, 50);
    let r = reward.extract(&mut emu);
    assert_eq!(r, 1.0); // Clipped from 5000.0

    emu.ram_write(0x0200, 0);
    let r = reward.extract(&mut emu);
    assert_eq!(r, -1.0); // Clipped from -5000.0
  }

  #[test]
  fn test_smb1_reward_construction() {
    let mut r = SuperMarioBrosReward::new();
    assert_eq!(r.name(), "super_mario_bros");
    r.reset();
  }
}
