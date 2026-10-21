//! Reservoir Punch-Out!!. Output goes to target/punchout/.
//!
//! Usage: cargo run --release --example punchout -- <command>
//!   gate                           boot the ROM and save a strip of frames as PPM

use greenstone::nes::gym::punchout::gate::{boot_strip, write_ppm};
use greenstone::nes::gym::punchout::PUNCH_OUT;
use std::path::Path;

/// Frames the gate keeps: the title, the intro, and into the first fight.
const GATE_FRAMES: [usize; 11] = [60, 300, 600, 900, 1200, 1800, 2400, 3000, 3600, 4800, 6000];

fn fail(msg: &str) -> ! {
  eprintln!("{}", msg);
  std::process::exit(1)
}

fn main() {
  let args: Vec<String> = std::env::args().skip(1).collect();
  let rom = std::fs::read(PUNCH_OUT).unwrap_or_else(|e| fail(&format!("{}: {}", PUNCH_OUT, e)));
  let out = Path::new("target/punchout");
  match args.first().map(String::as_str) {
    Some("gate") => {
      let dir = out.join("gate");
      std::fs::create_dir_all(&dir).unwrap_or_else(|e| fail(&e.to_string()));
      for (frame, rgb) in boot_strip(&rom, &GATE_FRAMES, 240) {
        let path = dir.join(format!("frame-{:05}.ppm", frame));
        write_ppm(&path, &rgb).unwrap_or_else(|e| fail(&e));
        println!("{}", path.display());
      }
    }
    _ => fail("usage: punchout gate"),
  }
}
