//! The Prospector: SMB's accidental levels. Output goes to target/prospector/.
//!
//! Usage: cargo run --release --example prospector -- <command>
//!   scan [--from HEX] [--to HEX]   load, fingerprint and walk every offset (default $8000-$FFFF)
//!   solve [--k N]                  beam-search the N furthest-walking distinct levels (default 50)
//!   calibrate                      solve real 1-1 and 4-1 the same way
//!   rank                           print the leaderboard and write ranking.json
//!   clips [--top N]                record takes for the top N (default 10)

use greenstone::nes::gym::prospector::catalog::{dedupe, load_catalog, write_view, SolveRecord};
use greenstone::nes::gym::prospector::clips::record_clip;
use greenstone::nes::gym::prospector::patch::check_rom;
use greenstone::nes::gym::prospector::rank::rows;
use greenstone::nes::gym::prospector::scan::{calibrate, scan_range, solve_top, CALIBRATION_FILE, CATALOG};
use greenstone::nes::gym::reservoir::demo::{guarded, quiet_guarded_panics};
use std::collections::BTreeMap;
use std::path::Path;

fn flag(args: &[String], name: &str) -> Option<String> {
  args.windows(2).find(|w| w[0] == name).map(|w| w[1].clone())
}

fn hex(s: &str) -> u16 {
  u16::from_str_radix(s.trim_start_matches('$').trim_start_matches("0x"), 16).unwrap_or_else(|_| fail(&format!("not a hex offset: {}", s)))
}

fn fail(msg: &str) -> ! {
  eprintln!("{}", msg);
  std::process::exit(1)
}

fn main() {
  let args: Vec<String> = std::env::args().skip(1).collect();
  let rom = std::fs::read("roms2/super_mario_bros.nes").unwrap_or_else(|e| fail(&format!("roms2/super_mario_bros.nes: {}", e)));
  check_rom(&rom).unwrap_or_else(|e| fail(&e));
  let out = Path::new("target/prospector");
  // Garbage levels panic the emulator thousands of times; each is caught and catalogued.
  quiet_guarded_panics();
  match args.first().map(String::as_str) {
    Some("scan") => {
      let from = flag(&args, "--from").map_or(0x8000, |s| hex(&s));
      let to = flag(&args, "--to").map_or(0xFFFF, |s| hex(&s));
      let n = scan_range(&rom, from, to, out).unwrap_or_else(|e| fail(&e));
      eprintln!("scanned {} new offsets", n);
    }
    Some("solve") => {
      let k = flag(&args, "--k").map_or(50, |s| s.parse().unwrap_or_else(|_| fail("--k takes a number")));
      let n = solve_top(&rom, out, k).unwrap_or_else(|e| fail(&e));
      eprintln!("solved {} levels", n);
    }
    Some("calibrate") => calibrate(&rom, out).unwrap_or_else(|e| fail(&e)),
    Some(command @ ("rank" | "clips")) => {
      let mut entries = load_catalog(&out.join(CATALOG));
      dedupe(&mut entries);
      let calibration: BTreeMap<String, SolveRecord> = std::fs::read(out.join(CALIBRATION_FILE))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
      let rows = rows(&entries, &calibration);
      if command == "rank" {
        for r in &rows {
          println!(
            "{:>3}  {}  {}  bottleneck {}  kill {:.2}  rounds {}  narrower than {}",
            r["rank"],
            r["hex"],
            r["clear"],
            r["bottleneck"],
            r["kill_rate"].as_f64().unwrap_or(0.0),
            r["rounds"],
            r["narrower_than"]
          );
        }
        std::fs::create_dir_all(out).unwrap_or_else(|e| fail(&e.to_string()));
        write_view(&out.join("catalog_view.jsonl"), &entries).unwrap_or_else(|e| fail(&e));
        std::fs::write(out.join("ranking.json"), serde_json::to_vec_pretty(&rows).unwrap()).unwrap_or_else(|e| fail(&e.to_string()));
      } else {
        let top = flag(&args, "--top").map_or(10, |s| s.parse().unwrap_or_else(|_| fail("--top takes a number")));
        for r in rows.iter().take(top) {
          let offset = r["offset"].as_u64().unwrap() as u16;
          let entry = entries.iter().find(|e| e.offset == offset).unwrap();
          match guarded(|| record_clip(&rom, entry, r, &out.join("takes"))) {
            Ok(Ok(n)) => eprintln!("recorded {} ({} frames)", entry.hex, n),
            Ok(Err(e)) => eprintln!("skipped {}: {}", entry.hex, e),
            Err(panic) => eprintln!("skipped {}: replay panicked: {}", entry.hex, panic),
          }
        }
      }
    }
    _ => fail("usage: prospector scan [--from HEX] [--to HEX] | solve [--k N] | calibrate | rank | clips [--top N]"),
  }
}
