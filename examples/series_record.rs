//! Record simulated play for the interpretability series.
//!
//! Usage: cargo run --release --example series_record -- GAME PLAYER FRAMES OUT [--seed N] [--footage]
//!   GAME    smb1 | smb3 | castlevania
//!   PLAYER  human | rule (rule: SMB1 only) | idle

use std::path::PathBuf;

use greenstone::nes::gym::series::ram::{CASTLEVANIA, SMB1, SMB3};
use greenstone::nes::gym::series::record::{record, HumanLike, Idle, Player, RuleTeacher};

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() < 5 {
    eprintln!("usage: series_record GAME PLAYER FRAMES OUT [--seed N] [--footage]");
    std::process::exit(2);
  }
  let game = match args[1].as_str() {
    "smb1" => &SMB1,
    "smb3" => &SMB3,
    "castlevania" => &CASTLEVANIA,
    other => {
      eprintln!("unknown game {}", other);
      std::process::exit(2);
    }
  };
  let frames: usize = args[3].parse().expect("FRAMES");
  let out = PathBuf::from(&args[4]);
  let seed = args
    .iter()
    .position(|a| a == "--seed")
    .map_or(1, |i| args[i + 1].parse().expect("--seed N"));
  let footage = args.iter().any(|a| a == "--footage");
  let mut player: Box<dyn Player> = match args[2].as_str() {
    "human" => Box::new(HumanLike::new(game, seed)),
    "rule" => Box::new(RuleTeacher::new()),
    "idle" => Box::new(Idle),
    other => {
      eprintln!("unknown player {}", other);
      std::process::exit(2);
    }
  };
  let rom = std::fs::read(game.rom).unwrap_or_else(|e| {
    eprintln!("read {}: {}", game.rom, e);
    std::process::exit(1);
  });
  match record(game, &rom, player.as_mut(), frames, &out, footage) {
    Ok(s) => println!(
      "{} {}: {} frames, airborne {:.3}, {} restarts",
      game.name,
      player.name(),
      s.frames,
      s.label_rate,
      s.restarts
    ),
    Err(e) => {
      eprintln!("{}", e);
      std::process::exit(1);
    }
  }
}
