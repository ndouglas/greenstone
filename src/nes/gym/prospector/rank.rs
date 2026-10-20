//! How narrowly a level was cleared, and the order that puts the worst first.

use crate::nes::gym::reservoir::demo::RoundStats;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Narrowness {
  /// Fewest distinct live states in any round. 1 means one way through.
  pub bottleneck: usize,
  /// Share of all children the search made that died or got stuck.
  pub kill_rate: f64,
  pub rounds: usize,
}

pub fn narrowness(stats: &[RoundStats]) -> Narrowness {
  let expanded: usize = stats.iter().map(|s| s.expanded).sum();
  let alive: usize = stats.iter().map(|s| s.alive).sum();
  Narrowness {
    bottleneck: stats.iter().map(|s| s.distinct).min().unwrap_or(usize::MAX),
    kill_rate: if expanded == 0 {
      0.0
    } else {
      (expanded - alive) as f64 / expanded as f64
    },
    rounds: stats.len(),
  }
}

/// `Less` when `a` is the worse (narrower) level and should rank first.
pub fn worse(a: &Narrowness, b: &Narrowness) -> Ordering {
  a.bottleneck
    .cmp(&b.bottleneck)
    .then(b.kill_rate.partial_cmp(&a.kill_rate).unwrap_or(Ordering::Equal))
    .then(b.rounds.cmp(&a.rounds))
}

#[cfg(test)]
mod tests {
  use super::*;

  fn r(expanded: usize, alive: usize, distinct: usize) -> RoundStats {
    RoundStats {
      expanded,
      alive,
      distinct,
      kept: distinct.min(16),
    }
  }

  #[test]
  fn narrowness_takes_the_tightest_round_and_the_overall_kill_rate() {
    let n = narrowness(&[r(6, 6, 6), r(36, 9, 2), r(12, 3, 3)]);
    assert_eq!(n.bottleneck, 2);
    assert!((n.kill_rate - 36.0 / 54.0).abs() < 1e-9);
    assert_eq!(n.rounds, 3);
  }

  #[test]
  fn narrowness_of_no_rounds_is_not_narrow() {
    assert_eq!(narrowness(&[]).bottleneck, usize::MAX);
  }

  #[test]
  fn worse_orders_by_bottleneck_then_kill_rate_then_rounds() {
    let n = |b, k, r| Narrowness {
      bottleneck: b,
      kill_rate: k,
      rounds: r,
    };
    use std::cmp::Ordering::*;
    assert_eq!(worse(&n(1, 0.1, 10), &n(2, 0.9, 99)), Less);
    assert_eq!(worse(&n(2, 0.9, 10), &n(2, 0.5, 99)), Less);
    assert_eq!(worse(&n(2, 0.5, 99), &n(2, 0.5, 10)), Less);
    assert_eq!(worse(&n(2, 0.5, 10), &n(2, 0.5, 10)), Equal);
  }
}
