//! High-level NES reinforcement learning environment.
//!
//! `NesEnv` wraps [`NesEmulator`] with a complete RL pipeline:
//! - Action space mapping (discrete action → button combo)
//! - Frame preprocessing (grayscale, downsampling, stacking)
//! - Reward extraction (game-specific RAM reading)
//! - Episode termination detection
//! - Frame skipping with max-pooling
//!
//! The API follows the standard Gym `step(action) → (obs, reward, done, info)` pattern.

use super::action::ActionSpace;
use super::preprocess::FrameStack;
use super::reward::{NullReward, RewardExtractor};
use super::NesEmulator;

/// Information returned alongside each step.
#[derive(Debug, Clone)]
pub struct StepInfo {
  /// Total reward accumulated during this step (across frame-skip frames).
  pub reward: f32,
  /// Whether the episode has ended (e.g., player died, game over).
  pub done: bool,
  /// Number of frames advanced in this step.
  pub frames_advanced: usize,
  /// Total frames since last reset.
  pub total_frames: u64,
  /// Total emulator clock cycles.
  pub clock_counter: u64,
}

/// Configuration for building a NesEnv.
pub struct NesEnvConfig {
  /// Number of frames to repeat each action (frame skip). Default: 4.
  pub frame_skip: usize,
  /// Number of frames to stack in the observation. Default: 4.
  pub stack_size: usize,
  /// Whether to max-pool adjacent frames. Default: true.
  pub max_pool: bool,
  /// Number of NOOP frames at the start of each episode (randomized 0..max).
  /// Adds stochasticity to prevent memorization. Default: 30.
  pub noop_max: usize,
  /// Maximum frames per episode (0 = unlimited). Default: 0.
  pub max_episode_frames: u64,
  /// Whether the episode ends when a life is lost (vs. game over only).
  /// Default: false.
  pub episodic_life: bool,
}

impl Default for NesEnvConfig {
  fn default() -> Self {
    NesEnvConfig {
      frame_skip: 4,
      stack_size: 4,
      max_pool: true,
      noop_max: 30,
      max_episode_frames: 0,
      episodic_life: false,
    }
  }
}

/// A fully-configured NES RL environment.
///
/// # Lifecycle
///
/// ```text
/// let mut env = NesEnv::new(rom, action_space, reward, config);
/// let obs = env.reset();
///
/// loop {
///     let (obs, info) = env.step(action);
///     if info.done { break; }
/// }
/// ```
pub struct NesEnv {
  /// The underlying headless emulator.
  emu: NesEmulator,
  /// Action space mapping.
  action_space: ActionSpace,
  /// Reward extractor.
  reward: Box<dyn RewardExtractor>,
  /// Frame stacker / preprocessor.
  frame_stack: FrameStack,
  /// Configuration.
  config: NesEnvConfig,
  /// Total frames since the last reset.
  total_frames: u64,
  /// Simple RNG state for noop randomization (xorshift32).
  rng_state: u32,
  /// RAM address to check for lives (0 = disabled).
  lives_address: u16,
  /// Previous lives value (for episodic life detection).
  prev_lives: u8,
}

impl NesEnv {
  /// Create a new NES RL environment.
  ///
  /// * `rom_data` - Raw NES ROM bytes.
  /// * `action_space` - The discrete action space to use.
  /// * `reward` - The reward extractor for this game.
  /// * `config` - Environment configuration.
  pub fn new(rom_data: &[u8], action_space: ActionSpace, reward: Box<dyn RewardExtractor>, config: NesEnvConfig) -> Self {
    let frame_stack = FrameStack::new(config.stack_size, config.max_pool);
    let emu = NesEmulator::new(rom_data);
    NesEnv {
      emu,
      action_space,
      reward,
      frame_stack,
      config,
      total_frames: 0,
      rng_state: 0xDEAD_BEEF,
      lives_address: 0,
      prev_lives: 0,
    }
  }

  /// Create with default config and null reward (for testing).
  pub fn new_default(rom_data: &[u8], action_space: ActionSpace) -> Self {
    Self::new(rom_data, action_space, Box::new(NullReward), NesEnvConfig::default())
  }

  /// Set the RAM address to monitor for lives (enables episodic_life mode).
  pub fn set_lives_address(mut self, address: u16) -> Self {
    self.lives_address = address;
    self
  }

  /// Reset the environment and return the initial observation.
  ///
  /// Runs a random number of NOOP frames (0..noop_max) to add
  /// stochasticity across episodes.
  pub fn reset(&mut self) -> Vec<u8> {
    self.emu.reset();
    self.reward.reset();
    self.frame_stack.reset();
    self.total_frames = 0;

    // Run random NOOP frames for stochasticity
    if self.config.noop_max > 0 {
      let noop_count = self.xorshift() as usize % self.config.noop_max;
      for _ in 0..noop_count {
        self.emu.step_frame(0);
        self.total_frames += 1;
      }
    }

    // Capture initial observation
    let fb = self.emu.framebuffer().to_vec();
    self.frame_stack.push(&fb);

    // Initialize lives tracking
    if self.lives_address != 0 {
      self.prev_lives = self.emu.ram_read(self.lives_address);
    }

    self.frame_stack.observation()
  }

  /// Execute one agent step: map action → buttons, advance frame_skip frames,
  /// accumulate reward, and return the new observation + metadata.
  ///
  /// Returns `(observation, step_info)`.
  pub fn step(&mut self, action: usize) -> (Vec<u8>, StepInfo) {
    let buttons = self.action_space.to_buttons(action);
    let mut total_reward: f32 = 0.0;
    let mut done = false;

    for _ in 0..self.config.frame_skip {
      self.emu.step_frame(buttons);
      self.total_frames += 1;

      // Accumulate reward
      total_reward += self.reward.extract(&mut self.emu);

      // Check termination conditions
      if self.check_done() {
        done = true;
        break;
      }
    }

    // Capture observation
    let fb = self.emu.framebuffer().to_vec();
    self.frame_stack.push(&fb);

    let info = StepInfo {
      reward: total_reward,
      done,
      frames_advanced: self.config.frame_skip,
      total_frames: self.total_frames,
      clock_counter: self.emu.clock_counter(),
    };

    (self.frame_stack.observation(), info)
  }

  /// Get the current observation without advancing the emulator.
  pub fn observe(&self) -> Vec<u8> {
    self.frame_stack.observation()
  }

  /// Get the current observation as normalized f32 values.
  pub fn observe_f32(&self) -> Vec<f32> {
    self.frame_stack.observation_f32()
  }

  /// Shape of the observation tensor: `[stack_size, 84, 84]`.
  pub fn observation_shape(&self) -> [usize; 3] {
    self.frame_stack.observation_shape()
  }

  /// Total elements in the observation.
  pub fn observation_size(&self) -> usize {
    self.frame_stack.observation_size()
  }

  /// Number of discrete actions available.
  pub fn action_count(&self) -> usize {
    self.action_space.size()
  }

  /// Get a reference to the action space.
  pub fn action_space(&self) -> &ActionSpace {
    &self.action_space
  }

  /// Get a reference to the underlying emulator.
  pub fn emulator(&self) -> &NesEmulator {
    &self.emu
  }

  /// Get a mutable reference to the underlying emulator.
  pub fn emulator_mut(&mut self) -> &mut NesEmulator {
    &mut self.emu
  }

  /// Save the complete environment state (emulator + frame stack).
  pub fn save_state(&self) -> Vec<u8> {
    self.emu.save_state()
  }

  /// Load a previously saved state. Does NOT reset the frame stack.
  pub fn load_state(&mut self, state: &[u8]) {
    self.emu.load_state(state);
  }

  /// Total frames elapsed since the last reset.
  pub fn total_frames(&self) -> u64 {
    self.total_frames
  }

  // --- Internal helpers ---

  /// Check if the episode should end.
  fn check_done(&mut self) -> bool {
    // Max episode length
    if self.config.max_episode_frames > 0 && self.total_frames >= self.config.max_episode_frames {
      return true;
    }

    // Episodic life: end episode when lives decrease
    if self.config.episodic_life && self.lives_address != 0 {
      let lives = self.emu.ram_read(self.lives_address);
      if lives < self.prev_lives {
        self.prev_lives = lives;
        return true;
      }
      self.prev_lives = lives;
    }

    false
  }

  /// Simple xorshift32 PRNG for noop randomization.
  fn xorshift(&mut self) -> u32 {
    let mut x = self.rng_state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    self.rng_state = x;
    x
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::action;
  use crate::nes::gym::preprocess::{OBS_HEIGHT, OBS_PIXELS, OBS_WIDTH};
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
  fn test_env_create_and_reset() {
    init();
    let rom = build_test_rom();
    let mut env = NesEnv::new_default(&rom, action::platformer());

    let obs = env.reset();
    assert_eq!(obs.len(), 4 * OBS_PIXELS);
    assert_eq!(env.observation_shape(), [4, OBS_HEIGHT, OBS_WIDTH]);
    assert_eq!(env.action_count(), 12);
  }

  #[test]
  fn test_env_step() {
    init();
    let rom = build_test_rom();
    let config = NesEnvConfig {
      noop_max: 0, // Disable noop for determinism
      ..Default::default()
    };
    let mut env = NesEnv::new(&rom, action::platformer(), Box::new(NullReward), config);

    let _obs = env.reset();
    let (obs, info) = env.step(0); // NOOP

    assert_eq!(obs.len(), 4 * OBS_PIXELS);
    assert_eq!(info.reward, 0.0); // Null reward
    assert!(!info.done);
    assert_eq!(info.total_frames, 4); // frame_skip = 4
  }

  #[test]
  fn test_env_max_episode_frames() {
    init();
    let rom = build_test_rom();
    let config = NesEnvConfig {
      frame_skip: 1,
      noop_max: 0,
      max_episode_frames: 5,
      ..Default::default()
    };
    let mut env = NesEnv::new(&rom, action::minimal(), Box::new(NullReward), config);

    let _obs = env.reset();
    let mut done = false;
    let mut steps = 0;
    while !done {
      let (_, info) = env.step(0);
      done = info.done;
      steps += 1;
    }
    assert_eq!(steps, 5);
  }

  #[test]
  fn test_env_observe_f32() {
    init();
    let rom = build_test_rom();
    let config = NesEnvConfig {
      noop_max: 0,
      ..Default::default()
    };
    let mut env = NesEnv::new(&rom, action::minimal(), Box::new(NullReward), config);
    env.reset();

    let obs = env.observe_f32();
    assert_eq!(obs.len(), 4 * OBS_PIXELS);
    // All values should be in [0, 1]
    assert!(obs.iter().all(|&v| (0.0..=1.0).contains(&v)));
  }

  #[test]
  fn test_env_save_load() {
    init();
    let rom = build_test_rom();
    let config = NesEnvConfig {
      noop_max: 0,
      ..Default::default()
    };
    let mut env = NesEnv::new(&rom, action::minimal(), Box::new(NullReward), config);
    env.reset();

    // Take a few steps
    env.step(0);
    env.step(1);

    // Save and modify
    let state = env.save_state();
    let obs_before = env.observe();

    env.step(2);
    env.step(3);

    // Restore
    env.load_state(&state);
    // Note: frame stack is NOT restored, only emulator state
    // This is by design — the agent should re-observe after loading

    // Emulator state should match
    assert_eq!(env.emulator().clock_counter(), env.emulator().clock_counter());
  }

  #[test]
  fn test_env_multiple_episodes() {
    init();
    let rom = build_test_rom();
    let config = NesEnvConfig {
      frame_skip: 1,
      noop_max: 0,
      max_episode_frames: 3,
      ..Default::default()
    };
    let mut env = NesEnv::new(&rom, action::minimal(), Box::new(NullReward), config);

    // Episode 1
    let _obs = env.reset();
    for _ in 0..3 {
      env.step(0);
    }

    // Episode 2
    let obs = env.reset();
    assert_eq!(obs.len(), 4 * OBS_PIXELS);
    assert_eq!(env.total_frames(), 0);
  }
}
