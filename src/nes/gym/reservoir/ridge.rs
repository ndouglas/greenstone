//! Ridge regression by normal equations, solved with Cholesky in f64.

/// A fitted linear readout. `weights` is `d x k`, row-major: `weights[i * k + j]`
/// is the weight from feature `i` to output `j`.
#[derive(Debug, Clone)]
pub struct Readout {
  pub d: usize,
  pub k: usize,
  pub weights: Vec<f64>,
  /// Outputs pass through a sigmoid: the readout was fitted by `logistic::fit_logistic`.
  pub logistic: bool,
}

impl Readout {
  /// Outputs for one feature row.
  pub fn predict(&self, row: &[f32]) -> Vec<f64> {
    assert_eq!(row.len(), self.d, "readout expects {} features, got {}", self.d, row.len());
    let mut out = vec![0.0; self.k];
    for (i, &x) in row.iter().enumerate() {
      if x == 0.0 {
        continue;
      }
      let x = x as f64;
      let w = &self.weights[i * self.k..(i + 1) * self.k];
      for j in 0..self.k {
        out[j] += x * w[j];
      }
    }
    if self.logistic {
      for o in &mut out {
        *o = super::logistic::sigmoid(*o);
      }
    }
    out
  }
}

/// Which normal equations to solve. Primal is `p x p` in the feature count,
/// dual is `n x n` in the row count. They give the same weights.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Solver {
  Primal,
  Dual,
}

/// Dual when there are fewer rows than penalised features.
pub fn choose_solver(n: usize, p: usize) -> Solver {
  if n < p {
    Solver::Dual
  } else {
    Solver::Primal
  }
}

/// Ridge regression where column 0 of every row is the intercept column.
///
/// Features 1.. are centered and penalised; the intercept is recovered from
/// the means and is never shrunk. Column 0 is assumed to be 1.0 in every row.
pub fn fit(rows: &[Vec<f32>], targets: &[Vec<f32>], lambda: f64) -> Result<Readout, String> {
  if rows.is_empty() {
    return Err("no rows to fit".to_string());
  }
  fit_with(rows, targets, lambda, choose_solver(rows.len(), rows[0].len().saturating_sub(1)))
}

pub fn fit_with(rows: &[Vec<f32>], targets: &[Vec<f32>], lambda: f64, solver: Solver) -> Result<Readout, String> {
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
  let p = d - 1;

  // Column means over the penalised features and the targets.
  let mut x_mean = vec![0.0f64; p];
  let mut y_mean = vec![0.0f64; k];
  for (row, target) in rows.iter().zip(targets) {
    for i in 0..p {
      x_mean[i] += row[1 + i] as f64;
    }
    for j in 0..k {
      y_mean[j] += target[j] as f64;
    }
  }
  x_mean.iter_mut().for_each(|m| *m /= n as f64);
  y_mean.iter_mut().for_each(|m| *m /= n as f64);

  // Centered copies, row-major.
  let mut xc = vec![0.0f64; n * p];
  let mut yc = vec![0.0f64; n * k];
  for (r, (row, target)) in rows.iter().zip(targets).enumerate() {
    for i in 0..p {
      xc[r * p + i] = row[1 + i] as f64 - x_mean[i];
    }
    for j in 0..k {
      yc[r * k + j] = target[j] as f64 - y_mean[j];
    }
  }

  // Penalised weights, p x k.
  let w = match solver {
    Solver::Primal => solve_primal(&xc, &yc, n, p, k, lambda)?,
    Solver::Dual => solve_dual(&xc, &yc, n, p, k, lambda)?,
  };

  let mut weights = vec![0.0; d * k];
  for j in 0..k {
    let mut intercept = y_mean[j];
    for i in 0..p {
      intercept -= x_mean[i] * w[i * k + j];
    }
    weights[j] = intercept;
    for i in 0..p {
      weights[(1 + i) * k + j] = w[i * k + j];
    }
  }
  Ok(Readout {
    d,
    k,
    weights,
    logistic: false,
  })
}

/// `W = (Xc^T Xc + lambda I)^-1 Xc^T Yc`.
fn solve_primal(xc: &[f64], yc: &[f64], n: usize, p: usize, k: usize, lambda: f64) -> Result<Vec<f64>, String> {
  let mut gram = vec![0.0f64; p * p];
  let mut xty = vec![0.0f64; p * k];
  for r in 0..n {
    let row = &xc[r * p..(r + 1) * p];
    let target = &yc[r * k..(r + 1) * k];
    for i in 0..p {
      let xi = row[i];
      if xi == 0.0 {
        continue;
      }
      let g = &mut gram[i * p..(i + 1) * p];
      for j in i..p {
        g[j] += xi * row[j];
      }
      let b = &mut xty[i * k..(i + 1) * k];
      for j in 0..k {
        b[j] += xi * target[j];
      }
    }
  }
  for i in 0..p {
    gram[i * p + i] += lambda;
    for j in 0..i {
      gram[i * p + j] = gram[j * p + i];
    }
  }
  let lower = cholesky(&gram, p)?;
  let mut w = vec![0.0; p * k];
  let mut rhs = vec![0.0; p];
  for j in 0..k {
    for i in 0..p {
      rhs[i] = xty[i * k + j];
    }
    let solution = cholesky_solve(&lower, p, &rhs);
    for i in 0..p {
      w[i * k + j] = solution[i];
    }
  }
  Ok(w)
}

/// `A = (Xc Xc^T + lambda I)^-1 Yc`, then `W = Xc^T A`.
fn solve_dual(xc: &[f64], yc: &[f64], n: usize, p: usize, k: usize, lambda: f64) -> Result<Vec<f64>, String> {
  let mut kern = vec![0.0f64; n * n];
  for a in 0..n {
    let ra = &xc[a * p..(a + 1) * p];
    for b in a..n {
      let rb = &xc[b * p..(b + 1) * p];
      let dot: f64 = ra.iter().zip(rb).map(|(x, y)| x * y).sum();
      kern[a * n + b] = dot;
      kern[b * n + a] = dot;
    }
    kern[a * n + a] += lambda;
  }
  let lower = cholesky(&kern, n)?;
  let mut alpha = vec![0.0; n * k];
  let mut rhs = vec![0.0; n];
  for j in 0..k {
    for r in 0..n {
      rhs[r] = yc[r * k + j];
    }
    let solution = cholesky_solve(&lower, n, &rhs);
    for r in 0..n {
      alpha[r * k + j] = solution[r];
    }
  }
  let mut w = vec![0.0; p * k];
  for r in 0..n {
    let row = &xc[r * p..(r + 1) * p];
    let a = &alpha[r * k..(r + 1) * k];
    for i in 0..p {
      let xi = row[i];
      if xi == 0.0 {
        continue;
      }
      for j in 0..k {
        w[i * k + j] += xi * a[j];
      }
    }
  }
  Ok(w)
}

/// Lower-triangular `L` with `a = L L^T`. `a` is `d x d` row-major and symmetric.
pub(super) fn cholesky(a: &[f64], d: usize) -> Result<Vec<f64>, String> {
  let mut l = vec![0.0; d * d];
  for i in 0..d {
    for j in 0..=i {
      let mut sum = a[i * d + j];
      for p in 0..j {
        sum -= l[i * d + p] * l[j * d + p];
      }
      if i == j {
        if sum <= 0.0 {
          return Err(format!("matrix is not positive definite at row {}", i));
        }
        l[i * d + i] = sum.sqrt();
      } else {
        l[i * d + j] = sum / l[j * d + j];
      }
    }
  }
  Ok(l)
}

/// Solve `L L^T x = b` by forward then back substitution.
pub(super) fn cholesky_solve(l: &[f64], d: usize, b: &[f64]) -> Vec<f64> {
  let mut y = vec![0.0; d];
  for i in 0..d {
    let mut sum = b[i];
    for p in 0..i {
      sum -= l[i * d + p] * y[p];
    }
    y[i] = sum / l[i * d + i];
  }
  let mut x = vec![0.0; d];
  for i in (0..d).rev() {
    let mut sum = y[i];
    for p in i + 1..d {
      sum -= l[p * d + i] * x[p];
    }
    x[i] = sum / l[i * d + i];
  }
  x
}

#[cfg(test)]
mod tests {
  use super::*;

  fn lcg(seed: &mut u64) -> f32 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((*seed >> 33) as f32 / (1u64 << 31) as f32) - 0.5
  }

  /// Rows with a leading 1.0, random true weights, exact targets.
  fn synthetic(n: usize, d: usize, k: usize) -> (Vec<Vec<f32>>, Vec<Vec<f32>>, Vec<f64>) {
    let mut seed = 7u64;
    let weights: Vec<f64> = (0..d * k).map(|_| lcg(&mut seed) as f64).collect();
    let mut rows = Vec::new();
    let mut targets = Vec::new();
    for _ in 0..n {
      let mut row = vec![1.0f32];
      row.extend((1..d).map(|_| lcg(&mut seed)));
      let target: Vec<f32> = (0..k)
        .map(|j| row.iter().enumerate().map(|(i, &x)| x as f64 * weights[i * k + j]).sum::<f64>() as f32)
        .collect();
      rows.push(row);
      targets.push(target);
    }
    (rows, targets, weights)
  }

  #[test]
  fn recovers_exact_weights_with_tiny_lambda() {
    let (rows, targets, truth) = synthetic(200, 5, 2);
    let readout = fit(&rows, &targets, 1e-9).unwrap();
    assert_eq!(readout.d, 5);
    assert_eq!(readout.k, 2);
    for (got, want) in readout.weights.iter().zip(&truth) {
      assert!((got - want).abs() < 1e-4, "got {} want {}", got, want);
    }
  }

  #[test]
  fn large_lambda_shrinks_weights_toward_zero() {
    let (rows, targets, _) = synthetic(200, 5, 2);
    let readout = fit(&rows, &targets, 1e6).unwrap();
    for w in &readout.weights[readout.k..] {
      assert!(w.abs() < 1e-3, "weight {} not shrunk", w);
    }
  }

  #[test]
  fn intercept_is_not_shrunk_by_lambda() {
    let (mut rows, mut targets, _) = synthetic(200, 5, 1);
    for t in targets.iter_mut() {
      t[0] += 5.0;
    }
    rows.iter_mut().for_each(|r| r[0] = 1.0);
    let readout = fit(&rows, &targets, 1e6).unwrap();
    assert!((readout.weights[0] - 5.0).abs() < 0.1, "intercept {} was shrunk", readout.weights[0]);
    for w in &readout.weights[1..] {
      assert!(w.abs() < 1e-3, "weight {} not shrunk", w);
    }
  }

  #[test]
  fn primal_and_dual_solvers_agree() {
    let (rows, targets, _) = synthetic(40, 10, 2);
    let primal = fit_with(&rows, &targets, 0.5, Solver::Primal).unwrap();
    let dual = fit_with(&rows, &targets, 0.5, Solver::Dual).unwrap();
    for (a, b) in primal.weights.iter().zip(&dual.weights) {
      assert!((a - b).abs() < 1e-8, "primal {} dual {}", a, b);
    }
  }

  #[test]
  fn dual_is_chosen_when_rows_are_fewer_than_features() {
    assert_eq!(choose_solver(20, 50), Solver::Dual);
    assert_eq!(choose_solver(200, 5), Solver::Primal);
    assert_eq!(choose_solver(50, 50), Solver::Primal);
  }

  #[test]
  fn fits_an_underdetermined_problem_and_reproduces_its_targets() {
    let (rows, targets, _) = synthetic(20, 60, 2);
    let readout = fit(&rows, &targets, 1e-9).unwrap();
    for (row, target) in rows.iter().zip(&targets) {
      let out = readout.predict(row);
      for (o, t) in out.iter().zip(target) {
        assert!((o - *t as f64).abs() < 1e-4, "got {} want {}", o, t);
      }
    }
  }

  #[test]
  fn rejects_nonpositive_lambda() {
    let (rows, targets, _) = synthetic(20, 3, 1);
    assert!(fit(&rows, &targets, 0.0).is_err());
    assert!(fit(&rows, &targets, -1.0).is_err());
  }

  #[test]
  fn rejects_mismatched_row_counts() {
    let (rows, targets, _) = synthetic(20, 3, 1);
    assert!(fit(&rows[..10], &targets, 1.0).is_err());
  }

  #[test]
  fn predict_reproduces_training_targets() {
    let (rows, targets, _) = synthetic(50, 4, 3);
    let readout = fit(&rows, &targets, 1e-9).unwrap();
    for (row, target) in rows.iter().zip(&targets) {
      let out = readout.predict(row);
      for (o, t) in out.iter().zip(target) {
        assert!((o - *t as f64).abs() < 1e-4);
      }
    }
  }

  #[test]
  #[should_panic(expected = "readout expects 4 features")]
  fn predict_rejects_wrong_row_length() {
    let (rows, targets, _) = synthetic(20, 4, 1);
    let readout = fit(&rows, &targets, 1.0).unwrap();
    readout.predict(&[1.0, 2.0]);
  }
}
