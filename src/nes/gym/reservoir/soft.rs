//! Reservoirs in software: the echo state network's expansion with no game in it.

use super::features::SMB_WINDOW_LEN;
use super::Lcg;

/// Window bytes the expansion reads.
pub const SOFT_INPUTS: usize = SMB_WINDOW_LEN;
/// Units in the expansion, the same width as Tetris's RAM.
pub const SOFT_UNITS: usize = 2048;
/// The one fixed seed, so every demonstration and episode sees the same reservoir.
pub const SOFT_SEED: u64 = 0x5EED_7E7A_15;
/// Input gain: pre-activations of centred bytes come out with unit-ish spread.
const INPUT_GAIN: f32 = 3.0;
/// Spectral radius of the recurrent matrix, below one so the echo state forgets.
const SPECTRAL_RADIUS: f32 = 0.9;
/// Leak rate of the echo state: how much of the new activation replaces the old.
const LEAK: f32 = 0.5;

/// A fixed random tanh expansion of the window, optionally with a random
/// recurrent matrix so the state carries history (an echo state network).
pub struct Expansion {
  w_in: Vec<f32>,
  bias: Vec<f32>,
  w_rec: Option<Vec<f32>>,
  state: Vec<f32>,
}

fn uniform(rng: &mut Lcg) -> f32 {
  (rng.next_f64() * 2.0 - 1.0) as f32
}

impl Expansion {
  pub fn new(seed: u64, recurrent: bool) -> Expansion {
    let mut rng = Lcg::new(seed);
    let in_scale = INPUT_GAIN / (SOFT_INPUTS as f32).sqrt();
    let w_in: Vec<f32> = (0..SOFT_UNITS * SOFT_INPUTS).map(|_| uniform(&mut rng) * in_scale).collect();
    let bias: Vec<f32> = (0..SOFT_UNITS).map(|_| uniform(&mut rng) * 0.5).collect();
    let w_rec = recurrent.then(|| {
      // Uniform(-1, 1) entries have spectral radius about sqrt(N / 3).
      let rec_scale = SPECTRAL_RADIUS * (3.0 / SOFT_UNITS as f32).sqrt();
      (0..SOFT_UNITS * SOFT_UNITS).map(|_| uniform(&mut rng) * rec_scale).collect()
    });
    Expansion {
      w_in,
      bias,
      w_rec,
      state: vec![0.0; SOFT_UNITS],
    }
  }

  /// Feed one window and return `[1.0, unit activations...]`.
  pub fn step(&mut self, window: &[u8]) -> Vec<f32> {
    let x: Vec<f32> = window.iter().map(|&b| b as f32 / 255.0 - 0.5).collect();
    let mut next = vec![0.0f32; SOFT_UNITS];
    for (u, out) in next.iter_mut().enumerate() {
      let w = &self.w_in[u * SOFT_INPUTS..(u + 1) * SOFT_INPUTS];
      let mut acc = self.bias[u];
      for (wi, xi) in w.iter().zip(&x) {
        acc += wi * xi;
      }
      if let Some(rec) = &self.w_rec {
        let r = &rec[u * SOFT_UNITS..(u + 1) * SOFT_UNITS];
        for (ri, hi) in r.iter().zip(&self.state) {
          acc += ri * hi;
        }
      }
      *out = acc.tanh();
    }
    if self.w_rec.is_some() {
      for (h, n) in self.state.iter_mut().zip(&next) {
        *h = (1.0 - LEAK) * *h + LEAK * n;
      }
    } else {
      self.state = next;
    }
    let mut row = Vec::with_capacity(1 + SOFT_UNITS);
    row.push(1.0);
    row.extend_from_slice(&self.state);
    row
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn window(seed: u8) -> Vec<u8> {
    (0..SOFT_INPUTS).map(|i| (i as u32 * 31 + seed as u32 * 7) as u8).collect()
  }

  #[test]
  fn expansion_is_deterministic_for_a_seed_and_bounded() {
    let mut a = Expansion::new(SOFT_SEED, false);
    let mut b = Expansion::new(SOFT_SEED, false);
    let row_a = a.step(&window(1));
    let row_b = b.step(&window(1));
    assert_eq!(row_a, row_b);
    assert_eq!(row_a.len(), 1 + SOFT_UNITS);
    assert_eq!(row_a[0], 1.0);
    assert!(row_a[1..].iter().all(|v| v.abs() < 1.0));
    let spread = row_a[1..].iter().filter(|v| v.abs() > 0.1).count();
    assert!(spread > SOFT_UNITS / 2, "units should not sit at zero: {} active", spread);
    let saturated = row_a[1..].iter().filter(|v| v.abs() > 0.99).count();
    assert!(saturated < SOFT_UNITS / 2, "units should not all saturate: {} saturated", saturated);
    let mut c = Expansion::new(SOFT_SEED + 1, false);
    assert_ne!(c.step(&window(1)), row_a, "another seed is another reservoir");
  }

  #[test]
  fn feedforward_has_no_memory_and_echo_does() {
    let mut ff = Expansion::new(SOFT_SEED, false);
    let first = ff.step(&window(1));
    ff.step(&window(2));
    assert_eq!(ff.step(&window(1)), first);
    let mut echo = Expansion::new(SOFT_SEED, true);
    let first = echo.step(&window(1));
    echo.step(&window(2));
    let later = echo.step(&window(1));
    assert_ne!(later, first, "the echo state carries history");
    assert!(later[1..].iter().all(|v| v.abs() < 1.0));
  }

  #[test]
  fn echo_state_forgets() {
    // After many identical inputs two echo states with different histories converge.
    let mut a = Expansion::new(SOFT_SEED, true);
    let mut b = Expansion::new(SOFT_SEED, true);
    a.step(&window(3));
    b.step(&window(9));
    let mut last = (vec![], vec![]);
    for _ in 0..200 {
      last = (a.step(&window(1)), b.step(&window(1)));
    }
    let gap: f32 = last.0.iter().zip(&last.1).map(|(x, y)| (x - y).abs()).fold(0.0, f32::max);
    assert!(gap < 1e-3, "states should converge, max gap {}", gap);
  }
}
