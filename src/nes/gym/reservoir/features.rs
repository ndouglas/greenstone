//! Feature rows for each experimental condition.

use super::drive::Drive;
use super::encode::{Encoder, ONEHOT_ROW_LEN};

/// Start of the SMB state window handed to the reservoir: zero page.
pub const SMB_WINDOW_START: u16 = 0x0000;
/// Length of the SMB state window.
pub const SMB_WINDOW_LEN: u16 = 256;
/// Tetris work RAM read as the reservoir state.
pub const TETRIS_RAM_LEN: u16 = 2048;
/// Number of drive bits appended for the controller-driven condition.
pub const DRIVE_BITS: usize = 8;
/// Slots in the clock condition's one-hot step index. Later steps clamp to the last slot.
pub const CLOCK_CAPACITY: usize = 1024;

/// Which features the readout sees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Condition {
  /// A one-hot of the step index and nothing else. The upper bound on what a
  /// readout can get from a clock, which is what undriven Tetris RAM amounts to.
  Clock,
  /// The SMB window itself. The real baseline.
  Raw,
  /// Tetris RAM with no input. The control.
  Undriven,
  /// Tetris RAM, Tetris driven through its controller by a fold of the window.
  DrivenController,
  /// Tetris RAM, the window copied into Tetris's playfield.
  DrivenPlayfield,
  /// The four placements as one-hot features, no Tetris. The control for the two below.
  PlacementOnehot,
  /// Tetris RAM after steering the four salient placements.
  PlacementTetris,
  /// Tetris RAM after steering placements hashed from the whole window.
  PlacementHash,
}

impl Condition {
  /// The default table.
  pub const ALL: [Condition; 5] = [
    Condition::Clock,
    Condition::Raw,
    Condition::PlacementOnehot,
    Condition::PlacementTetris,
    Condition::PlacementHash,
  ];
  /// The earlier drives, still runnable.
  pub const LEGACY: [Condition; 3] = [Condition::Undriven, Condition::DrivenController, Condition::DrivenPlayfield];

  pub fn name(self) -> &'static str {
    match self {
      Condition::Clock => "clock",
      Condition::Raw => "raw",
      Condition::Undriven => "undriven",
      Condition::DrivenController => "driven-controller",
      Condition::DrivenPlayfield => "driven-playfield",
      Condition::PlacementOnehot => "placement-onehot",
      Condition::PlacementTetris => "placement-tetris",
      Condition::PlacementHash => "placement-hash",
    }
  }

  pub fn drive(self) -> Option<Drive> {
    match self {
      Condition::DrivenController => Some(Drive::Controller),
      Condition::DrivenPlayfield => Some(Drive::Playfield),
      Condition::PlacementTetris => Some(Drive::Placements(Encoder::Salient)),
      Condition::PlacementHash => Some(Drive::Placements(Encoder::Hash)),
      _ => None,
    }
  }

  pub fn uses_reservoir(self) -> bool {
    !matches!(self, Condition::Raw | Condition::Clock | Condition::PlacementOnehot)
  }

  /// Placement drives play at level 19 so a piece drops in a few dozen frames.
  pub fn fast_tetris(self) -> bool {
    matches!(self.drive(), Some(Drive::Placements(_)))
  }

  pub fn appends_drive_bits(self) -> bool {
    matches!(self, Condition::DrivenController)
  }

  pub fn row_len(self) -> usize {
    match self {
      Condition::Clock => 1 + CLOCK_CAPACITY,
      Condition::PlacementOnehot => ONEHOT_ROW_LEN,
      _ => {
        let body = if self.uses_reservoir() { TETRIS_RAM_LEN } else { SMB_WINDOW_LEN } as usize;
        1 + body + if self.appends_drive_bits() { DRIVE_BITS } else { 0 }
      }
    }
  }
}

/// `[1.0, one-hot of step over CLOCK_CAPACITY slots]`.
pub fn clock_row(step: usize) -> Vec<f32> {
  let mut row = vec![0.0; 1 + CLOCK_CAPACITY];
  row[0] = 1.0;
  row[1 + step.min(CLOCK_CAPACITY - 1)] = 1.0;
  row
}

/// `[1.0, body / 255 ..., drive bit 0 .. drive bit 7]`.
pub fn feature_row(body: &[u8], drive_bits: Option<u8>) -> Vec<f32> {
  let mut row = Vec::with_capacity(1 + body.len() + DRIVE_BITS);
  row.push(1.0);
  row.extend(body.iter().map(|&b| b as f32 / 255.0));
  if let Some(bits) = drive_bits {
    for i in 0..DRIVE_BITS {
      row.push(((bits >> i) & 1) as f32);
    }
  }
  row
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn row_starts_with_bias_and_scales_bytes() {
    let row = feature_row(&[0, 255, 51], None);
    assert_eq!(row.len(), 4);
    assert_eq!(row[0], 1.0);
    assert_eq!(row[1], 0.0);
    assert_eq!(row[2], 1.0);
    assert!((row[3] - 0.2).abs() < 1e-6);
  }

  #[test]
  fn drive_bits_are_appended_low_bit_first() {
    let row = feature_row(&[9], Some(0b1000_0001));
    assert_eq!(row.len(), 1 + 1 + DRIVE_BITS);
    assert_eq!(row[2], 1.0);
    assert_eq!(row[3], 0.0);
    assert_eq!(row[9], 1.0);
  }

  #[test]
  fn clock_row_is_a_one_hot_of_the_step_index_after_the_bias() {
    let row = clock_row(3);
    assert_eq!(row.len(), 1 + CLOCK_CAPACITY);
    assert_eq!(row[0], 1.0);
    assert_eq!(row[1 + 3], 1.0);
    assert_eq!(row.iter().filter(|&&v| v != 0.0).count(), 2);
  }

  #[test]
  fn clock_row_clamps_steps_past_capacity_to_the_last_slot() {
    let row = clock_row(CLOCK_CAPACITY + 50);
    assert_eq!(row[1 + CLOCK_CAPACITY - 1], 1.0);
    assert_eq!(row.iter().filter(|&&v| v != 0.0).count(), 2);
  }

  #[test]
  fn row_lengths_match_conditions() {
    assert_eq!(Condition::Clock.row_len(), 1 + CLOCK_CAPACITY);
    assert_eq!(Condition::Raw.row_len(), 1 + 256);
    assert_eq!(Condition::Undriven.row_len(), 1 + 2048);
    assert_eq!(Condition::DrivenController.row_len(), 1 + 2048 + 8);
    assert_eq!(Condition::DrivenPlayfield.row_len(), 1 + 2048);
    assert_eq!(Condition::PlacementOnehot.row_len(), ONEHOT_ROW_LEN);
    assert_eq!(Condition::PlacementTetris.row_len(), 1 + 2048);
    assert_eq!(Condition::PlacementHash.row_len(), 1 + 2048);
  }

  #[test]
  fn reservoir_use_and_speed_follow_the_drive() {
    for c in [Condition::Clock, Condition::Raw, Condition::PlacementOnehot] {
      assert!(!c.uses_reservoir(), "{}", c.name());
      assert!(c.drive().is_none());
    }
    for c in [Condition::PlacementTetris, Condition::PlacementHash] {
      assert!(c.uses_reservoir(), "{}", c.name());
      assert!(c.fast_tetris(), "{}", c.name());
    }
    for c in Condition::LEGACY {
      assert!(c.uses_reservoir());
      assert!(!c.fast_tetris());
    }
    assert_eq!(Condition::PlacementTetris.drive(), Some(Drive::Placements(Encoder::Salient)));
    assert_eq!(Condition::PlacementHash.drive(), Some(Drive::Placements(Encoder::Hash)));
  }

  #[test]
  fn names_are_distinct_across_default_and_legacy() {
    let mut names: Vec<&str> = Condition::ALL.iter().chain(Condition::LEGACY.iter()).map(|c| c.name()).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), Condition::ALL.len() + Condition::LEGACY.len());
    assert_eq!(Condition::ALL.len(), 5);
    assert_eq!(Condition::LEGACY.len(), 3);
  }
}
