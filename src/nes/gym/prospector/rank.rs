//! How narrowly a level was cleared, and the order that puts the worst first.

use crate::nes::gym::prospector::catalog::{Entry, SolveRecord};
use crate::nes::gym::prospector::scan::CALIBRATION;
use crate::nes::gym::reservoir::demo::RoundStats;
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::BTreeMap;

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

/// The cleared levels, worst first, each with the calibration levels it is narrower than.
pub fn rows(entries: &[Entry], calibration: &BTreeMap<String, SolveRecord>) -> Vec<Value> {
  let mut cleared: Vec<(&Entry, &SolveRecord)> = entries
    .iter()
    .filter_map(|e| e.solve.as_ref().filter(|s| s.clear.is_some()).map(|s| (e, s)))
    .collect();
  cleared.sort_by(|a, b| worse(&a.1.narrowness(), &b.1.narrowness()).then(a.0.offset.cmp(&b.0.offset)));
  cleared
    .iter()
    .enumerate()
    .map(|(i, (e, s))| {
      let n = s.narrowness();
      let narrower: Vec<&str> = CALIBRATION
        .iter()
        .map(|(name, _)| *name)
        .filter(|name| calibration.get(*name).is_some_and(|c| worse(&n, &c.narrowness()) == Ordering::Less))
        .collect();
      json!({
        "rank": i + 1,
        "offset": e.offset,
        "hex": e.hex,
        "class": e.class,
        "clear": s.clear,
        "bottleneck": s.bottleneck,
        "kill_rate": s.kill_rate,
        "rounds": s.rounds,
        "x_reached": s.x_reached,
        "narrower_than": narrower,
      })
    })
    .collect()
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

  use crate::nes::gym::prospector::catalog::Class;
  use crate::nes::gym::reservoir::demo::Clear;

  fn cleared(offset: u16, bottleneck: usize, kill_rate: f64) -> Entry {
    let mut e = Entry::new(offset, Class::ClearedDistance);
    e.solve = Some(SolveRecord {
      clear: Some(Clear::Distance),
      x_reached: 3100,
      rounds: 90,
      panics: 0,
      bottleneck,
      kill_rate,
      path: vec![],
    });
    e
  }

  #[test]
  fn rows_rank_cleared_levels_worst_first_against_calibration() {
    let mut cal = BTreeMap::new();
    cal.insert("1-1".to_string(), cleared(0, 4, 0.5).solve.unwrap());
    cal.insert("4-1".to_string(), cleared(0, 2, 0.7).solve.unwrap());
    let entries = vec![cleared(0x9000, 3, 0.6), Entry::new(0x9100, Class::Unsolved), cleared(0x9200, 1, 0.2)];
    let rows = rows(&entries, &cal);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["hex"], "$9200");
    assert_eq!(rows[0]["rank"], 1);
    assert_eq!(rows[0]["narrower_than"], json!(["1-1", "4-1"]));
    assert_eq!(rows[1]["narrower_than"], json!(["1-1"]));
  }

  #[test]
  fn rows_are_empty_without_cleared_levels() {
    assert!(rows(&[Entry::new(0x9100, Class::Unsolved)], &BTreeMap::new()).is_empty());
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
