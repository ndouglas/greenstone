# The Prospector Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Load every offset of SMB's PRG ROM as 1-1's level layout, filter out what can't be played, beam-search the rest, and publish a ranked catalog of the narrowest clearable accidental levels with clips and a contact sheet.

**Architecture:** A new `src/nes/gym/prospector/` module patches our copy of the ROM so 1-1's layout pointer names offset X, then runs a funnel (load, fingerprint, walk, solve) whose verdicts are appended to `target/prospector/catalog.jsonl`. `reservoir/demo.rs`'s beam search gains a caller-supplied goal, per-round stats and panic isolation. An example CLI drives the scan; Python under `tools/prospector/` turns takes into clips and the catalog into a static HTML page.

**Tech Stack:** Rust (std threads, serde/serde_json, already in the tree), Python 3 with Pillow and numpy, ffmpeg.

**Spec:** `docs/superpowers/specs/2026-10-20-prospector-design.md`

## Global Constraints

- Never stage `src/main.rs` or `src/nes/bus/mod.rs` (they carry unrelated local edits). Leave `.claude/` and `AGENTS.md` alone.
- Format Rust with `rustfmt --edition 2021 <files>`; never `cargo fmt`.
- Run Rust tests with `RUSTFLAGS=-Awarnings`, release mode: `RUSTFLAGS=-Awarnings cargo test --release --lib <filter>`.
- Commit with `git commit -F <file>`; every message ends with a blank line then `Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e`.
- ROM: `roms2/super_mario_bros.nes`. ROM-dependent tests use `crate::nes::gym::reservoir::rom_or_skip` and return early when it is missing.
- Python: standard library, Pillow, numpy only. Tests are `unittest`, run from `tools/` with `python3 -m unittest discover -s prospector/tests -t .`.
- Values from the spec: `GOAL_X` = 3072; solve beam 16, at most 150 rounds; K = 50 for the first solve run; layout entry `$9D34`/`$9D56`; enemy entry `$9CEF`/`$9D11` (= `$9CE4+11`/`$9D06+11`); 1-1 layout `$A68E`, 1-1 enemies `$9F01`; 4-1 is `Stage { world: 3, level: 0 }`.
- The emulator runs about 300 frames/s per core (measured with `examples/headless_bench.rs`); budgets in this plan assume that.

## Review Focus

1. A scan killed mid-write leaves a truncated last line in `catalog.jsonl`: reloading must skip it and resume, not fail. (Task 5, `load_catalog_skips_a_truncated_line`.)
2. A different SMB revision or a non-SMB ROM: the CLI must refuse with a message, not scan garbage. (Task 2, `check_rom_rejects_a_rom_without_1_1_where_expected`.)
3. Offsets near the top of the address space (`$FFFF`): the patch must round-trip every u16. (Task 2, `patch_area_round_trips_the_top_offset`.)
4. A panic inside the beam search (garbage levels panic the emulator): one branch dies, the search finishes. (Task 1, `a_panicking_goal_kills_branches_not_the_search`.)
5. `clips --top N` with fewer than N cleared levels writes what exists and succeeds. (Task 6, `rows_are_empty_without_cleared_levels` plus the CLI looping over `rows.iter().take(n)`.)

## Rulings made while planning

- The spec's `patch_rom` both redirects enemies and must round-trip unchanged at `$A68E`, which can't both hold. Split: `patch_area` (pure layout pointer, round-trips) and `patch_rom = without_enemies(patch_area(..))`.
- The spec's single `scan` subcommand is split into `scan` (load, fingerprint, walk on every offset) and `solve --k N` (beam search the top K), so the cheap pass and the expensive pass can run and resume separately.
- No run of `$FF` bytes exists in this PRG, so the spec's "pointer into `$FF` is Blank/InstantDeath" test becomes a pure test of `classify_load` plus a ROM test that real 1-1 is `Alive`.
- The emulator panics on JAM/unknown opcodes rather than hanging, so `Jammed` means "the game is not running a level": engine state != 1 or the frame counter ($09) frozen.
- Thumbnails are written by fingerprint (`thumbs/<fingerprint>.ppm`), once per distinct level, to keep disk use near distinct-levels × 184 KB.
- Solve budget: at 300 frames/s a 150-round beam-16 search is roughly 10 minutes per level, so K = 50 is about 8 hours, not the spec's evening. It resumes, so it can run overnight.

---

### Task 1: Goal-driven beam search with round stats and panic isolation

**Files:**
- Modify: `src/nes/gym/reservoir/demo.rs` (the `beam_search` function at ~line 359 and its tests)

**Interfaces:**
- Produces (all `pub` in `crate::nes::gym::reservoir::demo`):
  - `enum Clear { Flag, Distance }` (serde, lowercase)
  - `struct RoundStats { expanded: usize, alive: usize, distinct: usize, kept: usize }`
  - `fn round_stats(expanded: &[Candidate], kept: usize) -> RoundStats`
  - `fn guarded<T>(f: impl FnOnce() -> T) -> Result<T, String>`
  - `struct SearchOutcome { demo: Demonstration, clear: Option<Clear>, panics: usize, expansions: usize }`
  - `type Goal<'a> = &'a (dyn Fn(&mut NesEmulator) -> Option<Clear> + Sync);`
  - `fn beam_search_until(rom: &[u8], start: &[u8], noops: usize, beam: usize, max_rounds: usize, seed: u64, stage: Stage, goal: Goal, on_round: &mut dyn FnMut(RoundStats)) -> SearchOutcome`
  - `beam_search` keeps its signature and behaviour.

- [ ] **Step 1: Write the failing tests** (in `demo.rs`'s `mod tests`, which already has `cand(x, dead)`)

```rust
  #[test]
  fn round_stats_counts_alive_distinct_and_kept() {
    let a = cand(10, false);
    let mut twin = cand(11, false);
    twin.key = a.key;
    let dead = cand(12, true);
    let mut stuck = cand(13, false);
    stuck.stuck = STUCK_LIMIT;
    let stats = round_stats(&[a, twin, dead, stuck], 1);
    assert_eq!(stats, RoundStats { expanded: 4, alive: 2, distinct: 1, kept: 1 });
  }

  #[test]
  fn guarded_returns_the_panic_message() {
    assert_eq!(guarded(|| 7), Ok(7));
    assert_eq!(guarded(|| -> u8 { panic!("boom {}", 1) }), Err("boom 1".to_string()));
    assert_eq!(guarded(|| -> u8 { panic!("static") }), Err("static".to_string()));
  }

  #[test]
  fn the_wrapper_matches_beam_search_until_with_the_stage_goal() {
    let rom = match rom_or_skip("roms2/super_mario_bros.nes") {
      Some(r) => r,
      None => return,
    };
    let start = smb_start_state(&rom);
    let plain = beam_search(&rom, &start, 0, 4, 8, 0, Stage::ONE_ONE);
    let goal = |emu: &mut NesEmulator| if smb_cleared_at(emu, Stage::ONE_ONE) { Some(Clear::Flag) } else { None };
    let mut rounds = Vec::new();
    let until = beam_search_until(&rom, &start, 0, 4, 8, 0, Stage::ONE_ONE, &goal, &mut |s| rounds.push(s));
    assert_eq!(plain.buttons, until.demo.buttons);
    assert_eq!(plain.x_reached, until.demo.x_reached);
    assert_eq!(until.clear, None);
    assert_eq!(until.panics, 0);
    assert_eq!(rounds.len(), 8);
    for s in &rounds {
      assert!(s.kept <= s.distinct && s.distinct <= s.alive && s.alive <= s.expanded, "{:?}", s);
    }
    assert_eq!(until.expansions, rounds.iter().map(|s| s.expanded).sum::<usize>());
  }

  #[test]
  fn a_panicking_goal_kills_branches_not_the_search() {
    let rom = match rom_or_skip("roms2/super_mario_bros.nes") {
      Some(r) => r,
      None => return,
    };
    let start = smb_start_state(&rom);
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let goal = |emu: &mut NesEmulator| -> Option<Clear> {
      if smb_x(emu) > 60 {
        panic!("goal exploded");
      }
      None
    };
    let out = beam_search_until(&rom, &start, 0, 4, 6, 0, Stage::ONE_ONE, &goal, &mut |_| {});
    std::panic::set_hook(prev);
    assert!(out.panics > 0, "the goal should have panicked on some branch");
    assert!(out.demo.x_reached <= 60 + 16, "every branch past x 60 died, got {}", out.demo.x_reached);
  }
```

- [ ] **Step 2: Run them to verify they fail**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib reservoir::demo 2>&1 | tail -20`
Expected: compile errors: `round_stats`, `RoundStats`, `guarded`, `beam_search_until`, `Clear` not found.

- [ ] **Step 3: Implement**

Add near the top of `demo.rs`, after `MACROS`:

```rust
/// How a goal says a candidate has finished: SMB itself moved on, or Mario
/// is far enough along alive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Clear {
  Flag,
  Distance,
}

/// A search's notion of done. Shared across worker threads.
pub type Goal<'a> = &'a (dyn Fn(&mut NesEmulator) -> Option<Clear> + Sync);

/// One round of the beam: children made, children neither dead nor stuck,
/// distinct states among those, and survivors kept for the next round.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RoundStats {
  pub expanded: usize,
  pub alive: usize,
  pub distinct: usize,
  pub kept: usize,
}

pub fn round_stats(expanded: &[Candidate], kept: usize) -> RoundStats {
  let alive: Vec<&Candidate> = expanded.iter().filter(|c| !c.dead && c.stuck < STUCK_LIMIT).collect();
  let distinct = alive.iter().map(|c| c.key).collect::<std::collections::HashSet<_>>().len();
  RoundStats {
    expanded: expanded.len(),
    alive: alive.len(),
    distinct,
    kept,
  }
}

/// Run `f`, turning a panic into its message.
pub fn guarded<T>(f: impl FnOnce() -> T) -> Result<T, String> {
  std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).map_err(|e| {
    e.downcast_ref::<String>()
      .cloned()
      .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
      .unwrap_or_else(|| "panic".to_string())
  })
}

/// What `beam_search_until` found, and how often the emulator panicked.
#[derive(Debug, Clone)]
pub struct SearchOutcome {
  pub demo: Demonstration,
  pub clear: Option<Clear>,
  pub panics: usize,
  pub expansions: usize,
}
```

Replace `beam_search` with the wrapper plus `beam_search_until` and an `expand` helper (the expansion body moves into `expand` unchanged except that `cleared` comes from `goal`):

```rust
pub fn beam_search(rom: &[u8], start: &[u8], noops: usize, beam: usize, max_rounds: usize, seed: u64, stage: Stage) -> Demonstration {
  let goal = move |emu: &mut NesEmulator| if smb_cleared_at(emu, stage) { Some(Clear::Flag) } else { None };
  beam_search_until(rom, start, noops, beam, max_rounds, seed, stage, &goal, &mut |_| {}).demo
}

fn expand(emu: &mut NesEmulator, parent: &Candidate, macro_index: usize, lives_at_start: u8, goal: Goal) -> Candidate {
  emu.load_state(&parent.state);
  let mut path = parent.path.clone();
  let mut dead = false;
  for &buttons in &MACROS[macro_index] {
    emu.step_frames(buttons, MACRO_FRAMES);
    path.push(buttons);
    if smb_dead(emu) || emu.ram_read(smb1_ram::LIVES) < lives_at_start {
      dead = true;
      break;
    }
  }
  let x = smb_x(emu);
  let stuck = next_stuck(parent.stuck, x > parent.x, smb_in_cutscene(emu));
  let cleared = goal(emu).is_some();
  let y = smb_y(emu);
  let key = state_key(emu);
  let state = emu.save_state();
  let dead = dead || (!cleared && doomed_next_round(emu, &state, lives_at_start));
  Candidate {
    x,
    y,
    dead,
    cleared,
    state,
    path,
    stuck,
    key,
  }
}

/// Beam search over save states until `goal` says done. Each round expands
/// every frontier candidate by every macro across threads, one emulator per
/// thread, and reports its `RoundStats` to `on_round`. An expansion that
/// panics is a dead branch, and its thread gets a fresh emulator.
#[allow(clippy::too_many_arguments)]
pub fn beam_search_until(
  rom: &[u8],
  start: &[u8],
  noops: usize,
  beam: usize,
  max_rounds: usize,
  seed: u64,
  stage: Stage,
  goal: Goal,
  on_round: &mut dyn FnMut(RoundStats),
) -> SearchOutcome {
  let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
  let (first, lives_at_start, key) = {
    let mut emu = NesEmulator::new(rom);
    emu.load_state(start);
    emu.step_frames(0, noops);
    (emu.save_state(), emu.ram_read(smb1_ram::LIVES), state_key(&mut emu))
  };
  let mut frontier = vec![Candidate {
    state: first,
    path: Vec::new(),
    x: 0,
    y: 0,
    dead: false,
    cleared: false,
    stuck: 0,
    key,
  }];
  let mut best = frontier[0].clone();
  let mut max_x = 0;
  let panics = std::sync::atomic::AtomicUsize::new(0);
  let mut expansions = 0;
  for _ in 0..max_rounds {
    let jobs: Vec<(usize, usize)> = (0..frontier.len()).flat_map(|p| (0..MACROS.len()).map(move |m| (p, m))).collect();
    let chunk = jobs.len().div_ceil(threads);
    let expanded: Vec<Candidate> = std::thread::scope(|s| {
      let handles: Vec<_> = jobs
        .chunks(chunk)
        .map(|jobs| {
          let frontier = &frontier;
          let panics = &panics;
          s.spawn(move || {
            let mut emu = NesEmulator::new(rom);
            let mut out = Vec::with_capacity(jobs.len());
            for &(parent, macro_index) in jobs {
              let p = &frontier[parent];
              match guarded(|| expand(&mut emu, p, macro_index, lives_at_start, goal)) {
                Ok(c) => out.push(c),
                Err(_) => {
                  panics.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                  emu = NesEmulator::new(rom);
                  out.push(Candidate { dead: true, ..p.clone() });
                }
              }
            }
            out
          })
        })
        .collect();
      handles.into_iter().flat_map(|h| h.join().expect("beam worker panicked")).collect()
    });
    expansions += expanded.len();
    let mut stats = round_stats(&expanded, 0);
    let survivors = prune_and_rank(expanded, beam, seed);
    stats.kept = survivors.len();
    on_round(stats);
    if survivors.is_empty() {
      break;
    }
    if survivors[0].cleared || survivors[0].x > best.x {
      best = survivors[0].clone();
    }
    max_x = max_x.max(survivors[0].x);
    if best.cleared {
      break;
    }
    frontier = survivors;
  }
  let clear = if best.cleared {
    let mut emu = NesEmulator::new(rom);
    emu.load_state(&best.state);
    guarded(|| goal(&mut emu)).ok().flatten()
  } else {
    None
  };
  SearchOutcome {
    demo: Demonstration {
      buttons: best.path,
      x_reached: max_x,
      cleared: best.cleared,
      noops,
      stage,
    },
    clear,
    panics: panics.into_inner(),
    expansions,
  }
}
```

If `rom_or_skip` is not already imported in `demo.rs`'s tests, add `use crate::nes::gym::reservoir::rom_or_skip;` there (it is used by existing tests, so it should be).

- [ ] **Step 4: Format and run the tests**

Run: `rustfmt --edition 2021 src/nes/gym/reservoir/demo.rs && RUSTFLAGS=-Awarnings cargo test --release --lib reservoir::demo 2>&1 | tail -20`
Expected: all `reservoir::demo` tests pass (the ignored slow ones stay ignored), including the four new ones.

- [ ] **Step 5: Run the whole library suite**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib > /private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/t1.log 2>&1; tail -5 /private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/t1.log`
Expected: `test result: ok.` with 0 failed.

- [ ] **Step 6: Commit**

```bash
git add src/nes/gym/reservoir/demo.rs
# message file: "Let the beam search take a goal, report each round, and survive a panicking branch."
git commit -F <msgfile>
```

---

### Task 2: The ROM patch

**Files:**
- Create: `src/nes/gym/prospector/mod.rs`, `src/nes/gym/prospector/patch.rs`
- Modify: `src/nes/gym/mod.rs` (add `pub mod prospector;` after `pub mod preprocess;`)

**Interfaces:**
- Produces (`crate::nes::gym::prospector::patch`):
  - `const ONE_ONE_AREA: u16 = 0xA68E; const ONE_ONE_ENEMIES: u16 = 0x9F01;`
  - `fn area_pointer(rom: &[u8]) -> u16`, `fn enemy_pointer(rom: &[u8]) -> u16`
  - `fn empty_enemy_list(rom: &[u8]) -> u16` (address of the first `$FF` in PRG)
  - `fn patch_area(rom: &[u8], area: u16) -> Vec<u8>`
  - `fn without_enemies(rom: &[u8]) -> Vec<u8>`
  - `fn patch_rom(rom: &[u8], area: u16) -> Vec<u8>` = `without_enemies(&patch_area(rom, area))`
  - `fn check_rom(rom: &[u8]) -> Result<(), String>`

- [ ] **Step 1: Write the module skeleton and failing tests**

`src/nes/gym/prospector/mod.rs`:

```rust
//! The Prospector: every offset of SMB's PRG ROM, loaded as 1-1's layout.
//! See docs/superpowers/specs/2026-10-20-prospector-design.md.

pub mod patch;
```

`src/nes/gym/prospector/patch.rs`, tests first (implementation stubs `todo!()` so it compiles and fails):

```rust
//! Pointing 1-1's level-pointer table entries somewhere else in our copy of the ROM.

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::reservoir::rom_or_skip;

  fn rom() -> Option<Vec<u8>> {
    rom_or_skip("roms2/super_mario_bros.nes")
  }

  #[test]
  fn the_real_rom_points_1_1_where_smb_keeps_it() {
    let Some(rom) = rom() else { return };
    assert_eq!(area_pointer(&rom), ONE_ONE_AREA);
    assert_eq!(enemy_pointer(&rom), ONE_ONE_ENEMIES);
    assert_eq!(check_rom(&rom), Ok(()));
  }

  #[test]
  fn patch_area_at_1_1_changes_nothing() {
    let Some(rom) = rom() else { return };
    assert_eq!(patch_area(&rom, ONE_ONE_AREA), rom);
  }

  #[test]
  fn patch_area_round_trips_the_top_offset() {
    let Some(rom) = rom() else { return };
    for area in [0x8000, 0xC4A0, 0xFFFF] {
      let patched = patch_area(&rom, area);
      assert_eq!(area_pointer(&patched), area);
      assert_eq!(enemy_pointer(&patched), ONE_ONE_ENEMIES);
      let differing = rom.iter().zip(&patched).filter(|(a, b)| a != b).count();
      assert!(differing <= 2, "only the two pointer bytes may change, {} did", differing);
    }
  }

  #[test]
  fn without_enemies_points_at_an_ff_byte() {
    let Some(rom) = rom() else { return };
    let patched = patch_rom(&rom, 0xC4A0);
    let empty = enemy_pointer(&patched);
    assert_eq!(empty, empty_enemy_list(&rom));
    assert_eq!(patched[PRG_START + (empty - 0x8000) as usize], 0xFF);
    assert_eq!(area_pointer(&patched), 0xC4A0);
  }

  #[test]
  fn check_rom_rejects_a_rom_without_1_1_where_expected() {
    let mut fake = vec![0u8; PRG_START + 0x8000];
    fake[..4].copy_from_slice(b"NES\x1a");
    assert!(check_rom(&fake).unwrap_err().contains("1-1"));
    assert!(check_rom(&[0u8; 10]).unwrap_err().contains("too short"));
  }
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib prospector::patch 2>&1 | tail -15`
Expected: compile errors for the missing names.

- [ ] **Step 3: Implement** (above the tests in `patch.rs`)

```rust
/// File offset of PRG ROM (after the iNES header). SMB is NROM-256: 32 KB at $8000.
pub const PRG_START: usize = 16;
const PRG_LEN: usize = 0x8000;

/// 1-1 is ground area 5: layout entry index 3 + 5 = 8 in SMB's tables at
/// $9D2C (low) and $9D4E (high); enemy entry index 6 + 5 = 11 in the tables
/// at $9CE4 (low) and $9D06 (high).
const AREA_LOW: u16 = 0x9D34;
const AREA_HIGH: u16 = 0x9D56;
const ENEMY_LOW: u16 = 0x9CEF;
const ENEMY_HIGH: u16 = 0x9D11;

pub const ONE_ONE_AREA: u16 = 0xA68E;
pub const ONE_ONE_ENEMIES: u16 = 0x9F01;

fn at(addr: u16) -> usize {
  PRG_START + (addr - 0x8000) as usize
}

fn pointer(rom: &[u8], low: u16, high: u16) -> u16 {
  rom[at(low)] as u16 | (rom[at(high)] as u16) << 8
}

fn set_pointer(rom: &mut [u8], low: u16, high: u16, value: u16) {
  rom[at(low)] = value as u8;
  rom[at(high)] = (value >> 8) as u8;
}

pub fn area_pointer(rom: &[u8]) -> u16 {
  pointer(rom, AREA_LOW, AREA_HIGH)
}

pub fn enemy_pointer(rom: &[u8]) -> u16 {
  pointer(rom, ENEMY_LOW, ENEMY_HIGH)
}

/// The first $FF byte in PRG. $FF ends an enemy list, so pointing there
/// gives a level no enemies.
pub fn empty_enemy_list(rom: &[u8]) -> u16 {
  let i = rom[PRG_START..PRG_START + PRG_LEN].iter().position(|&b| b == 0xFF).expect("SMB's PRG has an $FF byte");
  0x8000 + i as u16
}

pub fn patch_area(rom: &[u8], area: u16) -> Vec<u8> {
  let mut out = rom.to_vec();
  set_pointer(&mut out, AREA_LOW, AREA_HIGH, area);
  out
}

pub fn without_enemies(rom: &[u8]) -> Vec<u8> {
  let mut out = rom.to_vec();
  set_pointer(&mut out, ENEMY_LOW, ENEMY_HIGH, empty_enemy_list(rom));
  out
}

/// 1-1's layout at `area`, with no enemies.
pub fn patch_rom(rom: &[u8], area: u16) -> Vec<u8> {
  without_enemies(&patch_area(rom, area))
}

/// The Prospector only knows this SMB's table addresses.
pub fn check_rom(rom: &[u8]) -> Result<(), String> {
  if rom.len() < PRG_START + PRG_LEN {
    return Err(format!("ROM is too short ({} bytes) to be SMB", rom.len()));
  }
  if area_pointer(rom) != ONE_ONE_AREA || enemy_pointer(rom) != ONE_ONE_ENEMIES {
    return Err(format!(
      "this ROM does not keep 1-1 at ${:04X}/${:04X} (found ${:04X}/${:04X}); the Prospector needs the original Super Mario Bros.",
      ONE_ONE_AREA,
      ONE_ONE_ENEMIES,
      area_pointer(rom),
      enemy_pointer(rom)
    ));
  }
  Ok(())
}
```

- [ ] **Step 4: Format and run**

Run: `rustfmt --edition 2021 src/nes/gym/prospector/*.rs src/nes/gym/mod.rs && RUSTFLAGS=-Awarnings cargo test --release --lib prospector::patch 2>&1 | tail -10`
Expected: 5 passed.

- [ ] **Step 5: Commit**

```bash
git add src/nes/gym/mod.rs src/nes/gym/prospector/
# message: "Add the Prospector's ROM patch: 1-1's layout pointer, anywhere, with no enemies."
git commit -F <msgfile>
```

---

### Task 3: The funnel's cheap stages: load, fingerprint, walk

**Files:**
- Create: `src/nes/gym/prospector/funnel.rs`
- Modify: `src/nes/gym/prospector/mod.rs` (add `pub mod funnel;`)

**Interfaces:**
- Consumes: `patch::patch_rom`, `demo::{smb_start_state_at, smb_x, smb_dead, Stage, MACRO_FRAMES, guarded}`, `features::{SMB_BLOCK_BUFFER_1, SMB_BLOCK_BUFFER_2, SMB_BLOCK_BUFFER_LEN}`, `reward::smb1_ram::{LIVES, GAME_ENGINE_STATE}`.
- Produces (`crate::nes::gym::prospector::funnel`):
  - `enum LoadClass { Jammed, Blank, InstantDeath, Alive }`
  - `fn classify_load(engine_state: u8, ticking: bool, blank: bool, died: bool) -> LoadClass`
  - `struct Loaded { class: LoadClass, state: Vec<u8>, thumb: Vec<u8> }`
  - `fn load(rom: &[u8]) -> Loaded` (rom is already patched; may panic, callers wrap in `guarded`)
  - `fn fingerprint(rom: &[u8], state: &[u8]) -> u64`
  - `struct Walk { x: i32, moved: i32, died: bool }`
  - `fn walk(rom: &[u8], state: &[u8], seed: u64) -> Walk`
  - `const MOVED_ENOUGH: i32 = 64;`

- [ ] **Step 1: Write the failing tests** (in `funnel.rs`)

```rust
#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::prospector::patch::{patch_area, patch_rom, ONE_ONE_AREA};
  use crate::nes::gym::reservoir::demo::smb_start_state_at;
  use crate::nes::gym::reservoir::rom_or_skip;

  #[test]
  fn classify_load_checks_running_then_blank_then_death() {
    assert_eq!(classify_load(0, true, false, false), LoadClass::Jammed);
    assert_eq!(classify_load(1, false, false, false), LoadClass::Jammed);
    assert_eq!(classify_load(1, true, true, true), LoadClass::Blank);
    assert_eq!(classify_load(1, true, false, true), LoadClass::InstantDeath);
    assert_eq!(classify_load(1, true, false, false), LoadClass::Alive);
  }

  #[test]
  fn one_colour_spots_a_flat_frame() {
    assert!(one_colour(&[1, 2, 3, 1, 2, 3]));
    assert!(!one_colour(&[1, 2, 3, 1, 2, 4]));
  }

  #[test]
  fn real_1_1_without_enemies_loads_alive() {
    let Some(rom) = rom_or_skip("roms2/super_mario_bros.nes") else { return };
    let loaded = load(&patch_rom(&rom, ONE_ONE_AREA));
    assert_eq!(loaded.class, LoadClass::Alive);
    assert_eq!(loaded.thumb.len(), 256 * 240 * 3);
  }

  #[test]
  fn the_patch_removes_1_1s_goombas() {
    let Some(rom) = rom_or_skip("roms2/super_mario_bros.nes") else { return };
    let enemies_after_a_walk = |rom: &[u8]| {
      let mut emu = NesEmulator::new(rom);
      emu.load_state(&smb_start_state_at(rom, Stage::ONE_ONE));
      emu.step_frames(RUN, 200);
      (0..5).filter(|&i| emu.ram_read(ENEMY_FLAGS + i) != 0).count()
    };
    assert!(enemies_after_a_walk(&patch_area(&rom, ONE_ONE_AREA)) > 0, "control: real 1-1 has a Goomba on screen");
    assert_eq!(enemies_after_a_walk(&patch_rom(&rom, ONE_ONE_AREA)), 0);
  }

  #[test]
  fn fingerprint_is_stable_and_tells_levels_apart() {
    let Some(rom) = rom_or_skip("roms2/super_mario_bros.nes") else { return };
    let a = smb_start_state_at(&rom, Stage::ONE_ONE);
    let b = smb_start_state_at(&rom, Stage::ONE_ONE);
    let c = smb_start_state_at(&rom, Stage { world: 3, level: 0 });
    assert_eq!(fingerprint(&rom, &a), fingerprint(&rom, &b));
    assert_ne!(fingerprint(&rom, &a), fingerprint(&rom, &c));
  }

  #[test]
  fn walk_is_deterministic_and_moves_on_1_1() {
    let Some(rom) = rom_or_skip("roms2/super_mario_bros.nes") else { return };
    let rom = patch_rom(&rom, ONE_ONE_AREA);
    let state = smb_start_state_at(&rom, Stage::ONE_ONE);
    let one = walk(&rom, &state, 7);
    let two = walk(&rom, &state, 7);
    assert_eq!((one.x, one.moved, one.died), (two.x, two.moved, two.died));
    assert!(one.moved >= MOVED_ENOUGH, "walker should get going on 1-1, moved {}", one.moved);
  }
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib prospector::funnel 2>&1 | tail -15`
Expected: compile errors for the missing names.

- [ ] **Step 3: Implement** (above the tests)

```rust
//! The cheap stages: does the level load, which level is it, and can a
//! dumb walker get anywhere.

use crate::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_RIGHT};
use crate::nes::gym::reservoir::demo::{smb_dead, smb_start_state_at, smb_x, Stage, MACRO_FRAMES};
use crate::nes::gym::reservoir::features::{SMB_BLOCK_BUFFER_1, SMB_BLOCK_BUFFER_2, SMB_BLOCK_BUFFER_LEN};
use crate::nes::gym::reward::smb1_ram;
use crate::nes::gym::NesEmulator;

/// Frames of standing still after the level starts.
pub const IDLE_FRAMES: usize = 180;
/// Macro steps the walker takes (4 frames each).
pub const WALK_STEPS: usize = 100;
/// Less progress than this, in pixels, and the walker is stuck.
pub const MOVED_ENOUGH: i32 = 64;
/// SMB RAM: frame counter, ticked by every NMI.
const FRAME_COUNTER: u16 = 0x0009;
/// SMB RAM: the five enemy slots' active flags.
const ENEMY_FLAGS: u16 = 0x000F;

const RUN: u8 = BUTTON_RIGHT | BUTTON_B;
const JUMP: u8 = BUTTON_RIGHT | BUTTON_B | BUTTON_A;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadClass {
  /// The game isn't running a level: wrong engine state, or a frozen frame counter.
  Jammed,
  /// The first frame is one colour.
  Blank,
  /// Mario dies standing still.
  InstantDeath,
  Alive,
}

pub fn classify_load(engine_state: u8, ticking: bool, blank: bool, died: bool) -> LoadClass {
  if engine_state != 1 || !ticking {
    LoadClass::Jammed
  } else if blank {
    LoadClass::Blank
  } else if died {
    LoadClass::InstantDeath
  } else {
    LoadClass::Alive
  }
}

pub fn one_colour(rgb: &[u8]) -> bool {
  rgb.chunks(3).all(|p| p == &rgb[..3])
}

pub struct Loaded {
  pub class: LoadClass,
  /// The state the level starts in; every later stage starts here.
  pub state: Vec<u8>,
  /// The first frame of the level, 256x240 RGB.
  pub thumb: Vec<u8>,
}

/// Boot a patched ROM into 1-1, then stand still for `IDLE_FRAMES`.
pub fn load(rom: &[u8]) -> Loaded {
  let state = smb_start_state_at(rom, Stage::ONE_ONE);
  let mut emu = NesEmulator::new(rom);
  emu.load_state(&state);
  let thumb = emu.framebuffer().to_vec();
  let engine = emu.ram_read(smb1_ram::GAME_ENGINE_STATE);
  let ticks = emu.ram_read(FRAME_COUNTER);
  let lives = emu.ram_read(smb1_ram::LIVES);
  emu.step_frames(0, IDLE_FRAMES);
  let ticking = emu.ram_read(FRAME_COUNTER) != ticks;
  let died = smb_dead(&mut emu) || emu.ram_read(smb1_ram::LIVES) < lives;
  Loaded {
    class: classify_load(engine, ticking, one_colour(&thumb), died),
    state,
    thumb,
  }
}

/// Hash of both terrain block buffers as the level starts: two offsets with
/// the same fingerprint built the same opening screens.
pub fn fingerprint(rom: &[u8], state: &[u8]) -> u64 {
  let mut emu = NesEmulator::new(rom);
  emu.load_state(state);
  let mut h = 0xCBF2_9CE4_8422_2325u64;
  for base in [SMB_BLOCK_BUFFER_1, SMB_BLOCK_BUFFER_2] {
    for a in base..base + SMB_BLOCK_BUFFER_LEN {
      h = (h ^ emu.ram_read(a) as u64).wrapping_mul(0x100_0000_01B3);
    }
  }
  h
}

#[derive(Debug, Clone, Copy)]
pub struct Walk {
  /// Furthest x reached.
  pub x: i32,
  /// Furthest x minus the starting x.
  pub moved: i32,
  pub died: bool,
}

/// Run right holding B, starting a jump at random: hold A for five steps,
/// release for one. Seeded, so the same seed walks the same way.
pub fn walk(rom: &[u8], state: &[u8], seed: u64) -> Walk {
  let mut emu = NesEmulator::new(rom);
  emu.load_state(state);
  let start = smb_x(&mut emu);
  let lives = emu.ram_read(smb1_ram::LIVES);
  let mut rng = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
  let mut jump = 0;
  let mut best = start;
  for _ in 0..WALK_STEPS {
    rng ^= rng << 13;
    rng ^= rng >> 7;
    rng ^= rng << 17;
    if jump == 0 && rng % 4 == 0 {
      jump = 6;
    }
    let buttons = if jump > 1 { JUMP } else { RUN };
    jump = jump.saturating_sub(1);
    emu.step_frames(buttons, MACRO_FRAMES);
    best = best.max(smb_x(&mut emu));
    if smb_dead(&mut emu) || emu.ram_read(smb1_ram::LIVES) < lives {
      return Walk { x: best, moved: best - start, died: true };
    }
  }
  Walk { x: best, moved: best - start, died: false }
}
```

Make `smb_start_state_at`'s import in the tests module match; if `Stage` is not re-imported by `use super::*`, it is (it is imported at the top).

- [ ] **Step 4: Format and run**

Run: `rustfmt --edition 2021 src/nes/gym/prospector/*.rs && RUSTFLAGS=-Awarnings cargo test --release --lib prospector 2>&1 | tail -12`
Expected: all prospector tests pass (11).

- [ ] **Step 5: Commit**

```bash
git add src/nes/gym/prospector/
# message: "Add the Prospector's cheap stages: load, fingerprint, walk."
git commit -F <msgfile>
```

---

### Task 4: Solve and narrowness

**Files:**
- Create: `src/nes/gym/prospector/rank.rs`
- Modify: `src/nes/gym/prospector/funnel.rs` (add `solve`), `src/nes/gym/prospector/mod.rs` (add `pub mod rank;`)

**Interfaces:**
- Consumes: `demo::{beam_search_until, Clear, RoundStats, smb_stage, smb_x, smb_dead, Stage}`.
- Produces:
  - `rank::Narrowness { bottleneck: usize, kill_rate: f64, rounds: usize }`
  - `rank::narrowness(stats: &[RoundStats]) -> Narrowness`
  - `rank::worse(a: &Narrowness, b: &Narrowness) -> std::cmp::Ordering` (`Less` means `a` ranks first, i.e. is worse)
  - `funnel::{GOAL_X: i32 = 3072, SOLVE_BEAM: usize = 16, SOLVE_ROUNDS: usize = 150}`
  - `funnel::goal_for(stage: Stage) -> impl Fn(&mut NesEmulator) -> Option<Clear> + Sync`
  - `funnel::Solved { clear: Option<Clear>, x_reached: i32, panics: usize, expansions: usize, path: Vec<u8>, stats: Vec<RoundStats> }`
  - `funnel::solve(rom: &[u8], state: &[u8], stage: Stage) -> Solved`

- [ ] **Step 1: Write the failing tests** (in `rank.rs`)

```rust
#[cfg(test)]
mod tests {
  use super::*;

  fn r(expanded: usize, alive: usize, distinct: usize) -> RoundStats {
    RoundStats { expanded, alive, distinct, kept: distinct.min(16) }
  }

  #[test]
  fn narrowness_takes_the_tightest_round_and_the_overall_kill_rate() {
    let n = narrowness(&[r(6, 6, 6), r(36, 9, 2), r(12, 3, 3)]);
    assert_eq!(n.bottleneck, 2);
    assert!((n.kill_rate - 36.0 / 54.0).abs() < 1e-9);
    assert_eq!(n.rounds, 3);
  }

  #[test]
  fn narrowness_of_no_rounds_is_not_narrow() {
    assert_eq!(narrowness(&[]).bottleneck, usize::MAX);
  }

  #[test]
  fn worse_orders_by_bottleneck_then_kill_rate_then_rounds() {
    let n = |b, k, r| Narrowness { bottleneck: b, kill_rate: k, rounds: r };
    use std::cmp::Ordering::*;
    assert_eq!(worse(&n(1, 0.1, 10), &n(2, 0.9, 99)), Less);
    assert_eq!(worse(&n(2, 0.9, 10), &n(2, 0.5, 99)), Less);
    assert_eq!(worse(&n(2, 0.5, 99), &n(2, 0.5, 10)), Less);
    assert_eq!(worse(&n(2, 0.5, 10), &n(2, 0.5, 10)), Equal);
  }
}
```

And in `funnel.rs`'s tests:

```rust
  #[test]
  fn the_goal_needs_a_stage_change_or_distance_alive() {
    let Some(rom) = rom_or_skip("roms2/super_mario_bros.nes") else { return };
    let mut emu = NesEmulator::new(&rom);
    emu.load_state(&smb_start_state_at(&rom, Stage::ONE_ONE));
    let goal = goal_for(Stage::ONE_ONE);
    assert_eq!(goal(&mut emu), None);
    emu.ram_write(smb1_ram::PLAYER_X_PAGE, (GOAL_X / 256) as u8);
    emu.ram_write(smb1_ram::PLAYER_X_PIXEL, 0);
    assert_eq!(goal(&mut emu), Some(Clear::Distance));
    emu.ram_write(smb1_ram::LEVEL, 1);
    assert_eq!(goal(&mut emu), Some(Clear::Flag));
  }
```

- [ ] **Step 2: Run them to verify they fail**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib prospector 2>&1 | tail -15`
Expected: compile errors for `narrowness`, `Narrowness`, `worse`, `goal_for`, `GOAL_X`.

- [ ] **Step 3: Implement**

`rank.rs` (above the tests):

```rust
//! How narrowly a level was cleared, and the order that puts the worst first.

use crate::nes::gym::reservoir::demo::RoundStats;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Narrowness {
  /// Fewest distinct live states in any round. 1 means one way through.
  pub bottleneck: usize,
  /// Share of all children the search made that died or got stuck.
  pub kill_rate: f64,
  pub rounds: usize,
}

pub fn narrowness(stats: &[RoundStats]) -> Narrowness {
  let expanded: usize = stats.iter().map(|s| s.expanded).sum();
  let alive: usize = stats.iter().map(|s| s.alive).sum();
  Narrowness {
    bottleneck: stats.iter().map(|s| s.distinct).min().unwrap_or(usize::MAX),
    kill_rate: if expanded == 0 { 0.0 } else { (expanded - alive) as f64 / expanded as f64 },
    rounds: stats.len(),
  }
}

/// `Less` when `a` is the worse (narrower) level and should rank first.
pub fn worse(a: &Narrowness, b: &Narrowness) -> Ordering {
  a.bottleneck
    .cmp(&b.bottleneck)
    .then(b.kill_rate.partial_cmp(&a.kill_rate).unwrap_or(Ordering::Equal))
    .then(b.rounds.cmp(&a.rounds))
}
```

`funnel.rs` additions (imports: add `beam_search_until, smb_stage, Clear, RoundStats` from `demo`):

```rust
/// Alive at this x counts as cleared: about the length of 1-1.
pub const GOAL_X: i32 = 3072;
pub const SOLVE_BEAM: usize = 16;
pub const SOLVE_ROUNDS: usize = 150;

/// Cleared when SMB leaves `stage`, or when Mario is alive at `GOAL_X`.
pub fn goal_for(stage: Stage) -> impl Fn(&mut NesEmulator) -> Option<Clear> + Sync {
  move |emu: &mut NesEmulator| {
    if smb_stage(emu) != stage {
      Some(Clear::Flag)
    } else if smb_x(emu) >= GOAL_X && !smb_dead(emu) {
      Some(Clear::Distance)
    } else {
      None
    }
  }
}

#[derive(Debug, Clone)]
pub struct Solved {
  pub clear: Option<Clear>,
  pub x_reached: i32,
  pub panics: usize,
  pub expansions: usize,
  /// One button byte per macro step, from `state`.
  pub path: Vec<u8>,
  pub stats: Vec<RoundStats>,
}

pub fn solve(rom: &[u8], state: &[u8], stage: Stage) -> Solved {
  let goal = goal_for(stage);
  let mut stats = Vec::new();
  let out = beam_search_until(rom, state, 0, SOLVE_BEAM, SOLVE_ROUNDS, 0, stage, &goal, &mut |s| stats.push(s));
  Solved {
    clear: out.clear,
    x_reached: out.demo.x_reached,
    panics: out.panics,
    expansions: out.expansions,
    path: out.demo.buttons,
    stats,
  }
}
```

`Stage` must derive `PartialEq` already (it is compared in `smb_cleared_at`).

- [ ] **Step 4: Format and run**

Run: `rustfmt --edition 2021 src/nes/gym/prospector/*.rs && RUSTFLAGS=-Awarnings cargo test --release --lib prospector 2>&1 | tail -12`
Expected: all prospector tests pass (15).

- [ ] **Step 5: Commit**

```bash
git add src/nes/gym/prospector/
# message: "Add the Prospector's solver and its narrowness score."
git commit -F <msgfile>
```

---

### Task 5: The catalog and the scan

**Files:**
- Create: `src/nes/gym/prospector/catalog.rs`, `src/nes/gym/prospector/scan.rs`
- Modify: `src/nes/gym/prospector/mod.rs` (add `pub mod catalog; pub mod scan;`)

**Interfaces:**
- Consumes: `patch::patch_rom`, `funnel::{load, fingerprint, walk, solve, LoadClass, MOVED_ENOUGH, Solved}`, `rank::{narrowness, Narrowness}`, `demo::{guarded, Clear, Stage}`.
- Produces:
  - `catalog::Class` (serde snake_case): `Crashed, Jammed, Blank, InstantDeath, Duplicate, Stuck, WalkerDied, Walked, Unsolved, ClearedDistance, ClearedFlag`
  - `catalog::SolveRecord { clear: Option<Clear>, x_reached: i32, rounds: usize, panics: usize, bottleneck: usize, kill_rate: f64, path: Vec<u8> }` with `fn narrowness(&self) -> Narrowness` and `fn from_solved(s: &Solved) -> SolveRecord`
  - `catalog::Entry { offset: u16, hex: String, class: Class, message: Option<String>, fingerprint: Option<String>, duplicate_of: Option<u16>, walk_x: Option<i32>, solve: Option<SolveRecord> }` with `fn new(offset: u16, class: Class) -> Entry`
  - `catalog::append(path: &Path, e: &Entry) -> Result<(), String>`
  - `catalog::load_catalog(path: &Path) -> Vec<Entry>` (latest line per offset wins; unparseable lines skipped; sorted by offset)
  - `catalog::dedupe(entries: &mut [Entry])`
  - `scan::survey(rom: &[u8], offset: u16, thumbs: &Path) -> Entry`
  - `scan::scan_range(rom: &[u8], from: u16, to: u16, out: &Path) -> Result<usize, String>`
  - `scan::solve_top(rom: &[u8], out: &Path, k: usize) -> Result<usize, String>`
  - `scan::calibrate(rom: &[u8], out: &Path) -> Result<(), String>` (writes `calibration.json`: `{"1-1": SolveRecord, "4-1": SolveRecord}`)
  - `scan::CALIBRATION: [(&str, Stage); 2]`

- [ ] **Step 1: Write the failing tests**

In `catalog.rs`:

```rust
#[cfg(test)]
mod tests {
  use super::*;

  fn tmp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("prospector-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("catalog.jsonl")
  }

  #[test]
  fn entries_round_trip_through_a_line() {
    let mut e = Entry::new(0xC4A0, Class::Walked);
    e.walk_x = Some(812);
    e.fingerprint = Some("00000000deadbeef".into());
    let line = serde_json::to_string(&e).unwrap();
    assert!(line.contains("\"class\":\"walked\""), "{}", line);
    assert!(line.contains("\"hex\":\"$C4A0\""), "{}", line);
    assert_eq!(serde_json::from_str::<Entry>(&line).unwrap(), e);
  }

  #[test]
  fn load_catalog_takes_the_latest_line_per_offset() {
    let path = tmp("latest");
    append(&path, &Entry::new(0x9000, Class::Walked)).unwrap();
    append(&path, &Entry::new(0x8000, Class::Blank)).unwrap();
    append(&path, &Entry::new(0x9000, Class::Unsolved)).unwrap();
    let got = load_catalog(&path);
    assert_eq!(got.iter().map(|e| (e.offset, e.class)).collect::<Vec<_>>(), vec![(0x8000, Class::Blank), (0x9000, Class::Unsolved)]);
  }

  #[test]
  fn load_catalog_skips_a_truncated_line() {
    let path = tmp("truncated");
    append(&path, &Entry::new(0x8000, Class::Blank)).unwrap();
    use std::io::Write;
    std::fs::OpenOptions::new().append(true).open(&path).unwrap().write_all(b"{\"offset\":32769,\"hex\":\"$80").unwrap();
    assert_eq!(load_catalog(&path).len(), 1);
    assert!(load_catalog(&path.with_file_name("missing.jsonl")).is_empty());
  }

  #[test]
  fn dedupe_keeps_the_lowest_offset_per_fingerprint() {
    let mut a = Entry::new(0x8002, Class::Walked);
    a.fingerprint = Some("aa".into());
    let mut b = Entry::new(0x8001, Class::Stuck);
    b.fingerprint = Some("aa".into());
    let c = Entry::new(0x8000, Class::Crashed);
    let mut all = vec![a, b, c];
    all.sort_by_key(|e| e.offset);
    dedupe(&mut all);
    assert_eq!(all[0].class, Class::Crashed);
    assert_eq!(all[1].class, Class::Stuck);
    assert_eq!(all[2].class, Class::Duplicate);
    assert_eq!(all[2].duplicate_of, Some(0x8001));
  }
}
```

In `scan.rs`:

```rust
#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::prospector::patch::ONE_ONE_AREA;
  use crate::nes::gym::reservoir::rom_or_skip;

  #[test]
  fn survey_of_real_1_1_is_walked_with_a_thumbnail() {
    let Some(rom) = rom_or_skip("roms2/super_mario_bros.nes") else { return };
    let dir = std::env::temp_dir().join(format!("prospector-survey-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let e = survey(&rom, ONE_ONE_AREA, &dir.join("thumbs"));
    assert_eq!(e.class, Class::Walked);
    let fp = e.fingerprint.clone().unwrap();
    let thumb = std::fs::read(dir.join("thumbs").join(format!("{}.ppm", fp))).unwrap();
    assert!(thumb.starts_with(b"P6\n256 240\n255\n"));
    assert_eq!(thumb.len(), 15 + 256 * 240 * 3);
  }

  #[test]
  fn scan_range_writes_each_offset_once_and_resumes() {
    let Some(rom) = rom_or_skip("roms2/super_mario_bros.nes") else { return };
    let dir = std::env::temp_dir().join(format!("prospector-scan-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(scan_range(&rom, ONE_ONE_AREA, ONE_ONE_AREA + 1, &dir).unwrap(), 2);
    assert_eq!(scan_range(&rom, ONE_ONE_AREA, ONE_ONE_AREA + 1, &dir).unwrap(), 0);
    assert_eq!(load_catalog(&dir.join(CATALOG)).len(), 2);
  }

  #[test]
  fn solved_class_follows_the_clear_and_the_panics() {
    let s = |clear, panics, expansions| Solved { clear, x_reached: 0, panics, expansions, path: vec![], stats: vec![] };
    assert_eq!(solved_class(&s(Some(Clear::Flag), 0, 10)), Class::ClearedFlag);
    assert_eq!(solved_class(&s(Some(Clear::Distance), 3, 10)), Class::ClearedDistance);
    assert_eq!(solved_class(&s(None, 3, 10)), Class::Unsolved);
    assert_eq!(solved_class(&s(None, 10, 10)), Class::Crashed);
  }
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib prospector 2>&1 | tail -15`
Expected: compile errors for the missing names.

- [ ] **Step 3: Implement `catalog.rs`** (above its tests)

```rust
//! The catalog: one JSON line per offset, appended as verdicts arrive. The
//! latest line for an offset wins, so a solve can append over a walk.

use crate::nes::gym::prospector::funnel::Solved;
use crate::nes::gym::prospector::rank::{narrowness, Narrowness};
use crate::nes::gym::reservoir::demo::Clear;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Class {
  Crashed,
  Jammed,
  Blank,
  InstantDeath,
  Duplicate,
  Stuck,
  WalkerDied,
  Walked,
  Unsolved,
  ClearedDistance,
  ClearedFlag,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolveRecord {
  pub clear: Option<Clear>,
  pub x_reached: i32,
  pub rounds: usize,
  pub panics: usize,
  pub bottleneck: usize,
  pub kill_rate: f64,
  pub path: Vec<u8>,
}

impl SolveRecord {
  pub fn from_solved(s: &Solved) -> SolveRecord {
    let n = narrowness(&s.stats);
    SolveRecord {
      clear: s.clear,
      x_reached: s.x_reached,
      rounds: n.rounds,
      panics: s.panics,
      bottleneck: n.bottleneck,
      kill_rate: n.kill_rate,
      path: s.path.clone(),
    }
  }

  pub fn narrowness(&self) -> Narrowness {
    Narrowness {
      bottleneck: self.bottleneck,
      kill_rate: self.kill_rate,
      rounds: self.rounds,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
  pub offset: u16,
  pub hex: String,
  pub class: Class,
  #[serde(default)]
  pub message: Option<String>,
  #[serde(default)]
  pub fingerprint: Option<String>,
  #[serde(default)]
  pub duplicate_of: Option<u16>,
  #[serde(default)]
  pub walk_x: Option<i32>,
  #[serde(default)]
  pub solve: Option<SolveRecord>,
}

impl Entry {
  pub fn new(offset: u16, class: Class) -> Entry {
    Entry {
      offset,
      hex: format!("${:04X}", offset),
      class,
      message: None,
      fingerprint: None,
      duplicate_of: None,
      walk_x: None,
      solve: None,
    }
  }
}

pub fn append(path: &Path, e: &Entry) -> Result<(), String> {
  if let Some(dir) = path.parent() {
    std::fs::create_dir_all(dir).map_err(|err| format!("create {}: {}", dir.display(), err))?;
  }
  let mut f = std::fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(path)
    .map_err(|err| format!("open {}: {}", path.display(), err))?;
  let mut line = serde_json::to_string(e).map_err(|err| err.to_string())?;
  line.push('\n');
  f.write_all(line.as_bytes()).map_err(|err| err.to_string())
}

/// Every offset's latest entry, by offset. Lines that don't parse (a scan
/// killed mid-write) are skipped.
pub fn load_catalog(path: &Path) -> Vec<Entry> {
  let Ok(text) = std::fs::read_to_string(path) else {
    return Vec::new();
  };
  let mut latest = BTreeMap::new();
  for line in text.lines() {
    if let Ok(e) = serde_json::from_str::<Entry>(line) {
      latest.insert(e.offset, e);
    }
  }
  latest.into_values().collect()
}

/// Mark every entry whose fingerprint an earlier entry already has as a
/// duplicate of it. `entries` must be sorted by offset.
pub fn dedupe(entries: &mut [Entry]) {
  let mut first: HashMap<String, u16> = HashMap::new();
  for e in entries.iter_mut() {
    let Some(fp) = e.fingerprint.clone() else { continue };
    match first.get(&fp) {
      Some(&original) => {
        e.class = Class::Duplicate;
        e.duplicate_of = Some(original);
      }
      None => {
        first.insert(fp, e.offset);
      }
    }
  }
}
```

- [ ] **Step 4: Implement `scan.rs`** (above its tests)

```rust
//! Run the funnel over offsets and append the verdicts to the catalog.

use crate::nes::gym::prospector::catalog::{append, dedupe, load_catalog, Class, Entry, SolveRecord};
use crate::nes::gym::prospector::funnel::{fingerprint, load, solve, walk, LoadClass, Solved, MOVED_ENOUGH};
use crate::nes::gym::prospector::patch::patch_rom;
use crate::nes::gym::reservoir::demo::{guarded, smb_start_state_at, Clear, Stage};
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub const CATALOG: &str = "catalog.jsonl";
pub const CALIBRATION_FILE: &str = "calibration.json";
/// Real levels, solved the same way, to give narrowness a scale.
pub const CALIBRATION: [(&str, Stage); 2] = [("1-1", Stage::ONE_ONE), ("4-1", Stage { world: 3, level: 0 })];

fn write_thumb(dir: &Path, fp: &str, rgb: &[u8]) -> Result<(), String> {
  let path = dir.join(format!("{}.ppm", fp));
  if path.exists() {
    return Ok(());
  }
  std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
  let tmp = dir.join(format!("{}.ppm.partial", fp));
  let mut bytes = b"P6\n256 240\n255\n".to_vec();
  bytes.extend_from_slice(rgb);
  std::fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
  std::fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

/// Load, fingerprint and walk one offset. Never panics: a panic is a `Crashed` entry.
pub fn survey(rom: &[u8], offset: u16, thumbs: &Path) -> Entry {
  let patched = patch_rom(rom, offset);
  let loaded = match guarded(|| load(&patched)) {
    Ok(l) => l,
    Err(msg) => {
      let mut e = Entry::new(offset, Class::Crashed);
      e.message = Some(msg);
      return e;
    }
  };
  let class = match loaded.class {
    LoadClass::Jammed => Class::Jammed,
    LoadClass::Blank => Class::Blank,
    LoadClass::InstantDeath => Class::InstantDeath,
    LoadClass::Alive => Class::Walked,
  };
  let mut e = Entry::new(offset, class);
  if class != Class::Walked {
    return e;
  }
  let walked = guarded(|| (fingerprint(&patched, &loaded.state), walk(&patched, &loaded.state, offset as u64)));
  match walked {
    Err(msg) => {
      e.class = Class::Crashed;
      e.message = Some(msg);
    }
    Ok((fp, w)) => {
      let fp = format!("{:016x}", fp);
      if let Err(msg) = write_thumb(thumbs, &fp, &loaded.thumb) {
        e.message = Some(format!("thumbnail: {}", msg));
      }
      e.fingerprint = Some(fp);
      e.walk_x = Some(w.x);
      if w.moved < MOVED_ENOUGH {
        e.class = if w.died { Class::WalkerDied } else { Class::Stuck };
      }
    }
  }
  e
}

/// Survey every offset in `from..=to` not already in `out/catalog.jsonl`,
/// across threads. Returns how many entries it wrote.
pub fn scan_range(rom: &[u8], from: u16, to: u16, out: &Path) -> Result<usize, String> {
  let catalog = out.join(CATALOG);
  let thumbs = out.join("thumbs");
  let done: HashSet<u16> = load_catalog(&catalog).into_iter().map(|e| e.offset).collect();
  let todo: Vec<u16> = (from..=to).filter(|o| !done.contains(o)).collect();
  let next = AtomicUsize::new(0);
  let written = AtomicUsize::new(0);
  let lock = Mutex::new(());
  let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
  let errors: Mutex<Vec<String>> = Mutex::new(Vec::new());
  std::thread::scope(|s| {
    for _ in 0..threads {
      s.spawn(|| loop {
        let i = next.fetch_add(1, Ordering::Relaxed);
        let Some(&offset) = todo.get(i) else { break };
        let e = survey(rom, offset, &thumbs);
        let _guard = lock.lock().unwrap();
        if let Err(msg) = append(&catalog, &e) {
          errors.lock().unwrap().push(msg);
          break;
        }
        let n = written.fetch_add(1, Ordering::Relaxed) + 1;
        if n % 256 == 0 {
          eprintln!("scanned {} of {}", n, todo.len());
        }
      });
    }
  });
  match errors.into_inner().unwrap().into_iter().next() {
    Some(msg) => Err(msg),
    None => Ok(written.into_inner()),
  }
}

pub fn solved_class(s: &Solved) -> Class {
  match s.clear {
    Some(Clear::Flag) => Class::ClearedFlag,
    Some(Clear::Distance) => Class::ClearedDistance,
    None if s.expansions > 0 && s.panics == s.expansions => Class::Crashed,
    None => Class::Unsolved,
  }
}

/// Beam-search the `k` distinct walked levels that got furthest and haven't
/// been searched, appending each verdict. Returns how many it searched.
pub fn solve_top(rom: &[u8], out: &Path, k: usize) -> Result<usize, String> {
  let catalog = out.join(CATALOG);
  let mut entries = load_catalog(&catalog);
  dedupe(&mut entries);
  let mut walked: Vec<Entry> = entries.into_iter().filter(|e| e.class == Class::Walked).collect();
  walked.sort_by_key(|e| (std::cmp::Reverse(e.walk_x.unwrap_or(0)), e.offset));
  let mut n = 0;
  for (i, mut e) in walked.into_iter().take(k).enumerate() {
    let started = std::time::Instant::now();
    let patched = patch_rom(rom, e.offset);
    let state = smb_start_state_at(&patched, Stage::ONE_ONE);
    let solved = solve(&patched, &state, Stage::ONE_ONE);
    e.class = solved_class(&solved);
    e.solve = Some(SolveRecord::from_solved(&solved));
    append(&catalog, &e)?;
    eprintln!("solved {} of {}: {} {:?} in {:.0?}", i + 1, k, e.hex, e.class, started.elapsed());
    n += 1;
  }
  Ok(n)
}

/// Solve the calibration levels on the unpatched ROM, enemies and all.
pub fn calibrate(rom: &[u8], out: &Path) -> Result<(), String> {
  let mut map = serde_json::Map::new();
  for (name, stage) in CALIBRATION {
    let state = smb_start_state_at(rom, stage);
    let solved = solve(rom, &state, stage);
    let record = SolveRecord::from_solved(&solved);
    eprintln!("calibrated {}: {:?} bottleneck {}", name, record.clear, record.bottleneck);
    map.insert(name.to_string(), serde_json::to_value(&record).map_err(|e| e.to_string())?);
  }
  std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
  std::fs::write(out.join(CALIBRATION_FILE), serde_json::to_vec_pretty(&map).unwrap()).map_err(|e| e.to_string())
}
```

- [ ] **Step 5: Format and run**

Run: `rustfmt --edition 2021 src/nes/gym/prospector/*.rs && RUSTFLAGS=-Awarnings cargo test --release --lib prospector 2>&1 | tail -12`
Expected: all prospector tests pass (22).

- [ ] **Step 6: Commit**

```bash
git add src/nes/gym/prospector/
# message: "Add the Prospector's catalog and scan: resumable, deduplicated, one line per offset."
git commit -F <msgfile>
```

---

### Task 6: Ranking, clips and the CLI

**Files:**
- Create: `src/nes/gym/prospector/clips.rs`, `examples/prospector.rs`
- Modify: `src/nes/gym/prospector/rank.rs` (add `rows`), `src/nes/gym/prospector/mod.rs` (add `pub mod clips;`)

**Interfaces:**
- Consumes: `catalog::{Entry, Class, SolveRecord, load_catalog, dedupe}`, `scan::{scan_range, solve_top, calibrate, CATALOG, CALIBRATION_FILE}`, `patch::{patch_rom, check_rom}`, `film::TakeWriter`, `demo::{MACRO_FRAMES, smb_start_state_at, Stage}`.
- Produces:
  - `rank::rows(entries: &[Entry], calibration: &BTreeMap<String, SolveRecord>) -> Vec<serde_json::Value>`; each row: `rank` (1-based), `offset`, `hex`, `class`, `clear`, `bottleneck`, `kill_rate`, `rounds`, `x_reached`, `narrower_than` (list of calibration names, in `CALIBRATION` order)
  - `clips::INTRO_FRAMES: usize = 120`
  - `clips::record_clip(rom: &[u8], entry: &Entry, row: &serde_json::Value, out: &Path) -> Result<usize, String>` (returns frames written; take at `out/<hex without $>/`)
  - CLI: `cargo run --release --example prospector -- scan [--from HEX] [--to HEX] | solve [--k N] | calibrate | rank | clips [--top N]`; output under `target/prospector/`; `ranking.json` written by `rank`.

- [ ] **Step 1: Write the failing tests**

In `rank.rs` tests:

```rust
  use crate::nes::gym::prospector::catalog::{Class, Entry, SolveRecord};
  use std::collections::BTreeMap;

  fn cleared(offset: u16, bottleneck: usize, kill_rate: f64) -> Entry {
    let mut e = Entry::new(offset, Class::ClearedDistance);
    e.solve = Some(SolveRecord { clear: Some(crate::nes::gym::reservoir::demo::Clear::Distance), x_reached: 3100, rounds: 90, panics: 0, bottleneck, kill_rate, path: vec![] });
    e
  }

  #[test]
  fn rows_rank_cleared_levels_worst_first_against_calibration() {
    let mut cal = BTreeMap::new();
    cal.insert("1-1".to_string(), cleared(0, 4, 0.5).solve.unwrap());
    cal.insert("4-1".to_string(), cleared(0, 2, 0.7).solve.unwrap());
    let entries = vec![cleared(0x9000, 3, 0.6), Entry::new(0x9100, Class::Unsolved), cleared(0x9200, 1, 0.2)];
    let rows = rows(&entries, &cal);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["hex"], "$9200");
    assert_eq!(rows[0]["rank"], 1);
    assert_eq!(rows[0]["narrower_than"], serde_json::json!(["1-1", "4-1"]));
    assert_eq!(rows[1]["narrower_than"], serde_json::json!(["1-1"]));
  }

  #[test]
  fn rows_are_empty_without_cleared_levels() {
    assert!(rows(&[Entry::new(0x9100, Class::Unsolved)], &BTreeMap::new()).is_empty());
  }
```

In `clips.rs`:

```rust
#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::prospector::catalog::{Class, SolveRecord};
  use crate::nes::gym::prospector::patch::ONE_ONE_AREA;
  use crate::nes::gym::reservoir::rom_or_skip;

  #[test]
  fn a_clip_is_the_intro_then_the_path() {
    let Some(rom) = rom_or_skip("roms2/super_mario_bros.nes") else { return };
    let mut e = Entry::new(ONE_ONE_AREA, Class::ClearedDistance);
    e.solve = Some(SolveRecord { clear: None, x_reached: 0, rounds: 1, panics: 0, bottleneck: 1, kill_rate: 0.0, path: vec![0x81; 10] });
    let dir = std::env::temp_dir().join(format!("prospector-clip-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let n = record_clip(&rom, &e, &serde_json::json!({"rank": 1}), &dir).unwrap();
    assert_eq!(n, INTRO_FRAMES + 10 * MACRO_FRAMES);
    let meta: serde_json::Value = serde_json::from_slice(&std::fs::read(dir.join("A68E").join("take.json")).unwrap()).unwrap();
    assert_eq!(meta["streams"]["smb"], n);
    assert_eq!(meta["intro_frames"], INTRO_FRAMES);
    assert_eq!(meta["rank"], 1);
  }
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib prospector 2>&1 | tail -15`
Expected: compile errors for `rows`, `record_clip`, `INTRO_FRAMES`.

- [ ] **Step 3: Implement `rows` in `rank.rs`**

```rust
use crate::nes::gym::prospector::catalog::{Entry, SolveRecord};
use crate::nes::gym::prospector::scan::CALIBRATION;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// The cleared levels, worst first, each with the calibration levels it is narrower than.
pub fn rows(entries: &[Entry], calibration: &BTreeMap<String, SolveRecord>) -> Vec<Value> {
  let mut cleared: Vec<(&Entry, &SolveRecord)> = entries.iter().filter_map(|e| e.solve.as_ref().filter(|s| s.clear.is_some()).map(|s| (e, s))).collect();
  cleared.sort_by(|a, b| worse(&a.1.narrowness(), &b.1.narrowness()).then(a.0.offset.cmp(&b.0.offset)));
  cleared
    .iter()
    .enumerate()
    .map(|(i, (e, s))| {
      let n = s.narrowness();
      let narrower: Vec<&str> = CALIBRATION
        .iter()
        .map(|(name, _)| *name)
        .filter(|name| calibration.get(*name).is_some_and(|c| worse(&n, &c.narrowness()) == Ordering::Less))
        .collect();
      json!({
        "rank": i + 1, "offset": e.offset, "hex": e.hex, "class": e.class, "clear": s.clear,
        "bottleneck": s.bottleneck, "kill_rate": s.kill_rate, "rounds": s.rounds,
        "x_reached": s.x_reached, "narrower_than": narrower,
      })
    })
    .collect()
}
```

- [ ] **Step 4: Implement `clips.rs`** (above its tests)

```rust
//! Replay a cleared level's path into a take the Python clip maker reads.

use crate::nes::gym::prospector::catalog::Entry;
use crate::nes::gym::prospector::patch::patch_rom;
use crate::nes::gym::reservoir::demo::{smb_start_state_at, Stage, MACRO_FRAMES};
use crate::nes::gym::reservoir::film::TakeWriter;
use crate::nes::gym::NesEmulator;
use serde_json::Value;
use std::path::Path;

/// Frames the level's first screen is held before Mario moves.
pub const INTRO_FRAMES: usize = 120;

pub fn record_clip(rom: &[u8], entry: &Entry, row: &Value, out: &Path) -> Result<usize, String> {
  let solve = entry.solve.as_ref().ok_or_else(|| format!("{} was never solved", entry.hex))?;
  let patched = patch_rom(rom, entry.offset);
  let mut emu = NesEmulator::new(&patched);
  emu.load_state(&smb_start_state_at(&patched, Stage::ONE_ONE));
  let mut w = TakeWriter::new(&out.join(entry.hex.trim_start_matches('$')))?;
  let first = emu.framebuffer().to_vec();
  for _ in 0..INTRO_FRAMES {
    w.frame("smb", &first)?;
  }
  let mut n = INTRO_FRAMES;
  for &buttons in &solve.path {
    for _ in 0..MACRO_FRAMES {
      emu.step_frame(buttons);
      w.frame("smb", emu.framebuffer())?;
      n += 1;
    }
  }
  let mut meta = row.clone();
  meta["offset"] = entry.offset.into();
  meta["hex"] = entry.hex.clone().into();
  meta["intro_frames"] = INTRO_FRAMES.into();
  w.finish(meta)?;
  Ok(n)
}
```

Note: `beam_search_until` steps each macro step with `step_frames(buttons, MACRO_FRAMES)`; stepping the same buttons frame by frame with `step_frame` is the same emulation.

- [ ] **Step 5: Write `examples/prospector.rs`**

```rust
//! The Prospector: SMB's accidental levels. Output goes to target/prospector/.
//!
//! Usage: cargo run --release --example prospector -- <command>
//!   scan [--from HEX] [--to HEX]   load, fingerprint and walk every offset (default $8000-$FFFF)
//!   solve [--k N]                  beam-search the N furthest-walking distinct levels (default 50)
//!   calibrate                      solve real 1-1 and 4-1 the same way
//!   rank                           print the leaderboard and write ranking.json
//!   clips [--top N]                record takes for the top N (default 10)

use greenstone::nes::gym::prospector::catalog::{dedupe, load_catalog, SolveRecord};
use greenstone::nes::gym::prospector::clips::record_clip;
use greenstone::nes::gym::prospector::patch::check_rom;
use greenstone::nes::gym::prospector::rank::rows;
use greenstone::nes::gym::prospector::scan::{calibrate, scan_range, solve_top, CALIBRATION_FILE, CATALOG};
use std::collections::BTreeMap;
use std::path::Path;

fn flag(args: &[String], name: &str) -> Option<String> {
  args.windows(2).find(|w| w[0] == name).map(|w| w[1].clone())
}

fn hex(s: &str) -> u16 {
  u16::from_str_radix(s.trim_start_matches('$').trim_start_matches("0x"), 16).unwrap_or_else(|_| fail(&format!("not a hex offset: {}", s)))
}

fn fail(msg: &str) -> ! {
  eprintln!("{}", msg);
  std::process::exit(1)
}

fn main() {
  let args: Vec<String> = std::env::args().skip(1).collect();
  let rom = std::fs::read("roms2/super_mario_bros.nes").unwrap_or_else(|e| fail(&format!("roms2/super_mario_bros.nes: {}", e)));
  check_rom(&rom).unwrap_or_else(|e| fail(&e));
  let out = Path::new("target/prospector");
  // Garbage levels panic the emulator thousands of times; each is caught and catalogued.
  std::panic::set_hook(Box::new(|_| {}));
  match args.first().map(String::as_str) {
    Some("scan") => {
      let from = flag(&args, "--from").map_or(0x8000, |s| hex(&s));
      let to = flag(&args, "--to").map_or(0xFFFF, |s| hex(&s));
      let n = scan_range(&rom, from, to, out).unwrap_or_else(|e| fail(&e));
      eprintln!("scanned {} new offsets", n);
    }
    Some("solve") => {
      let k = flag(&args, "--k").map_or(50, |s| s.parse().unwrap_or_else(|_| fail("--k takes a number")));
      let n = solve_top(&rom, out, k).unwrap_or_else(|e| fail(&e));
      eprintln!("solved {} levels", n);
    }
    Some("calibrate") => calibrate(&rom, out).unwrap_or_else(|e| fail(&e)),
    Some("rank") | Some("clips") => {
      let mut entries = load_catalog(&out.join(CATALOG));
      dedupe(&mut entries);
      let calibration: BTreeMap<String, SolveRecord> = std::fs::read(out.join(CALIBRATION_FILE))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
      let rows = rows(&entries, &calibration);
      if args[0] == "rank" {
        for r in &rows {
          println!("{:>3}  {}  {}  bottleneck {}  kill {:.2}  rounds {}  narrower than {}", r["rank"], r["hex"], r["clear"], r["bottleneck"], r["kill_rate"].as_f64().unwrap_or(0.0), r["rounds"], r["narrower_than"]);
        }
        std::fs::write(out.join("ranking.json"), serde_json::to_vec_pretty(&rows).unwrap()).unwrap_or_else(|e| fail(&e.to_string()));
      } else {
        let top = flag(&args, "--top").map_or(10, |s| s.parse().unwrap_or_else(|_| fail("--top takes a number")));
        for r in rows.iter().take(top) {
          let offset = r["offset"].as_u64().unwrap() as u16;
          let entry = entries.iter().find(|e| e.offset == offset).unwrap();
          let n = record_clip(&rom, entry, r, &out.join("takes")).unwrap_or_else(|e| fail(&e));
          eprintln!("recorded {} ({} frames)", entry.hex, n);
        }
      }
    }
    _ => fail("usage: prospector scan [--from HEX] [--to HEX] | solve [--k N] | calibrate | rank | clips [--top N]"),
  }
}
```

- [ ] **Step 6: Format, run the tests, and smoke the CLI**

Run: `rustfmt --edition 2021 src/nes/gym/prospector/*.rs examples/prospector.rs && RUSTFLAGS=-Awarnings cargo test --release --lib prospector 2>&1 | tail -8`
Expected: all prospector tests pass (25).

Run: `RUSTFLAGS=-Awarnings cargo build --release --example prospector 2>&1 | tail -2`, then `target/release/examples/prospector scan --from A68E --to A69D; tail -3 target/prospector/catalog.jsonl; target/release/examples/prospector rank`
Expected: `scanned 16 new offsets`; three JSON lines; `rank` prints nothing (nothing solved yet) and writes `ranking.json` as `[]`. Then `target/release/examples/prospector bogus` prints the usage line and exits 1.

- [ ] **Step 7: Commit**

```bash
git add src/nes/gym/prospector/ examples/prospector.rs
# message: "Add the Prospector's ranking, clip takes and CLI."
git commit -F <msgfile>
```

---

### Task 7: Clips and the contact sheet (Python)

**Files:**
- Create: `tools/prospector/__init__.py`, `tools/prospector/clips.py`, `tools/prospector/sheet.py`, `tools/prospector/tests/__init__.py`, `tools/prospector/tests/test_clips.py`, `tools/prospector/tests/test_sheet.py`

**Interfaces:**
- Consumes: `target/prospector/{catalog.jsonl, ranking.json, calibration.json, thumbs/<fp>.ppm, takes/<HEX>/take.json + smb.rgb}`; `reservoir_film.draw` (`blank`, `nes`, `text`, `GOLD`, `DIM`, `INK`, `RED`), `reservoir_film.model.Take`.
- Produces:
  - `clips.compose(take, i) -> PIL.Image` (1280x720), `clips.encode(take, out_path)`, `clips.main()` writing `target/prospector/clips/<rank:02d>-<HEX>.mp4`
  - `sheet.load(root) -> (entries, ranking, calibration)`, `sheet.sections(entries, ranking) -> list[(title, list[dict])]`, `sheet.render(entries, ranking, calibration) -> str`, `sheet.main()` writing `target/prospector/catalog.html` and `thumbs/<fp>.png`

- [ ] **Step 1: Write the failing tests**

`tools/prospector/tests/test_sheet.py`:

```python
import unittest

from prospector import sheet


def entry(offset, cls, **kw):
    return dict({"offset": offset, "hex": f"${offset:04X}", "class": cls}, **kw)


class Sections(unittest.TestCase):
    def setUp(self):
        self.entries = [
            entry(0x8000, "crashed", message="Opcode 0x02 is not recognized"),
            entry(0x8001, "blank"),
            entry(0x8002, "walked", walk_x=900, fingerprint="aa"),
            entry(0x8003, "duplicate", duplicate_of=0x8002, fingerprint="aa"),
            entry(0x8004, "stuck", fingerprint="bb"),
            entry(0x8005, "unsolved", fingerprint="cc"),
            entry(0x8006, "cleared_distance", fingerprint="dd"),
            entry(0x8007, "cleared_flag", fingerprint="ee"),
            entry(0x8008, "instant_death"),
        ]
        self.ranking = [{"rank": 1, "offset": 0x8007, "hex": "$8007"}, {"rank": 2, "offset": 0x8006, "hex": "$8006"}]

    def test_sections_come_in_order_and_skip_duplicates(self):
        got = sheet.sections(self.entries, self.ranking)
        self.assertEqual([t for t, _ in got], list(sheet.SECTION_TITLES))
        flat = [e["offset"] for _, es in got for e in es]
        self.assertNotIn(0x8003, flat)
        self.assertEqual(sorted(flat), sorted(e["offset"] for e in self.entries if e["class"] != "duplicate"))

    def test_the_leaderboard_follows_the_ranking(self):
        title, leaders = sheet.sections(self.entries, self.ranking)[0]
        self.assertEqual([e["offset"] for e in leaders], [0x8007, 0x8006])

    def test_render_shows_counts_messages_and_calibration(self):
        html = sheet.render(self.entries, self.ranking, {"1-1": {"bottleneck": 4, "kill_rate": 0.5, "rounds": 80}})
        self.assertIn("Opcode 0x02 is not recognized", html)
        self.assertIn("crashed: 1", html)
        self.assertIn("1-1", html)
        self.assertIn("$8007", html)
        self.assertTrue(html.startswith("<!doctype html>"))


if __name__ == "__main__":
    unittest.main()
```

`tools/prospector/tests/test_clips.py`:

```python
import json
import shutil
import tempfile
import unittest
from pathlib import Path

import numpy as np

from prospector import clips
from reservoir_film.model import Take


def fake_take(root, frames=3):
    d = Path(root) / "A68E"
    d.mkdir(parents=True)
    (d / "smb.rgb").write_bytes(np.zeros((frames, 240, 256, 3), np.uint8).tobytes())
    meta = {"rank": 1, "offset": 0xA68E, "hex": "$A68E", "clear": "distance", "bottleneck": 2,
            "kill_rate": 0.81, "rounds": 97, "x_reached": 3080, "narrower_than": ["1-1"],
            "intro_frames": 1, "streams": {"smb": frames}}
    (d / "take.json").write_text(json.dumps(meta))
    return Take(d)


class Clips(unittest.TestCase):
    def test_compose_is_frame_sized(self):
        with tempfile.TemporaryDirectory() as root:
            img = clips.compose(fake_take(root), 1)
            self.assertEqual(img.size, (1280, 720))

    def test_caption_names_the_offset_and_comparison(self):
        with tempfile.TemporaryDirectory() as root:
            lines = clips.caption_lines(fake_take(root).meta)
            self.assertIn("$A68E", lines[0])
            self.assertTrue(any("narrower than 1-1" in l for l in lines))

    @unittest.skipUnless(shutil.which("ffmpeg"), "needs ffmpeg")
    def test_encode_writes_an_mp4(self):
        with tempfile.TemporaryDirectory() as root:
            out = Path(root) / "clip.mp4"
            clips.encode(fake_take(root), out)
            self.assertGreater(out.stat().st_size, 0)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cd tools && python3 -m unittest discover -s prospector/tests -t . 2>&1 | tail -5`
Expected: errors importing `prospector.sheet` / `prospector.clips`.

- [ ] **Step 3: Implement `tools/prospector/clips.py`** (and empty `__init__.py` files)

```python
"""Turn the Prospector's takes into short captioned clips.

Usage, from tools/: python3 -m prospector.clips
"""

import subprocess
from pathlib import Path

from reservoir_film import draw
from reservoir_film.model import REPO, Take

ROOT = REPO / "target" / "prospector"
W, H, FPS = 1280, 720, 60
CLEAR_NAMES = {"flag": "cleared: SMB moved on (flag)", "distance": "cleared: reached x 3072 alive"}


def caption_lines(meta):
    lines = [f"{meta['hex']}  (#{meta['rank']})", CLEAR_NAMES.get(meta.get("clear"), "not cleared"),
             f"bottleneck: {meta['bottleneck']} way(s) through",
             f"search killed {meta['kill_rate']:.0%} of what it tried",
             f"{meta['rounds']} rounds"]
    if meta.get("narrower_than"):
        lines.append("narrower than " + " and ".join(meta["narrower_than"]))
    return lines


def compose(take, i):
    img = draw.blank()
    draw.nes(img, take.frames("smb")[i], (0, 0, 768, 720))
    for k, line in enumerate(caption_lines(take.meta)):
        draw.text(img, (800, 60 + k * 56), line, 40 if k == 0 else 26, draw.GOLD if k == 0 else draw.INK, face=draw.mono(40) if k == 0 else None)
    draw.text(img, (800, 660), "The Prospector: SMB, loaded from the wrong address", 18, draw.DIM)
    return img


def encode(take, out):
    out = Path(out)
    out.parent.mkdir(parents=True, exist_ok=True)
    ff = subprocess.Popen(["ffmpeg", "-y", "-loglevel", "error", "-f", "rawvideo", "-pix_fmt", "rgb24",
                           "-s", f"{W}x{H}", "-r", str(FPS), "-i", "-", "-c:v", "libx264", "-crf", "20",
                           "-pix_fmt", "yuv420p", str(out)], stdin=subprocess.PIPE)
    for i in range(len(take.frames("smb"))):
        ff.stdin.write(compose(take, i).tobytes())
    ff.stdin.close()
    if ff.wait() != 0:
        raise RuntimeError(f"ffmpeg failed on {take.name}")


def main():
    takes = sorted((Take(p.parent) for p in (ROOT / "takes").glob("*/take.json")), key=lambda t: t.meta["rank"])
    if not takes:
        raise SystemExit("no takes; run: cargo run --release --example prospector -- clips")
    for take in takes:
        out = ROOT / "clips" / f"{take.meta['rank']:02d}-{take.name}.mp4"
        encode(take, out)
        print(out)


if __name__ == "__main__":
    main()
```

`draw.text`'s `face` is a Pillow font object; `draw.mono(size)` returns one.

- [ ] **Step 4: Implement `tools/prospector/sheet.py`**

```python
"""The Prospector's contact sheet: one static page, worst levels first.

Usage, from tools/: python3 -m prospector.sheet
"""

import html
import json
from pathlib import Path

from PIL import Image

from reservoir_film.model import REPO

ROOT = REPO / "target" / "prospector"
SECTION_TITLES = ("Cleared, worst first", "Unsolved", "Walked, not searched", "Going nowhere",
                  "Instant death", "Hall of shame")
SECTION_CLASSES = (None, {"unsolved"}, {"walked"}, {"stuck", "walker_died"}, {"instant_death"},
                   {"crashed", "jammed", "blank"})


def load(root=ROOT):
    root = Path(root)
    latest = {}
    for line in (root / "catalog.jsonl").read_text().splitlines():
        try:
            e = json.loads(line)
        except json.JSONDecodeError:
            continue
        latest[e["offset"]] = e
    ranking = json.loads((root / "ranking.json").read_text()) if (root / "ranking.json").exists() else []
    calibration = json.loads((root / "calibration.json").read_text()) if (root / "calibration.json").exists() else {}
    return [latest[k] for k in sorted(latest)], ranking, calibration


def sections(entries, ranking):
    """(title, entries) in SECTION_TITLES order. The Rust side has already
    marked duplicates; they are left out."""
    by_offset = {e["offset"]: e for e in entries}
    out = [(SECTION_TITLES[0], [by_offset[r["offset"]] for r in ranking if r["offset"] in by_offset])]
    ranked = {r["offset"] for r in ranking}
    for title, classes in zip(SECTION_TITLES[1:], SECTION_CLASSES[1:]):
        out.append((title, [e for e in entries if e["class"] in classes and e["offset"] not in ranked]))
    leftovers = [e for e in entries if e["class"] in {"cleared_distance", "cleared_flag"} and e["offset"] not in ranked]
    out[0][1].extend(leftovers)
    return out


def card(e, row):
    fp = e.get("fingerprint")
    img = f'<img src="thumbs/{fp}.png" alt="">' if fp else '<div class="none"></div>'
    lines = [f'<b>{html.escape(e["hex"])}</b> {html.escape(e["class"])}']
    if row:
        lines.append(f'#{row["rank"]} bottleneck {row["bottleneck"]}, killed {row["kill_rate"]:.0%}, {row["rounds"]} rounds')
        if row.get("narrower_than"):
            lines.append("narrower than " + " and ".join(row["narrower_than"]))
        name = e["hex"].lstrip("$")
        lines.append(f'<a href="clips/{row["rank"]:02d}-{name}.mp4">clip</a>')
    if e.get("walk_x") is not None:
        lines.append(f'walker reached x {e["walk_x"]}')
    if e.get("message"):
        lines.append(f'<code>{html.escape(e["message"])}</code>')
    return f'<figure>{img}<figcaption>{"<br>".join(lines)}</figcaption></figure>'


def render(entries, ranking, calibration):
    rows = {r["offset"]: r for r in ranking}
    counts = {}
    for e in entries:
        counts[e["class"]] = counts.get(e["class"], 0) + 1
    parts = ["<!doctype html><html><head><meta charset=\"utf-8\"><title>The Prospector</title>",
             "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><style>",
             "body{background:#0c0e14;color:#ebebeb;font:15px system-ui,sans-serif;margin:16px}",
             "figure{display:inline-block;width:256px;margin:8px;vertical-align:top}",
             "img,.none{width:256px;height:240px;image-rendering:pixelated;background:#1a1d28}",
             "code{color:#eb5a50;word-break:break-all}a{color:#f5c850}h2{margin-top:32px}</style></head><body>",
             "<h1>The Prospector</h1><p>Every offset of Super Mario Bros.' ROM, loaded as World 1-1.</p>",
             "<p>" + ", ".join(f"{html.escape(k)}: {v}" for k, v in sorted(counts.items())) + "</p>"]
    if calibration:
        parts.append("<p>Real levels, solved the same way: " + "; ".join(
            f"{html.escape(name)} bottleneck {c['bottleneck']}, killed {c['kill_rate']:.0%}, {c['rounds']} rounds"
            for name, c in calibration.items()) + "</p>")
    for title, es in sections(entries, ranking):
        parts.append(f"<h2>{html.escape(title)} ({len(es)})</h2>")
        parts.extend(card(e, rows.get(e["offset"])) for e in es)
    parts.append("</body></html>")
    return "\n".join(parts)


def main():
    entries, ranking, calibration = load()
    for ppm in (ROOT / "thumbs").glob("*.ppm"):
        png = ppm.with_suffix(".png")
        if not png.exists():
            Image.open(ppm).save(png)
    out = ROOT / "catalog.html"
    out.write_text(render(entries, ranking, calibration))
    print(out)


if __name__ == "__main__":
    main()
```

- [ ] **Step 5: Run the tests**

Run: `cd tools && python3 -m unittest discover -s prospector/tests -t . 2>&1 | tail -4` and `cd tools && python3 -m unittest discover -s reservoir_film/tests -t . 2>&1 | tail -3`
Expected: prospector 6 tests OK; reservoir_film suite still OK.

- [ ] **Step 6: Commit**

```bash
git add tools/prospector/
# message: "Add the Prospector's clip maker and contact sheet."
git commit -F <msgfile>
```

---

### Task 8: Prospect

**Files:**
- Modify: `docs/ideas/brain-cartridge.md` (Build order item 1: point at the spec, this plan and the results)

- [ ] **Step 1: Full scan in the background** (about 2.5 hours on 10 cores)

Run (background): `RUSTFLAGS=-Awarnings cargo build --release --example prospector` then `target/release/examples/prospector scan > /private/tmp/claude-501/-Users-nathan-Projects-ndouglas-greenstone/cf099f6b-6b2a-4507-8759-c42c3f7c29f4/scratchpad/scan.log 2>&1`
Expected: `scanned 32768 new offsets`; `wc -l target/prospector/catalog.jsonl` is 32768.

- [ ] **Step 2: Look at the class counts before spending the solve budget**

Run: `python3 -c "import json,collections;print(collections.Counter(json.loads(l)['class'] for l in open('target/prospector/catalog.jsonl')))"`
Expected: a spread across classes, with some `walked`. If there are no `walked` entries at all, stop and investigate the funnel (likely `load`'s Jammed or InstantDeath tests are misfiring on every level) before solving.

- [ ] **Step 3: Calibrate, then solve K = 50** (about 20 minutes, then about 8 hours, in the background)

Run: `target/release/examples/prospector calibrate` then `target/release/examples/prospector solve --k 50`
Expected: `calibration.json` with 1-1 cleared by flag; 50 `solved` log lines.

- [ ] **Step 4: Rank, record, render**

Run: `target/release/examples/prospector rank | head -20; target/release/examples/prospector clips --top 10; cd tools && python3 -m prospector.clips && python3 -m prospector.sheet`
Expected: a leaderboard, 10 takes, 10 mp4s under `target/prospector/clips/`, and `target/prospector/catalog.html`.

- [ ] **Step 5: Update the ideas doc and commit**

In `docs/ideas/brain-cartridge.md`, under Build order item 1, add a sentence pointing at `docs/superpowers/specs/2026-10-20-prospector-design.md`, `examples/prospector.rs`, and one line of what it found (counts per class, the worst find's offset and bottleneck).

```bash
git add docs/ideas/brain-cartridge.md
# message: "Record what the Prospector found."
git commit -F <msgfile>
```
