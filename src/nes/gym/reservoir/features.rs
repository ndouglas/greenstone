//! Feature rows for each experimental condition.

use super::drive::Drive;
#[cfg(test)]
use super::encode::Feature;
use super::encode::{Encoder, ONEHOT_ROW_LEN};

/// Start of the SMB state window handed to the reservoir: zero page.
pub const SMB_WINDOW_START: u16 = 0x0000;
/// The zero page, the first part of the window.
pub const SMB_ZERO_PAGE_LEN: u16 = 256;
/// SMB's metatile buffer: two screens of terrain columns, the second part of
/// the window. Pits, pipes and stairs live here and nowhere in the zero page.
pub const SMB_TERRAIN_START: u16 = 0x0500;
pub const SMB_TERRAIN_LEN: u16 = 0x01A0;
/// Length of the SMB state window: zero page then terrain.
pub const SMB_WINDOW_LEN: usize = SMB_ZERO_PAGE_LEN as usize + SMB_TERRAIN_LEN as usize;
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
  /// No readout: run right and press A by a two-state chain fitted to the
  /// demonstrations' jump statistics. The closed-loop floor.
  RandomA,
  /// Tetris RAM with no input. The control.
  Undriven,
  /// Tetris RAM, Tetris driven through its controller by a fold of the window.
  DrivenController,
  /// Tetris RAM, the window copied into Tetris's playfield.
  DrivenPlayfield,
  /// The encoder's four placements as one-hot features, no Tetris. The control for the next.
  PlacementOnehot(Encoder),
  /// Tetris RAM after steering the encoder's four placements.
  PlacementTetris(Encoder),
}

impl Condition {
  /// The earlier drives, still runnable.
  pub const LEGACY: [Condition; 3] = [Condition::Undriven, Condition::DrivenController, Condition::DrivenPlayfield];

  /// The default table: floors, raw, the hand-picked encoder straight and
  /// through Tetris, the searched encoder likewise when given, and the hash control.
  pub fn default_table(searched: Option<Encoder>) -> Vec<Condition> {
    let mut table = vec![
      Condition::Clock,
      Condition::Raw,
      Condition::RandomA,
      Condition::PlacementOnehot(Encoder::Salient),
      Condition::PlacementTetris(Encoder::Salient),
    ];
    if let Some(e) = searched {
      table.push(Condition::PlacementOnehot(e));
      table.push(Condition::PlacementTetris(e));
    }
    table.push(Condition::PlacementTetris(Encoder::Hash));
    table
  }

  pub fn name(self) -> String {
    let encoder = |e: Encoder| match e {
      Encoder::Salient => "placement",
      Encoder::Hash => "hashed",
      Encoder::Searched(_) => "searched",
    };
    match self {
      Condition::Clock => "clock".to_string(),
      Condition::Raw => "raw".to_string(),
      Condition::RandomA => "random-a".to_string(),
      Condition::Undriven => "undriven".to_string(),
      Condition::DrivenController => "driven-controller".to_string(),
      Condition::DrivenPlayfield => "driven-playfield".to_string(),
      Condition::PlacementOnehot(e) => format!("{}-onehot", encoder(e)),
      Condition::PlacementTetris(Encoder::Hash) => "placement-hash".to_string(),
      Condition::PlacementTetris(e) => format!("{}-tetris", encoder(e)),
    }
  }

  pub fn drive(self) -> Option<Drive> {
    match self {
      Condition::DrivenController => Some(Drive::Controller),
      Condition::DrivenPlayfield => Some(Drive::Playfield),
      Condition::PlacementTetris(e) => Some(Drive::Placements(e)),
      Condition::Clock | Condition::Raw | Condition::RandomA | Condition::Undriven | Condition::PlacementOnehot(_) => None,
    }
  }

  pub fn uses_reservoir(self) -> bool {
    matches!(
      self,
      Condition::Undriven | Condition::DrivenController | Condition::DrivenPlayfield | Condition::PlacementTetris(_)
    )
  }

  /// Placement drives play at level 19 so a piece drops in a few dozen frames.
  pub fn fast_tetris(self) -> bool {
    matches!(self.drive(), Some(Drive::Placements(_)))
  }

  pub fn appends_drive_bits(self) -> bool {
    matches!(self, Condition::DrivenController)
  }

  /// The one-hot encoder's placements for this row, when the condition has one.
  pub fn onehot_encoder(self) -> Option<Encoder> {
    match self {
      Condition::PlacementOnehot(e) => Some(e),
      _ => None,
    }
  }

  pub fn row_len(self) -> usize {
    match self {
      Condition::Clock | Condition::RandomA => 1 + CLOCK_CAPACITY,
      Condition::Raw => 1 + SMB_WINDOW_LEN,
      Condition::PlacementOnehot(_) => ONEHOT_ROW_LEN,
      Condition::DrivenController => 1 + TETRIS_RAM_LEN as usize + DRIVE_BITS,
      Condition::Undriven | Condition::DrivenPlayfield | Condition::PlacementTetris(_) => 1 + TETRIS_RAM_LEN as usize,
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
    assert_eq!(Condition::Raw.row_len(), 1 + 256 + 416);
    assert_eq!(Condition::RandomA.row_len(), 1 + CLOCK_CAPACITY);
    assert_eq!(Condition::Undriven.row_len(), 1 + 2048);
    assert_eq!(Condition::DrivenController.row_len(), 1 + 2048 + 8);
    assert_eq!(Condition::DrivenPlayfield.row_len(), 1 + 2048);
    assert_eq!(Condition::PlacementOnehot(Encoder::Salient).row_len(), ONEHOT_ROW_LEN);
    assert_eq!(Condition::PlacementTetris(Encoder::Salient).row_len(), 1 + 2048);
    assert_eq!(Condition::PlacementTetris(Encoder::Hash).row_len(), 1 + 2048);
  }

  #[test]
  fn reservoir_use_and_speed_follow_the_drive() {
    for c in [
      Condition::Clock,
      Condition::Raw,
      Condition::RandomA,
      Condition::PlacementOnehot(Encoder::Hash),
    ] {
      assert!(!c.uses_reservoir(), "{}", c.name());
      assert!(c.drive().is_none());
    }
    for c in [Condition::PlacementTetris(Encoder::Salient), Condition::PlacementTetris(Encoder::Hash)] {
      assert!(c.uses_reservoir(), "{}", c.name());
      assert!(c.fast_tetris(), "{}", c.name());
    }
    for c in Condition::LEGACY {
      assert!(c.uses_reservoir());
      assert!(!c.fast_tetris());
    }
    assert_eq!(
      Condition::PlacementTetris(Encoder::Salient).drive(),
      Some(Drive::Placements(Encoder::Salient))
    );
  }

  #[test]
  fn default_table_names_are_distinct_and_searched_rows_are_optional() {
    let f = Feature {
      address: 0x87,
      relative: true,
      shift: 4,
    };
    let searched = Encoder::Searched([f; 4]);
    let with = Condition::default_table(Some(searched));
    let without = Condition::default_table(None);
    assert_eq!(with.len(), without.len() + 2);
    let mut names: Vec<String> = with.iter().chain(Condition::LEGACY.iter()).map(|c| c.name()).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), with.len() + Condition::LEGACY.len());
    assert_eq!(Condition::PlacementTetris(Encoder::Hash).name(), "placement-hash");
    assert_eq!(Condition::PlacementOnehot(searched).name(), "searched-onehot");
    assert!(without.contains(&Condition::RandomA));
  }
}
