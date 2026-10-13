//! Ridge regression by normal equations, solved with Cholesky in f64.

/// A fitted linear readout. `weights` is `d x k`, row-major: `weights[i * k + j]`
/// is the weight from feature `i` to output `j`.
#[derive(Debug, Clone)]
pub struct Readout {
  pub d: usize,
  pub k: usize,
  pub weights: Vec<f64>,
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
    out
  }
}

/// Fit `W = (X^T X + lambda I)^-1 X^T Y`.
pub fn fit(rows: &[Vec<f32>], targets: &[Vec<f32>], lambda: f64) -> Result<Readout, String> {
  if lambda <= 0.0 {
    return Err(format!("lambda must be positive, got {}", lambda));
  }
  if rows.is_empty() {
    return Err("no rows to fit".to_string());
  }
  if rows.len() != targets.len() {
    return Err(format!("{} rows but {} targets", rows.len(), targets.len()));
  }
  let d = rows[0].len();
  let k = targets[0].len();
  let mut gram = vec![0.0f64; d * d];
  let mut xty = vec![0.0f64; d * k];
  for (row, target) in rows.iter().zip(targets) {
    if row.len() != d || target.len() != k {
      return Err(format!("ragged input: row {} target {}", row.len(), target.len()));
    }
    for i in 0..d {
      let xi = row[i] as f64;
      if xi == 0.0 {
        continue;
      }
      let g = &mut gram[i * d..(i + 1) * d];
      for j in i..d {
        g[j] += xi * row[j] as f64;
      }
      let b = &mut xty[i * k..(i + 1) * k];
      for j in 0..k {
        b[j] += xi * target[j] as f64;
      }
    }
  }
  for i in 0..d {
    gram[i * d + i] += lambda;
    for j in 0..i {
      gram[i * d + j] = gram[j * d + i];
    }
  }
  let lower = cholesky(&gram, d)?;
  let mut weights = vec![0.0; d * k];
  let mut rhs = vec![0.0; d];
  for j in 0..k {
    for i in 0..d {
      rhs[i] = xty[i * k + j];
    }
    let solution = cholesky_solve(&lower, d, &rhs);
    for i in 0..d {
      weights[i * k + j] = solution[i];
    }
  }
  Ok(Readout { d, k, weights })
}

/// Lower-triangular `L` with `a = L L^T`. `a` is `d x d` row-major and symmetric.
fn cholesky(a: &[f64], d: usize) -> Result<Vec<f64>, String> {
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
fn cholesky_solve(l: &[f64], d: usize, b: &[f64]) -> Vec<f64> {
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
    for w in &readout.weights {
      assert!(w.abs() < 1e-3, "weight {} not shrunk", w);
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
