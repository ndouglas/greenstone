//! Reservoir Punch-Out!!. Output goes to target/punchout/, or to
//! $PUNCHOUT_ROOT for train, test and takes.
//!
//! Usage: cargo run --release --example punchout -- <command>
//!   train --phase N [--sweep] [--kinds NAME,...]
//!                                  record teacher fights and DAgger rounds (resumable);
//!                                  --sweep trains phase 1's memory sweep; --kinds
//!                                  trains only the named brains (e.g. stack4,echo0.5)
//!   test --phase N [--sweep]       fight every opponent with every brain; write results
//!   takes --phase N                film each contestant's best training and held-out
//!                                  fights (and, for phase 1, always-Idle against Glass Joe)
//!
//! PUNCHOUT_SMOKE=1 cuts train and test to 200-step fights, one round and no
//! stop gate, to check the plumbing.
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
use greenstone::nes::gym::punchout::brains::{BrainKind, CONTESTANTS, SWEEP};
use greenstone::nes::gym::punchout::dagger::{final_readout, test_with, train_with, Budget, Store};
use greenstone::nes::gym::punchout::fight::run_fight;
use greenstone::nes::gym::punchout::gate::{boot_strip, write_ppm};
use greenstone::nes::gym::punchout::moves::{Move, Phase};
use greenstone::nes::gym::punchout::start::{fight_start, Opponent, HELD_OUT, TRAIN};
use greenstone::nes::gym::punchout::takes::{record_idle_take, record_take};
use greenstone::nes::gym::punchout::PUNCH_OUT;
use greenstone::nes::gym::reservoir::demo::quiet_guarded_panics;
use greenstone::nes::gym::NesEmulator;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::Instant;

const TETRIS: &str = "roms2/tetris.nes";

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

/// `--phase N` from the arguments.
fn phase_arg(args: &[String]) -> Phase {
  args
    .windows(2)
    .find(|w| w[0] == "--phase")
    .and_then(|w| w[1].parse().ok())
    .and_then(Phase::from_number)
    .unwrap_or_else(|| fail("--phase takes 1, 2 or 3"))
}

/// The brains a command runs: the contestants, with --sweep phase 1's memory
/// sweep, or with --kinds NAME,NAME... just those (train only: a test writes
/// every brain's results at once).
fn kinds_arg(args: &[String], phase: Phase) -> Vec<BrainKind> {
  let all = if !args.iter().any(|a| a == "--sweep") {
    CONTESTANTS.to_vec()
  } else if phase == Phase::Defence {
    SWEEP.to_vec()
  } else {
    fail("--sweep is phase 1's memory sweep")
  };
  match args.windows(2).find(|w| w[0] == "--kinds") {
    None => all,
    Some(_) if args.first().is_some_and(|c| c != "train") => fail("--kinds is for train only"),
    Some(w) => w[1]
      .split(',')
      .map(|name| match BrainKind::from_name(name) {
        Some(kind) if all.contains(&kind) => kind,
        _ => fail(&format!(
          "--kinds: no brain {} here; these are {}",
          name,
          all.iter().map(|k| k.name()).collect::<Vec<_>>().join(",")
        )),
      })
      .collect(),
  }
}

/// The experiment's budget, or a smoke run's under PUNCHOUT_SMOKE=1.
fn budget() -> Budget {
  if std::env::var("PUNCHOUT_SMOKE").is_ok_and(|v| v == "1") {
    Budget {
      max_steps: 200,
      rounds: 1,
      gate: false,
    }
  } else {
    Budget::default()
  }
}

fn root() -> PathBuf {
  std::env::var("PUNCHOUT_ROOT").map_or_else(|_| PathBuf::from("target/punchout"), PathBuf::from)
}

/// How good a fight was for the phase: fewer hits in defence, else health
/// dealt less twice health lost.
fn merit(phase: Phase, entry: &Value) -> f64 {
  let f = |k: &str| entry[k].as_f64().unwrap_or(f64::NAN);
  match phase {
    Phase::Defence => -f("hits_per_minute"),
    _ => f("damage_dealt") - 2.0 * f("damage_taken"),
  }
}

/// The opponent among `from` whose fight scored best in `fights`.
fn best_of(phase: Phase, fights: &Value, from: &[Opponent]) -> Option<Opponent> {
  from
    .iter()
    .filter(|o| fights[o.name()].get("error").is_none() && !fights[o.name()].is_null())
    .max_by(|a, b| merit(phase, &fights[a.name()]).total_cmp(&merit(phase, &fights[b.name()])))
    .copied()
}

fn takes(rom: &[u8], tetris: &[u8], phase: Phase, root: &Path) {
  let store = Store::new(root, phase);
  let results: Value = std::fs::read_to_string(store.path("results.json"))
    .ok()
    .and_then(|s| serde_json::from_str(&s).ok())
    .unwrap_or_else(|| {
      fail(&format!(
        "no results for phase {}: run test --phase {} first",
        phase.number(),
        phase.number()
      ))
    });
  let dir = root.join("takes");
  let mut jobs: Vec<(Option<BrainKind>, Opponent)> = vec![];
  if phase == Phase::Defence {
    jobs.push((None, Opponent::GlassJoe));
  }
  for &kind in CONTESTANTS.iter() {
    let rounds = &results["kinds"][kind.name()];
    let last = rounds
      .as_object()
      .and_then(|m| m.keys().max_by_key(|k| k[6..].parse::<usize>().unwrap_or(0)).cloned());
    let Some(fights) = last.map(|k| &rounds[k]) else {
      eprintln!("{}: no results", kind.name());
      continue;
    };
    for from in [&TRAIN[..], &HELD_OUT[..]] {
      if let Some(o) = best_of(phase, fights, from) {
        jobs.push((Some(kind), o));
      }
    }
  }
  std::thread::scope(|scope| {
    for (kind, opponent) in jobs {
      let (store, dir) = (&store, &dir);
      scope.spawn(move || {
        let name = kind.map_or("idle".to_string(), |k| k.name());
        let out = dir.join(format!("phase{}-{}-{}", phase.number(), name, opponent.name()));
        let steps = match kind {
          None => record_idle_take(rom, phase, opponent, &out),
          Some(kind) => final_readout(kind, phase, store, tetris).and_then(|readout| record_take(rom, tetris, phase, kind, opponent, &readout, &out)),
        };
        match steps {
          Ok(n) => println!("{}: {} steps", out.display(), n),
          Err(e) => eprintln!("{}: {}", out.display(), e),
        }
      });
    }
  });
}

fn main() {
  quiet_guarded_panics();
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
    Some("train") => {
      let phase = phase_arg(&args);
      let tetris = std::fs::read(TETRIS).unwrap_or_else(|e| fail(&format!("{}: {}", TETRIS, e)));
      let store = Store::new(&root(), phase);
      let started = Instant::now();
      let mut log = |line: &str| {
        let t = started.elapsed().as_secs();
        println!("[{:02}:{:02}:{:02}] {}", t / 3600, t / 60 % 60, t % 60, line);
      };
      train_with(&rom, &tetris, phase, &kinds_arg(&args, phase), budget(), &store, &mut log).unwrap_or_else(|e| fail(&e));
      log("done");
    }
    Some("test") => {
      let phase = phase_arg(&args);
      let tetris = std::fs::read(TETRIS).unwrap_or_else(|e| fail(&format!("{}: {}", TETRIS, e)));
      let root = root();
      let store = Store::new(&root, phase);
      let sweep = args.iter().any(|a| a == "--sweep");
      let file = if sweep { "sweep.json" } else { "results.json" };
      let results =
        test_with(&rom, &tetris, phase, &kinds_arg(&args, phase), budget(), &store, file).unwrap_or_else(|e| fail(&e));
      let text = serde_json::to_string_pretty(&results).unwrap_or_default();
      if sweep {
        std::fs::write(root.join("sweep.json"), &text).unwrap_or_else(|e| fail(&e.to_string()));
      }
      println!("{}", text);
    }
    Some("takes") => {
      let tetris = std::fs::read(TETRIS).unwrap_or_else(|e| fail(&format!("{}: {}", TETRIS, e)));
      takes(&rom, &tetris, phase_arg(&args), &root());
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
    _ => fail("usage: punchout train|test|takes --phase N [--sweep] | gate | starts | idle | probe [--punch] [--boot N] [--tap-until N] [--frames N] [--poke ADDR=VAL@FRAME]... [--press MASK@FRAME]... [--out NAME]"),
  }
}
