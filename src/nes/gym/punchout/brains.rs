//! What the brains see: a fixed random projection of the screen, and the
//! five ways of turning it into a readout's row.

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::nes::gym::reservoir::drive::{Drive, Reservoir};
use crate::nes::gym::reservoir::encode::{Encoder, Feature};
use crate::nes::gym::reservoir::features::feature_row;
use crate::nes::gym::reservoir::soft::{Expansion, SOFT_SEED};
use crate::nes::gym::reservoir::Lcg;

/// Numbers the projection makes from each screen.
pub const PROJ: usize = 256;
/// The projection's one fixed seed.
pub const PROJ_SEED: u64 = 0x9E3779B97F4A7C15;
/// Rows of the 84x84 screen that hold the HUD, cropped before projecting:
/// the HUD ends at screen row 44 of 240 (Task 2), and ceil(44 * 84 / 240) = 16.
pub const HUD_ROWS: usize = 16;

/// Pixels the projection reads: the 84x84 screen below the HUD.
const SEEN: usize = (84 - HUD_ROWS) * 84;
/// Typical spread of a centred pixel; dividing by it gives outputs of unit order.
const PIXEL_SPREAD: f32 = 0.3;

/// A fixed uniform random matrix from the screen below the HUD to `PROJ` numbers.
pub struct Projection {
  matrix: Vec<f32>,
}

impl Projection {
  pub fn new() -> Projection {
    let mut rng = Lcg::new(PROJ_SEED);
    // Uniform(-1, 1) has variance 1/3, so this scale gives each output about
    // the spread of one pixel, scaled up to unit order.
    let scale = (3.0 / SEEN as f32).sqrt() / PIXEL_SPREAD;
    let matrix = (0..PROJ * SEEN).map(|_| (rng.next_f64() * 2.0 - 1.0) as f32 * scale).collect();
    Projection { matrix }
  }

  /// `PROJ` numbers from an 84x84 grey screen, the HUD cropped off.
  pub fn project(&self, grey84: &[u8]) -> Vec<f32> {
    let x: Vec<f32> = grey84[HUD_ROWS * 84..].iter().map(|&p| p as f32 / 255.0 - 0.5).collect();
    self.matrix.chunks(SEEN).map(|w| w.iter().zip(&x).map(|(a, b)| a * b).sum()).collect()
  }
}

impl Default for Projection {
  fn default() -> Projection {
    Projection::new()
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BrainKind {
  /// The last `d` projections side by side.
  Stack(usize),
  /// A 2,048-unit echo state network with this leak.
  Echo(f32),
  /// Four Tetris boards driven by the projection.
  Bank,
  /// The game's own RAM: the reference no screen-reader should beat.
  Cheat,
}

impl BrainKind {
  pub fn name(self) -> String {
    match self {
      BrainKind::Stack(d) => format!("stack{}", d),
      BrainKind::Echo(leak) => format!("echo{}", leak),
      BrainKind::Bank => "bank".to_string(),
      BrainKind::Cheat => "cheat".to_string(),
    }
  }
}

/// The four contestants of every phase, and the RAM reference.
pub const CONTESTANTS: [BrainKind; 5] = [
  BrainKind::Stack(1),
  BrainKind::Stack(4),
  BrainKind::Echo(0.5),
  BrainKind::Bank,
  BrainKind::Cheat,
];

/// Phase one's memory sweep.
pub const SWEEP: [BrainKind; 7] = [
  BrainKind::Stack(1),
  BrainKind::Stack(2),
  BrainKind::Stack(4),
  BrainKind::Stack(8),
  BrainKind::Echo(0.2),
  BrainKind::Echo(0.5),
  BrainKind::Echo(0.9),
];

/// Tetris boards in the bank, one per quarter of the projection.
const BOARDS: usize = 4;

/// A brain's memory, if it has any, from one fight's steps so far.
pub enum Brain {
  Stack(usize, VecDeque<Vec<f32>>),
  Echo(Box<Expansion>),
  Bank(Vec<Reservoir>),
  Cheat,
}

impl Brain {
  pub fn new(kind: BrainKind, tetris_rom: &[u8]) -> Result<Brain, String> {
    Ok(match kind {
      BrainKind::Stack(d) => Brain::Stack(d, (0..d).map(|_| vec![0.0; PROJ]).collect()),
      BrainKind::Echo(leak) => Brain::Echo(Box::new(Expansion::shaped(SOFT_SEED, PROJ, true, leak))),
      BrainKind::Bank => Brain::Bank((0..BOARDS).map(|_| Reservoir::new_fast(tetris_rom)).collect::<Result<_, _>>()?),
      BrainKind::Cheat => Brain::Cheat,
    })
  }

  /// This step's row for the readout: `[1.0, ...]`. `ram` is the game's
  /// 2,048 bytes of work RAM, read only by `Cheat`.
  pub fn row(&mut self, proj: &[f32], ram: &[u8]) -> Vec<f32> {
    match self {
      Brain::Stack(d, past) => {
        past.push_front(proj.to_vec());
        past.truncate(*d);
        let mut row = Vec::with_capacity(1 + *d * PROJ);
        row.push(1.0);
        for p in past.iter() {
          row.extend_from_slice(p);
        }
        row
      }
      Brain::Echo(expansion) => expansion.step_centred(proj),
      Brain::Bank(boards) => {
        let window = bank_window(proj);
        let mut cells = Vec::new();
        for (b, board) in boards.iter_mut().enumerate() {
          let features = std::array::from_fn(|k| Feature {
            address: (b * 4 + k) as u16,
            relative: false,
            shift: 4,
          });
          board.step(Some(Drive::Placements(Encoder::Searched(features))), &window, 0);
          cells.extend(board.playfield());
        }
        feature_row(&cells, None)
      }
      Brain::Cheat => feature_row(ram, None),
    }
  }
}

/// The bank's 16-byte window: byte `b * 4 + k` is the mean of a 16-number
/// slice of the projection, mapped so `>> 4` lands on a Tetris column.
pub fn bank_window(proj: &[f32]) -> [u8; 16] {
  std::array::from_fn(|i| {
    let slice = &proj[i * 16..i * 16 + 16];
    let mean = slice.iter().sum::<f32>() / 16.0;
    (80.0 + 120.0 * mean).clamp(0.0, 159.0) as u8
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::gym::reservoir::rom_or_skip;

  fn grey(seed: u8) -> Vec<u8> {
    (0..84 * 84).map(|i| (i as u32 * 37 + seed as u32 * 11) as u8).collect()
  }

  #[test]
  fn the_projection_ignores_the_hud_and_is_deterministic() {
    let p = Projection::new();
    let a = p.project(&grey(1));
    assert_eq!(a.len(), PROJ);
    assert_eq!(a, Projection::new().project(&grey(1)));
    let mut hud_changed = grey(1);
    for v in &mut hud_changed[..HUD_ROWS * 84] {
      *v = v.wrapping_add(90);
    }
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
    let seq: Vec<Vec<f32>> = (0..6)
      .map(|i| (0..PROJ).map(|j| ((i * 7 + j) % 13) as f32 / 6.0 - 1.0).collect())
      .collect();
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

  #[test]
  fn names_are_as_the_tables_print_them() {
    let names: Vec<String> = CONTESTANTS.iter().map(|k| k.name()).collect();
    assert_eq!(names, ["stack1", "stack4", "echo0.5", "bank", "cheat"]);
  }
}
