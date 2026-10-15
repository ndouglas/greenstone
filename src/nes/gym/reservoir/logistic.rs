//! Logistic regression by Newton's method (iteratively reweighted least
//! squares), one output at a time, sharing the ridge solver.
//!
//! Ridge regression fits 0/1 targets by least squares, which puts a rare
//! conjunction such as "on the ground and an enemy is close" well under the
//! 0.5 threshold even when it is linearly separable: the squared error on
//! the many easy rows outweighs it. A logistic fit finds the separating
//! plane, so a readout can start a jump rather than only recognise one.

use super::ridge::{cholesky, cholesky_solve, Readout};

/// Newton steps per output.
pub const NEWTON_STEPS: usize = 8;
/// Stop once no weight moves more than this in a step.
const TOLERANCE: f64 = 1e-4;
/// Intercept for an output whose target never varies: sigmoid(8) is 0.9997.
const SATURATED: f64 = 8.0;
/// Tiny penalty on the intercept so the Hessian stays positive definite.
const INTERCEPT_PENALTY: f64 = 1e-8;

pub fn sigmoid(z: f64) -> f64 {
  1.0 / (1.0 + (-z).exp())
}

/// Logistic regression where column 0 of every row is the intercept column,
/// which is not penalised. Features 1.. carry the L2 penalty `lambda`.
pub fn fit_logistic(rows: &[Vec<f32>], targets: &[Vec<f32>], lambda: f64) -> Result<Readout, String> {
  if lambda <= 0.0 {
    return Err(format!("lambda must be positive, got {}", lambda));
  }
  if rows.is_empty() {
    return Err("no rows to fit".to_string());
  }
  if rows.len() != targets.len() {
    return Err(format!("{} rows but {} targets", rows.len(), targets.len()));
  }
  let n = rows.len();
  let d = rows[0].len();
  let k = targets[0].len();
  if d == 0 {
    return Err("rows need at least the intercept column".to_string());
  }
  for (row, target) in rows.iter().zip(targets) {
    if row.len() != d || target.len() != k {
      return Err(format!("ragged input: row {} target {}", row.len(), target.len()));
    }
  }
  let x: Vec<f64> = rows.iter().flat_map(|r| r.iter().map(|&v| v as f64)).collect();
  let mut weights = vec![0.0; d * k];
  for j in 0..k {
    let y: Vec<f64> = targets.iter().map(|t| t[j] as f64).collect();
    let w = fit_output(&x, &y, n, d, lambda);
    for i in 0..d {
      weights[i * k + j] = w[i];
    }
  }
  Ok(Readout {
    d,
    k,
    weights,
    logistic: true,
  })
}

fn linear(x: &[f64], w: &[f64], n: usize, d: usize) -> Vec<f64> {
  (0..n)
    .map(|r| {
      let row = &x[r * d..(r + 1) * d];
      row.iter().zip(w).map(|(a, b)| a * b).sum()
    })
    .collect()
}

/// Penalised negative log-likelihood.
fn loss(z: &[f64], y: &[f64], w: &[f64], lambda: f64) -> f64 {
  let nll: f64 = z
    .iter()
    .zip(y)
    .map(|(&z, &y)| {
      // log(1 + e^z) - y z, computed stably.
      z.max(0.0) + (-z.abs()).exp().ln_1p() - y * z
    })
    .sum();
  nll + 0.5 * lambda * w[1..].iter().map(|v| v * v).sum::<f64>()
}

fn fit_output(x: &[f64], y: &[f64], n: usize, d: usize, lambda: f64) -> Vec<f64> {
  let mean = y.iter().sum::<f64>() / n as f64;
  let mut w = vec![0.0; d];
  if mean <= 0.0 || mean >= 1.0 {
    w[0] = if mean <= 0.0 { -SATURATED } else { SATURATED };
    return w;
  }
  w[0] = (mean / (1.0 - mean)).ln();
  let mut z = linear(x, &w, n, d);
  let mut current = loss(&z, y, &w, lambda);
  for _ in 0..NEWTON_STEPS {
    // Gradient of the loss and the Hessian, both with the penalty.
    let mut grad = vec![0.0; d];
    let mut hess = vec![0.0; d * d];
    for r in 0..n {
      let row = &x[r * d..(r + 1) * d];
      let p = sigmoid(z[r]);
      let resid = p - y[r];
      let wt = p * (1.0 - p);
      for i in 0..d {
        let xi = row[i];
        if xi == 0.0 {
          continue;
        }
        grad[i] += xi * resid;
        let h = &mut hess[i * d..(i + 1) * d];
        let s = wt * xi;
        for j in i..d {
          h[j] += s * row[j];
        }
      }
    }
    for i in 0..d {
      let penalty = if i == 0 { INTERCEPT_PENALTY } else { lambda };
      grad[i] += penalty * w[i];
      hess[i * d + i] += penalty;
      for j in 0..i {
        hess[i * d + j] = hess[j * d + i];
      }
    }
    let lower = match cholesky(&hess, d) {
      Ok(l) => l,
      Err(_) => break,
    };
    let step = cholesky_solve(&lower, d, &grad);
    // Newton direction is -step; halve it until the loss falls.
    let mut scale = 1.0;
    let mut accepted = false;
    for _ in 0..8 {
      let trial: Vec<f64> = w.iter().zip(&step).map(|(w, s)| w - scale * s).collect();
      let trial_z = linear(x, &trial, n, d);
      let trial_loss = loss(&trial_z, y, &trial, lambda);
      if trial_loss <= current {
        let moved = step.iter().map(|s| (scale * s).abs()).fold(0.0, f64::max);
        w = trial;
        z = trial_z;
        current = trial_loss;
        accepted = moved > TOLERANCE;
        break;
      }
      scale *= 0.5;
    }
    if !accepted {
      break;
    }
  }
  w
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::reservoir::ridge::fit;

  /// Features (ground, close, rising) and whether A is pressed, with the
  /// rough proportions of a rule teacher's rows: the press onset is the
  /// rare conjunction ground AND close.
  const PATTERNS: [([f32; 3], f32, usize); 6] = [
    ([1.0, 1.0, 0.0], 1.0, 150),
    ([1.0, 0.0, 0.0], 0.0, 1500),
    ([0.0, 1.0, 0.0], 0.0, 300),
    ([0.0, 0.0, 1.0], 1.0, 400),
    ([0.0, 1.0, 1.0], 1.0, 150),
    ([0.0, 0.0, 0.0], 0.0, 300),
  ];

  fn toy() -> (Vec<Vec<f32>>, Vec<Vec<f32>>) {
    let mut rows = Vec::new();
    let mut targets = Vec::new();
    for (f, label, count) in PATTERNS {
      for _ in 0..count {
        rows.push(vec![1.0, f[0], f[1], f[2]]);
        targets.push(vec![label, 1.0 - label]);
      }
    }
    (rows, targets)
  }

  #[test]
  fn ridge_misses_the_onset_conjunction_and_logistic_finds_it() {
    let (rows, targets) = toy();
    let ridge = fit(&rows, &targets, 1.0).unwrap();
    let logistic = fit_logistic(&rows, &targets, 1.0).unwrap();
    let onset = vec![1.0, 1.0, 1.0, 0.0];
    assert!(ridge.predict(&onset)[0] < 0.5, "ridge {}", ridge.predict(&onset)[0]);
    assert!(logistic.predict(&onset)[0] > 0.5, "logistic {}", logistic.predict(&onset)[0]);
    for (f, label, _) in PATTERNS {
      let out = logistic.predict(&[1.0, f[0], f[1], f[2]])[0];
      assert_eq!(out > 0.5, label > 0.5, "pattern {:?} output {}", f, out);
      let other = logistic.predict(&[1.0, f[0], f[1], f[2]])[1];
      assert_eq!(other > 0.5, label < 0.5, "pattern {:?} second output {}", f, other);
    }
  }

  #[test]
  fn logistic_outputs_are_probabilities() {
    let (rows, targets) = toy();
    let readout = fit_logistic(&rows, &targets, 1.0).unwrap();
    for row in rows.iter().step_by(97) {
      for out in readout.predict(row) {
        assert!((0.0..=1.0).contains(&out), "{}", out);
      }
    }
  }

  #[test]
  fn a_constant_target_saturates_without_failing() {
    let (rows, _) = toy();
    let targets: Vec<Vec<f32>> = rows.iter().map(|_| vec![1.0, 0.0]).collect();
    let readout = fit_logistic(&rows, &targets, 1.0).unwrap();
    let out = readout.predict(&rows[0]);
    assert!(out[0] > 0.99 && out[1] < 0.01, "{:?}", out);
  }

  #[test]
  fn logistic_rejects_bad_input() {
    let (rows, targets) = toy();
    assert!(fit_logistic(&rows, &targets, 0.0).is_err());
    assert!(fit_logistic(&[], &[], 1.0).is_err());
    assert!(fit_logistic(&rows[..2], &targets[..1], 1.0).is_err());
  }
}
