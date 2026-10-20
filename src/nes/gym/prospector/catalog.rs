//! The catalog: one JSON line per offset, appended as verdicts arrive. The
//! latest line for an offset wins, so a solve can append over a walk.

use crate::nes::gym::prospector::funnel::Solved;
use crate::nes::gym::prospector::rank::{narrowness, Narrowness};
use crate::nes::gym::reservoir::demo::Clear;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Class {
  Crashed,
  Jammed,
  Blank,
  InstantDeath,
  Duplicate,
  Stuck,
  WalkerDied,
  Walked,
  Unsolved,
  ClearedDistance,
  ClearedFlag,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolveRecord {
  pub clear: Option<Clear>,
  pub x_reached: i32,
  pub rounds: usize,
  pub panics: usize,
  pub bottleneck: usize,
  pub kill_rate: f64,
  pub path: Vec<u8>,
}

impl SolveRecord {
  pub fn from_solved(s: &Solved) -> SolveRecord {
    let n = narrowness(&s.stats);
    SolveRecord {
      clear: s.clear,
      x_reached: s.x_reached,
      rounds: n.rounds,
      panics: s.panics,
      bottleneck: n.bottleneck,
      kill_rate: n.kill_rate,
      path: s.path.clone(),
    }
  }

  pub fn narrowness(&self) -> Narrowness {
    Narrowness {
      bottleneck: self.bottleneck,
      kill_rate: self.kill_rate,
      rounds: self.rounds,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
  pub offset: u16,
  pub hex: String,
  pub class: Class,
  #[serde(default)]
  pub message: Option<String>,
  #[serde(default)]
  pub fingerprint: Option<String>,
  #[serde(default)]
  pub duplicate_of: Option<u16>,
  #[serde(default)]
  pub walk_x: Option<i32>,
  #[serde(default)]
  pub solve: Option<SolveRecord>,
}

impl Entry {
  pub fn new(offset: u16, class: Class) -> Entry {
    Entry {
      offset,
      hex: format!("${:04X}", offset),
      class,
      message: None,
      fingerprint: None,
      duplicate_of: None,
      walk_x: None,
      solve: None,
    }
  }
}

pub fn append(path: &Path, e: &Entry) -> Result<(), String> {
  if let Some(dir) = path.parent() {
    std::fs::create_dir_all(dir).map_err(|err| format!("create {}: {}", dir.display(), err))?;
  }
  let mut f = std::fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(path)
    .map_err(|err| format!("open {}: {}", path.display(), err))?;
  let mut line = serde_json::to_string(e).map_err(|err| err.to_string())?;
  line.push('\n');
  f.write_all(line.as_bytes()).map_err(|err| err.to_string())
}

/// Every offset's latest entry, by offset. Lines that don't parse (a scan
/// killed mid-write) are skipped.
pub fn load_catalog(path: &Path) -> Vec<Entry> {
  let Ok(text) = std::fs::read_to_string(path) else {
    return Vec::new();
  };
  let mut latest = BTreeMap::new();
  for line in text.lines() {
    if let Ok(e) = serde_json::from_str::<Entry>(line) {
      latest.insert(e.offset, e);
    }
  }
  latest.into_values().collect()
}

/// Mark every entry whose fingerprint an earlier entry already has as a
/// duplicate of it. `entries` must be sorted by offset.
pub fn dedupe(entries: &mut [Entry]) {
  let mut first: HashMap<String, u16> = HashMap::new();
  for e in entries.iter_mut() {
    let Some(fp) = e.fingerprint.clone() else { continue };
    match first.get(&fp) {
      Some(&original) => {
        e.class = Class::Duplicate;
        e.duplicate_of = Some(original);
      }
      None => {
        first.insert(fp, e.offset);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn tmp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("prospector-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("catalog.jsonl")
  }

  #[test]
  fn entries_round_trip_through_a_line() {
    let mut e = Entry::new(0xC4A0, Class::Walked);
    e.walk_x = Some(812);
    e.fingerprint = Some("00000000deadbeef".into());
    let line = serde_json::to_string(&e).unwrap();
    assert!(line.contains("\"class\":\"walked\""), "{}", line);
    assert!(line.contains("\"hex\":\"$C4A0\""), "{}", line);
    assert_eq!(serde_json::from_str::<Entry>(&line).unwrap(), e);
  }

  #[test]
  fn load_catalog_takes_the_latest_line_per_offset() {
    let path = tmp("latest");
    append(&path, &Entry::new(0x9000, Class::Walked)).unwrap();
    append(&path, &Entry::new(0x8000, Class::Blank)).unwrap();
    append(&path, &Entry::new(0x9000, Class::Unsolved)).unwrap();
    let got = load_catalog(&path);
    assert_eq!(
      got.iter().map(|e| (e.offset, e.class)).collect::<Vec<_>>(),
      vec![(0x8000, Class::Blank), (0x9000, Class::Unsolved)]
    );
  }

  #[test]
  fn load_catalog_skips_a_truncated_line() {
    let path = tmp("truncated");
    append(&path, &Entry::new(0x8000, Class::Blank)).unwrap();
    std::fs::OpenOptions::new()
      .append(true)
      .open(&path)
      .unwrap()
      .write_all(b"{\"offset\":32769,\"hex\":\"$80")
      .unwrap();
    assert_eq!(load_catalog(&path).len(), 1);
    assert!(load_catalog(&path.with_file_name("missing.jsonl")).is_empty());
  }

  #[test]
  fn dedupe_keeps_the_lowest_offset_per_fingerprint() {
    let mut a = Entry::new(0x8002, Class::Walked);
    a.fingerprint = Some("aa".into());
    let mut b = Entry::new(0x8001, Class::Stuck);
    b.fingerprint = Some("aa".into());
    let c = Entry::new(0x8000, Class::Crashed);
    let mut all = vec![a, b, c];
    all.sort_by_key(|e| e.offset);
    dedupe(&mut all);
    assert_eq!(all[0].class, Class::Crashed);
    assert_eq!(all[1].class, Class::Stuck);
    assert_eq!(all[2].class, Class::Duplicate);
    assert_eq!(all[2].duplicate_of, Some(0x8001));
  }
}
