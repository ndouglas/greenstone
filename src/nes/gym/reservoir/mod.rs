//! Tetris as a reservoir computer, driven by Super Mario Bros.
//!
//! See `docs/superpowers/specs/2026-10-13-tetris-reservoir-design.md`.

pub mod demo;
pub mod drive;
pub mod features;
pub mod ridge;

use crate::nes::bus::{BUTTON_SELECT, BUTTON_START};
use crate::nes::gym::NesEmulator;
use demo::{beam_search, smb_cleared, smb_dead, smb_start_state, smb_window, smb_x, Demonstration, MACRO_FRAMES};
use drive::{fold_to_controller, Reservoir};
use features::{clock_row, feature_row, Condition};
use ridge::{fit, Readout};

pub struct ExperimentConfig {
  pub lambda: f64,
  pub beam: usize,
  pub demo_max_rounds: usize,
  pub episodes: usize,
  pub max_steps: usize,
  pub noop_max: usize,
  pub train_fraction: f64,
}

impl Default for ExperimentConfig {
  fn default() -> Self {
    ExperimentConfig {
      lambda: 1.0,
      beam: 16,
      demo_max_rounds: 100,
      episodes: 5,
      max_steps: 750,
      noop_max: 30,
      train_fraction: 0.8,
    }
  }
}

#[derive(Debug, Clone)]
pub struct ConditionResult {
  pub condition: Condition,
  pub rows: usize,
  pub accuracy: f64,
  /// Mean squared error of the raw outputs on the held-out rows, over the active bits.
  pub mse: f64,
  pub mean_x: f64,
  pub max_x: i32,
  pub resets: u32,
}

#[derive(Debug, Clone)]
pub struct ExperimentReport {
  pub demo_steps: usize,
  pub demo_x: i32,
  pub demo_cleared: bool,
  pub active_bits: u8,
  pub results: Vec<ConditionResult>,
}

/// First index of the test split. Needs at least ten rows so the test split is non-empty.
pub fn split_index(n: usize, train_fraction: f64) -> Result<usize, String> {
  if n < 10 {
    return Err(format!("demonstration has only {} steps; need at least 10", n));
  }
  if !(train_fraction > 0.0 && train_fraction < 1.0) {
    return Err(format!("train_fraction must be strictly between 0 and 1, got {}", train_fraction));
  }
  let split = ((n as f64) * train_fraction).floor() as usize;
  if split == 0 || split >= n {
    return Err(format!("split {} of {} rows leaves an empty train or test set", split, n));
  }
  Ok(split)
}

pub fn buttons_to_targets(buttons: u8) -> Vec<f32> {
  (0..8).map(|i| ((buttons >> i) & 1) as f32).collect()
}

/// Threshold each output at 0.5. Start and Select are never pressed.
pub fn outputs_to_buttons(outputs: &[f64]) -> u8 {
  let mut buttons = 0u8;
  for (i, &o) in outputs.iter().enumerate().take(8) {
    if o > 0.5 {
      buttons |= 1 << i;
    }
  }
  buttons & !(BUTTON_START | BUTTON_SELECT)
}

pub fn active_bits(buttons: &[u8]) -> u8 {
  buttons.iter().fold(0, |acc, &b| acc | b)
}

/// Fraction of masked bits on which prediction and truth agree.
pub fn bit_accuracy(pred: &[u8], truth: &[u8], mask: u8) -> f64 {
  let per_row = mask.count_ones() as usize;
  if per_row == 0 || pred.is_empty() {
    return 0.0;
  }
  let correct: usize = pred.iter().zip(truth).map(|(&p, &t)| (!(p ^ t) & mask).count_ones() as usize).sum();
  correct as f64 / (per_row * pred.len()) as f64
}

/// Mean squared error of raw outputs against 0/1 targets, over the masked bits.
pub fn output_mse(outputs: &[Vec<f64>], targets: &[Vec<f32>], mask: u8) -> f64 {
  let per_row = mask.count_ones() as usize;
  if per_row == 0 || outputs.is_empty() {
    return 0.0;
  }
  let total: f64 = outputs
    .iter()
    .zip(targets)
    .map(|(out, target)| {
      (0..8)
        .filter(|i| (mask >> i) & 1 == 1)
        .map(|i| (out[i] - target[i] as f64).powi(2))
        .sum::<f64>()
    })
    .sum();
  total / (per_row * outputs.len()) as f64
}

/// Episode indices per worker thread, round-robin, with no more threads than episodes.
pub fn assign_round_robin(episodes: usize, threads: usize) -> Vec<Vec<usize>> {
  let threads = threads.min(episodes);
  (0..threads).map(|t| (t..episodes).step_by(threads).collect()).collect()
}

/// Deterministic per-episode no-op count in `[0, noop_max)`.
pub fn noop_frames(episode: usize, noop_max: usize) -> usize {
  if noop_max == 0 {
    return 0;
  }
  let h = (episode as u64 + 1).wrapping_mul(2654435761).wrapping_mul(0x9E3779B97F4A7C15);
  ((h >> 33) as usize) % noop_max
}

struct Dataset {
  rows: Vec<Vec<f32>>,
  targets: Vec<Vec<f32>>,
  buttons: Vec<u8>,
  resets: u32,
}

/// Step the reservoir with the current window (if the condition has one) and build the row.
fn observe(condition: Condition, window: &[u8], reservoir: Option<&mut Reservoir>, step: usize) -> Vec<f32> {
  match (condition, reservoir) {
    (Condition::Clock, _) => clock_row(step),
    (Condition::Raw, _) => feature_row(window, None),
    (c, Some(r)) => {
      r.step(c.drive(), window, MACRO_FRAMES);
      let bits = if c.appends_drive_bits() {
        Some(fold_to_controller(window))
      } else {
        None
      };
      feature_row(&r.ram(), bits)
    }
    (c, None) => panic!("condition {} needs a reservoir", c.name()),
  }
}

fn collect(condition: Condition, smb_rom: &[u8], tetris_rom: &[u8], start: &[u8], demo: &Demonstration) -> Result<Dataset, String> {
  let mut smb = NesEmulator::new(smb_rom);
  smb.load_state(start);
  let mut reservoir = if condition.uses_reservoir() {
    Some(Reservoir::new(tetris_rom)?)
  } else {
    None
  };
  let mut data = Dataset {
    rows: Vec::new(),
    targets: Vec::new(),
    buttons: Vec::new(),
    resets: 0,
  };
  for (step, &action) in demo.buttons.iter().enumerate() {
    let window = smb_window(&mut smb);
    data.rows.push(observe(condition, &window, reservoir.as_mut(), step));
    data.targets.push(buttons_to_targets(action));
    data.buttons.push(action);
    smb.step_frames(action, MACRO_FRAMES);
  }
  data.resets = reservoir.map(|r| r.resets).unwrap_or(0);
  Ok(data)
}

/// One closed-loop episode. Returns (max x reached, reservoir resets).
fn episode(
  condition: Condition,
  readout: &Readout,
  smb_rom: &[u8],
  tetris_rom: &[u8],
  start: &[u8],
  noops: usize,
  max_steps: usize,
) -> Result<(i32, u32), String> {
  let mut smb = NesEmulator::new(smb_rom);
  smb.load_state(start);
  smb.step_frames(0, noops);
  let mut reservoir = if condition.uses_reservoir() {
    Some(Reservoir::new(tetris_rom)?)
  } else {
    None
  };
  let mut max_x = 0;
  for step in 0..max_steps {
    let window = smb_window(&mut smb);
    let row = observe(condition, &window, reservoir.as_mut(), step);
    let buttons = outputs_to_buttons(&readout.predict(&row));
    smb.step_frames(buttons, MACRO_FRAMES);
    max_x = max_x.max(smb_x(&mut smb));
    if smb_dead(&mut smb) || smb_cleared(&mut smb) {
      break;
    }
  }
  Ok((max_x, reservoir.map(|r| r.resets).unwrap_or(0)))
}

/// Run every condition and return the report. `log` receives progress lines.
pub fn run(smb_rom: &[u8], tetris_rom: &[u8], config: &ExperimentConfig, log: &mut dyn FnMut(&str)) -> Result<ExperimentReport, String> {
  let start = smb_start_state(smb_rom);
  log("searching for a demonstration");
  let demo = beam_search(smb_rom, &start, config.beam, config.demo_max_rounds);
  log(&format!(
    "demonstration: {} steps, x={}, cleared={}",
    demo.buttons.len(),
    demo.x_reached,
    demo.cleared
  ));
  let split = split_index(demo.buttons.len(), config.train_fraction)?;
  let mask = active_bits(&demo.buttons);
  let mut results = Vec::new();
  for condition in Condition::ALL {
    log(&format!("{}: collecting features", condition.name()));
    let data = collect(condition, smb_rom, tetris_rom, &start, &demo)?;
    log(&format!(
      "{}: fitting readout on {} rows of {}",
      condition.name(),
      split,
      data.rows[0].len()
    ));
    let readout = fit(&data.rows[..split], &data.targets[..split], config.lambda)?;
    let outputs: Vec<Vec<f64>> = data.rows[split..].iter().map(|r| readout.predict(r)).collect();
    let preds: Vec<u8> = outputs.iter().map(|o| outputs_to_buttons(o)).collect();
    let accuracy = bit_accuracy(&preds, &data.buttons[split..], mask);
    let mse = output_mse(&outputs, &data.targets[split..], mask);
    log(&format!("{}: running {} episodes", condition.name(), config.episodes));
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
    let outcomes: Vec<(i32, u32)> = std::thread::scope(|s| {
      let handles: Vec<_> = assign_round_robin(config.episodes, threads)
        .into_iter()
        .map(|episodes| {
          let readout = &readout;
          let start = &start;
          let noop_max = config.noop_max;
          let max_steps = config.max_steps;
          s.spawn(move || {
            episodes
              .into_iter()
              .map(|e| episode(condition, readout, smb_rom, tetris_rom, start, noop_frames(e, noop_max), max_steps))
              .collect::<Result<Vec<_>, _>>()
          })
        })
        .collect();
      handles
        .into_iter()
        .map(|h| h.join().expect("episode worker panicked"))
        .collect::<Result<Vec<Vec<_>>, _>>()
        .map(|v| v.into_iter().flatten().collect())
    })?;
    let mean_x = outcomes.iter().map(|o| o.0 as f64).sum::<f64>() / outcomes.len().max(1) as f64;
    let max_x = outcomes.iter().map(|o| o.0).max().unwrap_or(0);
    let resets = data.resets + outcomes.iter().map(|o| o.1).sum::<u32>();
    results.push(ConditionResult {
      condition,
      rows: data.rows.len(),
      accuracy,
      mse,
      mean_x,
      max_x,
      resets,
    });
  }
  Ok(ExperimentReport {
    demo_steps: demo.buttons.len(),
    demo_x: demo.x_reached,
    demo_cleared: demo.cleared,
    active_bits: mask,
    results,
  })
}

/// Read a ROM for an integration test, or print a skip notice and return None.
#[cfg(test)]
pub(crate) fn rom_or_skip(path: &str) -> Option<Vec<u8>> {
  match std::fs::read(path) {
    Ok(bytes) => Some(bytes),
    Err(_) => {
      eprintln!("skipping: {} is not present", path);
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_RIGHT, BUTTON_SELECT, BUTTON_START};

  #[test]
  fn episodes_are_assigned_round_robin_to_capped_threads() {
    assert_eq!(assign_round_robin(7, 3), vec![vec![0, 3, 6], vec![1, 4], vec![2, 5]]);
    assert_eq!(assign_round_robin(2, 8), vec![vec![0], vec![1]]);
    assert!(assign_round_robin(0, 3).is_empty());
  }

  #[test]
  fn observe_clock_needs_no_reservoir_and_encodes_the_step() {
    let row = observe(Condition::Clock, &[1, 2, 3], None, 7);
    assert_eq!(row.len(), Condition::Clock.row_len());
    assert_eq!(row[1 + 7], 1.0);
    let raw = observe(Condition::Raw, &[1, 2, 3], None, 7);
    assert_eq!(raw.len(), 4);
  }

  #[test]
  fn split_rejects_fewer_than_ten_rows() {
    assert!(split_index(9, 0.8).is_err());
    assert_eq!(split_index(10, 0.8).unwrap(), 8);
    assert_eq!(split_index(100, 0.8).unwrap(), 80);
  }

  #[test]
  fn split_rejects_fractions_that_leave_no_test_rows() {
    assert!(split_index(100, 1.0).is_err());
    assert!(split_index(100, 1.5).is_err());
    assert!(split_index(100, 0.0).is_err());
    assert!(split_index(100, -0.2).is_err());
    assert_eq!(split_index(10, 0.95).unwrap(), 9);
  }

  #[test]
  fn targets_and_buttons_round_trip() {
    let b = BUTTON_A | BUTTON_RIGHT;
    let t = buttons_to_targets(b);
    assert_eq!(t.len(), 8);
    let out: Vec<f64> = t.iter().map(|&v| v as f64).collect();
    assert_eq!(outputs_to_buttons(&out), b);
  }

  #[test]
  fn outputs_never_press_start_or_select() {
    let out = vec![1.0; 8];
    let b = outputs_to_buttons(&out);
    assert_eq!(b & (BUTTON_START | BUTTON_SELECT), 0);
    assert_eq!(b & BUTTON_A, BUTTON_A);
  }

  #[test]
  fn active_bits_is_the_union() {
    assert_eq!(active_bits(&[BUTTON_A, BUTTON_RIGHT, 0]), BUTTON_A | BUTTON_RIGHT);
  }

  #[test]
  fn bit_accuracy_counts_only_masked_bits() {
    let truth = [BUTTON_A | BUTTON_RIGHT, BUTTON_B];
    let pred = [BUTTON_A, BUTTON_B | BUTTON_RIGHT];
    let mask = BUTTON_A | BUTTON_B | BUTTON_RIGHT;
    // Row 0: A right, B right, Right wrong. Row 1: A right, B right, Right wrong. 4 of 6.
    assert!((bit_accuracy(&pred, &truth, mask) - 4.0 / 6.0).abs() < 1e-9);
    assert_eq!(bit_accuracy(&pred, &truth, BUTTON_A), 1.0);
  }

  #[test]
  fn output_mse_averages_squared_error_over_masked_bits() {
    // Two rows, mask of two bits (A and B). Outputs are off by 0.5 on A only.
    let outputs = vec![vec![0.5, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], vec![0.5, 0.0, 9.0, 0.0, 0.0, 0.0, 0.0, 0.0]];
    let targets = vec![buttons_to_targets(BUTTON_A | BUTTON_B), buttons_to_targets(0)];
    let mse = output_mse(&outputs, &targets, BUTTON_A | BUTTON_B);
    assert!((mse - 0.125).abs() < 1e-9, "got {}", mse);
  }

  #[test]
  fn noop_frames_are_deterministic_and_bounded() {
    for e in 0..100 {
      let n = noop_frames(e, 30);
      assert!(n < 30);
      assert_eq!(n, noop_frames(e, 30));
    }
    assert_eq!(noop_frames(5, 0), 0);
  }
}
