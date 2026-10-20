//! Run the funnel over offsets and append the verdicts to the catalog.

use crate::nes::gym::prospector::catalog::{append, dedupe, load_catalog, Class, Entry, SolveRecord};
use crate::nes::gym::prospector::funnel::{fingerprint, level_start_at, load, solve, walk, LoadClass, Solved, MOVED_ENOUGH};
use crate::nes::gym::prospector::patch::patch_rom;
use crate::nes::gym::reservoir::demo::{guarded, Clear, Stage};
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub const CATALOG: &str = "catalog.jsonl";
pub const CALIBRATION_FILE: &str = "calibration.json";
/// Real levels, solved the same way, to give narrowness a scale.
pub const CALIBRATION: [(&str, Stage); 2] = [("1-1", Stage::ONE_ONE), ("4-1", Stage { world: 3, level: 0 })];

fn write_thumb(dir: &Path, fp: &str, rgb: &[u8]) -> Result<(), String> {
  let path = dir.join(format!("{}.ppm", fp));
  if path.exists() {
    return Ok(());
  }
  std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
  let tmp = dir.join(format!("{}.ppm.partial.{:?}", fp, std::thread::current().id()));
  let mut bytes = b"P6\n256 240\n255\n".to_vec();
  bytes.extend_from_slice(rgb);
  std::fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
  std::fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

/// Load, fingerprint and walk one offset. Never panics: a panic is a `Crashed` entry.
pub fn survey(rom: &[u8], offset: u16, thumbs: &Path) -> Entry {
  let patched = patch_rom(rom, offset);
  let loaded = match guarded(|| load(&patched)) {
    Ok(l) => l,
    Err(msg) => {
      let mut e = Entry::new(offset, Class::Crashed);
      e.message = Some(msg);
      return e;
    }
  };
  let class = match loaded.class {
    LoadClass::Jammed => Class::Jammed,
    LoadClass::Blank => Class::Blank,
    LoadClass::InstantDeath => Class::InstantDeath,
    LoadClass::Alive => Class::Walked,
  };
  let mut e = Entry::new(offset, class);
  if class != Class::Walked {
    return e;
  }
  let walked = guarded(|| (fingerprint(&patched, &loaded.state), walk(&patched, &loaded.state, offset as u64)));
  match walked {
    Err(msg) => {
      e.class = Class::Crashed;
      e.message = Some(msg);
    }
    Ok((fp, w)) => {
      let fp = format!("{:016x}", fp);
      if let Err(msg) = write_thumb(thumbs, &fp, &loaded.thumb) {
        e.message = Some(format!("thumbnail: {}", msg));
      }
      e.fingerprint = Some(fp);
      e.walk_x = Some(w.x);
      if w.moved < MOVED_ENOUGH {
        e.class = if w.died { Class::WalkerDied } else { Class::Stuck };
      }
    }
  }
  e
}

/// Survey every offset in `from..=to` not already in `out/catalog.jsonl`,
/// across threads. Returns how many entries it wrote.
pub fn scan_range(rom: &[u8], from: u16, to: u16, out: &Path) -> Result<usize, String> {
  let catalog = out.join(CATALOG);
  let thumbs = out.join("thumbs");
  let done: HashSet<u16> = load_catalog(&catalog).into_iter().map(|e| e.offset).collect();
  let todo: Vec<u16> = (from..=to).filter(|o| !done.contains(o)).collect();
  let next = AtomicUsize::new(0);
  let written = AtomicUsize::new(0);
  let lock = Mutex::new(());
  let errors: Mutex<Vec<String>> = Mutex::new(Vec::new());
  let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
  std::thread::scope(|s| {
    for _ in 0..threads {
      s.spawn(|| loop {
        let i = next.fetch_add(1, Ordering::Relaxed);
        let Some(&offset) = todo.get(i) else { break };
        let e = survey(rom, offset, &thumbs);
        let _guard = lock.lock().unwrap();
        if let Err(msg) = append(&catalog, &e) {
          errors.lock().unwrap().push(msg);
          next.store(todo.len(), Ordering::Relaxed);
          break;
        }
        let n = written.fetch_add(1, Ordering::Relaxed) + 1;
        if n % 256 == 0 {
          eprintln!("scanned {} of {}", n, todo.len());
        }
      });
    }
  });
  match errors.into_inner().unwrap().into_iter().next() {
    Some(msg) => Err(msg),
    None => Ok(written.into_inner()),
  }
}

pub fn solved_class(s: &Solved) -> Class {
  match s.clear {
    Some(Clear::Flag) => Class::ClearedFlag,
    Some(Clear::Distance) => Class::ClearedDistance,
    None if s.expansions > 0 && s.panics == s.expansions => Class::Crashed,
    None => Class::Unsolved,
  }
}

/// Beam-search the `k` distinct walked levels that got furthest and haven't
/// been searched, appending each verdict. Returns how many it searched.
pub fn solve_top(rom: &[u8], out: &Path, k: usize) -> Result<usize, String> {
  let catalog = out.join(CATALOG);
  let mut entries = load_catalog(&catalog);
  dedupe(&mut entries);
  let mut walked: Vec<Entry> = entries.into_iter().filter(|e| e.class == Class::Walked).collect();
  walked.sort_by_key(|e| (std::cmp::Reverse(e.walk_x.unwrap_or(0)), e.offset));
  let mut n = 0;
  for (i, mut e) in walked.into_iter().take(k).enumerate() {
    let started = std::time::Instant::now();
    let patched = patch_rom(rom, e.offset);
    match guarded(|| level_start_at(&patched, Stage::ONE_ONE)) {
      Ok(Some((state, _))) => {
        let solved = solve(&patched, &state, Stage::ONE_ONE);
        e.class = solved_class(&solved);
        e.solve = Some(SolveRecord::from_solved(&solved));
      }
      Ok(None) => e.class = Class::Jammed,
      Err(msg) => {
        e.class = Class::Crashed;
        e.message = Some(msg);
      }
    }
    append(&catalog, &e)?;
    eprintln!("solved {} of {}: {} {:?} in {:.0?}", i + 1, k, e.hex, e.class, started.elapsed());
    n += 1;
  }
  Ok(n)
}

/// Solve the calibration levels on the unpatched ROM, enemies and all.
pub fn calibrate(rom: &[u8], out: &Path) -> Result<(), String> {
  let mut map = serde_json::Map::new();
  for (name, stage) in CALIBRATION {
    let (state, _) = level_start_at(rom, stage).ok_or_else(|| format!("{} never started", name))?;
    let solved = solve(rom, &state, stage);
    let record = SolveRecord::from_solved(&solved);
    eprintln!("calibrated {}: {:?} bottleneck {}", name, record.clear, record.bottleneck);
    map.insert(name.to_string(), serde_json::to_value(&record).map_err(|e| e.to_string())?);
  }
  std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
  std::fs::write(out.join(CALIBRATION_FILE), serde_json::to_vec_pretty(&map).unwrap()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::prospector::patch::ONE_ONE_AREA;
  use crate::nes::gym::reservoir::rom_or_skip;

  #[test]
  fn survey_of_real_1_1_is_walked_with_a_thumbnail() {
    let Some(rom) = rom_or_skip("roms2/super_mario_bros.nes") else { return };
    let dir = std::env::temp_dir().join(format!("prospector-survey-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let e = survey(&rom, ONE_ONE_AREA, &dir.join("thumbs"));
    assert_eq!(e.class, Class::Walked);
    let fp = e.fingerprint.clone().unwrap();
    let thumb = std::fs::read(dir.join("thumbs").join(format!("{}.ppm", fp))).unwrap();
    assert!(thumb.starts_with(b"P6\n256 240\n255\n"));
    assert_eq!(thumb.len(), 15 + 256 * 240 * 3);
  }

  #[test]
  fn scan_range_writes_each_offset_once_and_resumes() {
    let Some(rom) = rom_or_skip("roms2/super_mario_bros.nes") else { return };
    let dir = std::env::temp_dir().join(format!("prospector-scan-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(scan_range(&rom, ONE_ONE_AREA, ONE_ONE_AREA + 1, &dir).unwrap(), 2);
    assert_eq!(scan_range(&rom, ONE_ONE_AREA, ONE_ONE_AREA + 1, &dir).unwrap(), 0);
    assert_eq!(load_catalog(&dir.join(CATALOG)).len(), 2);
  }

  #[test]
  fn solved_class_follows_the_clear_and_the_panics() {
    let s = |clear, panics, expansions| Solved {
      clear,
      x_reached: 0,
      panics,
      expansions,
      path: vec![],
      stats: vec![],
    };
    assert_eq!(solved_class(&s(Some(Clear::Flag), 0, 10)), Class::ClearedFlag);
    assert_eq!(solved_class(&s(Some(Clear::Distance), 3, 10)), Class::ClearedDistance);
    assert_eq!(solved_class(&s(None, 3, 10)), Class::Unsolved);
    assert_eq!(solved_class(&s(None, 10, 10)), Class::Crashed);
  }
}
