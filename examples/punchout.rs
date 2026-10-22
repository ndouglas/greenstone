//! Reservoir Punch-Out!!. Output goes to target/punchout/.
//!
//! Usage: cargo run --release --example punchout -- <command>
//!   gate                           boot the ROM and save a strip of frames as PPM
//!   starts                         save each opponent's first can-act frame as PPM
//!   idle                           fight each opponent standing still; print the scores
//!   probe [--punch] [--boot N] [--tap-until N] [--frames N] [--poke ADDR=VAL@FRAME]...
//!         [--press MASK@FRAME]... [--out NAME]
//!                                  boot into the fight, then record RAM every frame
//!                                  and a PPM every 30th frame (3,600 frames by default);
//!                                  each --poke writes a RAM byte (hex) at a probe frame;
//!                                  Start taps go on until frame --tap-until (default --boot);
//!                                  each --press MASK@FRAME holds buttons (hex) for 4 frames

use greenstone::nes::bus::{BUTTON_A, BUTTON_START, BUTTON_UP};
use greenstone::nes::gym::punchout::fight::run_fight;
use greenstone::nes::gym::punchout::gate::{boot_strip, write_ppm};
use greenstone::nes::gym::punchout::moves::Move;
use greenstone::nes::gym::punchout::start::{fight_start, HELD_OUT, TRAIN};
use greenstone::nes::gym::punchout::PUNCH_OUT;
use greenstone::nes::gym::NesEmulator;
use std::path::Path;

/// Frames the gate keeps: the title, the intro, and into the first fight.
const GATE_FRAMES: [usize; 11] = [60, 300, 600, 900, 1200, 1800, 2400, 3000, 3600, 4800, 6000];

/// How long the probe records by default, once the fight has begun.
const PROBE_FRAMES: usize = 3600;

/// One RAM write the probe makes: (address, value, probe frame).
type Poke = (u16, u8, usize);

fn parse_poke(arg: &str) -> Poke {
  let parsed = arg.split_once('=').and_then(|(a, rest)| {
    let (v, f) = rest.split_once('@')?;
    Some((u16::from_str_radix(a, 16).ok()?, u8::from_str_radix(v, 16).ok()?, f.parse().ok()?))
  });
  parsed.unwrap_or_else(|| fail("--poke takes ADDR=VAL@FRAME, address and value in hex"))
}

/// Boot with the gate's Start taps for `boot` frames, then record `frames`
/// frames: all 2,048 bytes of RAM per frame into ram.bin, and every 30th frame
/// as a PPM. Start taps go on until frame `tap_until`, counted from power-on.
/// With `punch`, Mac jabs high on alternate 8-frame blocks.
fn probe(rom: &[u8], dir: &Path, boot: usize, tap_until: usize, frames: usize, pokes: &[Poke], presses: &[(u8, usize)], punch: bool) {
  std::fs::create_dir_all(dir).unwrap_or_else(|e| fail(&e.to_string()));
  let mut emu = NesEmulator::new(rom);
  for f in 1..=boot {
    emu.step_frames(if f % 240 < 4 { BUTTON_START } else { 0 }, 1);
  }
  let mut ram = Vec::with_capacity(frames * 2048);
  for f in 0..frames {
    for &(address, value, _) in pokes.iter().filter(|p| p.2 == f) {
      emu.ram_write(address, value);
    }
    let tap = boot + f < tap_until && (boot + f) % 240 < 4;
    let pressed = presses.iter().find(|p| f >= p.1 && f < p.1 + 4);
    let buttons = if let Some(&(mask, _)) = pressed {
      mask
    } else if tap {
      BUTTON_START
    } else if punch && (f / 8) % 2 == 0 {
      BUTTON_UP | BUTTON_A
    } else {
      0
    };
    emu.step_frames(buttons, 1);
    ram.extend(emu.ram_read_range(0, 2048));
    if f % 30 == 0 {
      write_ppm(&dir.join(format!("frame-{:05}.ppm", f)), emu.framebuffer()).unwrap_or_else(|e| fail(&e));
    }
  }
  std::fs::write(dir.join("ram.bin"), ram).unwrap_or_else(|e| fail(&e.to_string()));
  println!("{}", dir.display());
}

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
    Some("starts") => {
      let dir = out.join("starts");
      std::fs::create_dir_all(&dir).unwrap_or_else(|e| fail(&e.to_string()));
      for o in TRAIN.iter().chain(HELD_OUT.iter()) {
        let (state, frame) = fight_start(&rom, *o).unwrap_or_else(|e| fail(&e));
        let mut emu = NesEmulator::new(&rom);
        emu.load_state(&state);
        emu.step_frames(0, 1);
        let path = dir.join(format!("{}.ppm", o.name()));
        write_ppm(&path, emu.framebuffer()).unwrap_or_else(|e| fail(&e));
        println!("{} at frame {}", path.display(), frame);
      }
    }
    Some("idle") => {
      for o in TRAIN.iter().chain(HELD_OUT.iter()) {
        let score = run_fight(&rom, *o, &mut |_| Move::Idle).unwrap_or_else(|e| fail(&e));
        println!(
          "{:<13} hits/min {:6.2}  hits {:3}  active {:4}/{:4}  down at {:?}  survival {:5.1}s",
          o.name(),
          score.hits_per_minute(),
          score.hits,
          score.active_steps,
          score.steps,
          score.mac_down_at,
          score.survival_seconds()
        );
      }
    }
    Some("probe") => {
      let punch = args.iter().any(|a| a == "--punch");
      let value = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1));
      let count = |flag: &str, default: usize| {
        value(flag).map_or(default, |n| n.parse().unwrap_or_else(|_| fail(&format!("{} takes a frame count", flag))))
      };
      let pokes: Vec<Poke> =
        args.windows(2).filter(|w| w[0] == "--poke").map(|w| parse_poke(&w[1])).collect();
      let name = value("--out").cloned().unwrap_or_else(|| (if punch { "probe-punch" } else { "probe" }).to_string());
      let presses: Vec<(u8, usize)> = args
        .windows(2)
        .filter(|w| w[0] == "--press")
        .map(|w| {
          let parsed = w[1].split_once('@').and_then(|(m, f)| Some((u8::from_str_radix(m, 16).ok()?, f.parse().ok()?)));
          parsed.unwrap_or_else(|| fail("--press takes MASK@FRAME, the mask in hex"))
        })
        .collect();
      let boot = count("--boot", 1300);
      let (tap_until, frames) = (count("--tap-until", boot), count("--frames", PROBE_FRAMES));
      probe(&rom, &out.join(name), boot, tap_until, frames, &pokes, &presses, punch);
    }
    _ => fail("usage: punchout gate | starts | idle | probe [--punch] [--boot N] [--tap-until N] [--frames N] [--poke ADDR=VAL@FRAME]... [--press MASK@FRAME]... [--out NAME]"),
  }
}
