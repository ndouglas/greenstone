//! Record the reservoir film's takes into target/reservoir-film/.
//!
//! Usage: cargo run --release --example reservoir_film -- [--take NAME]... [--force]

use greenstone::nes::gym::reservoir::film::{record, takes, FilmContext};
use std::path::Path;

fn main() {
  let args: Vec<String> = std::env::args().skip(1).collect();
  let force = args.iter().any(|a| a == "--force");
  let named: Vec<&str> = args.windows(2).filter(|w| w[0] == "--take").map(|w| w[1].as_str()).collect();
  let ctx = FilmContext::load(
    Path::new("roms2/super_mario_bros.nes"),
    Path::new("roms2/tetris.nes"),
    Path::new("target"),
  )
  .unwrap_or_else(|e| {
    eprintln!("{}", e);
    std::process::exit(1);
  });
  let out = Path::new("target/reservoir-film");
  for (name, _) in takes() {
    let wanted = if named.is_empty() {
      force || !out.join(name).join("take.json").exists()
    } else {
      named.contains(&name)
    };
    if !wanted {
      continue;
    }
    eprintln!("recording {}", name);
    let start = std::time::Instant::now();
    if let Err(e) = record(&ctx, name, out) {
      eprintln!("{}: {}", name, e);
      std::process::exit(1);
    }
    eprintln!("{} done in {:.0?}", name, start.elapsed());
  }
}
