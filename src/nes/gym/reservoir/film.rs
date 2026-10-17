//! Takes for the reservoir film: each runs the emulators with one
//! intervention and writes frames plus a `take.json`. The compositor in
//! `tools/reservoir_film/` reads them. See
//! `docs/superpowers/specs/2026-10-17-reservoir-film-design.md`.

use super::demo::{load_cached, smb_dead, smb_start_state_at, smb_window, smb_x, smb_y, Demonstration, Stage, MACRO_FRAMES};
use super::teacher::{tripwires, Player, Rule, Tripwires};
use super::trace::enemies;
use super::Lcg;
use crate::nes::bus::{BUTTON_RIGHT, BUTTON_START};
use crate::nes::gym::reward::smb1_ram;
use crate::nes::gym::NesEmulator;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

pub const FRAME_BYTES: usize = 256 * 240 * 3;

/// Demonstration caches the takes read, and the command that makes each.
pub const DEMO_CACHES: &[(&str, &str)] = &[
  ("tetris-reservoir-demos", "cargo run --release --example tetris_reservoir"),
  ("tetris-reservoir-demos-rule", "cargo run --release --example tetris_reservoir -- --rule"),
  (
    "tetris-reservoir-demos-rule-1-1+6-1",
    "cargo run --release --example tetris_reservoir -- --rule --stages=1-1,6-1",
  ),
];

/// The rule played when a cache has no `rule.bin`: run 30's.
pub const DEFAULT_RULE: &str = "40,3,3,9";

/// The cache whose rule the world takes play.
const RULE_CACHE: &str = "tetris-reservoir-demos-rule";

pub struct FilmContext {
  pub smb_rom: Vec<u8>,
  pub tetris_rom: Vec<u8>,
  pub target: PathBuf,
}

impl FilmContext {
  pub fn load(smb: &Path, tetris: &Path, target: &Path) -> Result<FilmContext, String> {
    let read = |p: &Path| std::fs::read(p).map_err(|e| format!("cannot read ROM {}: {}", p.display(), e));
    Ok(FilmContext {
      smb_rom: read(smb)?,
      tetris_rom: read(tetris)?,
      target: target.to_path_buf(),
    })
  }

  fn missing(&self, cache: &str) -> String {
    let dir = self.target.join(cache);
    let make = DEMO_CACHES.iter().find(|(c, _)| *c == cache).map_or("(no known command)", |(_, m)| m);
    format!("missing demonstrations in {}; make them with: {}", dir.display(), make)
  }

  pub fn demos(&self, cache: &str, count: usize) -> Result<Vec<Demonstration>, String> {
    load_cached(&self.target.join(cache), count).ok_or_else(|| self.missing(cache))
  }

  pub fn rule(&self, cache: &str) -> Result<Rule, String> {
    match std::fs::read(self.target.join(cache).join("rule.bin")) {
      Ok(bytes) => Rule::from_bytes(&bytes),
      Err(_) => Rule::parse(DEFAULT_RULE),
    }
  }
}

/// Frame streams and the metadata of one take, written under its directory.
pub struct TakeWriter {
  dir: PathBuf,
  streams: BTreeMap<String, (BufWriter<File>, usize)>,
}

impl TakeWriter {
  pub fn new(dir: &Path) -> Result<TakeWriter, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("create {}: {}", dir.display(), e))?;
    Ok(TakeWriter {
      dir: dir.to_path_buf(),
      streams: BTreeMap::new(),
    })
  }

  pub fn frame(&mut self, stream: &str, rgb: &[u8]) -> Result<(), String> {
    if rgb.len() != FRAME_BYTES {
      return Err(format!("frame for {} is {} bytes, not {}", stream, rgb.len(), FRAME_BYTES));
    }
    if !self.streams.contains_key(stream) {
      let path = self.dir.join(format!("{}.rgb", stream));
      let file = File::create(&path).map_err(|e| format!("create {}: {}", path.display(), e))?;
      self.streams.insert(stream.to_string(), (BufWriter::new(file), 0));
    }
    let (w, n) = self.streams.get_mut(stream).unwrap();
    w.write_all(rgb).map_err(|e| e.to_string())?;
    *n += 1;
    Ok(())
  }

  pub fn finish(self, mut meta: Value) -> Result<(), String> {
    let mut counts = serde_json::Map::new();
    for (name, (mut w, n)) in self.streams {
      w.flush().map_err(|e| e.to_string())?;
      counts.insert(name, json!(n));
    }
    meta["streams"] = Value::Object(counts);
    let tmp = self.dir.join("take.json.partial");
    std::fs::write(&tmp, serde_json::to_vec(&meta).unwrap()).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, self.dir.join("take.json")).map_err(|e| e.to_string())
  }
}

pub type TakeFn = fn(&FilmContext, &mut TakeWriter) -> Result<Value, String>;

/// Every take, in recording order.
pub fn takes() -> Vec<(&'static str, TakeFn)> {
  vec![
    ("lives_glyphs", lives_glyphs as TakeFn),
    ("matrix_walk", matrix_walk),
    ("poke_enemy", poke_enemy),
    ("poke_terrain", poke_terrain),
    ("noise_bytes", noise_bytes),
  ]
}

/// Record one take into `out/<name>/`, by way of `<name>.partial` so a
/// failed recording never looks finished.
pub fn record(ctx: &FilmContext, name: &str, out: &Path) -> Result<(), String> {
  let (_, f) = takes()
    .into_iter()
    .find(|(n, _)| *n == name)
    .ok_or_else(|| format!("no take named {}", name))?;
  let partial = out.join(format!("{}.partial", name));
  let _ = std::fs::remove_dir_all(&partial);
  let mut w = TakeWriter::new(&partial)?;
  let meta = f(ctx, &mut w)?;
  w.finish(meta)?;
  let done = out.join(name);
  let _ = std::fs::remove_dir_all(&done);
  std::fs::rename(&partial, &done).map_err(|e| e.to_string())
}

pub fn hex(bytes: &[u8]) -> String {
  bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// The block-buffer byte for an absolute level column and row, laid out as
/// `terrain_ahead` reads it: even pages at $0500, odd at $05D0, 16 columns a row.
pub fn block_address(column: usize, row: usize) -> u16 {
  let base: u16 = if (column / 16) % 2 == 0 { 0x0500 } else { 0x05D0 };
  base + (row * 16 + column % 16) as u16
}

/// A fresh SMB at the start of `stage` after `noops` idle frames.
fn smb_at(ctx: &FilmContext, stage: Stage, noops: usize) -> NesEmulator {
  let mut smb = NesEmulator::new(&ctx.smb_rom);
  smb.load_state(&smb_start_state_at(&ctx.smb_rom, stage));
  smb.step_frames(0, noops);
  smb
}

/// Play the rule until `until` holds, recording nothing. Errors after `limit` steps.
fn play_until(smb: &mut NesEmulator, player: &mut Player, limit: usize, until: &mut dyn FnMut(&mut NesEmulator) -> bool) -> Result<usize, String> {
  for step in 0..limit {
    if until(smb) {
      return Ok(step);
    }
    let window = smb_window(smb);
    let buttons = player.buttons(&window);
    smb.step_frames(buttons, MACRO_FRAMES);
  }
  Err(format!("condition not reached in {} steps", limit))
}

fn on_ground(smb: &mut NesEmulator) -> bool {
  smb.ram_read(smb1_ram::PLAYER_FLOAT_STATE) == 0
}

fn tripwire_json(t: Tripwires) -> Value {
  json!({"enemy": t.enemy, "pit": t.pit, "wall": t.wall})
}

/// The intermission screen for every lives value, and where its ones digit is.
fn lives_glyphs(ctx: &FilmContext, w: &mut TakeWriter) -> Result<Value, String> {
  let mut frames = Vec::new();
  for lives in 0..=255u8 {
    let mut emu = NesEmulator::new(&ctx.smb_rom);
    emu.step_frames(0, 40);
    for f in 0..90 {
      emu.ram_write(smb1_ram::LIVES, lives);
      emu.step_frame(if f < 6 { BUTTON_START } else { 0 });
    }
    w.frame("screen", emu.framebuffer())?;
    frames.push(emu.framebuffer().to_vec());
  }
  // The ones digit: where lives 20 and 21 differ.
  let (a, b) = (&frames[20], &frames[21]);
  let (mut x0, mut y0, mut x1, mut y1) = (256, 240, 0, 0);
  for y in 0..240 {
    for x in 0..256 {
      let i = (y * 256 + x) * 3;
      if a[i..i + 3] != b[i..i + 3] {
        x0 = x0.min(x);
        y0 = y0.min(y);
        x1 = x1.max(x);
        y1 = y1.max(y);
      }
    }
  }
  if x1 < x0 {
    return Err("lives 20 and 21 drew the same screen".into());
  }
  Ok(json!({"digit_box": [x0, y0, x1 - x0 + 1, y1 - y0 + 1]}))
}

/// The rule plays 1-1 for 300 steps; every frame, and every step's window.
fn matrix_walk(ctx: &FilmContext, w: &mut TakeWriter) -> Result<Value, String> {
  let rule = ctx.rule(RULE_CACHE)?;
  let mut smb = smb_at(ctx, Stage::ONE_ONE, 0);
  let mut player = Player::new(rule);
  let mut steps = Vec::new();
  for _ in 0..300 {
    let window = smb_window(&mut smb);
    let buttons = player.buttons(&window);
    steps.push(json!({
      "x": smb_x(&mut smb),
      "y": smb_y(&mut smb),
      "airborne": window[smb1_ram::PLAYER_FLOAT_STATE as usize],
      "buttons": buttons,
      "window": hex(&window),
      "enemies": enemies(&window).iter().map(|e| json!({"slot": e.slot, "kind": e.kind, "x": e.x, "y": e.y})).collect::<Vec<_>>(),
      "tripwires": tripwire_json(tripwires(rule, &window)),
    }));
    for _ in 0..MACRO_FRAMES {
      smb.step_frame(buttons);
      w.frame("smb", smb.framebuffer())?;
    }
  }
  Ok(json!({
    "steps": steps,
    "labels": [
      {"address": 0x86, "span": 1, "name": "Mario's x"},
      {"address": 0x1D, "span": 1, "name": "on the ground, jumping, falling"},
      {"address": 0x0F, "span": 5, "name": "enemy slots"},
      {"address": 0x87, "span": 5, "name": "enemy x"},
      {"address": 0x100, "span": 104, "name": "the terrain ahead"},
    ],
  }))
}

/// How far each poke moves the enemy. Further and it leaves the screen and despawns.
const POKE_PX: i32 = 40;

/// An enemy ahead is pushed `POKE_PX` right four times while Mario stands still.
fn poke_enemy(ctx: &FilmContext, w: &mut TakeWriter) -> Result<Value, String> {
  let mut smb = smb_at(ctx, Stage::ONE_ONE, 0);
  let mut player = Player::new(ctx.rule(RULE_CACHE)?);
  play_until(&mut smb, &mut player, 300, &mut |s| {
    let x = smb_x(s);
    let window = smb_window(s);
    on_ground(s) && enemies(&window).iter().any(|e| (40..110).contains(&(e.x - x)))
  })?;
  let window = smb_window(&mut smb);
  let x = smb_x(&mut smb);
  let slot = enemies(&window)
    .into_iter()
    .find(|e| (40..110).contains(&(e.x - x)))
    .ok_or("the enemy left")?
    .slot as u16;
  let mut pokes = Vec::new();
  for frame in 0..210 {
    if frame >= 60 && (frame - 60) % 30 == 0 && pokes.len() < 4 {
      let (page, pixel) = (0x6E + slot, 0x87 + slot);
      let before = smb.ram_read(page) as i32 * 256 + smb.ram_read(pixel) as i32;
      let after = before + POKE_PX;
      smb.ram_write(page, (after / 256) as u8);
      smb.ram_write(pixel, (after % 256) as u8);
      pokes.push(json!({"frame": frame, "address": pixel, "before": before % 256, "after": after % 256, "x_before": before, "x_after": after}));
    }
    smb.step_frame(0);
    w.frame("smb", smb.framebuffer())?;
  }
  Ok(json!({"slot": slot, "pokes": pokes}))
}

/// Two scenes: bricks written into the map but not the screen, and ground
/// erased from the map but not the screen.
fn poke_terrain(ctx: &FilmContext, w: &mut TakeWriter) -> Result<Value, String> {
  let rule = ctx.rule(RULE_CACHE)?;
  let mut scenes = serde_json::Map::new();
  for scene in ["wall", "floor"] {
    let mut smb = smb_at(ctx, Stage::ONE_ONE, 0);
    let mut player = Player::new(rule);
    play_until(&mut smb, &mut player, 400, &mut |s| smb_x(s) >= 400 && on_ground(s))?;
    let column = smb_x(&mut smb) as usize / 16;
    let cells: Vec<(usize, usize, u8)> = match scene {
      "wall" => (8..=10).map(|r| (column + 3, r, 0x51)).collect(),
      _ => [2, 3].iter().flat_map(|&d| [(column + d, 11, 0), (column + d, 12, 0)]).collect(),
    };
    let mut pokes = Vec::new();
    for &(c, r, v) in &cells {
      let a = block_address(c, r);
      pokes.push(json!({"address": a, "before": smb.ram_read(a), "after": v, "column": c, "row": r}));
      smb.ram_write(a, v);
    }
    let mut xs = Vec::new();
    let mut died = false;
    for _ in 0..180 {
      // Run right, no jumping: the point is what the numbers do to him.
      smb.step_frame(BUTTON_RIGHT);
      w.frame(scene, smb.framebuffer())?;
      xs.push(smb_x(&mut smb));
      died |= smb_dead(&mut smb);
    }
    scenes.insert(scene.into(), json!({"pokes": pokes, "x": xs, "died": died, "start_column": column}));
  }
  Ok(Value::Object(scenes))
}

/// Random bytes sprayed into memory while the rule plays, four seeds.
fn noise_bytes(ctx: &FilmContext, w: &mut TakeWriter) -> Result<Value, String> {
  let rule = ctx.rule(RULE_CACHE)?;
  let mut seeds = serde_json::Map::new();
  for seed in 1..=4u64 {
    let stream = format!("seed{}", seed);
    let mut smb = smb_at(ctx, Stage::ONE_ONE, 0);
    let mut player = Player::new(rule);
    play_until(&mut smb, &mut player, 400, &mut |s| smb_x(s) >= 300)?;
    let mut rng = Lcg::new(seed);
    let mut pokes = Vec::new();
    let mut buttons = 0;
    for frame in 0..240 {
      if frame % MACRO_FRAMES == 0 {
        buttons = player.buttons(&smb_window(&mut smb));
      }
      if frame % 10 == 0 {
        for _ in 0..3 {
          let address = noise_address(&mut rng);
          let value = (rng.next_f64() * 256.0) as u8;
          smb.ram_write(address, value);
          pokes.push(json!({"frame": frame, "address": address, "value": value}));
        }
      }
      smb.step_frame(buttons);
      w.frame(&stream, smb.framebuffer())?;
    }
    seeds.insert(stream, json!({"pokes": pokes}));
  }
  Ok(Value::Object(seeds))
}

/// A random address in the zero page or $0200-$07FF, never the stack page
/// and never $0770-$077F, which switch the game's modes.
fn noise_address(rng: &mut Lcg) -> u16 {
  loop {
    let a = (rng.next_f64() * 0x800 as f64) as u16;
    if !(0x100..0x200).contains(&a) && !(0x770..0x780).contains(&a) {
      return a;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_missing_cache_names_its_path_and_the_command_that_makes_it() {
    let ctx = FilmContext {
      smb_rom: vec![],
      tetris_rom: vec![],
      target: PathBuf::from("/nonexistent-target"),
    };
    let err = ctx.demos("tetris-reservoir-demos-rule", 6).unwrap_err();
    assert!(err.contains("/nonexistent-target/tetris-reservoir-demos-rule"), "{}", err);
    assert!(err.contains("--example tetris_reservoir -- --rule"), "{}", err);
  }

  #[test]
  fn a_missing_rom_is_an_error_not_a_panic() {
    let err = FilmContext::load(Path::new("/nope/smb.nes"), Path::new("/nope/tetris.nes"), Path::new("target"))
      .err()
      .unwrap();
    assert!(err.contains("/nope/smb.nes"), "{}", err);
  }

  #[test]
  fn the_writer_counts_frames_per_stream_and_finishes_atomically() {
    let dir = std::env::temp_dir().join(format!("film-writer-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut w = TakeWriter::new(&dir).unwrap();
    let frame = vec![0u8; FRAME_BYTES];
    w.frame("smb", &frame).unwrap();
    w.frame("smb", &frame).unwrap();
    w.frame("tetris", &frame).unwrap();
    assert!(w.frame("smb", &[0u8; 3]).is_err(), "wrong frame size");
    w.finish(serde_json::json!({"hello": 1})).unwrap();
    let meta: serde_json::Value = serde_json::from_slice(&std::fs::read(dir.join("take.json")).unwrap()).unwrap();
    assert_eq!(meta["streams"]["smb"], 2);
    assert_eq!(meta["streams"]["tetris"], 1);
    assert_eq!(meta["hello"], 1);
    assert_eq!(std::fs::metadata(dir.join("smb.rgb")).unwrap().len(), 2 * FRAME_BYTES as u64);
    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn block_address_follows_the_page_parity() {
    assert_eq!(block_address(0, 0), 0x0500);
    assert_eq!(block_address(17, 11), 0x05D0 + 11 * 16 + 1);
    assert_eq!(block_address(32 + 3, 2), 0x0500 + 2 * 16 + 3);
  }

  #[test]
  fn noise_addresses_avoid_the_stack_and_the_mode_bytes() {
    let mut rng = Lcg::new(9);
    for _ in 0..5000 {
      let a = noise_address(&mut rng);
      assert!(a < 0x800 && !(0x100..0x200).contains(&a) && !(0x770..0x780).contains(&a), "{:04X}", a);
    }
  }

  fn read_meta(dir: &Path, name: &str) -> serde_json::Value {
    serde_json::from_slice(&std::fs::read(dir.join(name).join("take.json")).unwrap()).unwrap()
  }

  #[test]
  #[ignore = "needs roms2/super_mario_bros.nes and roms2/tetris.nes"]
  fn the_world_takes_record() {
    let ctx = FilmContext::load(
      Path::new("roms2/super_mario_bros.nes"),
      Path::new("roms2/tetris.nes"),
      Path::new("target"),
    )
    .unwrap();
    let out = std::env::temp_dir().join(format!("film-world-{}", std::process::id()));
    for name in ["lives_glyphs", "matrix_walk", "poke_enemy", "poke_terrain", "noise_bytes"] {
      record(&ctx, name, &out).unwrap();
      let meta = read_meta(&out, name);
      assert!(meta["streams"].as_object().unwrap().values().all(|n| n.as_u64().unwrap() > 0), "{}", name);
    }
    assert_eq!(read_meta(&out, "lives_glyphs")["streams"]["screen"], 256);
    let _ = std::fs::remove_dir_all(&out);
  }
}
