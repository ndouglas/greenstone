//! NES action space for RL agents.
//!
//! Maps discrete action indices to NES controller button combinations.
//! Multiple predefined action spaces are provided, from minimal (movement only)
//! to full (all 256 possible button combinations).

use crate::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT, BUTTON_UP};

/// A predefined set of meaningful NES button combinations.
///
/// Different games benefit from different action spaces. A platformer like
/// Super Mario Bros needs A (jump), B (run), and directions, while a puzzle
/// game might only need directions and A.
#[derive(Debug, Clone)]
pub struct ActionSpace {
  /// Human-readable name of this action space.
  pub name: &'static str,
  /// The button combinations available, indexed by action ID.
  actions: Vec<u8>,
}

impl ActionSpace {
  /// Number of discrete actions in this space.
  pub fn size(&self) -> usize {
    self.actions.len()
  }

  /// Convert an action index to the NES button byte.
  /// Panics if `action >= self.size()`.
  pub fn to_buttons(&self, action: usize) -> u8 {
    self.actions[action]
  }

  /// Find the action index for a given button combination.
  /// Returns `None` if the combination isn't in this action space.
  pub fn from_buttons(&self, buttons: u8) -> Option<usize> {
    self.actions.iter().position(|&b| b == buttons)
  }

  /// Create a custom action space from arbitrary button combinations.
  pub fn custom(name: &'static str, actions: Vec<u8>) -> Self {
    ActionSpace { name, actions }
  }
}

/// Minimal action space: 6 actions.
/// NOOP, Up, Down, Left, Right, A
///
/// Good for simple games or initial experiments.
pub fn minimal() -> ActionSpace {
  ActionSpace {
    name: "minimal",
    actions: vec![
      0,            // 0: NOOP
      BUTTON_UP,    // 1: Up
      BUTTON_DOWN,  // 2: Down
      BUTTON_LEFT,  // 3: Left
      BUTTON_RIGHT, // 4: Right
      BUTTON_A,     // 5: A
    ],
  }
}

/// Standard platformer action space: 12 actions.
/// Covers the useful combinations for side-scrolling platformers
/// like Super Mario Bros, Mega Man, etc.
pub fn platformer() -> ActionSpace {
  ActionSpace {
    name: "platformer",
    actions: vec![
      0,                                  // 0: NOOP
      BUTTON_RIGHT,                       // 1: Right
      BUTTON_RIGHT | BUTTON_A,            // 2: Right + A (run-jump right)
      BUTTON_RIGHT | BUTTON_B,            // 3: Right + B (run right)
      BUTTON_RIGHT | BUTTON_A | BUTTON_B, // 4: Right + A + B (run-jump right)
      BUTTON_A,                           // 5: A (jump)
      BUTTON_LEFT,                        // 6: Left
      BUTTON_LEFT | BUTTON_A,             // 7: Left + A (run-jump left)
      BUTTON_LEFT | BUTTON_B,             // 8: Left + B (run left)
      BUTTON_LEFT | BUTTON_A | BUTTON_B,  // 9: Left + A + B (run-jump left)
      BUTTON_DOWN,                        // 10: Down (duck/pipe)
      BUTTON_UP,                          // 11: Up (climb/door)
    ],
  }
}

/// Full standard action space: 16 actions.
/// All single-button presses plus the most useful combinations.
/// Based on the "RIGHT_ONLY" / "SIMPLE_MOVEMENT" / "COMPLEX_MOVEMENT"
/// hierarchy used in gym-retro and stable-retro.
pub fn full() -> ActionSpace {
  ActionSpace {
    name: "full",
    actions: vec![
      0,                                  // 0: NOOP
      BUTTON_A,                           // 1: A
      BUTTON_B,                           // 2: B
      BUTTON_UP,                          // 3: Up
      BUTTON_DOWN,                        // 4: Down
      BUTTON_LEFT,                        // 5: Left
      BUTTON_RIGHT,                       // 6: Right
      BUTTON_A | BUTTON_B,                // 7: A + B
      BUTTON_UP | BUTTON_A,               // 8: Up + A
      BUTTON_DOWN | BUTTON_A,             // 9: Down + A
      BUTTON_LEFT | BUTTON_A,             // 10: Left + A
      BUTTON_RIGHT | BUTTON_A,            // 11: Right + A
      BUTTON_LEFT | BUTTON_B,             // 12: Left + B
      BUTTON_RIGHT | BUTTON_B,            // 13: Right + B
      BUTTON_RIGHT | BUTTON_A | BUTTON_B, // 14: Right + A + B
      BUTTON_LEFT | BUTTON_A | BUTTON_B,  // 15: Left + A + B
    ],
  }
}

/// Brute-force action space: all 256 possible button combinations.
/// Probably only useful for exhaustive testing or MCTS-style search.
pub fn all_combinations() -> ActionSpace {
  ActionSpace {
    name: "all_combinations",
    actions: (0..=255u8).collect(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::bus::{BUTTON_SELECT, BUTTON_START};

  #[test]
  fn test_minimal_space() {
    let space = minimal();
    assert_eq!(space.size(), 6);
    assert_eq!(space.to_buttons(0), 0); // NOOP
    assert_eq!(space.to_buttons(5), BUTTON_A);
  }

  #[test]
  fn test_platformer_space() {
    let space = platformer();
    assert_eq!(space.size(), 12);
    // Right + A + B should be action 4
    assert_eq!(space.to_buttons(4), BUTTON_RIGHT | BUTTON_A | BUTTON_B);
  }

  #[test]
  fn test_full_space() {
    let space = full();
    assert_eq!(space.size(), 16);
    // NOOP is action 0
    assert_eq!(space.to_buttons(0), 0);
  }

  #[test]
  fn test_all_combinations() {
    let space = all_combinations();
    assert_eq!(space.size(), 256);
    for i in 0..256 {
      assert_eq!(space.to_buttons(i), i as u8);
    }
  }

  #[test]
  fn test_from_buttons_roundtrip() {
    let space = platformer();
    for i in 0..space.size() {
      let buttons = space.to_buttons(i);
      assert_eq!(space.from_buttons(buttons), Some(i));
    }
    // Button combo not in space
    assert_eq!(space.from_buttons(BUTTON_SELECT), None);
  }

  #[test]
  fn test_custom_space() {
    let space = ActionSpace::custom("test", vec![0, BUTTON_A, BUTTON_START]);
    assert_eq!(space.size(), 3);
    assert_eq!(space.name, "test");
    assert_eq!(space.to_buttons(2), BUTTON_START);
  }
}
