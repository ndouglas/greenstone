//! Dump what the reservoir video needs: frames, per-step state and readout outputs.
//!
//! Usage: cargo run --release --example reservoir_dump -- [out-dir] [smb.nes] [tetris.nes] [demo-dir]
//! Needs the cached demonstrations from `tetris_reservoir` first.

use greenstone::nes::bus::BUTTON_A;
use greenstone::nes::gym::reservoir::demo::load_cached;
use greenstone::nes::gym::reservoir::encode::Encoder;
use greenstone::nes::gym::reservoir::features::Condition;
use greenstone::nes::gym::reservoir::ridge::Readout;
use greenstone::nes::gym::reservoir::trace::{fit_condition, replay, trace_episode};
use greenstone::nes::gym::reservoir::{episode_noops, ExperimentConfig};
use serde_json::json;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

fn hex(bytes: &[u8]) -> String {
  bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// The weight from every feature to button A, intercept first.
fn a_weights(readout: &Readout) -> Vec<f64> {
  (0..readout.d)
    .map(|i| readout.weights[i * readout.k + BUTTON_A.trailing_zeros() as usize])
    .collect()
}

fn main() {
  let args: Vec<String> = std::env::args().collect();
  let out = PathBuf::from(args.get(1).map(String::as_str).unwrap_or("target/tetris-reservoir-dump"));
  let smb_path = args.get(2).map(String::as_str).unwrap_or("roms2/super_mario_bros.nes");
  let tetris_path = args.get(3).map(String::as_str).unwrap_or("roms2/tetris.nes");
  let demo_dir = PathBuf::from(args.get(4).map(String::as_str).unwrap_or("target/tetris-reservoir-demos"));
  let smb_rom = std::fs::read(smb_path).expect("read smb");
  let tetris_rom = std::fs::read(tetris_path).expect("read tetris");
  let config = ExperimentConfig::default();
  let demos = load_cached(&demo_dir, config.demonstrations).unwrap_or_else(|| {
    eprintln!(
      "no cached demonstrations in {}; run the tetris_reservoir example first",
      demo_dir.display()
    );
    std::process::exit(1);
  });
  std::fs::create_dir_all(&out).expect("create out dir");

  let conditions = [
    Condition::Raw,
    Condition::PlacementOnehot(Encoder::Salient),
    Condition::PlacementTetris(Encoder::Salient),
    Condition::Clock,
  ];
  let readouts: Vec<(Condition, Readout)> = conditions
    .iter()
    .map(|&c| {
      eprintln!("fitting {} on demonstrations 1..{}", c.name(), demos.len());
      (c, fit_condition(c, &smb_rom, &tetris_rom, &demos[1..], config.lambda).expect("fit"))
    })
    .collect();

  eprintln!("replaying demonstration 0 with frames");
  let mut smb_sink = BufWriter::new(File::create(out.join("demo.smb.rgb")).expect("create"));
  let mut tetris_sink = BufWriter::new(File::create(out.join("demo.tetris.rgb")).expect("create"));
  let steps = replay(&smb_rom, &tetris_rom, &demos[0], &readouts, &mut smb_sink, &mut tetris_sink).expect("replay");
  let demo_json = json!({
    "demo": {"noops": demos[0].noops, "steps": demos[0].buttons.len(), "x_reached": demos[0].x_reached, "cleared": demos[0].cleared},
    "conditions": conditions.iter().map(|c| c.name()).collect::<Vec<_>>(),
    "weights": {
      "raw_a": a_weights(&readouts[0].1),
      "onehot_a": a_weights(&readouts[1].1),
      "tetris_a": a_weights(&readouts[2].1),
    },
    "steps": steps.iter().map(|s| json!({
      "x": s.x, "y": s.y, "buttons": s.buttons, "cutscene": s.cutscene,
      "window": hex(&s.window), "board": hex(&s.board), "tetris_ram": hex(&s.tetris_ram),
      "enemies": s.enemies.iter().map(|e| json!({"slot": e.slot, "kind": e.kind, "x": e.x, "y": e.y})).collect::<Vec<_>>(),
      "placements": s.placements,
      "outputs": s.outputs,
    })).collect::<Vec<_>>(),
  });
  std::fs::write(out.join("demo.json"), serde_json::to_string(&demo_json).unwrap()).expect("write json");

  for (name, index) in [("raw", 0usize), ("clock", 3usize)] {
    eprintln!("tracing a closed-loop {} episode", name);
    let noops = episode_noops(0, demos.len(), config.noop_max);
    let mut sink = BufWriter::new(File::create(out.join(format!("episode-{}.smb.rgb", name))).expect("create"));
    let (condition, readout) = &readouts[index];
    let steps = trace_episode(*condition, readout, &smb_rom, &tetris_rom, noops, config.max_steps, &mut sink).expect("episode");
    let v = json!({
      "noops": noops,
      "steps": steps.iter().map(|s| json!({"x": s.x, "buttons": s.buttons, "dead": s.dead})).collect::<Vec<_>>(),
    });
    std::fs::write(out.join(format!("episode-{}.json", name)), serde_json::to_string(&v).unwrap()).expect("write json");
  }
  eprintln!("wrote {}", out.display());
}
