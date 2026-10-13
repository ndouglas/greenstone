//! Tetris as a reservoir computer for Super Mario Bros.
//!
//! Usage: cargo run --release --example tetris_reservoir -- [smb.nes] [tetris.nes]

use greenstone::nes::gym::reservoir::{run, ExperimentConfig};
use std::path::Path;

fn main() {
  let args: Vec<String> = std::env::args().collect();
  let smb_path = args.get(1).map(String::as_str).unwrap_or("roms2/super_mario_bros.nes");
  let tetris_path = args.get(2).map(String::as_str).unwrap_or("roms2/tetris.nes");
  for path in [smb_path, tetris_path] {
    if !Path::new(path).exists() {
      println!("missing ROM: {}", path);
      return;
    }
  }
  let smb_rom = std::fs::read(smb_path).expect("read smb");
  let tetris_rom = std::fs::read(tetris_path).expect("read tetris");
  let config = ExperimentConfig::default();
  let report = match run(&smb_rom, &tetris_rom, &config, &mut |line| eprintln!("{}", line)) {
    Ok(r) => r,
    Err(e) => {
      eprintln!("error: {}", e);
      std::process::exit(1);
    }
  };

  println!();
  println!(
    "demonstration: {} steps, x reached {}, cleared {}",
    report.demo_steps, report.demo_x, report.demo_cleared
  );
  println!("accuracy is over the buttons the demonstration uses: {:08b}", report.active_bits);
  println!();
  println!(
    "{:<18} | {:>5} | {:>17} | {:>7} | {:>5} | {:>16}",
    "condition", "rows", "held-out accuracy", "mean x", "max x", "reservoir resets"
  );
  println!("{}", "-".repeat(84));
  for r in &report.results {
    println!(
      "{:<18} | {:>5} | {:>17.3} | {:>7.1} | {:>5} | {:>16}",
      r.condition.name(),
      r.rows,
      r.accuracy,
      r.mean_x,
      r.max_x,
      r.resets
    );
  }
}
