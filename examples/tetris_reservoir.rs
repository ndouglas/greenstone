//! Tetris as a reservoir computer for Super Mario Bros.
//!
//! Usage: cargo run --release --example tetris_reservoir -- [smb.nes] [tetris.nes] [demo-dir] [--legacy]
//!
//! `--legacy` adds the undriven, controller-driven and playfield conditions to the table.
//! Demonstrations are cached in `demo-dir` (default `target/tetris-reservoir-demos`)
//! as `demo-N.bin`; delete the directory to search again.

use greenstone::nes::gym::reservoir::demo::{load_cached, save_cached};
use greenstone::nes::gym::reservoir::drive::{FAST_LEVEL, PIECE_FRAME_LIMIT};
use greenstone::nes::gym::reservoir::encode::{ENEMY_BIN_PX, PIECES_PER_STEP};
use greenstone::nes::gym::reservoir::features::Condition;
use greenstone::nes::gym::reservoir::{find_demonstrations, run, ExperimentConfig};
use std::path::{Path, PathBuf};

fn main() {
  let legacy = std::env::args().any(|a| a == "--legacy");
  let args: Vec<String> = std::env::args().filter(|a| !a.starts_with("--")).collect();
  let smb_path = args.get(1).map(String::as_str).unwrap_or("roms2/super_mario_bros.nes");
  let tetris_path = args.get(2).map(String::as_str).unwrap_or("roms2/tetris.nes");
  let demo_dir = PathBuf::from(args.get(3).map(String::as_str).unwrap_or("target/tetris-reservoir-demos"));
  for path in [smb_path, tetris_path] {
    if !Path::new(path).exists() {
      println!("missing ROM: {}", path);
      return;
    }
  }
  let smb_rom = std::fs::read(smb_path).expect("read smb");
  let tetris_rom = std::fs::read(tetris_path).expect("read tetris");
  let config = ExperimentConfig::default();
  let mut log = |line: &str| eprintln!("{}", line);
  let demos = match load_cached(&demo_dir, config.demonstrations) {
    Some(demos) => {
      log(&format!("loaded {} demonstrations from {}", demos.len(), demo_dir.display()));
      demos
    }
    None => {
      let demos = find_demonstrations(&smb_rom, &config, &mut log);
      save_cached(&demo_dir, &demos).expect("write demos");
      log(&format!("saved {} demonstrations to {}", demos.len(), demo_dir.display()));
      demos
    }
  };
  let mut conditions: Vec<Condition> = Condition::ALL.to_vec();
  if legacy {
    conditions.extend(Condition::LEGACY);
  }
  let report = match run(&smb_rom, &tetris_rom, &demos, &conditions, &config, &mut log) {
    Ok(r) => r,
    Err(e) => {
      eprintln!("error: {}", e);
      std::process::exit(1);
    }
  };

  println!();
  for (i, d) in report.demos.iter().enumerate() {
    println!(
      "demonstration {}: {} idle frames, {} steps, x reached {}, cleared {}",
      i, d.noops, d.steps, d.x_reached, d.cleared
    );
  }
  println!("accuracy is over the buttons the demonstrations use: {:08b}", report.active_bits);
  println!("held-out columns are leave-one-demonstration-out, pooled over folds");
  println!(
    "placements: {} per step, enemy bins {} px, Tetris level {} via the menu, {} frames per piece at most",
    PIECES_PER_STEP, ENEMY_BIN_PX, FAST_LEVEL, PIECE_FRAME_LIMIT
  );
  println!();
  println!(
    "{:<18} | {:>5} | {:>9} | {:>12} | {:>12} | {:>7} | {:>5} | {:>6}",
    "condition", "rows", "train acc", "held-out acc", "held-out mse", "mean x", "max x", "resets"
  );
  println!("{}", "-".repeat(95));
  for r in &report.results {
    println!(
      "{:<18} | {:>5} | {:>9.3} | {:>12.3} | {:>12.4} | {:>7.1} | {:>5} | {:>6}",
      r.condition.name(),
      r.rows,
      r.train_accuracy,
      r.accuracy,
      r.mse,
      r.mean_x,
      r.max_x,
      r.resets
    );
  }
}
