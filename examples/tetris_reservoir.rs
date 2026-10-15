//! Tetris as a reservoir computer for Super Mario Bros.
//!
//! Usage: cargo run --release --example tetris_reservoir -- [smb.nes] [tetris.nes] [demo-dir] [--legacy] [--horizon=N]
//!
//! `--legacy` adds the undriven, controller-driven and playfield conditions to the table.
//! `--horizon=N` sets the jump-label horizon in steps (default 0, the raw label).
//! `--no-tetris` drops every condition that runs a Tetris emulator, for a table in minutes.
//! `--lambda=X` sets the ridge penalty (default 1).
//! Demonstrations are cached in `demo-dir` (default `target/tetris-reservoir-demos`)
//! as `demo-N.bin`; delete the directory to search again.

use greenstone::nes::gym::reservoir::demo::{load_cached, save_cached};
use greenstone::nes::gym::reservoir::drive::{FAST_LEVEL, PIECE_FRAME_LIMIT};
use greenstone::nes::gym::reservoir::encode::Encoder;
use greenstone::nes::gym::reservoir::encode::{ENEMY_BIN_PX, PIECES_PER_STEP};
use greenstone::nes::gym::reservoir::features::{Condition, SMB_WINDOW_LEN};
use greenstone::nes::gym::reservoir::search::search_encoder;
use greenstone::nes::gym::reservoir::{active_bits, find_demonstrations, run, ExperimentConfig};
use std::path::{Path, PathBuf};

fn main() {
  let legacy = std::env::args().any(|a| a == "--legacy");
  let no_tetris = std::env::args().any(|a| a == "--no-tetris");
  let lambda = std::env::args().find_map(|a| a.strip_prefix("--lambda=").map(|v| v.parse::<f64>().expect("--lambda=X")));
  let horizon = std::env::args().find_map(|a| a.strip_prefix("--horizon=").map(|v| v.parse::<usize>().expect("--horizon=N")));
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
  let mut config = ExperimentConfig::default();
  if let Some(h) = horizon {
    config.jump_horizon = h;
  }
  if let Some(l) = lambda {
    config.lambda = l;
  }
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
  let mask = demos.iter().fold(0u8, |acc, d| acc | active_bits(&d.buttons));
  log("searching the encoder on the one-hot control");
  let searched = search_encoder(&smb_rom, &demos, config.lambda, mask, config.jump_horizon, &mut log);
  let mut conditions = Condition::default_table(Some(searched));
  if legacy {
    conditions.extend(Condition::LEGACY);
  }
  if no_tetris {
    conditions.retain(|c| !c.uses_reservoir());
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
  println!(
    "held-out columns are leave-one-demonstration-out, pooled over folds; ridge lambda {}",
    config.lambda
  );
  if config.jump_horizon == 0 {
    println!(
      "A label: raw; window: zero page (controller bytes hidden) plus the terrain buffer, {} bytes",
      SMB_WINDOW_LEN
    );
  } else {
    println!(
      "A label: pressed within the next {} steps ({} frames); window: zero page plus 8 terrain columns ahead, {} bytes",
      config.jump_horizon,
      config.jump_horizon * 4,
      SMB_WINDOW_LEN
    );
  }
  println!(
    "placements: {} per step, enemy bins {} px, Tetris level {} via the menu, {} frames per piece at most",
    PIECES_PER_STEP, ENEMY_BIN_PX, FAST_LEVEL, PIECE_FRAME_LIMIT
  );
  if let Encoder::Searched(features) = searched {
    let picks: Vec<String> = features
      .iter()
      .map(|f| format!("${:02X}{} /{}", f.address, if f.relative { "-x" } else { "" }, 1 << f.shift))
      .collect();
    println!("searched encoder: {}", picks.join(", "));
  }
  println!();
  println!(
    "{:<18} | {:>5} | {:>9} | {:>12} | {:>7} | {:>12} | {:>7} | {:>5} | {:>6}",
    "condition", "rows", "train acc", "held-out acc", "A acc", "held-out mse", "mean x", "max x", "resets"
  );
  println!("{}", "-".repeat(105));
  for r in &report.results {
    println!(
      "{:<18} | {:>5} | {:>9.3} | {:>12.3} | {:>7.3} | {:>12.4} | {:>7.1} | {:>5} | {:>6}",
      r.condition.name(),
      r.rows,
      r.train_accuracy,
      r.accuracy,
      r.a_accuracy,
      r.mse,
      r.mean_x,
      r.max_x,
      r.resets
    );
  }
}
