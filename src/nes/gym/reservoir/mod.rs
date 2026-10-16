//! Tetris as a reservoir computer, driven by Super Mario Bros.
//!
//! See `docs/superpowers/specs/2026-10-13-tetris-reservoir-design.md`.

pub mod demo;
pub mod drive;
pub mod encode;
pub mod features;
pub mod logistic;
pub mod ridge;
pub mod search;
pub mod soft;
pub mod teacher;
pub mod trace;

use crate::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_RIGHT, BUTTON_SELECT, BUTTON_START};
use crate::nes::gym::NesEmulator;
use demo::{beam_search, smb_cleared_at, smb_dead, smb_in_cutscene, smb_start_state_at, smb_window, smb_x, Demonstration, Stage, MACRO_FRAMES};
use drive::{column_heights, fold_to_controller, tidy_placement, Drive, Reservoir, PLAYFIELD_ROWS};
use encode::{bank_onehot_row, encode, onehot_row, BankEncoder, Encoder, BANK_BOARDS};
#[cfg(test)]
use encode::{PIECES_PER_STEP, PLACEMENT_ONEHOT};
#[cfg(test)]
use features::SMB_WINDOW_LEN;
use features::{clock_row, feature_row, Condition};
use logistic::fit_logistic;
use ridge::{fit, Readout};
use soft::{Expansion, SOFT_SEED};
use teacher::{rollout, search_rule, stage_starts, Rule};

pub struct ExperimentConfig {
  pub lambda: f64,
  pub beam: usize,
  pub demo_max_rounds: usize,
  /// Demonstrations to search for, each from its own no-op offset and tie-break seed.
  pub demonstrations: usize,
  pub episodes: usize,
  pub max_steps: usize,
  pub noop_max: usize,
  /// The levels demonstrations come from, round robin by index; episodes
  /// follow the demonstrations' stages the same way.
  pub stages: Vec<Stage>,
  /// Play this rule instead of searching the grid, so two runs can share a teacher.
  pub pinned_rule: Option<Rule>,
  /// Levels kept out of every search and fit: the rule is played there for
  /// demonstrations that are only scored, and episodes run there too.
  pub test_stages: Vec<Stage>,
  /// The A label at a step is "A within the next `jump_horizon` steps", which
  /// absorbs the demonstrations' arbitrary jump phase. 0 is the raw label; the
  /// smoothed label raised the floors more than the readouts, so 0 is the default.
  pub jump_horizon: usize,
  /// Fit the readouts by logistic regression instead of ridge. Ridge puts a
  /// rare conjunction like a press onset under the 0.5 threshold; logistic
  /// finds the separating plane if the features hold one.
  pub logistic: bool,
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
      noop_max: 120,
      stages: vec![Stage::ONE_ONE],
      pinned_rule: None,
      test_stages: Vec::new(),
      jump_horizon: 0,
      logistic: false,
    }
  }
}

/// Fit a readout of the kind the experiment is configured for.
pub fn fit_readout(rows: &[Vec<f32>], targets: &[Vec<f32>], lambda: f64, logistic: bool) -> Result<Readout, String> {
  if logistic {
    fit_logistic(rows, targets, lambda)
  } else {
    fit(rows, targets, lambda)
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
  /// Held-out accuracy of the A bit alone.
  pub a_accuracy: f64,
  /// Fraction of held-out press onsets the readout pressed A on.
  pub onset_recall: f64,
  /// The same with one stage held out at a time: fitted on the other
  /// levels' demonstrations. None with a single stage.
  pub cross_onset_recall: Option<f64>,
  /// Onset recall on the test levels' demonstrations, fitted on every
  /// training demonstration. None without test levels.
  pub test_onset_recall: Option<f64>,
  /// Mean closed-loop x on the test levels. None without test levels.
  pub test_mean_x: Option<f64>,
  /// Mean squared error of the raw outputs on the held-out rows, over the active bits.
  pub mse: f64,
  pub mean_x: f64,
  pub max_x: i32,
  /// Closed-loop (stage, mean x, max x, is a test level) per stage, training
  /// stages first in the demonstrations' order.
  pub by_stage: Vec<(Stage, f64, i32, bool)>,
  pub resets: u32,
}

#[derive(Debug, Clone)]
pub struct DemoSummary {
  pub steps: usize,
  pub x_reached: i32,
  pub cleared: bool,
  pub noops: usize,
  pub stage: Stage,
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

/// Press onsets in `truth` (the first step of each A press, in step order)
/// and how many of them `pred` presses A on: (hits, onsets).
pub fn onset_hits(pred: &[u8], truth: &[u8]) -> (usize, usize) {
  let mut hits = 0;
  let mut onsets = 0;
  for i in 0..truth.len() {
    let onset = truth[i] & BUTTON_A != 0 && (i == 0 || truth[i - 1] & BUTTON_A == 0);
    if onset {
      onsets += 1;
      hits += (pred[i] & BUTTON_A != 0) as usize;
    }
  }
  (hits, onsets)
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

/// The clock's step: macro steps since the level started, idle frames
/// included. Counted from the first button instead, every offset has the
/// same step at every x, because Mario starts in one place and nothing
/// moves until he does, and a clock readout memorises the whole level.
pub fn level_step(noops: usize, step: usize) -> usize {
  noops / MACRO_FRAMES + step
}

/// Whether cached demonstrations are the configured number at the offsets the
/// configuration would search from, so a cache from another spread is not reused.
pub fn demos_at_configured_offsets(demos: &[Demonstration], config: &ExperimentConfig) -> bool {
  demos.len() == config.demonstrations
    && demos
      .iter()
      .enumerate()
      .all(|(i, d)| d.noops == demonstration_noops(i, config.demonstrations, config.noop_max) && d.stage == demonstration_stage(i, &config.stages))
}

/// The stage demonstration `index` is played on: the stages in turn.
pub fn demonstration_stage(index: usize, stages: &[Stage]) -> Stage {
  stages.get(index % stages.len().max(1)).copied().unwrap_or(Stage::ONE_ONE)
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

#[derive(Clone)]
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
  /// Held-out accuracy of the A bit alone: the jump decision, without the
  /// walk and wait macros' noise on B and Right.
  pub a_accuracy: f64,
  /// Fraction of the held-out demonstrations' press onsets at which the
  /// readout pressed A: whether it starts jumps, not just recognises them.
  pub onset_recall: f64,
  /// Fraction of the held-out steps without A at which the readout did not
  /// press A: whether it leaves the ground alone.
  pub a_specificity: f64,
  pub mse: f64,
}

impl FoldMetrics {
  /// What the encoder search maximises: onsets started plus A specificity.
  /// Always-A and never-A both score one, so the search cannot buy onsets
  /// with presses. Accuracy alone is dominated by the hold steps and picks
  /// bytes that recognise a jump already under way; onsets plus A accuracy
  /// still paid for presses, since never-A already scores 0.7 on accuracy.
  pub fn onset_score(&self) -> f64 {
    self.onset_recall + self.a_specificity
  }
}

/// Fraction of rows without A in `truth` where `pred` has no A either.
pub fn a_specificity(pred: &[u8], truth: &[u8]) -> f64 {
  let quiet: Vec<usize> = (0..truth.len()).filter(|&i| truth[i] & BUTTON_A == 0).collect();
  let kept = quiet.iter().filter(|&&i| pred[i] & BUTTON_A == 0).count();
  kept as f64 / quiet.len().max(1) as f64
}

/// Hold each dataset out in turn, fit on the rest, and pool the predictions.
pub(crate) fn evaluate_folds(datasets: &[Dataset], lambda: f64, logistic: bool, mask: u8) -> Result<FoldMetrics, String> {
  let singletons: Vec<Vec<usize>> = (0..datasets.len()).map(|i| vec![i]).collect();
  evaluate_groups(datasets, &singletons, lambda, logistic, mask)
    .map_err(|_| format!("leave-one-out needs at least two demonstrations, got {}", datasets.len()))
}

/// The demonstration indices of each stage, in first-seen stage order.
pub fn stage_groups(stages: &[Stage]) -> Vec<Vec<usize>> {
  let mut groups: Vec<(Stage, Vec<usize>)> = Vec::new();
  for (i, &stage) in stages.iter().enumerate() {
    match groups.iter_mut().find(|(s, _)| *s == stage) {
      Some((_, g)) => g.push(i),
      None => groups.push((stage, vec![i])),
    }
  }
  groups.into_iter().map(|(_, g)| g).collect()
}

/// Hold each group of datasets out in turn, fit on the rest, and pool the
/// predictions. Groups of one dataset are leave-one-out; one group per
/// stage asks whether what was learned on one level starts jumps on another.
pub(crate) fn evaluate_groups(datasets: &[Dataset], groups: &[Vec<usize>], lambda: f64, logistic: bool, mask: u8) -> Result<FoldMetrics, String> {
  if groups.len() < 2 {
    return Err(format!("holding groups out needs at least two groups, got {}", groups.len()));
  }
  let mut train_preds = Vec::new();
  let mut train_truth = Vec::new();
  let mut test_outputs = Vec::new();
  let mut test_targets = Vec::new();
  let mut test_preds = Vec::new();
  let mut test_truth = Vec::new();
  let (mut hits, mut onsets) = (0, 0);
  for held_out in groups {
    let rows: Vec<Vec<f32>> = datasets
      .iter()
      .enumerate()
      .filter(|(i, _)| !held_out.contains(i))
      .flat_map(|(_, d)| d.rows.iter().cloned())
      .collect();
    let targets: Vec<Vec<f32>> = datasets
      .iter()
      .enumerate()
      .filter(|(i, _)| !held_out.contains(i))
      .flat_map(|(_, d)| d.targets.iter().cloned())
      .collect();
    let readout = fit_readout(&rows, &targets, lambda, logistic)?;
    for (i, d) in datasets.iter().enumerate() {
      let testing = held_out.contains(&i);
      let mut fold_preds = Vec::new();
      for (row, &buttons) in d.rows.iter().zip(&d.buttons) {
        let out = readout.predict(row);
        if testing {
          fold_preds.push(outputs_to_buttons(&out));
          test_truth.push(buttons);
          test_targets.push(buttons_to_targets(buttons));
          test_outputs.push(out);
        } else {
          train_preds.push(outputs_to_buttons(&out));
          train_truth.push(buttons);
        }
      }
      if testing {
        let (h, o) = onset_hits(&fold_preds, &d.buttons);
        hits += h;
        onsets += o;
        test_preds.extend(fold_preds);
      }
    }
  }
  Ok(FoldMetrics {
    rows: test_preds.len(),
    train_accuracy: bit_accuracy(&train_preds, &train_truth, mask),
    accuracy: bit_accuracy(&test_preds, &test_truth, mask),
    a_accuracy: bit_accuracy(&test_preds, &test_truth, BUTTON_A),
    onset_recall: hits as f64 / onsets.max(1) as f64,
    a_specificity: a_specificity(&test_preds, &test_truth),
    mse: output_mse(&test_outputs, &test_targets, mask),
  })
}

/// A condition's hidden layer: a game, a software expansion, or nothing.
pub(crate) enum Hidden {
  None,
  Tetris(Reservoir),
  Bank(Vec<Reservoir>),
  Soft(Expansion),
  /// Whether A was held on the last step, for the hold control.
  Hold(bool),
}

impl Hidden {
  pub(crate) fn tetris(&self) -> Option<&Reservoir> {
    match self {
      Hidden::Tetris(r) => Some(r),
      _ => None,
    }
  }

  pub(crate) fn tetris_mut(&mut self) -> Option<&mut Reservoir> {
    match self {
      Hidden::Tetris(r) => Some(r),
      _ => None,
    }
  }

  pub(crate) fn resets(&self) -> u32 {
    match self {
      Hidden::Tetris(r) => r.resets,
      Hidden::Bank(boards) => boards.iter().map(|r| r.resets).sum(),
      Hidden::None | Hidden::Soft(_) | Hidden::Hold(_) => 0,
    }
  }

  /// Tell the hidden layer which buttons were pressed this step. Only the
  /// hold control remembers them.
  pub(crate) fn record(&mut self, buttons: u8) {
    if let Hidden::Hold(held) = self {
      *held = buttons & BUTTON_A != 0;
    }
  }
}

/// The hidden layer a condition needs, booted the way its drive needs it.
pub(crate) fn hidden_for(condition: Condition, tetris_rom: &[u8]) -> Result<Hidden, String> {
  if let Condition::BankHold(_) = condition {
    return Ok(Hidden::Hold(false));
  }
  if let Some(recurrent) = condition.soft_reservoir() {
    return Ok(Hidden::Soft(Expansion::new(SOFT_SEED, recurrent)));
  }
  if !condition.uses_reservoir() {
    return Ok(Hidden::None);
  }
  if let Condition::BankTetris(..) | Condition::BankHeights(..) = condition {
    return (0..BANK_BOARDS)
      .map(|_| Reservoir::new_fast(tetris_rom))
      .collect::<Result<Vec<_>, _>>()
      .map(Hidden::Bank);
  }
  if condition.fast_tetris() {
    Reservoir::new_fast(tetris_rom).map(Hidden::Tetris)
  } else {
    Reservoir::new(tetris_rom).map(Hidden::Tetris)
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
  let (mut hits, mut onsets) = (0, 0);
  for held_out in 0..datasets.len() {
    let others: Vec<&[u8]> = datasets
      .iter()
      .enumerate()
      .filter(|(i, _)| *i != held_out)
      .map(|(_, d)| d.buttons.as_slice())
      .collect();
    let chain = JumpChain::fit(&others);
    let d = &datasets[held_out];
    let fold_preds: Vec<u8> = chain.sample(held_out as u64, d.buttons.len()).into_iter().map(random_a_buttons).collect();
    let (h, o) = onset_hits(&fold_preds, &d.buttons);
    hits += h;
    onsets += o;
    for (p, &b) in fold_preds.into_iter().zip(&d.buttons) {
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
    a_accuracy: bit_accuracy(&preds, &truth, BUTTON_A),
    onset_recall: hits as f64 / onsets.max(1) as f64,
    a_specificity: a_specificity(&preds, &truth),
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

/// Step the hidden layer with the current window (if the condition has one) and build the row.
pub(crate) fn observe(condition: Condition, window: &[u8], hidden: &mut Hidden, step: usize) -> Vec<f32> {
  match (condition, hidden) {
    (Condition::Clock | Condition::RandomA, _) => clock_row(step),
    (Condition::Raw, _) => feature_row(window, None),
    (Condition::PlacementOnehot(e), _) => onehot_row(&encode(e, window)),
    (Condition::BankOnehot(e), _) => bank_onehot_row(e, window),
    (Condition::BankHold(e), Hidden::Hold(held)) => {
      let mut row = bank_onehot_row(e, window);
      row.push(*held as u8 as f32);
      row
    }
    (Condition::BankTetris(e, tidy), Hidden::Bank(boards)) => {
      let body: Vec<u8> = step_bank(boards, e, tidy, window).concat();
      feature_row(&body, None)
    }
    (Condition::BankHeights(e, tidy), Hidden::Bank(boards)) => {
      let mut row = vec![1.0];
      for field in step_bank(boards, e, tidy, window) {
        row.extend(column_heights(&field).iter().map(|&h| h as f32 / PLAYFIELD_ROWS as f32));
      }
      row
    }
    (_, Hidden::Soft(e)) => e.step(window),
    (c, Hidden::Tetris(r)) => {
      r.step(c.drive(), window, MACRO_FRAMES);
      let bits = if c.appends_drive_bits() {
        Some(fold_to_controller(window))
      } else {
        None
      };
      feature_row(&r.ram(), bits)
    }
    (c, Hidden::None | Hidden::Bank(_) | Hidden::Hold(_)) => panic!("condition {} has the wrong hidden layer", c.name()),
  }
}

/// Steer each board of the bank by its four placements (and a tidy piece if
/// asked) and return the four playfields.
fn step_bank(boards: &mut [Reservoir], encoder: BankEncoder, tidy: bool, window: &[u8]) -> Vec<Vec<u8>> {
  boards
    .iter_mut()
    .zip(encoder.features())
    .map(|(board, features)| {
      board.step(Some(Drive::Placements(Encoder::Searched(features))), window, MACRO_FRAMES);
      if tidy {
        let target = tidy_placement(&board.playfield());
        board.place(target);
      }
      board.playfield()
    })
    .collect()
}

pub(crate) fn collect(condition: Condition, smb_rom: &[u8], tetris_rom: &[u8], demo: &Demonstration, horizon: usize) -> Result<Dataset, String> {
  let mut smb = NesEmulator::new(smb_rom);
  smb.load_state(&smb_start_state_at(smb_rom, demo.stage));
  smb.step_frames(0, demo.noops);
  let mut hidden = hidden_for(condition, tetris_rom)?;
  let mut data = Dataset {
    rows: Vec::new(),
    targets: Vec::new(),
    buttons: Vec::new(),
    resets: 0,
  };
  for (step, &action) in demo.buttons.iter().enumerate() {
    let window = smb_window(&mut smb);
    let row = observe(condition, &window, &mut hidden, level_step(demo.noops, step));
    if row_is_informative(smb_in_cutscene(&mut smb)) {
      data.rows.push(row);
      data.targets.push(buttons_to_targets(action));
      data.buttons.push(action);
    }
    hidden.record(action);
    smb.step_frames(action, MACRO_FRAMES);
  }
  data.buttons = smooth_a(&data.buttons, horizon);
  data.targets = data.buttons.iter().map(|&b| buttons_to_targets(b)).collect();
  data.resets = hidden.resets();
  Ok(data)
}

/// One closed-loop episode. Returns (max x reached, reservoir resets).
fn episode(
  condition: Condition,
  mut driver: Driver,
  smb_rom: &[u8],
  tetris_rom: &[u8],
  stage: Stage,
  noops: usize,
  max_steps: usize,
) -> Result<(i32, u32), String> {
  let mut smb = NesEmulator::new(smb_rom);
  smb.load_state(&smb_start_state_at(smb_rom, stage));
  smb.step_frames(0, noops);
  let mut hidden = hidden_for(condition, tetris_rom)?;
  let mut max_x = 0;
  for step in 0..max_steps {
    let window = smb_window(&mut smb);
    let row = observe(condition, &window, &mut hidden, level_step(noops, step));
    let buttons = driver.buttons(&row);
    hidden.record(buttons);
    smb.step_frames(buttons, MACRO_FRAMES);
    max_x = max_x.max(smb_x(&mut smb));
    if smb_dead(&mut smb) || smb_cleared_at(&mut smb, stage) {
      break;
    }
  }
  Ok((max_x, hidden.resets()))
}

/// (stage, idle frames) for `per_stage` demonstrations on each test stage,
/// spread over `noop_max` like the training demonstrations.
pub fn test_offsets(stages: &[Stage], per_stage: usize, noop_max: usize) -> Vec<(Stage, usize)> {
  stages
    .iter()
    .flat_map(|&s| (0..per_stage).map(move |i| (s, demonstration_noops(i, per_stage, noop_max))))
    .collect()
}

/// Closed-loop episodes as (stage, idle frames, is a test level): `episodes`
/// on the training stages, following the demonstrations' stages in turn,
/// then `per_test_stage` on each test stage.
pub fn episode_plan(
  demo_stages: &[Stage],
  episodes: usize,
  noop_max: usize,
  test_stages: &[Stage],
  per_test_stage: usize,
) -> Vec<(Stage, usize, bool)> {
  let n = demo_stages.len().max(1);
  let mut plan: Vec<(Stage, usize, bool)> = (0..episodes)
    .map(|e| {
      (
        demo_stages.get(e % n).copied().unwrap_or(Stage::ONE_ONE),
        episode_noops(e, n, noop_max),
        false,
      )
    })
    .collect();
  for &stage in test_stages {
    plan.extend((0..per_test_stage).map(|e| (stage, episode_noops(e, per_test_stage, noop_max), true)));
  }
  plan
}

/// Demonstrations per test stage: as many as each training stage has.
pub fn per_test_stage(config: &ExperimentConfig) -> usize {
  (config.demonstrations / config.stages.len().max(1)).max(2)
}

/// Play `rule` on the test stages for demonstrations that are scored and never fitted.
pub fn find_test_demonstrations(smb_rom: &[u8], rule: Rule, config: &ExperimentConfig) -> Vec<Demonstration> {
  let offsets = test_offsets(&config.test_stages, per_test_stage(config), config.noop_max);
  let starts = stage_starts(smb_rom, offsets.iter().map(|&(s, _)| s));
  offsets
    .iter()
    .map(|&(stage, noops)| rollout(smb_rom, &starts[&stage], noops, rule, config.max_steps, stage))
    .collect()
}

/// A fitted readout's metrics on demonstrations it never saw.
pub(crate) fn score_unseen(readout: &Readout, datasets: &[Dataset], mask: u8) -> FoldMetrics {
  let (mut preds, mut truth, mut outputs, mut targets) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
  let (mut hits, mut onsets) = (0, 0);
  for d in datasets {
    let mut fold = Vec::with_capacity(d.rows.len());
    for row in &d.rows {
      let out = readout.predict(row);
      fold.push(outputs_to_buttons(&out));
      outputs.push(out);
    }
    let (h, o) = onset_hits(&fold, &d.buttons);
    hits += h;
    onsets += o;
    preds.extend(fold);
    truth.extend(d.buttons.iter().copied());
    targets.extend(d.buttons.iter().map(|&b| buttons_to_targets(b)));
  }
  let accuracy = bit_accuracy(&preds, &truth, mask);
  FoldMetrics {
    rows: preds.len(),
    train_accuracy: accuracy,
    accuracy,
    a_accuracy: bit_accuracy(&preds, &truth, BUTTON_A),
    onset_recall: hits as f64 / onsets.max(1) as f64,
    a_specificity: a_specificity(&preds, &truth),
    mse: output_mse(&outputs, &targets, mask),
  }
}

/// Search for `config.demonstrations` demonstrations, each from its own no-op
/// offset and tie-break seed. `log` receives progress lines.
pub fn find_demonstrations(smb_rom: &[u8], config: &ExperimentConfig, log: &mut dyn FnMut(&str)) -> Vec<Demonstration> {
  let starts = stage_starts(smb_rom, config.stages.iter().copied());
  (0..config.demonstrations)
    .map(|i| {
      let noops = demonstration_noops(i, config.demonstrations, config.noop_max);
      let stage = demonstration_stage(i, &config.stages);
      log(&format!(
        "searching for demonstration {} on {} from {} idle frames",
        i,
        stage.name(),
        noops
      ));
      let demo = beam_search(smb_rom, &starts[&stage], noops, config.beam, config.demo_max_rounds, i as u64, stage);
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

/// Search the rule grid and play the best rule from each demonstration offset.
pub fn find_rule_demonstrations(smb_rom: &[u8], config: &ExperimentConfig, log: &mut dyn FnMut(&str)) -> (Rule, Vec<Demonstration>) {
  let offsets: Vec<(Stage, usize)> = (0..config.demonstrations)
    .map(|i| {
      (
        demonstration_stage(i, &config.stages),
        demonstration_noops(i, config.demonstrations, config.noop_max),
      )
    })
    .collect();
  let names: Vec<String> = config.stages.iter().map(|s| s.name()).collect();
  let (rule, demos) = match config.pinned_rule {
    Some(rule) => {
      log(&format!(
        "playing the pinned rule {} from {} offsets on {}",
        rule.key(),
        offsets.len(),
        names.join(", ")
      ));
      let starts = stage_starts(smb_rom, offsets.iter().map(|&(s, _)| s));
      let demos = offsets
        .iter()
        .map(|&(stage, noops)| rollout(smb_rom, &starts[&stage], noops, rule, config.max_steps, stage))
        .collect();
      (rule, demos)
    }
    None => {
      log(&format!(
        "searching {} rules from {} offsets on {}",
        Rule::grid().len(),
        offsets.len(),
        names.join(", ")
      ));
      search_rule(smb_rom, &offsets, config.max_steps, log)
    }
  };
  for (i, demo) in demos.iter().enumerate() {
    log(&format!(
      "demonstration {}: {} steps, x={}, cleared={}",
      i,
      demo.buttons.len(),
      demo.x_reached,
      demo.cleared
    ));
  }
  (rule, demos)
}

/// Run the given conditions on the given demonstrations and return the report.
/// `log` receives progress lines.
pub fn run(
  smb_rom: &[u8],
  tetris_rom: &[u8],
  demos: &[Demonstration],
  test_demos: &[Demonstration],
  conditions: &[Condition],
  config: &ExperimentConfig,
  log: &mut dyn FnMut(&str),
) -> Result<ExperimentReport, String> {
  if demos.len() < 2 {
    return Err(format!("leave-one-out needs at least two demonstrations, got {}", demos.len()));
  }
  let mask = demos.iter().fold(0, |acc, d| acc | active_bits(&d.buttons));
  let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
  let mut results = Vec::new();
  for &condition in conditions {
    log(&format!("{}: collecting features from {} demonstrations", condition.name(), demos.len()));
    let mut datasets: Vec<Dataset> = std::thread::scope(|s| {
      let handles: Vec<_> = demos
        .iter()
        .chain(test_demos)
        .map(|demo| {
          let horizon = config.jump_horizon;
          s.spawn(move || collect(condition, smb_rom, tetris_rom, demo, horizon))
        })
        .collect();
      handles
        .into_iter()
        .map(|h| h.join().expect("collect worker panicked"))
        .collect::<Result<Vec<_>, _>>()
    })?;
    let test_datasets = datasets.split_off(demos.len());
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
      evaluate_folds(&datasets, config.lambda, config.logistic, mask)?
    };
    let stages: Vec<Stage> = demos.iter().map(|d| d.stage).collect();
    let groups = stage_groups(&stages);
    let cross_onset_recall = if random || groups.len() < 2 {
      None
    } else {
      Some(evaluate_groups(&datasets, &groups, config.lambda, config.logistic, mask)?.onset_recall)
    };
    let all_rows: Vec<Vec<f32>> = datasets.iter().flat_map(|d| d.rows.iter().cloned()).collect();
    let all_targets: Vec<Vec<f32>> = datasets.iter().flat_map(|d| d.targets.iter().cloned()).collect();
    let readout = if random {
      None
    } else {
      Some(fit_readout(&all_rows, &all_targets, config.lambda, config.logistic)?)
    };
    let test_onset_recall = match (&readout, test_datasets.is_empty()) {
      (Some(r), false) => Some(score_unseen(r, &test_datasets, mask).onset_recall),
      _ => None,
    };
    let chain = JumpChain::fit(&datasets.iter().map(|d| d.buttons.as_slice()).collect::<Vec<_>>());
    let plan = episode_plan(&stages, config.episodes, config.noop_max, &config.test_stages, per_test_stage(config));
    log(&format!("{}: running {} episodes", condition.name(), plan.len()));
    let outcomes: Vec<(i32, u32)> = std::thread::scope(|s| {
      let handles: Vec<_> = assign_round_robin(plan.len(), threads)
        .into_iter()
        .map(|episodes| {
          let readout = &readout;
          let plan = &plan;
          let max_steps = config.max_steps;
          s.spawn(move || {
            episodes
              .into_iter()
              .map(|e| {
                let (stage, noops, _) = plan[e];
                let driver = match readout {
                  Some(r) => Driver::Readout(r),
                  None => Driver::Random(chain, Lcg::new(1000 + e as u64), false),
                };
                episode(condition, driver, smb_rom, tetris_rom, stage, noops, max_steps)
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
    let xs_where =
      |keep: &dyn Fn(&(Stage, usize, bool)) -> bool| -> Vec<i32> { plan.iter().zip(&outcomes).filter(|(p, _)| keep(p)).map(|(_, o)| o.0).collect() };
    let mean = |xs: &[i32]| xs.iter().map(|&x| x as f64).sum::<f64>() / xs.len().max(1) as f64;
    let training = xs_where(&|p| !p.2);
    let mean_x = mean(&training);
    let max_x = training.iter().copied().max().unwrap_or(0);
    let test_xs = xs_where(&|p| p.2);
    let test_mean_x = if test_xs.is_empty() { None } else { Some(mean(&test_xs)) };
    let mut stage_order: Vec<(Stage, bool)> = groups.iter().map(|g| (stages[g[0]], false)).collect();
    stage_order.extend(config.test_stages.iter().map(|&s| (s, true)));
    let by_stage: Vec<(Stage, f64, i32, bool)> = stage_order
      .into_iter()
      .map(|(stage, test)| {
        let xs = xs_where(&|p| p.0 == stage && p.2 == test);
        (stage, mean(&xs), xs.iter().copied().max().unwrap_or(0), test)
      })
      .collect();
    let resets = datasets.iter().map(|d| d.resets).sum::<u32>() + outcomes.iter().map(|o| o.1).sum::<u32>();
    results.push(ConditionResult {
      condition,
      rows,
      train_accuracy: folds.train_accuracy,
      accuracy: folds.accuracy,
      a_accuracy: folds.a_accuracy,
      onset_recall: folds.onset_recall,
      cross_onset_recall,
      test_onset_recall,
      test_mean_x,
      mse: folds.mse,
      mean_x,
      max_x,
      by_stage,
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
        stage: d.stage,
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
  fn observe_bank_onehot_needs_no_reservoir_and_has_sixteen_placements() {
    let window = vec![0u8; SMB_WINDOW_LEN];
    let row = observe(Condition::BankOnehot(BankEncoder::Hand), &window, &mut Hidden::None, 0);
    assert_eq!(row.len(), Condition::BankOnehot(BankEncoder::Hand).row_len());
    assert_eq!(row.len(), 1 + BANK_BOARDS * PIECES_PER_STEP * PLACEMENT_ONEHOT);
    assert_eq!(row.iter().filter(|&&v| v == 1.0).count(), 1 + 2 * BANK_BOARDS * PIECES_PER_STEP);
  }

  #[test]
  fn the_hold_control_reads_whether_a_was_down_last_step() {
    let window = vec![0u8; SMB_WINDOW_LEN];
    let c = Condition::BankHold(BankEncoder::Hand);
    let mut hidden = hidden_for(c, &[]).unwrap();
    let row = observe(c, &window, &mut hidden, 0);
    assert_eq!(row.len(), c.row_len());
    assert_eq!(row[row.len() - 1], 0.0, "nothing pressed before the first step");
    assert_eq!(row[..row.len() - 1], bank_onehot_row(BankEncoder::Hand, &window)[..]);
    hidden.record(BUTTON_A | BUTTON_RIGHT);
    assert_eq!(*observe(c, &window, &mut hidden, 1).last().unwrap(), 1.0);
    hidden.record(BUTTON_RIGHT);
    assert_eq!(*observe(c, &window, &mut hidden, 2).last().unwrap(), 0.0);
    let mut none = Hidden::None;
    none.record(BUTTON_A); // other hidden layers ignore it
  }

  #[test]
  fn the_episode_plan_runs_training_stages_then_each_test_stage() {
    let one = Stage::ONE_ONE;
    let six = Stage { world: 5, level: 0 };
    let two = Stage { world: 1, level: 0 };
    let plan = episode_plan(&[one, six, one, six, one, six], 5, 120, &[two], 3);
    assert_eq!(plan.len(), 8);
    assert_eq!(plan[..5].iter().map(|p| p.0).collect::<Vec<_>>(), vec![one, six, one, six, one]);
    assert!(plan[..5].iter().all(|p| !p.2), "training episodes are not test episodes");
    assert_eq!(plan[1].1, episode_noops(1, 6, 120));
    assert!(plan[5..].iter().all(|p| p.0 == two && p.2));
    assert_eq!(
      plan[5..].iter().map(|p| p.1).collect::<Vec<_>>(),
      (0..3).map(|e| episode_noops(e, 3, 120)).collect::<Vec<_>>()
    );
    assert_eq!(episode_plan(&[one, one], 4, 120, &[], 3).len(), 4);
  }

  #[test]
  fn test_offsets_spread_each_test_stage_like_the_demonstrations() {
    let two = Stage { world: 1, level: 0 };
    let three = Stage { world: 2, level: 0 };
    let offsets = test_offsets(&[two, three], 3, 120);
    assert_eq!(offsets, vec![(two, 0), (two, 40), (two, 80), (three, 0), (three, 40), (three, 80)]);
  }

  #[test]
  fn a_fitted_readout_is_scored_on_unseen_demonstrations() {
    let train: Vec<Dataset> = (0..3).map(|s| synthetic_dataset(s, 40)).collect();
    let rows: Vec<Vec<f32>> = train.iter().flat_map(|d| d.rows.iter().cloned()).collect();
    let targets: Vec<Vec<f32>> = train.iter().flat_map(|d| d.targets.iter().cloned()).collect();
    let readout = fit_readout(&rows, &targets, 1e-6, false).unwrap();
    let test: Vec<Dataset> = (7..9).map(|s| synthetic_dataset(s, 40)).collect();
    let m = score_unseen(&readout, &test, BUTTON_A);
    assert_eq!(m.rows, 80);
    assert!(m.onset_recall > 0.9, "{}", m.onset_recall);
    assert!(m.a_accuracy > 0.9);
    let mut inverted = test.clone();
    for d in &mut inverted {
      d.buttons = d.buttons.iter().map(|&b| b ^ BUTTON_A).collect();
    }
    assert!(score_unseen(&readout, &inverted, BUTTON_A).onset_recall < 0.1);
  }

  #[test]
  fn observe_onehot_needs_no_reservoir() {
    let window = vec![0u8; 256];
    let row = observe(Condition::PlacementOnehot(Encoder::Salient), &window, &mut Hidden::None, 0);
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
    let row = observe(Condition::Clock, &[1, 2, 3], &mut Hidden::None, 7);
    assert_eq!(row.len(), Condition::Clock.row_len());
    assert_eq!(row[1 + 7], 1.0);
    let raw = observe(Condition::Raw, &[1, 2, 3], &mut Hidden::None, 7);
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
  fn onset_hits_count_predicted_a_at_each_first_step_of_a_press() {
    let truth = [0, BUTTON_A, BUTTON_A, 0, BUTTON_A, BUTTON_A];
    let pred = [BUTTON_A, BUTTON_A, 0, 0, 0, BUTTON_A];
    assert_eq!(onset_hits(&pred, &truth), (1, 2));
    assert_eq!(onset_hits(&[BUTTON_A, 0], &[BUTTON_A, 0]), (1, 1));
    assert_eq!(onset_hits(&[0, 0], &[0, 0]), (0, 0));
  }

  #[test]
  fn folds_hold_out_each_dataset_in_turn() {
    let datasets: Vec<Dataset> = (0..3).map(|s| synthetic_dataset(s, 40)).collect();
    let m = evaluate_folds(&datasets, 1e-6, false, BUTTON_A).unwrap();
    assert!(m.accuracy > 0.9, "held-out accuracy {}", m.accuracy);
    assert!(m.train_accuracy > 0.9, "train accuracy {}", m.train_accuracy);
    assert!(m.mse < 0.1, "mse {}", m.mse);
    assert!(m.a_accuracy > 0.9, "A accuracy {}", m.a_accuracy);
    assert!(m.onset_recall > 0.9, "onset recall {}", m.onset_recall);
    assert!(m.a_specificity > 0.9, "A specificity {}", m.a_specificity);
    assert_eq!(m.rows, 120);
  }

  #[test]
  fn a_readout_that_never_presses_a_scores_one_on_onsets_plus_specificity() {
    // A constant feature and A on a quarter of the steps: the readout outputs
    // the A rate, under one half, so it never presses A. Specificity 1,
    // onsets 0: the same score as always-A.
    let datasets: Vec<Dataset> = (0..3)
      .map(|_| {
        let buttons: Vec<u8> = (0..40).map(|i| if i % 4 == 0 { BUTTON_A } else { 0 }).collect();
        Dataset {
          rows: vec![vec![1.0, 0.0]; 40],
          targets: buttons.iter().map(|&b| buttons_to_targets(b)).collect(),
          buttons,
          resets: 0,
        }
      })
      .collect();
    let m = evaluate_folds(&datasets, 1.0, false, BUTTON_A).unwrap();
    assert_eq!(m.onset_recall, 0.0);
    assert_eq!(m.a_specificity, 1.0);
    assert!((m.onset_score() - 1.0).abs() < 1e-9);
  }

  #[test]
  fn holding_out_a_group_scores_transfer_between_groups() {
    // Two groups of two datasets. With the same rule in both, a readout fitted
    // on one group starts the other's onsets; with the rule inverted in the
    // second group, it starts none of them, though leave-one-out still would.
    let same: Vec<Dataset> = (0..4).map(|s| synthetic_dataset(s, 40)).collect();
    let groups = vec![vec![0, 1], vec![2, 3]];
    let m = evaluate_groups(&same, &groups, 1e-6, false, BUTTON_A).unwrap();
    assert_eq!(m.rows, 160);
    assert!(m.onset_recall > 0.9, "same rule: {}", m.onset_recall);
    let mut inverted = same.clone();
    for d in &mut inverted[2..] {
      d.buttons = d.buttons.iter().map(|&b| b ^ BUTTON_A).collect();
      d.targets = d.buttons.iter().map(|&b| buttons_to_targets(b)).collect();
    }
    let m = evaluate_groups(&inverted, &groups, 1e-6, false, BUTTON_A).unwrap();
    assert!(m.onset_recall < 0.1, "inverted rule: {}", m.onset_recall);
    assert!(
      evaluate_groups(&same, &[vec![0, 1, 2, 3]], 1.0, false, BUTTON_A).is_err(),
      "one group is no test"
    );
  }

  #[test]
  fn stage_groups_collect_the_demonstrations_of_each_stage_in_order() {
    let one = Stage::ONE_ONE;
    let six = Stage { world: 5, level: 0 };
    assert_eq!(stage_groups(&[one, six, one, six, one]), vec![vec![0, 2, 4], vec![1, 3]]);
    assert_eq!(stage_groups(&[one, one]), vec![vec![0, 1]]);
  }

  #[test]
  fn folds_need_at_least_two_datasets() {
    let datasets = vec![synthetic_dataset(0, 40)];
    assert!(evaluate_folds(&datasets, 1.0, false, BUTTON_A).is_err());
    assert!(evaluate_folds(&[], 1.0, false, BUTTON_A).is_err());
  }

  #[test]
  fn the_clock_counts_macro_steps_from_the_level_start() {
    assert_eq!(level_step(0, 3), 3);
    assert_eq!(level_step(20, 3), 8, "twenty idle frames are five steps");
    assert_eq!(level_step(23, 0), 5);
  }

  #[test]
  fn demonstration_offsets_spread_over_at_least_a_hundred_frames() {
    // Six offsets inside 30 frames differ by at most six steps, so a clock
    // readout can memorise where the deterministic teacher jumps.
    assert!(ExperimentConfig::default().noop_max >= 100);
  }

  #[test]
  fn cached_demonstrations_count_only_at_the_configured_offsets() {
    let config = ExperimentConfig {
      demonstrations: 3,
      noop_max: 90,
      ..ExperimentConfig::default()
    };
    let demo = |noops: usize, stage: Stage| Demonstration {
      buttons: vec![],
      x_reached: 0,
      cleared: false,
      noops,
      stage,
    };
    let one = Stage::ONE_ONE;
    assert!(demos_at_configured_offsets(&[demo(0, one), demo(30, one), demo(60, one)], &config));
    assert!(
      !demos_at_configured_offsets(&[demo(0, one), demo(5, one), demo(10, one)], &config),
      "the old spread"
    );
    assert!(!demos_at_configured_offsets(&[demo(0, one), demo(30, one)], &config), "too few");
    let six = Stage { world: 5, level: 0 };
    assert!(
      !demos_at_configured_offsets(&[demo(0, one), demo(30, six), demo(60, one)], &config),
      "wrong stage"
    );
    let two = ExperimentConfig {
      stages: vec![one, six],
      ..config
    };
    assert!(demos_at_configured_offsets(&[demo(0, one), demo(30, six), demo(60, one)], &two));
  }

  #[test]
  fn demonstrations_take_the_stages_in_turn() {
    let one = Stage::ONE_ONE;
    let six = Stage { world: 5, level: 0 };
    let stages: Vec<Stage> = (0..6).map(|i| demonstration_stage(i, &[one, six])).collect();
    assert_eq!(stages, vec![one, six, one, six, one, six]);
    assert_eq!(demonstration_stage(4, &[]), one, "no stages means 1-1");
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

#[cfg(test)]
mod rom_tests {
  use super::*;

  const PLAYFIELD_EMPTY_UNIT: f32 = drive::PLAYFIELD_EMPTY as f32 / 255.0;

  #[test]
  #[ignore]
  fn the_bank_steps_four_boards_and_reads_their_playfields() {
    let tetris = match rom_or_skip("roms2/tetris.nes") {
      Some(r) => r,
      None => return,
    };
    let plain = Condition::BankTetris(BankEncoder::Hand, false);
    let mut hidden = hidden_for(plain, &tetris).unwrap();
    assert!(matches!(&hidden, Hidden::Bank(boards) if boards.len() == BANK_BOARDS));
    let window = vec![0u8; SMB_WINDOW_LEN];
    let row = observe(plain, &window, &mut hidden, 0);
    assert_eq!(row.len(), plain.row_len());
    assert_eq!(row[0], 1.0);
    let cells = |row: &[f32], b: usize| row[1 + b * 200..1 + (b + 1) * 200].iter().filter(|&&v| v != PLAYFIELD_EMPTY_UNIT).count();
    // Four pieces locked on every board: at least a dozen cells each, after any clears.
    for b in 0..BANK_BOARDS {
      assert!(cells(&row, b) >= 12, "board {} has {} cells", b, cells(&row, b));
    }
    assert_eq!(hidden.resets(), 0);
    // The heights readout: forty column heights over twenty, nonzero where pieces locked.
    let heights = Condition::BankHeights(BankEncoder::Hand, false);
    let mut h = hidden_for(heights, &tetris).unwrap();
    let hrow = observe(heights, &window, &mut h, 0);
    assert_eq!(hrow.len(), heights.row_len());
    assert_eq!(hrow[0], 1.0);
    for b in 0..BANK_BOARDS {
      let board = &hrow[1 + b * 10..1 + (b + 1) * 10];
      assert!(board.iter().all(|&v| (0.0..=1.0).contains(&v)));
      assert!(board.iter().any(|&v| v > 0.0), "board {} has no height", b);
    }
    // The tidy variant adds a fifth piece per board per step.
    let tidy = Condition::BankTetris(BankEncoder::Hand, true);
    let mut tidied = hidden_for(tidy, &tetris).unwrap();
    let tidy_row = observe(tidy, &window, &mut tidied, 0);
    for b in 0..BANK_BOARDS {
      assert!(cells(&tidy_row, b) >= 16, "tidied board {} has {} cells", b, cells(&tidy_row, b));
    }
  }
}

#[cfg(test)]
mod diagnostics {
  use super::*;
  use crate::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_RIGHT};
  use demo::load_cached;
  use std::path::Path;

  /// What a readout outputs on the A bit at press onsets, the step before them,
  /// the rest of each press, and everywhere else, with onset recall and
  /// false-positive rate at several thresholds.
  #[test]
  #[ignore]
  fn what_the_readout_says_at_onsets() {
    let smb = match rom_or_skip("roms2/super_mario_bros.nes") {
      Some(r) => r,
      None => return,
    };
    let tetris = std::fs::read("roms2/tetris.nes").unwrap_or_default();
    let dir = std::env::var("DEMO_DIR").unwrap_or_else(|_| "target/tetris-reservoir-demos".to_string());
    let demos = load_cached(Path::new(&dir), 6).expect("cached demonstrations");
    let lambda: f64 = std::env::var("LAMBDA").ok().and_then(|v| v.parse().ok()).unwrap_or(100.0);
    let logistic = std::env::var("LOGISTIC").is_ok();
    let a_index = BUTTON_A.trailing_zeros() as usize;
    let mut conditions = vec![Condition::Raw, Condition::PlacementOnehot(Encoder::Salient)];
    if std::env::var("SKIP_SOFT").is_err() {
      conditions.extend([Condition::RandomTanh, Condition::EchoState]);
    }
    for condition in conditions {
      let datasets: Vec<Dataset> = demos.iter().map(|d| collect(condition, &smb, &tetris, d, 0).unwrap()).collect();
      // (sum, count) of the A output in each class.
      let mut classes = [(0.0f64, 0usize); 4];
      let mut onset_outputs = Vec::new();
      let mut quiet_outputs = Vec::new();
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
        let readout = fit_readout(&rows, &targets, lambda, logistic).unwrap();
        let d = &datasets[held_out];
        let a: Vec<bool> = d.buttons.iter().map(|&b| b & BUTTON_A != 0).collect();
        for (i, row) in d.rows.iter().enumerate() {
          let out = readout.predict(row)[a_index];
          let prev = i > 0 && a[i - 1];
          let next = i + 1 < a.len() && a[i + 1];
          let class = match (prev, a[i], next) {
            (false, true, _) => 0,
            (_, false, true) => 1,
            (true, true, _) => 2,
            _ => 3,
          };
          classes[class].0 += out;
          classes[class].1 += 1;
          if class == 0 {
            onset_outputs.push(out);
          }
          if class == 3 {
            quiet_outputs.push(out);
          }
        }
      }
      let mean = |c: (f64, usize)| c.0 / c.1.max(1) as f64;
      println!(
        "{}: {} lambda {}: mean A output at onsets {:.3} ({}), the step before {:.3} ({}), mid-press {:.3} ({}), elsewhere {:.3} ({})",
        condition.name(),
        if logistic { "logistic" } else { "ridge" },
        lambda,
        mean(classes[0]),
        classes[0].1,
        mean(classes[1]),
        classes[1].1,
        mean(classes[2]),
        classes[2].1,
        mean(classes[3]),
        classes[3].1
      );
      for t in [0.5, 0.4, 0.3, 0.2, 0.1] {
        let recall = onset_outputs.iter().filter(|&&o| o > t).count() as f64 / onset_outputs.len().max(1) as f64;
        let fpr = quiet_outputs.iter().filter(|&&o| o > t).count() as f64 / quiet_outputs.len().max(1) as f64;
        println!("   threshold {:.1}: onset recall {:.3}, false A elsewhere {:.3}", t, recall, fpr);
      }
    }
  }

  /// Where the held-out A errors of a readout fall, relative to the demonstrations' presses.
  #[test]
  #[ignore]
  fn where_the_a_errors_are() {
    let smb = match rom_or_skip("roms2/super_mario_bros.nes") {
      Some(r) => r,
      None => return,
    };
    let tetris = std::fs::read("roms2/tetris.nes").unwrap_or_default();
    let dir = std::env::var("DEMO_DIR").unwrap_or_else(|_| "target/tetris-reservoir-demos".to_string());
    let demos = load_cached(Path::new(&dir), 6).expect("cached demonstrations");
    let lambda: f64 = std::env::var("LAMBDA").ok().and_then(|v| v.parse().ok()).unwrap_or(100.0);
    for condition in [Condition::Raw, Condition::PlacementOnehot(Encoder::Salient), Condition::Clock] {
      let datasets: Vec<Dataset> = demos.iter().map(|d| collect(condition, &smb, &tetris, d, 0).unwrap()).collect();
      let (mut onset, mut release, mut mid, mut ground, mut a_rows, mut a_err, mut other_err, mut other_bits) = (0, 0, 0, 0, 0, 0, 0, 0);
      let mut truth_a = 0;
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
        let readout = fit(&rows, &targets, lambda).unwrap();
        let d = &datasets[held_out];
        let a: Vec<bool> = d.buttons.iter().map(|&b| b & BUTTON_A != 0).collect();
        for (i, row) in d.rows.iter().enumerate() {
          let pred = outputs_to_buttons(&readout.predict(row));
          a_rows += 1;
          truth_a += a[i] as usize;
          for bit in [BUTTON_B, BUTTON_RIGHT] {
            other_bits += 1;
            other_err += ((pred ^ d.buttons[i]) & bit != 0) as usize;
          }
          if (pred & BUTTON_A != 0) != a[i] {
            a_err += 1;
            let prev = i > 0 && a[i - 1];
            let next = i + 1 < a.len() && a[i + 1];
            match (prev, a[i], next) {
              (false, true, _) => onset += 1,
              (true, false, _) => release += 1,
              (true, true, true) | (_, true, true) => mid += 1,
              _ => ground += 1,
            }
          }
        }
      }
      println!(
        "{}: lambda {}: A bit {:.3} accurate ({} errors in {} rows, A held {:.1}% of steps); B/Right bits {:.3}",
        condition.name(),
        lambda,
        1.0 - a_err as f64 / a_rows as f64,
        a_err,
        a_rows,
        100.0 * truth_a as f64 / a_rows as f64,
        1.0 - other_err as f64 / other_bits as f64
      );
      println!(
        "   A errors: {} at press onsets, {} at releases, {} mid-press, {} on the ground with no press nearby",
        onset, release, mid, ground
      );
    }
  }
}
