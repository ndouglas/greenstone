//! Feature rows for each experimental condition.

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
}

impl Condition {
  pub const ALL: [Condition; 5] = [
    Condition::Clock,
    Condition::Raw,
    Condition::Undriven,
    Condition::DrivenController,
    Condition::DrivenPlayfield,
  ];

  pub fn name(self) -> &'static str {
    match self {
      Condition::Clock => "clock",
      Condition::Raw => "raw",
      Condition::Undriven => "undriven",
      Condition::DrivenController => "driven-controller",
      Condition::DrivenPlayfield => "driven-playfield",
    }
  }

  pub fn drive(self) -> Option<super::drive::Drive> {
    match self {
      Condition::DrivenController => Some(super::drive::Drive::Controller),
      Condition::DrivenPlayfield => Some(super::drive::Drive::Playfield),
      _ => None,
    }
  }

  pub fn uses_reservoir(self) -> bool {
    !matches!(self, Condition::Raw | Condition::Clock)
  }

  pub fn appends_drive_bits(self) -> bool {
    matches!(self, Condition::DrivenController)
  }

  pub fn row_len(self) -> usize {
    if self == Condition::Clock {
      return 1 + CLOCK_CAPACITY;
    }
    let body = if self.uses_reservoir() { TETRIS_RAM_LEN } else { SMB_WINDOW_LEN } as usize;
    1 + body + if self.appends_drive_bits() { DRIVE_BITS } else { 0 }
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
  }

  #[test]
  fn only_clock_and_raw_skip_the_reservoir() {
    assert!(!Condition::Clock.uses_reservoir());
    assert!(!Condition::Raw.uses_reservoir());
    for c in Condition::ALL.iter().filter(|c| **c != Condition::Raw && **c != Condition::Clock) {
      assert!(c.uses_reservoir(), "{}", c.name());
    }
  }

  #[test]
  fn names_are_distinct() {
    let mut names: Vec<&str> = Condition::ALL.iter().map(|c| c.name()).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), Condition::ALL.len());
    assert_eq!(Condition::ALL.len(), 5);
  }
}
