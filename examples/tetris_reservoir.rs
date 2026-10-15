//! Tetris as a reservoir computer for Super Mario Bros.
//!
//! Usage: cargo run --release --example tetris_reservoir -- [smb.nes] [tetris.nes] [demo-dir] [--legacy] [--horizon=N]
//!
//! `--legacy` adds the undriven, controller-driven and playfield conditions to the table.
//! `--horizon=N` sets the jump-label horizon in steps (default 0, the raw label).
//! `--no-tetris` drops every condition that runs a Tetris emulator, for a table in minutes.
//! `--lambda=X` sets the ridge penalty (default 1).
//! `--logistic` fits the readouts by logistic regression instead of ridge.
//! `--only=PREFIX` keeps only the conditions whose names start with PREFIX.
//! `--stages=1-1,6-1` takes the demonstrations from those levels in turn (default 1-1);
//! the default cache directory then carries the stage names.
//! `--noop-max=N` spreads the demonstration offsets over N idle frames (default 120).
//! A cache whose demonstrations start at other offsets is searched again.
//! `--rule` takes the demonstrations from the rule teacher (a jump rule over the
//! window, searched over a grid) instead of the beam search; its default cache is
//! `target/tetris-reservoir-demos-rule` and the rule is saved beside the demonstrations.
//! Demonstrations are cached in `demo-dir` (default `target/tetris-reservoir-demos`)
//! as `demo-N.bin`; delete the directory to search again.

use greenstone::nes::gym::reservoir::demo::{load_cached, save_cached, Stage};
use greenstone::nes::gym::reservoir::drive::{FAST_LEVEL, PIECE_FRAME_LIMIT};
use greenstone::nes::gym::reservoir::encode::{BankEncoder, Encoder};
use greenstone::nes::gym::reservoir::encode::{BANK_BOARDS, ENEMY_BIN_PX, PIECES_PER_STEP};
use greenstone::nes::gym::reservoir::features::{Condition, SMB_WINDOW_LEN};
use greenstone::nes::gym::reservoir::search::{search_bank_encoder, search_encoder, Region};
use greenstone::nes::gym::reservoir::teacher::Rule;
use greenstone::nes::gym::reservoir::{
  active_bits, demos_at_configured_offsets, find_demonstrations, find_rule_demonstrations, run, ExperimentConfig,
};
use std::path::{Path, PathBuf};

fn main() {
  let legacy = std::env::args().any(|a| a == "--legacy");
  let no_tetris = std::env::args().any(|a| a == "--no-tetris");
  let rule_teacher = std::env::args().any(|a| a == "--rule");
  let logistic = std::env::args().any(|a| a == "--logistic");
  let lambda = std::env::args().find_map(|a| a.strip_prefix("--lambda=").map(|v| v.parse::<f64>().expect("--lambda=X")));
  let horizon = std::env::args().find_map(|a| a.strip_prefix("--horizon=").map(|v| v.parse::<usize>().expect("--horizon=N")));
  let noop_max = std::env::args().find_map(|a| a.strip_prefix("--noop-max=").map(|v| v.parse::<usize>().expect("--noop-max=N")));
  let only = std::env::args().find_map(|a| a.strip_prefix("--only=").map(str::to_string));
  let stages: Vec<Stage> = std::env::args()
    .find_map(|a| a.strip_prefix("--stages=").map(str::to_string))
    .map(|v| v.split(',').map(|s| Stage::parse(s).expect("--stages=W-L,W-L")).collect())
    .unwrap_or_else(|| vec![Stage::ONE_ONE]);
  let stage_suffix = if stages == [Stage::ONE_ONE] {
    String::new()
  } else {
    format!("-{}", stages.iter().map(|s| s.name()).collect::<Vec<_>>().join("+"))
  };
  let args: Vec<String> = std::env::args().filter(|a| !a.starts_with("--")).collect();
  let smb_path = args.get(1).map(String::as_str).unwrap_or("roms2/super_mario_bros.nes");
  let tetris_path = args.get(2).map(String::as_str).unwrap_or("roms2/tetris.nes");
  let default_dir = format!("target/tetris-reservoir-demos{}{}", if rule_teacher { "-rule" } else { "" }, stage_suffix);
  let demo_dir = PathBuf::from(args.get(3).map(String::as_str).unwrap_or(&default_dir));
  let rule_path = demo_dir.join("rule.bin");
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
  config.logistic = logistic;
  if let Some(n) = noop_max {
    config.noop_max = n;
  }
  config.stages = stages;
  let mut log = |line: &str| eprintln!("{}", line);
  let mut rule: Option<Rule> = None;
  let cached = load_cached(&demo_dir, config.demonstrations).filter(|demos| {
    let matches = demos_at_configured_offsets(demos, &config);
    if !matches {
      log(&format!(
        "cached demonstrations in {} start at other offsets than {} over {} frames; searching again",
        demo_dir.display(),
        config.demonstrations,
        config.noop_max
      ));
    }
    matches
  });
  let demos = match cached {
    Some(demos) => {
      log(&format!("loaded {} demonstrations from {}", demos.len(), demo_dir.display()));
      rule = std::fs::read(&rule_path).ok().and_then(|b| Rule::from_bytes(&b).ok());
      demos
    }
    None => {
      let demos = if rule_teacher {
        let (found, demos) = find_rule_demonstrations(&smb_rom, &config, &mut log);
        rule = Some(found);
        demos
      } else {
        find_demonstrations(&smb_rom, &config, &mut log)
      };
      save_cached(&demo_dir, &demos).expect("write demos");
      if let Some(r) = rule {
        std::fs::write(&rule_path, r.to_bytes()).expect("write rule");
      }
      log(&format!("saved {} demonstrations to {}", demos.len(), demo_dir.display()));
      demos
    }
  };
  let mask = demos.iter().fold(0u8, |acc, d| acc | active_bits(&d.buttons));
  log("searching the encoder on the one-hot control");
  let searched = search_encoder(&smb_rom, &demos, config.lambda, config.logistic, mask, config.jump_horizon, &mut log);
  log("searching four features per region for the bank");
  let bank = search_bank_encoder(&smb_rom, &demos, config.lambda, config.logistic, mask, config.jump_horizon, &mut log);
  let mut conditions = Condition::default_table(Some(searched), Some(bank));
  if legacy {
    conditions.extend(Condition::LEGACY);
  }
  if no_tetris {
    conditions.retain(|c| !c.uses_reservoir());
  }
  if let Some(prefix) = &only {
    conditions.retain(|c| c.name().starts_with(prefix.as_str()));
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
      "demonstration {}: {} from {} idle frames, {} steps, x reached {}, cleared {}",
      i,
      d.stage.name(),
      d.noops,
      d.steps,
      d.x_reached,
      d.cleared
    );
  }
  match rule {
    Some(r) => println!("teacher: rule, {}", r.describe()),
    None => println!("teacher: beam search over save states, fewest A presses first"),
  }
  println!("accuracy is over the buttons the demonstrations use: {:08b}", report.active_bits);
  println!(
    "held-out columns are leave-one-demonstration-out, pooled over folds; {} readout, penalty {}",
    if config.logistic { "logistic" } else { "ridge" },
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
    "demonstration offsets: {} spread over {} idle frames; episodes start between them",
    config.demonstrations, config.noop_max
  );
  println!(
    "placements: {} per step, enemy bins {} px, Tetris level {} via the menu, {} frames per piece at most",
    PIECES_PER_STEP, ENEMY_BIN_PX, FAST_LEVEL, PIECE_FRAME_LIMIT
  );
  println!(
    "bank: {} boards of {} placements each, hand: Mario, enemies, the floor ahead, the wall ahead; searched: four per region by onsets plus A specificity; readout sees the playfields; tidy adds a fifth piece on the lowest column",
    BANK_BOARDS, PIECES_PER_STEP
  );
  if let Encoder::Searched(features) = searched {
    let picks: Vec<String> = features.iter().map(|f| f.describe()).collect();
    println!("searched encoder: {}", picks.join(", "));
  }
  if let BankEncoder::Searched(boards) = bank {
    for (region, board) in Region::ALL.iter().zip(boards.iter()) {
      let picks: Vec<String> = board.iter().map(|f| f.describe()).collect();
      println!("searched bank, {}: {}", region.name(), picks.join(", "));
    }
  }
  println!();
  println!(
    "{:<18} | {:>5} | {:>9} | {:>12} | {:>7} | {:>6} | {:>12} | {:>7} | {:>5} | {:>6}",
    "condition", "rows", "train acc", "held-out acc", "A acc", "onset", "held-out mse", "mean x", "max x", "resets"
  );
  println!("{}", "-".repeat(114));
  for r in &report.results {
    println!(
      "{:<18} | {:>5} | {:>9.3} | {:>12.3} | {:>7.3} | {:>6.3} | {:>12.4} | {:>7.1} | {:>5} | {:>6}",
      r.condition.name(),
      r.rows,
      r.train_accuracy,
      r.accuracy,
      r.a_accuracy,
      r.onset_recall,
      r.mse,
      r.mean_x,
      r.max_x,
      r.resets
    );
  }
}
