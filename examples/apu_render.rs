//! Play a register log through Greenstone's APU and write a WAV.
//!
//! Usage: cargo run --release --example apu_render -- <log> <wav>

use greenstone::nes::apu::render::{parse_log, render_log, wav_bytes};

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() != 3 {
    eprintln!("usage: apu_render <log> <wav>");
    std::process::exit(2);
  }
  let text = std::fs::read_to_string(&args[1]).unwrap_or_else(|e| {
    eprintln!("read {}: {}", args[1], e);
    std::process::exit(1);
  });
  let (frames, writes) = parse_log(&text).unwrap_or_else(|e| {
    eprintln!("{}: {}", args[1], e);
    std::process::exit(1);
  });
  std::fs::write(&args[2], wav_bytes(&render_log(frames, &writes))).expect("write wav");
}
