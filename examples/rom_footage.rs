//! Film any ROM with no buttons pressed, for the series: a take of its frames that the video tools read.
//!
//! Usage: cargo run --release --example rom_footage -- ROM FRAMES OUT
//!   writes OUT/footage/game.rgb (256×240 RGB frames) and OUT/footage/take.json

use std::path::PathBuf;

use greenstone::nes::gym::reservoir::film::TakeWriter;
use greenstone::nes::gym::NesEmulator;
use serde_json::json;

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 4 {
    eprintln!("usage: rom_footage ROM FRAMES OUT");
    std::process::exit(2);
  }
  let rom = std::fs::read(&args[1]).unwrap_or_else(|e| {
    eprintln!("read {}: {}", args[1], e);
    std::process::exit(1);
  });
  let frames: usize = args[2].parse().expect("FRAMES");
  let dir = PathBuf::from(&args[3]).join("footage");
  let mut emu = NesEmulator::new(&rom);
  let mut take = TakeWriter::new(&dir).unwrap_or_else(|e| {
    eprintln!("{}", e);
    std::process::exit(1);
  });
  for _ in 0..frames {
    emu.step_frame(0);
    take.frame("game", emu.framebuffer()).expect("write frame");
  }
  take.finish(json!({ "rom": args[1] })).expect("finish take");
  println!("{} frames of {} in {}", frames, args[1], dir.display());
}
