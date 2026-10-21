# Reservoir Punch-Out!! Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Run Mike Tyson's Punch-Out!! in Greenstone, then train four brains that read the screen (no memory, frame stack, echo state network, four-Tetris bank) by DAgger against a save-state lookahead teacher, in three phases, and produce the results and film takes.

**Architecture:** Mapper 9 joins the cartridge mappers. A new `src/nes/gym/punchout/` module holds the RAM map, fight starts, moves, the screen projection and brains, the teacher, and the DAgger loop; `examples/punchout.rs` drives it. Every brain is a pure function of the per-step projection sequence (plus RAM for the cheat reference), so fights are stored as projections and brains are refitted offline.

**Tech Stack:** Rust (std threads, serde_json, bincode), the existing reservoir code (`soft.rs`, `ridge.rs`, `drive.rs`, `encode.rs`, `features.rs`, `film.rs`, `demo.rs::guarded`), ffmpeg for PNG conversion.

**Spec:** `docs/superpowers/specs/2026-10-21-punchout-reservoir-design.md`

**Scope split (ruling at planning time):** this plan ends with results, takes and the ideas-doc entry. The film (`tools/punchout_film/`) gets its own plan once results exist, because the spec requires its captions to be chosen from results; a film plan written now would be written against numbers that do not exist.

## Global Constraints

- Screen in, RAM only for scoring. No brain reads RAM except the RAM-cheat reference.
- Three phases, each from fresh brains with its own teacher and moves: 1 Defence, 2 Counter, 3 Whole fight.
- One readout for every brain: `ridge::fit` on one-hot move targets, argmax at play time.
- Train opponents: Glass Joe, Von Kaiser, Piston Honda. Held out: Don Flamenco, Bald Bull.
- `MOVE_FRAMES` = 4. `PROJ` = 256. `HORIZON` = 60 frames. ESN: 2,048 units, default leak 0.5.
- DAgger: round 0 = teacher fights, 3 per training opponent, shared by all brains; rounds 1-4 = 1 student fight per training opponent, teacher labels every 4th step; refit on all labels so far.
- Test: 3 fights per opponent (training and held out), no labelling.
- Sweep (phase 1 only): frame stack depths 1, 2, 4, 8; ESN leaks 0.2, 0.5, 0.9.
- Teacher score: phase 1 = hits taken, ties to Idle; phases 2-3 = damage dealt - 2 x damage taken.
- Stop gate: after phase 1 round 0, if no brain takes fewer hits per minute than always-Idle, stop and show Nate.
- Each emulator step that can panic runs under `reservoir::demo::guarded`; a panic ends that fight, not the run. Fixed seeds everywhere.
- ROM paths: `roms2/mike_tysons_punch_out.nes`, `roms2/tetris.nes`. ROM-dependent tests use `rom_or_skip`.
- Output under `target/punchout/`.
- Repo rules: never stage `src/main.rs` or `src/nes/bus/mod.rs`; leave `.claude/` and `AGENTS.md` alone; format with `rustfmt --edition 2021 <files>` (never `cargo fmt`); test with `RUSTFLAGS=-Awarnings cargo test --release --lib <filter>`; commit with `git commit -F <file>`, message ending with a blank line then `Claude-Session: https://claude.ai/code/session_01GxcjqSAGjzrqp9ufw5r46e`.
- Definition used throughout: **dodge rate** = 1 - (brain's hits per minute / always-Idle's hits per minute against the same opponent). The spec's "share of punches dodged" needs a count of punches thrown, which nothing observes; this ratio against the no-defence baseline measures the same thing.
- Definition: a **hit** is a drop in Mac's health during a move. The spec also names Mac's got-hit state; Task 3 adds it to `hit` only if the probe finds a hit that leaves health unchanged (ledgered either way).
- Definition: a **fight** runs from `fight_start` until the fight is over or `MAX_STEPS` = 2,400 steps (9,600 frames). Hits per minute use only steps where Mac can act.

## Review Focus

1. A save state taken mid-fight must restore the MMC2 latches and banks exactly; otherwise teacher rollouts diverge from the real fight. Pinned in Task 1 (serde round trip) and Task 3 (save, step, load, step gives identical frames).
2. The teacher must leave the caller's emulator exactly as it found it. Pinned in Task 6.
3. Steps where Mac cannot act (intros, between rounds, knockdowns) must be neither labelled nor counted in hits per minute. Pinned in Task 4 and Task 7.
4. A run killed mid-write must resume without losing or duplicating labels (truncated last line). Pinned in Task 7.
5. Refitting replays stored projections through fresh brains, so every brain must be deterministic for a seed; the bank and ESN especially. Pinned in Task 5.

---

### Task 1: Mapper 9 (MMC2)

**Files:**
- Create: `src/nes/cartridge/mapper/mapper9.rs`
- Modify: `src/nes/cartridge/mapper/mod.rs` (module, enum variant `Mmc2(Mapper9)`, every match arm)
- Modify: `src/nes/cartridge/mod.rs:25-32` (add `9 => Mapper::Mmc2(Mapper9::new(data)),`)

**Interfaces:**
- Consumes: `Data`, `Pager::read_u8(Page, u16)`, `get_page_count(PageSize)`, `Mappable`.
- Produces: `Mapper9::new(data: Data) -> Mapper9`; `Mapper::Mmc2`.

- [ ] **Step 1: Write the failing tests** at the bottom of `mapper9.rs` (create the file with only the doc comment, imports, an empty `pub struct Mapper9 { data: Data }`, `new`, and `todo!()` bodies for every `Mappable` method so the tests compile):

```rust
#[cfg(test)]
mod test {
  use super::*;
  use crate::nes::cartridge::Data;

  /// 8 KB PRG banks start with their bank number; 4 KB CHR banks start with theirs.
  fn build_mapper9(prg_8k: usize, chr_4k: usize) -> Mapper9 {
    let mut data = vec![0x4e, 0x45, 0x53, 0x1a, (prg_8k / 2) as u8, (chr_4k / 2) as u8, 0x90, 0x00, 0, 0, 0, 0, 0, 0, 0, 0];
    for bank in 0..prg_8k {
      data.extend((0..0x2000usize).map(|i| if i < 4 { bank as u8 } else { (i & 0xFF) as u8 }));
    }
    for bank in 0..chr_4k {
      data.extend((0..0x1000usize).map(|i| if i < 4 { bank as u8 } else { 0xA0 | (bank as u8 & 0x0F) }));
    }
    Mapper9::new(Data::new(&data))
  }

  #[test]
  fn prg_switches_8000_and_fixes_the_last_three_banks() {
    let mut m = build_mapper9(16, 32);
    assert_eq!(m.read_prg_u8(0x8000), 0);
    assert_eq!(m.read_prg_u8(0xA000), 13);
    assert_eq!(m.read_prg_u8(0xC000), 14);
    assert_eq!(m.read_prg_u8(0xE000), 15);
    m.write_prg_u8(0xA000, 5);
    assert_eq!(m.read_prg_u8(0x8000), 5);
    assert_eq!(m.read_prg_u8(0xA000), 13, "writing $A000 does not move the fixed banks");
  }

  #[test]
  fn chr_halves_follow_their_latches_which_start_at_fe() {
    let mut m = build_mapper9(16, 32);
    m.write_prg_u8(0xB000, 1); // half 0, FD
    m.write_prg_u8(0xC000, 2); // half 0, FE
    m.write_prg_u8(0xD000, 3); // half 1, FD
    m.write_prg_u8(0xE000, 4); // half 1, FE
    assert_eq!(m.read_chr_u8(0x0000), 2, "power-on latch 0 is FE");
    assert_eq!(m.read_chr_u8(0x1000), 4, "power-on latch 1 is FE");
  }

  #[test]
  fn reading_tile_fd_or_fe_flips_the_latch_after_the_read() {
    let mut m = build_mapper9(16, 32);
    m.write_prg_u8(0xB000, 1);
    m.write_prg_u8(0xC000, 2);
    m.write_prg_u8(0xD000, 3);
    m.write_prg_u8(0xE000, 4);
    assert_eq!(m.read_chr_u8(0x0FD8), 0xA2, "the triggering read still uses the FE bank");
    assert_eq!(m.read_chr_u8(0x0000), 1, "now FD");
    m.read_chr_u8(0x0FE8);
    assert_eq!(m.read_chr_u8(0x0000), 2, "back to FE");
    m.read_chr_u8(0x1FDC);
    assert_eq!(m.read_chr_u8(0x1000), 3, "half 1 flips anywhere in $1FD8-$1FDF");
    m.read_chr_u8(0x1FEF);
    assert_eq!(m.read_chr_u8(0x1000), 4);
    m.read_chr_u8(0x0FD9);
    assert_eq!(m.read_chr_u8(0x0000), 2, "half 0 flips only on exactly $0FD8");
  }

  #[test]
  fn f000_sets_mirroring() {
    let mut m = build_mapper9(16, 32);
    m.write_prg_u8(0xF000, 0);
    assert!(matches!(m.get_mirroring_mode(), MirroringMode::Vertical));
    m.write_prg_u8(0xF000, 1);
    assert!(matches!(m.get_mirroring_mode(), MirroringMode::Horizontal));
  }

  #[test]
  fn a_serde_round_trip_keeps_banks_and_latches() {
    let mut m = build_mapper9(16, 32);
    m.write_prg_u8(0xA000, 7);
    m.write_prg_u8(0xB000, 1);
    m.read_chr_u8(0x0FD8);
    let bytes = bincode::serialize(&m).unwrap();
    let r: Mapper9 = bincode::deserialize(&bytes).unwrap();
    assert_eq!(r.read_prg_u8(0x8000), 7);
    assert_eq!(r.read_chr_u8(0x0000), 1, "latch 0 restored as FD");
  }

  #[test]
  fn chr_rom_ignores_writes() {
    let mut m = build_mapper9(16, 32);
    m.write_chr_u8(0x0010, 0x55);
    assert_eq!(m.read_chr_u8(0x0010), 0xA0);
  }
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib mapper9`
Expected: 6 tests FAIL (panicked at `not yet implemented`).

- [ ] **Step 3: Implement `Mapper9`**

```rust
/// Mapper 9 (MMC2)
///
/// PRG ROM: one switchable 8KB bank at $8000, the last three 8KB banks fixed at $A000-$FFFF.
/// CHR ROM: two 4KB halves, each switched between two banks by a latch that the
/// PPU sets by fetching tile $FD or $FE.
/// Games: Mike Tyson's Punch-Out!!, Punch-Out!!
use std::cell::Cell;

use super::super::{Data, MirroringMode, Page, PageSize};
use crate::traits::Mappable;

use Page::*;
use PageSize::*;

/// Latch values: which of a half's two banks is showing.
const FD: u8 = 0;
const FE: u8 = 1;

#[derive(Serialize, Deserialize)]
pub struct Mapper9 {
  data: Data,
  prg_bank: u8,
  /// `chr_banks[half][latch]`: the 4KB bank for each half under each latch value.
  chr_banks: [[u8; 2]; 2],
  /// Set by PPU reads, so they change behind `&self`.
  latches: [Cell<u8>; 2],
  horizontal: bool,
}

impl Mapper9 {
  pub fn new(data: Data) -> Mapper9 {
    Mapper9 {
      data,
      prg_bank: 0,
      chr_banks: [[0; 2]; 2],
      latches: [Cell::new(FE), Cell::new(FE)],
      horizontal: false,
    }
  }
}

impl Mappable for Mapper9 {
  #[named]
  fn read_prg_u8(&self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let count = self.data.prg_rom.get_page_count(EightKb);
    let result = match address {
      0x4020..=0x7FFF => 0x00,
      0x8000..=0x9FFF => self.data.prg_rom.read_u8(Number(self.prg_bank as usize % count, EightKb), address - 0x8000),
      0xA000..=0xFFFF => {
        let fixed = ((address - 0xA000) / 0x2000) as usize;
        self.data.prg_rom.read_u8(Number(count - 3 + fixed, EightKb), address & 0x1FFF)
      }
      _ => panic!("bad address: {}", format_u16!(address)),
    };
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  fn write_prg_u8(&mut self, address: u16, value: u8) {
    trace_enter!();
    trace_u16!(address);
    trace_u8!(value);
    match address {
      0x4020..=0x9FFF => {}
      0xA000..=0xAFFF => self.prg_bank = value & 0x0F,
      0xB000..=0xBFFF => self.chr_banks[0][FD as usize] = value & 0x1F,
      0xC000..=0xCFFF => self.chr_banks[0][FE as usize] = value & 0x1F,
      0xD000..=0xDFFF => self.chr_banks[1][FD as usize] = value & 0x1F,
      0xE000..=0xEFFF => self.chr_banks[1][FE as usize] = value & 0x1F,
      0xF000..=0xFFFF => self.horizontal = value & 1 == 1,
      _ => panic!("bad address: {}", format_u16!(address)),
    }
    trace_exit!();
  }

  #[named]
  fn read_chr_u8(&self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let half = ((address >> 12) & 1) as usize;
    let count = self.data.chr_rom.get_page_count(FourKb);
    let bank = self.chr_banks[half][self.latches[half].get() as usize] as usize % count;
    let result = self.data.chr_rom.read_u8(Number(bank, FourKb), address & 0x0FFF);
    match address {
      0x0FD8 => self.latches[0].set(FD),
      0x0FE8 => self.latches[0].set(FE),
      0x1FD8..=0x1FDF => self.latches[1].set(FD),
      0x1FE8..=0x1FEF => self.latches[1].set(FE),
      _ => {}
    }
    trace_u8!(result);
    trace_exit!();
    result
  }

  fn write_chr_u8(&mut self, _address: u16, _value: u8) {
    // CHR ROM: writes are ignored.
  }

  fn get_mirroring_mode(&self) -> MirroringMode {
    if self.horizontal {
      MirroringMode::Horizontal
    } else {
      MirroringMode::Vertical
    }
  }
}
```

Register it: in `mapper/mod.rs` add `pub mod mapper9; pub use mapper9::*;`, the variant `Mmc2(Mapper9)`, and a `Mapper::Mmc2(m) => m.<method>(...)` arm in each of the seven `match`es. In `cartridge/mod.rs` add `9 => Mapper::Mmc2(Mapper9::new(data)),`.

- [ ] **Step 4: Run the tests**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib mapper9`
Expected: 6 passed.

- [ ] **Step 5: Run the whole library suite**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib`
Expected: all pass (470 + 6).

- [ ] **Step 6: Format and commit**

```bash
rustfmt --edition 2021 src/nes/cartridge/mapper/mapper9.rs src/nes/cartridge/mapper/mod.rs src/nes/cartridge/mod.rs
git add src/nes/cartridge/mapper/mapper9.rs src/nes/cartridge/mapper/mod.rs src/nes/cartridge/mod.rs
git commit -F <msg>   # "Add mapper 9 (MMC2), for Punch-Out!!."
```

---

### Task 2: The gate

**Files:**
- Create: `src/nes/gym/punchout/mod.rs` (declares `pub mod gate;` now; later tasks add modules)
- Create: `src/nes/gym/punchout/gate.rs`
- Modify: `src/nes/gym/mod.rs` (add `pub mod punchout;`)
- Create: `examples/punchout.rs` (subcommand `gate` only for now)

**Interfaces:**
- Consumes: `NesEmulator::{new, step_frames, framebuffer}`, `BUTTON_START`.
- Produces: `gate::write_ppm(path: &Path, rgb: &[u8]) -> Result<(), String>`; `gate::boot_strip(rom: &[u8], frames: &[usize], press_start_every: usize) -> Vec<(usize, Vec<u8>)>`; `pub const PUNCH_OUT: &str = "roms2/mike_tysons_punch_out.nes";` in `punchout/mod.rs`.

- [ ] **Step 1: Write the failing test** in `gate.rs`:

```rust
#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::reservoir::rom_or_skip;

  #[test]
  fn punch_out_boots_and_draws_more_than_one_colour() {
    let Some(rom) = rom_or_skip(super::super::PUNCH_OUT) else { return };
    let strip = boot_strip(&rom, &[60, 600], 120);
    assert_eq!(strip.len(), 2);
    for (frame, rgb) in &strip {
      assert_eq!(rgb.len(), 256 * 240 * 3);
      let first = &rgb[0..3];
      assert!(rgb.chunks(3).any(|p| p != first), "frame {} is one flat colour", frame);
    }
  }
}
```

`rom_or_skip` is `pub(crate)` in `reservoir/mod.rs` today; this test uses it from a sibling module, which `pub(crate)` allows.

- [ ] **Step 2: Run it**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib punch_out_boots`
Expected: FAIL to compile (`boot_strip` not found). Before mapper 9 it would have panicked with "Mapper 9 not implemented"; that is Task 1's job.

- [ ] **Step 3: Implement**

```rust
//! The mapper gate: does Punch-Out!! boot and draw?

use std::path::Path;

use crate::nes::bus::BUTTON_START;
use crate::nes::gym::NesEmulator;

/// Write a 256x240 RGB frame as a binary PPM.
pub fn write_ppm(path: &Path, rgb: &[u8]) -> Result<(), String> {
  let mut bytes = b"P6\n256 240\n255\n".to_vec();
  bytes.extend_from_slice(rgb);
  std::fs::write(path, bytes).map_err(|e| e.to_string())
}

/// Boot the ROM and keep the frames numbered in `frames`, tapping Start for
/// four frames every `press_start_every` frames so the game leaves its title
/// and intro screens.
pub fn boot_strip(rom: &[u8], frames: &[usize], press_start_every: usize) -> Vec<(usize, Vec<u8>)> {
  let mut emu = NesEmulator::new(rom);
  let last = frames.iter().copied().max().unwrap_or(0);
  let mut out = Vec::new();
  for f in 1..=last {
    let buttons = if press_start_every > 0 && f % press_start_every < 4 { BUTTON_START } else { 0 };
    emu.step_frames(buttons, 1);
    if frames.contains(&f) {
      out.push((f, emu.framebuffer().to_vec()));
    }
  }
  out
}
```

`examples/punchout.rs`: parse `std::env::args()`; `gate` reads `PUNCH_OUT`, calls `boot_strip(&rom, &[60, 300, 600, 900, 1200, 1800, 2400, 3000, 3600, 4800, 6000], 240)`, writes `target/punchout/gate/frame-<NNNNN>.ppm` for each, then prints the paths. Same `main` style as `examples/prospector.rs` (subcommand `match`, `Result<(), String>`, `exit(1)` on error).

- [ ] **Step 4: Run the test**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib punch_out_boots`
Expected: PASS.

- [ ] **Step 5: Run the gate and convert**

```bash
RUSTFLAGS=-Awarnings cargo build --release --example punchout
target/release/examples/punchout gate
for f in target/punchout/gate/*.ppm; do ffmpeg -loglevel error -y -i "$f" "${f%.ppm}.png"; done
```

Expected: 11 PNGs. Read every one. Expected: a title screen, then the intro, then Glass Joe in the ring. If no frame shows a fight, rerun with a longer frame list until one does.

- [ ] **Step 6: STOP — show Nate** the title PNG and the best mid-fight PNG (`SendUserFile` or read them inline) and ask whether Glass Joe draws correctly. This is the spec's gate; nothing below starts until he answers. If tiles are wrong, the fix (PPU sprite fetch timing) becomes a sub-task here with its own failing test before continuing, and is ledgered.

- [ ] **Step 7: Record the HUD rows.** From the mid-fight PNG, note the last screen row (of 240) occupied by the hearts/health/clock bar. Ledger it as `Task 2: HUD bottom row = N`; Task 5 uses it.

- [ ] **Step 8: Format and commit**

```bash
rustfmt --edition 2021 src/nes/gym/punchout/mod.rs src/nes/gym/punchout/gate.rs examples/punchout.rs src/nes/gym/mod.rs
git add src/nes/gym/punchout/mod.rs src/nes/gym/punchout/gate.rs examples/punchout.rs src/nes/gym/mod.rs
git commit -F <msg>   # "Boot Punch-Out!!: the mapper gate."
```

---

### Task 3: RAM map and fight starts

This task finds addresses. The procedure is fixed; the values are discoveries, each pinned by a behavioural test that does not assume the value.

**Files:**
- Create: `src/nes/gym/punchout/ram.rs`
- Create: `src/nes/gym/punchout/start.rs`
- Modify: `src/nes/gym/punchout/mod.rs` (add modules)
- Modify: `examples/punchout.rs` (subcommand `probe`)

**Interfaces:**
- Consumes: `NesEmulator`, Task 2's `boot_strip`, `write_ppm`.
- Produces (`ram.rs`):
  - `pub fn mac_health(emu: &mut NesEmulator) -> u8`
  - `pub fn opponent_health(emu: &mut NesEmulator) -> u8`
  - `pub fn mac_can_act(emu: &mut NesEmulator) -> bool` (in a round, Mac standing, not knocked down, not in a cutscene)
  - `pub fn fight_over(emu: &mut NesEmulator) -> bool`
  - `pub fn mac_down(emu: &mut NesEmulator) -> bool`
  - `pub fn opponent_down(emu: &mut NesEmulator) -> bool`
  - `pub fn stars(emu: &mut NesEmulator) -> u8`
  - `pub fn opponent_id(emu: &mut NesEmulator) -> u8`
- Produces (`start.rs`):
  - `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)] pub enum Opponent { GlassJoe, VonKaiser, PistonHonda, DonFlamenco, BaldBull }`
  - `pub const TRAIN: [Opponent; 3]`, `pub const HELD_OUT: [Opponent; 2]`
  - `impl Opponent { pub fn name(self) -> &'static str }` ("glass-joe", ...)
  - `pub fn fight_start(rom: &[u8], opponent: Opponent) -> Result<(NesState, usize), String>`: the state at the first frame Mac can act, and that frame number.

- [ ] **Step 1: Add a `probe` subcommand** that boots with the Task 2 Start taps until the fight begins (frame from the gate ledger), then for 3,600 frames: holds no buttons, writes `target/punchout/probe/ram.bin` (2,048 bytes per frame, frames concatenated) and every 30th frame as PPM. A second mode `probe --punch` holds `BUTTON_UP | BUTTON_A` on alternate 8-frame blocks, so the opponent takes damage.

- [ ] **Step 2: Find the addresses.**
  1. Health: in the idle probe, Mac's health bar shrinks when he is hit (visible in the PPMs). Candidate bytes are monotone non-increasing across the probe, with at least two drops, each drop at a frame where the PPMs show a hit. In the `--punch` probe, the opponent's health is the analogous byte. Cross-check against community memory maps (Data Crystal's Punch-Out!! RAM map); where they disagree, the probe wins.
  2. Knockdowns and "Mac can act": in the idle probe Mac is eventually knocked down; find the byte(s) that change at that moment and stay changed while he is on the canvas; "can act" is the fight-state value held while both fighters are up and the clock runs.
  3. Fight over: let the idle probe run on until the count-out; the byte that changes at the KO screen.
  4. Opponent id: enter the Major circuit password 005 737 5423 (Don Flamenco) from the title "Continue" screen; diff RAM at Don Flamenco's first can-act frame against Glass Joe's; the opponent id is a small integer that differs. Then poke Glass Joe's start with each candidate value (before the pre-fight intro) and check the PPM shows the other boxer.
  5. Stars: in the `--punch` probe, a byte that increments when a star icon appears.
  Write each address as a `const` in `ram.rs` with a one-line comment saying how it was found, and the accessors above.

- [ ] **Step 3: Write the failing tests** (`ram.rs` and `start.rs`; all use `rom_or_skip(PUNCH_OUT)`):

```rust
#[test]
fn idle_mac_loses_health_and_never_gains_it() {
  let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
  let (state, _) = fight_start(&rom, Opponent::GlassJoe).unwrap();
  let mut emu = NesEmulator::new(&rom);
  emu.load_state(&state);
  let start = mac_health(&mut emu);
  let mut last = start;
  let mut drops = 0;
  for _ in 0..3600 {
    emu.step_frames(0, 1);
    if !mac_can_act(&mut emu) { break; }
    let h = mac_health(&mut emu);
    assert!(h <= last, "health rose from {} to {}", last, h);
    if h < last { drops += 1; }
    last = h;
  }
  assert!(drops >= 2, "Glass Joe hit an idle Mac only {} times", drops);
}

#[test]
fn jabbing_lowers_the_opponents_health() {
  let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
  let (state, _) = fight_start(&rom, Opponent::GlassJoe).unwrap();
  let mut emu = NesEmulator::new(&rom);
  emu.load_state(&state);
  let start = opponent_health(&mut emu);
  for i in 0..600 {
    emu.step_frames(if i % 16 < 4 { BUTTON_UP | BUTTON_A } else { 0 }, 1);
  }
  assert!(opponent_health(&mut emu) < start);
}

#[test]
fn every_opponent_starts_with_mac_able_to_act_and_its_own_id() {
  let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
  let mut ids = std::collections::BTreeSet::new();
  for o in TRAIN.iter().chain(HELD_OUT.iter()) {
    let (state, frame) = fight_start(&rom, *o).unwrap();
    let mut emu = NesEmulator::new(&rom);
    emu.load_state(&state);
    assert!(mac_can_act(&mut emu), "{} start: Mac cannot act", o.name());
    assert!(frame > 0);
    ids.insert(opponent_id(&mut emu));
  }
  assert_eq!(ids.len(), 5, "five opponents, five ids");
}

#[test]
fn save_step_load_step_gives_the_same_frames() {
  let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
  let (state, _) = fight_start(&rom, Opponent::GlassJoe).unwrap();
  let mut emu = NesEmulator::new(&rom);
  emu.load_state(&state);
  emu.step_frames(0, 100);
  let mid = emu.save_state();
  emu.step_frames(BUTTON_LEFT, 40);
  let first = emu.framebuffer().to_vec();
  emu.load_state(&mid);
  emu.step_frames(BUTTON_LEFT, 40);
  assert_eq!(emu.framebuffer(), &first[..], "MMC2 state survives a save state");
}
```

- [ ] **Step 4: Run them; they fail** (functions are `todo!()` until Step 5).

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib punchout`
Expected: FAIL (`not yet implemented`).

- [ ] **Step 5: Implement** the accessors from the constants found in Step 2, and `fight_start`: boot; for Glass Joe, tap Start through the title (frame numbers from the probe); for the others, poke `opponent_id` to the opponent's value at the pre-intro point found in Step 2.4 (fallback, ledgered: circuit passwords); step one frame at a time until `mac_can_act`, at most 6,000 frames, else `Err("<name> never reached a fight")`; return `(emu.save_state(), frame)`.

- [ ] **Step 6: Run them**

Run: `RUSTFLAGS=-Awarnings cargo test --release --lib punchout`
Expected: all pass. Then save each opponent's start frame as PPM and read all five: each must show its own boxer.

- [ ] **Step 7: Format and commit**: "Map Punch-Out!!'s RAM and start fights against any opponent."

---

### Task 4: Moves, the fight loop and scores

**Files:**
- Create: `src/nes/gym/punchout/moves.rs`, `src/nes/gym/punchout/fight.rs`

**Interfaces:**
- Consumes: Task 3's `ram::*`, `fight_start`, `Opponent`; `guarded`.
- Produces (`moves.rs`):
  - `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)] pub enum Phase { Defence, Counter, Fight }` with `pub fn number(self) -> u8` (1, 2, 3) and `pub fn from_number(n: u8) -> Option<Phase>`
  - `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)] pub enum Move { Idle, DodgeLeft, DodgeRight, Duck, JabLeft, JabRight, Uppercut }`
  - `impl Move { pub fn buttons(self) -> u8; pub fn index_in(self, phase: Phase) -> usize }`
  - `pub fn moves(phase: Phase) -> &'static [Move]` (Defence: Idle, DodgeLeft, DodgeRight, Duck; Counter: + JabLeft, JabRight; Fight: + Uppercut)
  - `pub const MOVE_FRAMES: usize = 4;`
  - `pub struct Outcome { pub hit: bool, pub damage_taken: u8, pub damage_dealt: u8 }`
  - `pub fn play(emu: &mut NesEmulator, mv: Move) -> Outcome` (holds `mv.buttons()` for `MOVE_FRAMES` frames; health deltas measured across them)
- Produces (`fight.rs`):
  - `pub const MAX_STEPS: usize = 2400;`
  - `#[derive(Debug, Clone, Default, Serialize, Deserialize)] pub struct Score { pub steps: usize, pub active_steps: usize, pub hits: usize, pub damage_taken: u32, pub damage_dealt: u32, pub knockdowns_dealt: u32, pub mac_down_at: Option<usize>, pub won: bool }`
  - `impl Score { pub fn hits_per_minute(&self) -> f64; pub fn survival_seconds(&self) -> f64 }` (minutes = `active_steps * MOVE_FRAMES / 3600`; survival = `mac_down_at.unwrap_or(steps) * MOVE_FRAMES / 60`)
  - `pub struct Step<'a> { pub index: usize, pub emu: &'a mut NesEmulator, pub can_act: bool }`
  - `pub fn run_fight(rom: &[u8], opponent: Opponent, policy: &mut dyn FnMut(&mut Step) -> Move) -> Result<Score, String>`: loads `fight_start`, loops to `fight_over` or `MAX_STEPS`; it calls the policy on every step, so brains with memory see every frame, but when Mac cannot act it passes `can_act: false` and plays `Move::Idle` whatever the policy returns. The whole loop runs under `guarded`; a panic returns `Err`.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn moves_grow_by_phase_and_keep_their_indices() {
  assert_eq!(moves(Phase::Defence).len(), 4);
  assert_eq!(moves(Phase::Counter).len(), 6);
  assert_eq!(moves(Phase::Fight).len(), 7);
  for p in [Phase::Defence, Phase::Counter, Phase::Fight] {
    for (i, m) in moves(p).iter().enumerate() {
      assert_eq!(m.index_in(p), i);
    }
  }
  assert_eq!(&moves(Phase::Fight)[..4], moves(Phase::Defence));
}

#[test]
fn always_idle_gets_hit_and_scores_count_only_active_steps() {
  let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
  let mut asked_inactive = 0;
  let score = run_fight(&rom, Opponent::GlassJoe, &mut |s| {
    if !s.can_act { asked_inactive += 1; }
    Move::Idle
  }).unwrap();
  assert!(score.hits > 0);
  assert!(score.active_steps < score.steps, "knockdowns and intros are not active");
  assert!(asked_inactive > 0, "the policy still sees inactive steps");
  assert!(score.hits_per_minute() > 0.0);
}

#[test]
fn holding_a_move_changes_the_fight() {
  let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
  let idle = run_fight(&rom, Opponent::GlassJoe, &mut |_| Move::Idle).unwrap();
  let block = run_fight(&rom, Opponent::GlassJoe, &mut |_| Move::Duck).unwrap();
  assert_ne!(idle.hits_per_minute(), block.hits_per_minute(), "moves change the fight");
}
```

(The third test asserts only that moves matter: whether holding Duck beats Idle is the game's business, not ours.)

- [ ] **Step 2: Run; expect FAIL** (`todo!()`).
- [ ] **Step 3: Implement** as specified in Interfaces. Buttons: Idle 0; DodgeLeft LEFT; DodgeRight RIGHT; Duck DOWN; JabLeft UP|B; JabRight UP|A; Uppercut START. `won` = `fight_over` with `opponent_down`. `knockdowns_dealt` counts rising edges of `opponent_down`. `mac_down_at` = first step `mac_down` is true.
- [ ] **Step 4: Run; expect PASS.** Then print idle's `hits_per_minute` for each of the five opponents and ledger them (Task 7's stop gate and the dodge rate use them).
- [ ] **Step 5: Format and commit**: "Add Punch-Out!! moves, the fight loop and scores."

---

### Task 5: The projection and the brains

**Files:**
- Create: `src/nes/gym/punchout/brains.rs`
- Modify: `src/nes/gym/reservoir/soft.rs` (generalise `Expansion`; existing behaviour unchanged)

**Interfaces:**
- Consumes: `NesEmulator::framebuffer_grayscale_84x84`, `Expansion`, `Reservoir::new_fast`, `Reservoir::{step, playfield}`, `Drive::Placements`, `Encoder::Searched`, `Feature`, `feature_row`, the HUD row from the Task 2 ledger.
- Produces (`soft.rs`):
  - `pub fn shaped(seed: u64, inputs: usize, recurrent: bool, leak: f32) -> Expansion` (`new(seed, r)` becomes `shaped(seed, SOFT_INPUTS, r, LEAK)`)
  - `pub fn step_centred(&mut self, x: &[f32]) -> Vec<f32>` (`step` maps bytes to `b/255 - 0.5` and calls it)
- Produces (`brains.rs`):
  - `pub const PROJ: usize = 256; pub const PROJ_SEED: u64 = 0x9E3779B97F4A7C15;`
  - `pub const HUD_ROWS: usize` = ceil(HUD bottom row x 84 / 240) from the Task 2 ledger
  - `pub struct Projection` with `pub fn new() -> Projection` and `pub fn project(&self, grey84: &[u8]) -> Vec<f32>` (crops the first `HUD_ROWS` rows, centres pixels as `p/255 - 0.5`, multiplies by a fixed uniform matrix scaled so outputs have unit-order spread)
  - `#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)] pub enum BrainKind { Stack(usize), Echo(f32), Bank, Cheat }` with `pub fn name(self) -> String` ("stack1", "stack4", "echo0.5", "bank", "cheat")
  - `pub const CONTESTANTS: [BrainKind; 5] = [Stack(1), Stack(4), Echo(0.5), Bank, Cheat]`
  - `pub const SWEEP: [BrainKind; 7] = [Stack(1), Stack(2), Stack(4), Stack(8), Echo(0.2), Echo(0.5), Echo(0.9)]`
  - `pub struct Brain` with `pub fn new(kind: BrainKind, tetris_rom: &[u8]) -> Result<Brain, String>` and `pub fn row(&mut self, proj: &[f32], ram: &[u8]) -> Vec<f32>` (every row starts with `1.0`; `ram` is the 2,048-byte work RAM, read only by `Cheat`)
  - `pub fn bank_window(proj: &[f32]) -> [u8; 16]`

Rows:
- `Stack(d)`: `[1.0]` then the last `d` projections, newest first, zeros before the fight's first step. Length `1 + d * PROJ`.
- `Echo(leak)`: `Expansion::shaped(SOFT_SEED, PROJ, true, leak).step_centred(proj)`. Length `1 + 2048`.
- `Bank`: four `Reservoir::new_fast` boards; each step, board `b` is stepped with `Drive::Placements(Encoder::Searched(features_b))` on `bank_window(proj)`, where `features_b[k] = Feature { address: (b * 4 + k) as u16, relative: false, shift: 4 }`; the row is `feature_row(&concat(playfields), None)`. Length `1 + 800`.
- `bank_window`: byte `b*4+k` = `clamp(80 + 120 * mean(proj[b*64 + k*16 .. b*64 + k*16 + 16]), 0, 159)` (a mean of 16 unit-spread values has spread about 0.25, so the byte spans most of 0..159 and `>> 4` lands in columns 0..9).
- `Cheat`: `feature_row(ram, None)`. Length `1 + 2048`.

- [ ] **Step 1: Write the failing tests**

```rust
fn grey(seed: u8) -> Vec<u8> { (0..84 * 84).map(|i| (i as u32 * 37 + seed as u32 * 11) as u8).collect() }

#[test]
fn the_projection_ignores_the_hud_and_is_deterministic() {
  let p = Projection::new();
  let a = p.project(&grey(1));
  assert_eq!(a.len(), PROJ);
  assert_eq!(a, Projection::new().project(&grey(1)));
  let mut hud_changed = grey(1);
  for v in &mut hud_changed[..HUD_ROWS * 84] { *v = v.wrapping_add(90); }
  assert_eq!(p.project(&hud_changed), a, "the HUD is cropped");
  let spread = (a.iter().map(|v| v * v).sum::<f32>() / PROJ as f32).sqrt();
  assert!(spread > 0.1 && spread < 10.0, "spread {}", spread);
}

#[test]
fn a_frame_stack_holds_the_last_depth_projections_newest_first() {
  let mut b = Brain::new(BrainKind::Stack(2), &[]).unwrap();
  let one = vec![1.0; PROJ];
  let two = vec![2.0; PROJ];
  let r1 = b.row(&one, &[]);
  assert_eq!(r1.len(), 1 + 2 * PROJ);
  assert!(r1[1 + PROJ..].iter().all(|&v| v == 0.0), "nothing before the first step");
  let r2 = b.row(&two, &[]);
  assert_eq!(r2[1], 2.0);
  assert_eq!(r2[1 + PROJ], 1.0);
}

#[test]
fn the_echo_brain_has_memory_and_is_deterministic() {
  let mut a = Brain::new(BrainKind::Echo(0.5), &[]).unwrap();
  let mut b = Brain::new(BrainKind::Echo(0.5), &[]).unwrap();
  let x = vec![0.3; PROJ];
  let y = vec![-0.3; PROJ];
  assert_eq!(a.row(&x, &[]), b.row(&x, &[]));
  a.row(&y, &[]);
  b.row(&x, &[]);
  assert_ne!(a.row(&x, &[]), b.row(&x, &[]), "different histories, different states");
  assert_eq!(a.row(&x, &[]).len(), 1 + 2048);
}

#[test]
fn bank_window_bytes_cover_tetris_columns() {
  let lo = bank_window(&vec![-2.0; PROJ]);
  let hi = bank_window(&vec![2.0; PROJ]);
  assert!(lo.iter().all(|&v| v >> 4 == 0));
  assert!(hi.iter().all(|&v| v >> 4 == 9));
}

#[test]
fn the_bank_replays_identically() {
  let Some(tetris) = rom_or_skip("roms2/tetris.nes") else { return };
  let seq: Vec<Vec<f32>> = (0..6).map(|i| (0..PROJ).map(|j| ((i * 7 + j) % 13) as f32 / 6.0 - 1.0).collect()).collect();
  let run = || {
    let mut b = Brain::new(BrainKind::Bank, &tetris).unwrap();
    seq.iter().map(|p| b.row(p, &[])).collect::<Vec<_>>()
  };
  let first = run();
  assert_eq!(first, run(), "the bank is deterministic, so offline refits are valid");
  assert_eq!(first[0].len(), 1 + 800);
}

#[test]
fn the_cheat_reads_ram_and_nothing_else() {
  let mut b = Brain::new(BrainKind::Cheat, &[]).unwrap();
  let ram = vec![255u8; 2048];
  let r = b.row(&vec![0.0; PROJ], &ram);
  assert_eq!(r.len(), 1 + 2048);
  assert_eq!(r[1], 1.0);
}
```

Plus, in `soft.rs`: `shaped_with_smb_width_matches_new` (`Expansion::new(SOFT_SEED, true)` and `Expansion::shaped(SOFT_SEED, SOFT_INPUTS, true, 0.5)` give identical rows for the existing `window(1)`).

- [ ] **Step 2: Run; expect FAIL** (compile errors: missing items).
- [ ] **Step 3: Implement** as specified. In `soft.rs`, store `inputs` and `leak` in `Expansion`; keep the RNG draw order identical so `new` reproduces old reservoirs (the existing `soft` tests and the new equality test prove it). For `Stack(d)`, `Brain` keeps a `VecDeque<Vec<f32>>` of length `d`.
- [ ] **Step 4: Run** `RUSTFLAGS=-Awarnings cargo test --release --lib soft` and `... --lib punchout::brains`. Expected: PASS, and the existing `soft` tests still pass.
- [ ] **Step 5: Format and commit**: "Add the Punch-Out!! screen projection and its five brains."

---

### Task 6: The teacher

**Files:**
- Create: `src/nes/gym/punchout/teacher.rs`

**Interfaces:**
- Consumes: `moves::{play, moves, Move, Phase, MOVE_FRAMES}`, `ram::{mac_health, opponent_health, stars, mac_can_act}`.
- Produces:
  - `pub const HORIZON: usize = 60;`
  - `pub fn continuations(phase: Phase, stars: u8) -> Vec<Vec<Move>>` (Defence: `[[Idle; 14]]`; Counter: that plus `[JabRight, Idle, JabRight, Idle, ...]` to 14 moves; Fight: both plus, when `stars > 0`, `[Uppercut, Idle, ...]`). 14 = (HORIZON - MOVE_FRAMES) / MOVE_FRAMES.
  - `pub fn score(phase: Phase, taken: u32, dealt: u32) -> i64` (Defence: `-(taken as i64)` where `taken` counts hits; Counter/Fight: `dealt as i64 - 2 * taken as i64` where both are health points)
  - `pub fn label(emu: &mut NesEmulator, phase: Phase) -> Move`: saves the state, tries every move of the phase followed by every continuation, keeps each move's best score, restores the saved state, returns the best move; ties go to the earlier move in `moves(phase)` (Idle first).

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn the_teacher_leaves_the_emulator_as_it_found_it() {
  let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
  let (state, _) = fight_start(&rom, Opponent::GlassJoe).unwrap();
  let mut emu = NesEmulator::new(&rom);
  emu.load_state(&state);
  emu.step_frames(0, 200);
  let before = emu.save_state();
  label(&mut emu, Phase::Defence);
  emu.step_frames(0, 30);
  let after_label = emu.framebuffer().to_vec();
  emu.load_state(&before);
  emu.step_frames(0, 30);
  assert_eq!(emu.framebuffer(), &after_label[..]);
}

#[test]
fn the_teacher_dodges_before_a_hit_and_idles_otherwise() {
  let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
  let (state, _) = fight_start(&rom, Opponent::GlassJoe).unwrap();
  let mut emu = NesEmulator::new(&rom);
  emu.load_state(&state);
  // Walk an idle fight; remember the state a few steps before the first hit.
  let mut history = Vec::new();
  loop {
    history.push(emu.save_state());
    if play(&mut emu, Move::Idle).hit { break; }
    assert!(history.len() < 2000, "no hit in 2000 steps");
  }
  let before_hit = &history[history.len() - 3];
  emu.load_state(before_hit);
  assert_ne!(label(&mut emu, Phase::Defence), Move::Idle, "a punch is coming");
  emu.load_state(&history[0]);
  assert_eq!(label(&mut emu, Phase::Defence), Move::Idle, "nothing is coming at the bell");
}

#[test]
fn scores_weigh_damage_taken_twice() {
  assert_eq!(score(Phase::Defence, 2, 50), -2);
  assert_eq!(score(Phase::Counter, 3, 10), 4);
  assert!(continuations(Phase::Fight, 1).len() > continuations(Phase::Fight, 0).len());
}
```

(The dodge test uses the state 3 steps (12 frames) before the hit: inside the reaction window of a jab, and before the punch has landed. If Glass Joe's first hit cannot be avoided from 3 steps out, try 2 and 4 and ledger which one the test uses.)

- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement.** Defence counts `taken` as the number of `play` outcomes with `hit`; Counter/Fight sum `damage_taken` and `damage_dealt`. Stop a rollout early when `!mac_can_act` (a knockdown ends the line; count it as a hit in Defence).
- [ ] **Step 4: Run; expect PASS.** Time one `label` call in release and ledger it.
- [ ] **Step 5: Format and commit**: "Add the Punch-Out!! lookahead teacher."

---

### Task 7: DAgger

**Files:**
- Create: `src/nes/gym/punchout/dagger.rs`

**Interfaces:**
- Consumes: Tasks 3-6; `ridge::{fit, Readout}`.
- Produces:
  - `#[derive(Serialize, Deserialize)] pub struct FightRecord { pub opponent: Opponent, pub driver: String, pub round: usize, pub index: usize, pub projections: Vec<Vec<f32>>, pub labels: Vec<(usize, Move)>, pub rams: Vec<(usize, Vec<u8>)>, pub score: Score }` (`driver` = "teacher" or a brain name; `labels`/`rams` keyed by step; `rams` only at labelled steps)
  - `pub fn record_fight(rom: &[u8], opponent: Opponent, phase: Phase, driver: Driver, label_every: usize, threads: usize) -> Result<FightRecord, String>` where `pub enum Driver<'a> { Teacher, Student(&'a mut Brain, &'a Readout) }`. A teacher fight labels every can-act step (it must choose every move anyway). A student fight saves the state at every `label_every`-th can-act step, and after the fight labels the saved states on `threads` threads.
  - `pub fn rows_for(kind: BrainKind, record: &FightRecord, tetris: &[u8]) -> Result<(Vec<Vec<f32>>, Vec<Move>), String>`: replays `record.projections` through a fresh brain; returns rows and labels at labelled steps.
  - `pub fn fit_brain(kind: BrainKind, records: &[FightRecord], phase: Phase, tetris: &[u8]) -> Result<Readout, String>` (one-hot targets over `moves(phase)`, `lambda` = 1.0)
  - `pub fn choose(readout: &Readout, row: &[f32], phase: Phase) -> Move` (argmax; ties to the earlier move)
  - `pub struct Store { root: PathBuf }`: `pub fn new(root: &Path, phase: Phase) -> Store` (dir `target/punchout/phase<N>/`), `pub fn append(&self, file: &str, record: &FightRecord) -> Result<(), String>` (one JSON line, leading `\n` if the file ends unfinished, as `prospector::catalog::append` does), `pub fn load(&self, file: &str) -> Vec<FightRecord>` (skips unparseable lines)
  - `pub fn train(rom: &[u8], tetris: &[u8], phase: Phase, kinds: &[BrainKind], store: &Store, log: &mut dyn FnMut(&str)) -> Result<(), String>`:
    1. Round 0: `teacher.jsonl` must hold 3 fights per `TRAIN` opponent; record the missing ones (teacher fights in parallel, one per thread).
    2. For each kind, for rounds 1..=4: if `<kind>.jsonl` already holds that round's 3 fights, skip; else fit on teacher + this kind's fights so far, record 1 student fight per `TRAIN` opponent (label every 4th step), append. Kinds run in parallel threads.
    3. Stop gate (Defence only, after round 0): fit every kind on teacher fights, run one fight each against Glass Joe; if none beats always-Idle's hits per minute, return `Err("stop gate: ...")` with the numbers.
  - `pub fn test(rom, tetris, phase, kinds, store) -> Result<Value, String>`: fits each kind on all its labels; runs 3 fights per opponent (train and held out) plus always-Idle; returns and writes `results.json`: per kind, per opponent, per round (`round 0` = teacher-only fit, the imitation baseline, and `round 4` = final), the mean `Score` fields, hits per minute, dodge rate, survival seconds.

- [ ] **Step 1: Write the failing tests** (pure ones need no ROM):

```rust
fn record(round: usize, labels: usize) -> FightRecord {
  FightRecord {
    opponent: Opponent::GlassJoe, driver: "stack1".into(), round, index: 0,
    projections: vec![vec![0.0; PROJ]; labels * 4],
    labels: (0..labels).map(|i| (i * 4, Move::Idle)).collect(),
    rams: vec![], score: Score::default(),
  }
}

#[test]
fn resume_after_a_truncated_line_keeps_every_whole_record() {
  let dir = std::env::temp_dir().join(format!("punchout-store-{}", std::process::id()));
  let _ = std::fs::remove_dir_all(&dir);
  let store = Store::new(&dir, Phase::Defence);
  store.append("stack1.jsonl", &record(1, 2)).unwrap();
  let path = dir.join("phase1").join("stack1.jsonl");
  let mut f = std::fs::OpenOptions::new().append(true).open(&path).unwrap();
  std::io::Write::write_all(&mut f, b"{\"opponent\":").unwrap();
  store.append("stack1.jsonl", &record(2, 3)).unwrap();
  let back = store.load("stack1.jsonl");
  assert_eq!(back.iter().map(|r| r.round).collect::<Vec<_>>(), vec![1, 2]);
}

#[test]
fn rows_come_only_from_labelled_steps() {
  let (rows, labels) = rows_for(BrainKind::Stack(1), &record(0, 5), &[]).unwrap();
  assert_eq!(rows.len(), 5);
  assert_eq!(labels.len(), 5);
}

#[test]
fn choose_takes_the_argmax_and_breaks_ties_to_idle() {
  let r = Readout { d: 1, k: 4, weights: vec![0.0, 0.0, 0.0, 0.0], logistic: false };
  assert_eq!(choose(&r, &[1.0], Phase::Defence), Move::Idle);
  let r = Readout { d: 1, k: 4, weights: vec![0.0, 0.0, 1.0, 0.0], logistic: false };
  assert_eq!(choose(&r, &[1.0], Phase::Defence), Move::DodgeRight);
}

#[test]
fn the_label_set_grows_each_round() {
  let records = vec![record(0, 3), record(1, 2), record(2, 2)];
  let n = |upto: usize| records.iter().filter(|r| r.round <= upto).map(|r| r.labels.len()).sum::<usize>();
  assert!(n(0) < n(1) && n(1) < n(2));
  let fitted = fit_brain(BrainKind::Stack(1), &records, Phase::Defence, &[]).unwrap();
  assert_eq!(fitted.k, 4);
}

#[test]
fn a_student_fight_labels_only_steps_where_mac_can_act() {
  let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
  let mut brain = Brain::new(BrainKind::Stack(1), &[]).unwrap();
  let readout = Readout { d: 1 + PROJ, k: 4, weights: vec![0.0; (1 + PROJ) * 4], logistic: false };
  let rec = record_fight(&rom, Opponent::GlassJoe, Phase::Defence, Driver::Student(&mut brain, &readout), 4, 4).unwrap();
  assert_eq!(rec.projections.len(), rec.score.steps);
  assert!(!rec.labels.is_empty());
  assert!(rec.labels.len() <= rec.score.active_steps / 4 + 1);
}
```

- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement** as specified. Threads: `std::thread::scope`, each thread with its own `NesEmulator`; labelling parallelises over saved states. Every fight runs under `guarded`; an `Err` is logged and that fight is retried once with the next `index`, then skipped (ledger-visible in the log).
- [ ] **Step 4: Run; expect PASS.**
- [ ] **Step 5: Format and commit**: "Add DAgger for Punch-Out!!: fight records, resume, refits and the stop gate."

---

### Task 8: The command line and takes

**Files:**
- Modify: `examples/punchout.rs`
- Create: `src/nes/gym/punchout/takes.rs`

**Interfaces:**
- Consumes: Task 7's `train`, `test`, `Store`, `FightRecord`; `reservoir::film::TakeWriter::{new, frame, finish}`.
- Produces:
  - Subcommands: `gate`, `probe [--punch]`, `train --phase N [--sweep]`, `test --phase N [--sweep]`, `takes --phase N`. Phase 1 trains `CONTESTANTS`; `--sweep` trains/tests `SWEEP` (phase 1 only; error otherwise) and writes `target/punchout/sweep.json`.
  - `takes::record_take(rom, tetris, phase, kind, opponent, readout, out: &Path) -> Result<usize, String>`: plays one fight, writing stream `"punchout"` (the screen) and metadata per step `{step, move, can_act, mac_health, opponent_health, hit}` plus a summary (`Score`, kind, opponent, phase) via `finish`.
  - `takes --phase N`: for each contestant, its best-scoring held-out fight and its best training fight; for phase 1 also always-Idle against Glass Joe (the cold open). Output `target/punchout/takes/phase<N>-<kind>-<opponent>/`.

- [ ] **Step 1: Write the failing test** (`takes.rs`):

```rust
#[test]
fn a_take_has_one_metadata_step_per_screen_frame_group() {
  let Some(rom) = rom_or_skip(PUNCH_OUT) else { return };
  let dir = std::env::temp_dir().join(format!("punchout-take-{}", std::process::id()));
  let _ = std::fs::remove_dir_all(&dir);
  let readout = Readout { d: 1 + PROJ, k: 4, weights: vec![0.0; (1 + PROJ) * 4], logistic: false };
  let steps = record_take(&rom, &[], Phase::Defence, BrainKind::Stack(1), Opponent::GlassJoe, &readout, &dir).unwrap();
  let meta: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("meta.json")).unwrap()).unwrap();
  assert_eq!(meta["steps"].as_array().unwrap().len(), steps);
  assert_eq!(meta["streams"]["punchout"].as_u64().unwrap() as usize, steps * MOVE_FRAMES);
}
```

(Before writing it, read `film.rs::TakeWriter::finish` to confirm the metadata file name and the `streams` key; adjust the two lookups to match, and ledger it if they differ.)

- [ ] **Step 2: Run; expect FAIL.**
- [ ] **Step 3: Implement** the subcommands and `record_take`.
- [ ] **Step 4: Run; expect PASS.** Then smoke the CLI on a throwaway root: `train --phase 1` with an env var `PUNCHOUT_SMOKE=1` that cuts fights to 200 steps and rounds to 1 (honoured only by the CLI, not the library), check that `phase1/teacher.jsonl` and one brain file appear and that a second run skips the finished round.
- [ ] **Step 5: Format and commit**: "Add the Punch-Out!! command line and film takes."

---

### Task 9: Run it

**Files:**
- Modify: `docs/ideas/brain-cartridge.md` (Build order: a Punch-Out!! entry with results)

- [ ] **Step 1:** `RUSTFLAGS=-Awarnings cargo build --release --example punchout`, then in the background `target/release/examples/punchout train --phase 1 > <scratch>/train1.log 2>&1`. Expected: round 0, then the stop gate passes. If it stops: STOP and show Nate the gate's numbers.
- [ ] **Step 2:** In the background: `train --phase 1 --sweep`; when both finish, `test --phase 1` and `test --phase 1 --sweep`. Read `results.json` and `sweep.json`; ledger the headline numbers.
- [ ] **Step 3:** `train --phase 2`, `test --phase 2`; `train --phase 3`, `test --phase 3`. Read each `results.json`.
- [ ] **Step 4:** `takes --phase 1`, `--phase 2`, `--phase 3`. Convert two takes' first frames to PNG and look at them.
- [ ] **Step 5:** Add the ideas-doc entry: what was built (pointers to the spec, plan, `examples/punchout.rs`), each phase's headline (best brain and its dodge rate or wins against each held-out opponent, the imitation-only round-0 numbers next to round 4), and the sweep's answer to "does the game need more memory than four frames?". Commit: "Record Reservoir Punch-Out!!'s results in the ideas doc."
- [ ] **Step 6:** Tell Nate the results and propose the film plan.

**Budget note:** the Tetris bank dominates. Each step it places 4 pieces on each of 4 boards, about 700 Tetris frames per Punch-Out!! step, so a full bank fight is about 1.7 M frames, roughly 90 min on one core. Other brains cost about as much as the fight itself. Bank fights run in parallel with everything else; expect phase 1 (with sweep) to take most of a day of wall time and the whole run 1-2 days, all resumable.
