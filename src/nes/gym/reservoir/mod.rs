//! Tetris as a reservoir computer, driven by Super Mario Bros.
//!
//! See `docs/superpowers/specs/2026-10-13-tetris-reservoir-design.md`.

pub mod demo;
pub mod drive;
pub mod encode;
pub mod features;
pub mod ridge;
pub mod search;
pub mod trace;

use crate::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_RIGHT, BUTTON_SELECT, BUTTON_START};
use crate::nes::gym::NesEmulator;
use demo::{beam_search, smb_cleared, smb_dead, smb_in_cutscene, smb_start_state, smb_window, smb_x, Demonstration, MACRO_FRAMES};
use drive::{fold_to_controller, Reservoir};
use encode::{encode, onehot_row, Encoder};
use features::{clock_row, feature_row, Condition};
use ridge::{fit, Readout};

pub struct ExperimentConfig {
  pub lambda: f64,
  pub beam: usize,
  pub demo_max_rounds: usize,
  /// Demonstrations to search for, each from its own no-op offset and tie-break seed.
  pub demonstrations: usize,
  pub episodes: usize,
  pub max_steps: usize,
  pub noop_max: usize,
  /// The A label at a step is "A within the next `jump_horizon` steps", which
  /// absorbs the demonstrations' arbitrary jump phase. 0 is the raw label.
  pub jump_horizon: usize,
}

impl Default for ExperimentConfig {
  fn default() -> Self {
    ExperimentConfig {
      lambda: 1.0,
      beam: 16,
      demo_max_rounds: 100,
      demonstrations: 6,
      episodes: 5,
      max_steps: 750,
      noop_max: 30,
      jump_horizon: 4,
    }
  }
}

/// Set A at every step that has A pressed within the next `horizon` steps
/// (including itself). Other bits are untouched.
pub fn smooth_a(buttons: &[u8], horizon: usize) -> Vec<u8> {
  (0..buttons.len())
    .map(|i| {
      let due = buttons[i..buttons.len().min(i + horizon + 1)].iter().any(|&b| b & BUTTON_A != 0);
      if due {
        buttons[i] | BUTTON_A
      } else {
        buttons[i]
      }
    })
    .collect()
}

#[derive(Debug, Clone)]
pub struct ConditionResult {
  pub condition: Condition,
  /// Informative rows over all demonstrations.
  pub rows: usize,
  /// Bit accuracy on the training rows, pooled over folds.
  pub train_accuracy: f64,
  /// Bit accuracy on the held-out demonstration, pooled over folds.
  pub accuracy: f64,
  /// Mean squared error of the raw outputs on the held-out rows, over the active bits.
  pub mse: f64,
  pub mean_x: f64,
  pub max_x: i32,
  pub resets: u32,
}

#[derive(Debug, Clone)]
pub struct DemoSummary {
  pub steps: usize,
  pub x_reached: i32,
  pub cleared: bool,
  pub noops: usize,
}

#[derive(Debug, Clone)]
pub struct ExperimentReport {
  pub demos: Vec<DemoSummary>,
  pub active_bits: u8,
  pub results: Vec<ConditionResult>,
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

/// A demonstration step is a training row only while the player has control.
/// During a cutscene, such as the castle countdown after the flagpole, the
/// recorded buttons are arbitrary and say nothing about the policy.
pub fn row_is_informative(in_cutscene: bool) -> bool {
  !in_cutscene
}

/// Idle frames before demonstration `index` starts: evenly spaced over `[0, noop_max)`.
pub fn demonstration_noops(index: usize, demonstrations: usize, noop_max: usize) -> usize {
  index * noop_max / demonstrations.max(1)
}

/// Idle frames before closed-loop episode `index` starts: the midpoints between
/// demonstration offsets, so an episode never starts where a demonstration did.
pub fn episode_noops(index: usize, demonstrations: usize, noop_max: usize) -> usize {
  let demonstrations = demonstrations.max(1);
  (2 * (index % demonstrations) + 1) * noop_max / (2 * demonstrations)
}

pub(crate) struct Dataset {
  pub(crate) rows: Vec<Vec<f32>>,
  pub(crate) targets: Vec<Vec<f32>>,
  pub(crate) buttons: Vec<u8>,
  pub(crate) resets: u32,
}

/// A small deterministic generator for the random-A policy.
#[derive(Debug, Clone)]
pub struct Lcg(u64);

impl Lcg {
  pub fn new(seed: u64) -> Lcg {
    Lcg(seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407))
  }

  /// Uniform in [0, 1).
  pub fn next_f64(&mut self) -> f64 {
    self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (self.0 >> 11) as f64 / (1u64 << 53) as f64
  }
}

/// A two-state chain over the A button: how often a jump starts, how long it holds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JumpChain {
  /// P(A this step | no A last step).
  pub p_start: f64,
  /// P(A this step | A last step).
  pub p_hold: f64,
}

impl JumpChain {
  /// Fit the transition rates to button sequences.
  pub fn fit(sequences: &[&[u8]]) -> JumpChain {
    let (mut from_off, mut starts, mut from_on, mut holds) = (0usize, 0usize, 0usize, 0usize);
    for seq in sequences {
      for pair in seq.windows(2) {
        let (prev, next) = (pair[0] & BUTTON_A != 0, pair[1] & BUTTON_A != 0);
        if prev {
          from_on += 1;
          holds += next as usize;
        } else {
          from_off += 1;
          starts += next as usize;
        }
      }
    }
    let rate = |n: usize, d: usize| if d == 0 { 0.0 } else { n as f64 / d as f64 };
    JumpChain {
      p_start: rate(starts, from_off),
      p_hold: rate(holds, from_on),
    }
  }

  /// A deterministic sequence of `len` A decisions for `seed`, starting from no A.
  pub fn sample(&self, seed: u64, len: usize) -> Vec<bool> {
    let mut rng = Lcg::new(seed);
    let mut prev = false;
    (0..len)
      .map(|_| {
        let p = if prev { self.p_hold } else { self.p_start };
        prev = rng.next_f64() < p;
        prev
      })
      .collect()
  }
}

/// The random-A policy's buttons: always run right, A when the chain says so.
pub fn random_a_buttons(a: bool) -> u8 {
  BUTTON_B | BUTTON_RIGHT | if a { BUTTON_A } else { 0 }
}

/// Pooled leave-one-dataset-out metrics.
#[derive(Debug, Clone)]
pub struct FoldMetrics {
  pub rows: usize,
  pub train_accuracy: f64,
  pub accuracy: f64,
  pub mse: f64,
}

/// Hold each dataset out in turn, fit on the rest, and pool the predictions.
pub(crate) fn evaluate_folds(datasets: &[Dataset], lambda: f64, mask: u8) -> Result<FoldMetrics, String> {
  if datasets.len() < 2 {
    return Err(format!("leave-one-out needs at least two demonstrations, got {}", datasets.len()));
  }
  let mut train_preds = Vec::new();
  let mut train_truth = Vec::new();
  let mut test_outputs = Vec::new();
  let mut test_targets = Vec::new();
  let mut test_preds = Vec::new();
  let mut test_truth = Vec::new();
  for held_out in 0..datasets.len() {
    let rows: Vec<Vec<f32>> = datasets
      .iter()
      .enumerate()
      .filter(|(i, _)| *i != held_out)
      .flat_map(|(_, d)| d.rows.iter().cloned())
      .collect();
    let targets: Vec<Vec<f32>> = datasets
      .iter()
      .enumerate()
      .filter(|(i, _)| *i != held_out)
      .flat_map(|(_, d)| d.targets.iter().cloned())
      .collect();
    let readout = fit(&rows, &targets, lambda)?;
    for (i, d) in datasets.iter().enumerate() {
      for (row, &buttons) in d.rows.iter().zip(&d.buttons) {
        let out = readout.predict(row);
        if i == held_out {
          test_preds.push(outputs_to_buttons(&out));
          test_truth.push(buttons);
          test_targets.push(buttons_to_targets(buttons));
          test_outputs.push(out);
        } else {
          train_preds.push(outputs_to_buttons(&out));
          train_truth.push(buttons);
        }
      }
    }
  }
  Ok(FoldMetrics {
    rows: test_preds.len(),
    train_accuracy: bit_accuracy(&train_preds, &train_truth, mask),
    accuracy: bit_accuracy(&test_preds, &test_truth, mask),
    mse: output_mse(&test_outputs, &test_targets, mask),
  })
}

/// The reservoir a condition needs, booted the way its drive needs it, or None.
pub(crate) fn reservoir_for(condition: Condition, tetris_rom: &[u8]) -> Result<Option<Reservoir>, String> {
  if !condition.uses_reservoir() {
    return Ok(None);
  }
  if condition.fast_tetris() {
    Reservoir::new_fast(tetris_rom).map(Some)
  } else {
    Reservoir::new(tetris_rom).map(Some)
  }
}

/// Leave-one-out metrics of the random-A policy: the chain is fitted on the
/// other demonstrations and sampled once per held-out demonstration.
fn random_a_folds(datasets: &[Dataset], mask: u8) -> Result<FoldMetrics, String> {
  if datasets.len() < 2 {
    return Err(format!("leave-one-out needs at least two demonstrations, got {}", datasets.len()));
  }
  let mut preds = Vec::new();
  let mut truth = Vec::new();
  let mut outputs = Vec::new();
  let mut targets = Vec::new();
  for held_out in 0..datasets.len() {
    let others: Vec<&[u8]> = datasets
      .iter()
      .enumerate()
      .filter(|(i, _)| *i != held_out)
      .map(|(_, d)| d.buttons.as_slice())
      .collect();
    let chain = JumpChain::fit(&others);
    let d = &datasets[held_out];
    for (a, &b) in chain.sample(held_out as u64, d.buttons.len()).into_iter().zip(&d.buttons) {
      let p = random_a_buttons(a);
      preds.push(p);
      truth.push(b);
      outputs.push(buttons_to_targets(p).into_iter().map(|v| v as f64).collect());
      targets.push(buttons_to_targets(b));
    }
  }
  let accuracy = bit_accuracy(&preds, &truth, mask);
  Ok(FoldMetrics {
    rows: preds.len(),
    train_accuracy: accuracy,
    accuracy,
    mse: output_mse(&outputs, &targets, mask),
  })
}

/// What chooses the buttons in a closed-loop episode.
enum Driver<'a> {
  Readout(&'a Readout),
  Random(JumpChain, Lcg, bool),
}

impl Driver<'_> {
  fn buttons(&mut self, row: &[f32]) -> u8 {
    match self {
      Driver::Readout(r) => outputs_to_buttons(&r.predict(row)),
      Driver::Random(chain, rng, prev) => {
        let p = if *prev { chain.p_hold } else { chain.p_start };
        *prev = rng.next_f64() < p;
        random_a_buttons(*prev)
      }
    }
  }
}

/// Step the reservoir with the current window (if the condition has one) and build the row.
pub(crate) fn observe(condition: Condition, window: &[u8], reservoir: Option<&mut Reservoir>, step: usize) -> Vec<f32> {
  match (condition, reservoir) {
    (Condition::Clock | Condition::RandomA, _) => clock_row(step),
    (Condition::Raw, _) => feature_row(window, None),
    (Condition::PlacementOnehot(e), _) => onehot_row(&encode(e, window)),
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

pub(crate) fn collect(
  condition: Condition,
  smb_rom: &[u8],
  tetris_rom: &[u8],
  start: &[u8],
  demo: &Demonstration,
  horizon: usize,
) -> Result<Dataset, String> {
  let mut smb = NesEmulator::new(smb_rom);
  smb.load_state(start);
  smb.step_frames(0, demo.noops);
  let mut reservoir = reservoir_for(condition, tetris_rom)?;
  let mut data = Dataset {
    rows: Vec::new(),
    targets: Vec::new(),
    buttons: Vec::new(),
    resets: 0,
  };
  for (step, &action) in demo.buttons.iter().enumerate() {
    let window = smb_window(&mut smb);
    let row = observe(condition, &window, reservoir.as_mut(), step);
    if row_is_informative(smb_in_cutscene(&mut smb)) {
      data.rows.push(row);
      data.targets.push(buttons_to_targets(action));
      data.buttons.push(action);
    }
    smb.step_frames(action, MACRO_FRAMES);
  }
  data.buttons = smooth_a(&data.buttons, horizon);
  data.targets = data.buttons.iter().map(|&b| buttons_to_targets(b)).collect();
  data.resets = reservoir.map(|r| r.resets).unwrap_or(0);
  Ok(data)
}

/// One closed-loop episode. Returns (max x reached, reservoir resets).
fn episode(
  condition: Condition,
  mut driver: Driver,
  smb_rom: &[u8],
  tetris_rom: &[u8],
  start: &[u8],
  noops: usize,
  max_steps: usize,
) -> Result<(i32, u32), String> {
  let mut smb = NesEmulator::new(smb_rom);
  smb.load_state(start);
  smb.step_frames(0, noops);
  let mut reservoir = reservoir_for(condition, tetris_rom)?;
  let mut max_x = 0;
  for step in 0..max_steps {
    let window = smb_window(&mut smb);
    let row = observe(condition, &window, reservoir.as_mut(), step);
    let buttons = driver.buttons(&row);
    smb.step_frames(buttons, MACRO_FRAMES);
    max_x = max_x.max(smb_x(&mut smb));
    if smb_dead(&mut smb) || smb_cleared(&mut smb) {
      break;
    }
  }
  Ok((max_x, reservoir.map(|r| r.resets).unwrap_or(0)))
}

/// Search for `config.demonstrations` demonstrations, each from its own no-op
/// offset and tie-break seed. `log` receives progress lines.
pub fn find_demonstrations(smb_rom: &[u8], config: &ExperimentConfig, log: &mut dyn FnMut(&str)) -> Vec<Demonstration> {
  let start = smb_start_state(smb_rom);
  (0..config.demonstrations)
    .map(|i| {
      let noops = demonstration_noops(i, config.demonstrations, config.noop_max);
      log(&format!("searching for demonstration {} from {} idle frames", i, noops));
      let demo = beam_search(smb_rom, &start, noops, config.beam, config.demo_max_rounds, i as u64);
      log(&format!(
        "demonstration {}: {} steps, x={}, cleared={}",
        i,
        demo.buttons.len(),
        demo.x_reached,
        demo.cleared
      ));
      demo
    })
    .collect()
}

/// Run the given conditions on the given demonstrations and return the report.
/// `log` receives progress lines.
pub fn run(
  smb_rom: &[u8],
  tetris_rom: &[u8],
  demos: &[Demonstration],
  conditions: &[Condition],
  config: &ExperimentConfig,
  log: &mut dyn FnMut(&str),
) -> Result<ExperimentReport, String> {
  if demos.len() < 2 {
    return Err(format!("leave-one-out needs at least two demonstrations, got {}", demos.len()));
  }
  let start = smb_start_state(smb_rom);
  let mask = demos.iter().fold(0, |acc, d| acc | active_bits(&d.buttons));
  let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
  let mut results = Vec::new();
  for &condition in conditions {
    log(&format!("{}: collecting features from {} demonstrations", condition.name(), demos.len()));
    let datasets: Vec<Dataset> = std::thread::scope(|s| {
      let handles: Vec<_> = demos
        .iter()
        .map(|demo| {
          let start = &start;
          let horizon = config.jump_horizon;
          s.spawn(move || collect(condition, smb_rom, tetris_rom, start, demo, horizon))
        })
        .collect();
      handles
        .into_iter()
        .map(|h| h.join().expect("collect worker panicked"))
        .collect::<Result<Vec<_>, _>>()
    })?;
    let rows: usize = datasets.iter().map(|d| d.rows.len()).sum();
    log(&format!(
      "{}: {} informative rows of {}; leave-one-out over {} folds",
      condition.name(),
      rows,
      datasets[0].rows[0].len(),
      datasets.len()
    ));
    let random = condition == Condition::RandomA;
    let folds = if random {
      random_a_folds(&datasets, mask)?
    } else {
      evaluate_folds(&datasets, config.lambda, mask)?
    };
    let all_rows: Vec<Vec<f32>> = datasets.iter().flat_map(|d| d.rows.iter().cloned()).collect();
    let all_targets: Vec<Vec<f32>> = datasets.iter().flat_map(|d| d.targets.iter().cloned()).collect();
    let readout = if random {
      None
    } else {
      Some(fit(&all_rows, &all_targets, config.lambda)?)
    };
    let chain = JumpChain::fit(&datasets.iter().map(|d| d.buttons.as_slice()).collect::<Vec<_>>());
    log(&format!("{}: running {} episodes", condition.name(), config.episodes));
    let outcomes: Vec<(i32, u32)> = std::thread::scope(|s| {
      let handles: Vec<_> = assign_round_robin(config.episodes, threads)
        .into_iter()
        .map(|episodes| {
          let readout = &readout;
          let start = &start;
          let demonstrations = demos.len();
          let noop_max = config.noop_max;
          let max_steps = config.max_steps;
          s.spawn(move || {
            episodes
              .into_iter()
              .map(|e| {
                let noops = episode_noops(e, demonstrations, noop_max);
                let driver = match readout {
                  Some(r) => Driver::Readout(r),
                  None => Driver::Random(chain, Lcg::new(1000 + e as u64), false),
                };
                episode(condition, driver, smb_rom, tetris_rom, start, noops, max_steps)
              })
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
    let resets = datasets.iter().map(|d| d.resets).sum::<u32>() + outcomes.iter().map(|o| o.1).sum::<u32>();
    results.push(ConditionResult {
      condition,
      rows,
      train_accuracy: folds.train_accuracy,
      accuracy: folds.accuracy,
      mse: folds.mse,
      mean_x,
      max_x,
      resets,
    });
  }
  Ok(ExperimentReport {
    demos: demos
      .iter()
      .map(|d| DemoSummary {
        steps: d.buttons.len(),
        x_reached: d.x_reached,
        cleared: d.cleared,
        noops: d.noops,
      })
      .collect(),
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
  fn cutscene_steps_are_not_training_rows() {
    assert!(row_is_informative(false));
    assert!(!row_is_informative(true));
  }

  #[test]
  fn episodes_are_assigned_round_robin_to_capped_threads() {
    assert_eq!(assign_round_robin(7, 3), vec![vec![0, 3, 6], vec![1, 4], vec![2, 5]]);
    assert_eq!(assign_round_robin(2, 8), vec![vec![0], vec![1]]);
    assert!(assign_round_robin(0, 3).is_empty());
  }

  #[test]
  fn observe_onehot_needs_no_reservoir() {
    let window = vec![0u8; 256];
    let row = observe(Condition::PlacementOnehot(Encoder::Salient), &window, None, 0);
    assert_eq!(row.len(), Condition::PlacementOnehot(Encoder::Salient).row_len());
    assert_eq!(row[0], 1.0);
  }

  #[test]
  fn smoothing_sets_a_when_a_jump_is_due_within_the_horizon() {
    let a = BUTTON_A | BUTTON_RIGHT;
    let r = BUTTON_RIGHT;
    let seq = [r, r, a, r, r, r, a, a, r];
    let out = smooth_a(&seq, 2);
    let expect = [a, a, a, r, a, a, a, a, r];
    assert_eq!(out, expect);
    assert_eq!(smooth_a(&seq, 0), seq, "horizon 0 is the raw label");
    assert_eq!(smooth_a(&[], 3), Vec::<u8>::new());
    let b = BUTTON_B;
    assert_eq!(smooth_a(&[b, a], 1), vec![b | BUTTON_A, a], "other bits are kept");
  }

  #[test]
  fn jump_chain_fits_start_and_hold_rates() {
    let a = BUTTON_A | BUTTON_RIGHT;
    let seq = [a, a, a, 0, 0, 0, a, a, 0, 0];
    let chain = JumpChain::fit(&[&seq]);
    assert!((chain.p_hold - 0.6).abs() < 1e-9, "p_hold {}", chain.p_hold);
    assert!((chain.p_start - 0.25).abs() < 1e-9, "p_start {}", chain.p_start);
  }

  #[test]
  fn jump_chain_samples_are_deterministic_per_seed_and_match_the_rate() {
    let chain = JumpChain { p_start: 0.2, p_hold: 0.75 };
    let a: Vec<bool> = chain.sample(7, 4000);
    let b: Vec<bool> = chain.sample(7, 4000);
    assert_eq!(a, b);
    assert_ne!(a, chain.sample(8, 4000));
    // Stationary A rate is p_start / (p_start + 1 - p_hold) = 0.2 / 0.45.
    let rate = a.iter().filter(|&&x| x).count() as f64 / a.len() as f64;
    assert!((rate - 0.2 / 0.45).abs() < 0.05, "rate {}", rate);
  }

  #[test]
  fn random_a_buttons_hold_run_and_toggle_only_a() {
    assert_eq!(random_a_buttons(true), BUTTON_A | BUTTON_B | BUTTON_RIGHT);
    assert_eq!(random_a_buttons(false), BUTTON_B | BUTTON_RIGHT);
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

  fn synthetic_dataset(seed: u32, n: usize) -> Dataset {
    // Target bit A is set exactly when the second feature is above one half.
    let mut data = Dataset {
      rows: Vec::new(),
      targets: Vec::new(),
      buttons: Vec::new(),
      resets: 0,
    };
    for i in 0..n {
      let f = ((i as u32 * 7919 + seed * 104729) % 1000) as f32 / 1000.0;
      let buttons = if f > 0.5 { BUTTON_A } else { 0 };
      data.rows.push(vec![1.0, f]);
      data.targets.push(buttons_to_targets(buttons));
      data.buttons.push(buttons);
    }
    data
  }

  #[test]
  fn folds_hold_out_each_dataset_in_turn() {
    let datasets: Vec<Dataset> = (0..3).map(|s| synthetic_dataset(s, 40)).collect();
    let m = evaluate_folds(&datasets, 1e-6, BUTTON_A).unwrap();
    assert!(m.accuracy > 0.9, "held-out accuracy {}", m.accuracy);
    assert!(m.train_accuracy > 0.9, "train accuracy {}", m.train_accuracy);
    assert!(m.mse < 0.1, "mse {}", m.mse);
    assert_eq!(m.rows, 120);
  }

  #[test]
  fn folds_need_at_least_two_datasets() {
    let datasets = vec![synthetic_dataset(0, 40)];
    assert!(evaluate_folds(&datasets, 1.0, BUTTON_A).is_err());
    assert!(evaluate_folds(&[], 1.0, BUTTON_A).is_err());
  }

  #[test]
  fn demonstration_and_episode_offsets_are_disjoint_for_small_counts() {
    let demos: Vec<usize> = (0..6).map(|i| demonstration_noops(i, 6, 30)).collect();
    assert_eq!(demos, vec![0, 5, 10, 15, 20, 25]);
    let episodes: Vec<usize> = (0..8).map(|e| episode_noops(e, 6, 30)).collect();
    assert_eq!(episodes, vec![2, 7, 12, 17, 22, 27, 2, 7]);
    for e in &episodes {
      assert!(!demos.contains(e), "episode offset {} replays a demonstration", e);
    }
    assert_eq!(demonstration_noops(0, 0, 30), 0);
    assert_eq!(episode_noops(3, 0, 30), 15);
  }
}
