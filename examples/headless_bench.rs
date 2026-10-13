//! Headless throughput benchmark.
//!
//! Usage: cargo run --release --example headless_bench -- path/to/rom.nes [frames]
//!
//! Runs the emulator with no display or audio, holding Start for the first
//! second and Right afterward, and reports frames per second on one core.

use greenstone::nes::bus::{BUTTON_RIGHT, BUTTON_START};
use greenstone::nes::gym::NesEmulator;
use std::time::Instant;

fn main() {
  let args: Vec<String> = std::env::args().collect();
  let rom_path = args.get(1).expect("usage: headless_bench <rom.nes> [frames]");
  let frames: usize = args.get(2).map(|s| s.parse().expect("frames must be an integer")).unwrap_or(6000);
  let rom = std::fs::read(rom_path).expect("could not read ROM");
  let mut emu = NesEmulator::new(&rom);

  let start = Instant::now();
  let mut instructions: u64 = 0;
  for frame in 0..frames {
    let buttons = if frame < 60 { BUTTON_START } else { BUTTON_RIGHT };
    instructions += emu.step_frame(buttons).instructions;
  }
  let elapsed = start.elapsed();

  let fps = frames as f64 / elapsed.as_secs_f64();
  println!("rom:          {}", rom_path);
  println!("frames:       {}", frames);
  println!("elapsed:      {:.3}s", elapsed.as_secs_f64());
  println!("frames/sec:   {:.0}", fps);
  println!("realtime x:   {:.1}", fps / 60.0988);
  println!("instr/frame:  {}", instructions / frames as u64);

  // Save/load state round trips, which bound any search over save states.
  let rounds = 200;
  let state = emu.save_state();
  let start = Instant::now();
  for _ in 0..rounds {
    let s = emu.save_state();
    emu.load_state(&s);
  }
  let elapsed = start.elapsed();
  println!("state bytes:  {}", state.len());
  println!("save+load:    {:.0} us", elapsed.as_micros() as f64 / rounds as f64);
}
