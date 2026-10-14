//! Mario's window as tetromino placements.

use super::trace::enemies;

/// Placements sent to Tetris per SMB step.
pub const PIECES_PER_STEP: usize = 4;
/// Enemy distance bin width in pixels. Nine bins cover 288 pixels; the tenth column means "none".
pub const ENEMY_BIN_PX: i32 = 32;
/// Features per placement in the one-hot row: ten columns and four rotations.
pub const PLACEMENT_ONEHOT: usize = 14;
/// Intercept plus one-hot of every placement.
pub const ONEHOT_ROW_LEN: usize = 1 + PIECES_PER_STEP * PLACEMENT_ONEHOT;

const MARIO_X_PAGE: usize = 0x006D;
const MARIO_X_PIXEL: usize = 0x0086;
const MARIO_Y: usize = 0x00CE;
const MARIO_X_SPEED: usize = 0x0057;
const MARIO_Y_SPEED: usize = 0x009F;
const MARIO_AIRBORNE: usize = 0x001D;
const MARIO_STATE: usize = 0x000E;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
  /// Target column for the piece's x register, 0..=9.
  pub column: u8,
  /// Target rotation index into the piece's A-rotation cycle, 0..=3.
  pub rotation: u8,
}

/// One window byte binned into a placement: a candidate for the encoder search.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Feature {
  /// Window index of the byte: zero page first, then the terrain buffer.
  pub address: u16,
  /// Subtract Mario's x pixel first, so the byte reads as a distance from Mario.
  pub relative: bool,
  /// Bits dropped before the column: the bin width is `1 << shift`.
  pub shift: u8,
}

impl Feature {
  pub fn placement(self, window: &[u8]) -> Placement {
    let mut v = window[self.address as usize];
    if self.relative {
      v = v.wrapping_sub(window[MARIO_X_PIXEL]);
    }
    let column = (v >> self.shift).min(9);
    let rotation = (v >> self.shift.saturating_sub(2)) & 3;
    Placement { column, rotation }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoder {
  /// Four hand-chosen features: the two nearest enemies ahead, Mario's height, Mario's speed.
  Salient,
  /// A hash of the whole window. Control: shows that locality in the encoding matters.
  Hash,
  /// Four features chosen by `search::search_features`.
  Searched([Feature; PIECES_PER_STEP]),
}

fn mario_x(window: &[u8]) -> i32 {
  window[MARIO_X_PAGE] as i32 * 256 + window[MARIO_X_PIXEL] as i32
}

/// Distance bin for an enemy `ahead` pixels in front of Mario, or 9 for none.
fn distance_column(ahead: Option<i32>) -> u8 {
  match ahead {
    Some(d) => (d / ENEMY_BIN_PX).clamp(0, 8) as u8,
    None => 9,
  }
}

fn y_band(y: u8) -> u8 {
  y / 64
}

/// The enemies ahead of Mario, nearest first, as (distance, y).
fn enemies_ahead(window: &[u8]) -> Vec<(i32, u8)> {
  let mx = mario_x(window);
  let mut ahead: Vec<(i32, u8)> = enemies(window).into_iter().filter(|e| e.x > mx).map(|e| (e.x - mx, e.y)).collect();
  ahead.sort();
  ahead
}

fn salient(window: &[u8]) -> [Placement; PIECES_PER_STEP] {
  let ahead = enemies_ahead(window);
  let enemy = |i: usize| match ahead.get(i) {
    Some(&(d, y)) => Placement {
      column: distance_column(Some(d)),
      rotation: y_band(y),
    },
    None => Placement { column: 9, rotation: 0 },
  };
  let airborne = (window[MARIO_AIRBORNE] != 0) as u8;
  let rising = ((window[MARIO_Y_SPEED] as i8) < 0) as u8;
  let height = Placement {
    column: (window[MARIO_Y] / 24).min(9),
    rotation: airborne * 2 + rising,
  };
  let speed = window[MARIO_X_SPEED] as i8 as i32;
  let motion = Placement {
    column: ((speed + 40) / 8).clamp(0, 9) as u8,
    rotation: window[MARIO_STATE] & 3,
  };
  [enemy(0), enemy(1), height, motion]
}

fn hash(window: &[u8]) -> [Placement; PIECES_PER_STEP] {
  let mut out = [Placement { column: 0, rotation: 0 }; PIECES_PER_STEP];
  for (i, slot) in out.iter_mut().enumerate() {
    let mut h = 0xCBF2_9CE4_8422_2325u64 ^ (i as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    for &b in window {
      h = (h ^ b as u64).wrapping_mul(0x100_0000_01B3);
    }
    *slot = Placement {
      column: (h % 10) as u8,
      rotation: ((h >> 8) % 4) as u8,
    };
  }
  out
}

/// Four placements for one SMB window.
pub fn encode(encoder: Encoder, window: &[u8]) -> [Placement; PIECES_PER_STEP] {
  match encoder {
    Encoder::Salient => salient(window),
    Encoder::Hash => hash(window),
    Encoder::Searched(features) => features.map(|f| f.placement(window)),
  }
}

/// `[1.0, then per placement: ten column bits, four rotation bits]`.
pub fn onehot_row(placements: &[Placement]) -> Vec<f32> {
  let mut row = vec![0.0; 1 + placements.len() * PLACEMENT_ONEHOT];
  row[0] = 1.0;
  for (i, p) in placements.iter().enumerate() {
    let base = 1 + i * PLACEMENT_ONEHOT;
    row[base + p.column as usize] = 1.0;
    row[base + 10 + p.rotation as usize] = 1.0;
  }
  row
}

#[cfg(test)]
mod tests {
  use super::*;

  fn window_with_mario(x: i32, y: u8) -> Vec<u8> {
    let mut w = vec![0u8; 256];
    w[MARIO_X_PAGE] = (x / 256) as u8;
    w[MARIO_X_PIXEL] = (x % 256) as u8;
    w[MARIO_Y] = y;
    w
  }

  fn add_enemy(w: &mut [u8], slot: usize, x: i32, y: u8) {
    w[0x000F + slot] = 1;
    w[0x0016 + slot] = 0x06;
    w[0x006E + slot] = (x / 256) as u8;
    w[0x0087 + slot] = (x % 256) as u8;
    w[0x00CF + slot] = y;
  }

  #[test]
  fn nearest_enemy_ahead_sets_the_first_column_in_32_pixel_bins() {
    let mut w = window_with_mario(1000, 176);
    add_enemy(&mut w, 0, 1000 + 70, 0x80);
    add_enemy(&mut w, 1, 1000 + 200, 0x40);
    add_enemy(&mut w, 2, 900, 0x80); // behind Mario: ignored
    let p = encode(Encoder::Salient, &w);
    assert_eq!(p[0].column, 2, "70 px ahead is bin 2");
    assert_eq!(p[0].rotation, 2, "y 0x80 is band 2 of four");
    assert_eq!(p[1].column, 6, "200 px ahead is bin 6");
    assert_eq!(p[1].rotation, 1);
  }

  #[test]
  fn no_enemy_ahead_means_column_nine() {
    let mut w = window_with_mario(1000, 176);
    add_enemy(&mut w, 0, 900, 0x80);
    let p = encode(Encoder::Salient, &w);
    assert_eq!(p[0].column, 9);
    assert_eq!(p[1].column, 9);
    assert_eq!(p[0].rotation, 0);
  }

  #[test]
  fn far_enemies_clamp_to_the_last_distance_bin() {
    let mut w = window_with_mario(0, 176);
    add_enemy(&mut w, 0, 5000, 0);
    assert_eq!(encode(Encoder::Salient, &w)[0].column, 8);
  }

  #[test]
  fn height_speed_and_state_fill_the_last_two_placements() {
    let mut w = window_with_mario(0, 96);
    w[MARIO_AIRBORNE] = 1;
    w[MARIO_Y_SPEED] = 0xFC; // rising
    w[MARIO_X_SPEED] = 0x28; // 40, full run speed
    w[MARIO_STATE] = 0x08;
    let p = encode(Encoder::Salient, &w);
    assert_eq!(p[2].column, 4, "y 96 in 24-pixel bins");
    assert_eq!(p[2].rotation, 3, "airborne and rising");
    assert_eq!(p[3].column, 9, "speed 40 is the top bin");
    assert_eq!(p[3].rotation, 0, "state 8 & 3");
    w[MARIO_X_SPEED] = 0xD8; // -40
    w[MARIO_Y] = 255;
    assert_eq!(encode(Encoder::Salient, &w)[3].column, 0);
    assert_eq!(encode(Encoder::Salient, &w)[2].column, 9, "y clamps to the last bin");
  }

  #[test]
  fn a_feature_bins_one_byte_optionally_relative_to_mario() {
    let mut w = window_with_mario(0x0120, 176);
    w[0x87] = 0x60; // enemy x pixel
    let absolute = Feature {
      address: 0x87,
      relative: false,
      shift: 4,
    };
    assert_eq!(
      absolute.placement(&w),
      Placement { column: 6, rotation: 0 },
      "0x60 >> 4 = 6, bits below are 0"
    );
    let relative = Feature {
      address: 0x87,
      relative: true,
      shift: 4,
    };
    // 0x60 - 0x20 = 0x40: column 4, rotation from the two bits under the column bits.
    assert_eq!(relative.placement(&w), Placement { column: 4, rotation: 0 });
    w[0x87] = 0x2C;
    assert_eq!(
      relative.placement(&w),
      Placement { column: 0, rotation: 3 },
      "0x0C: column 0, bits 3..2 are 11"
    );
    let wide = Feature {
      address: 0x87,
      relative: false,
      shift: 1,
    };
    assert_eq!(wide.placement(&w).column, 9, "0x2C >> 1 = 22 clamps to 9");
  }

  #[test]
  fn searched_encoder_applies_its_four_features() {
    let mut w = window_with_mario(0, 0);
    w[0x10] = 0x30;
    let f = Feature {
      address: 0x10,
      relative: false,
      shift: 4,
    };
    let p = encode(Encoder::Searched([f; PIECES_PER_STEP]), &w);
    assert!(p.iter().all(|p| p.column == 3));
  }

  #[test]
  fn hash_encoder_is_deterministic_in_range_and_sensitive_to_every_byte() {
    let w = vec![7u8; 256];
    let a = encode(Encoder::Hash, &w);
    assert_eq!(a, encode(Encoder::Hash, &w));
    for p in a {
      assert!(p.column <= 9 && p.rotation <= 3);
    }
    let mut w2 = w.clone();
    w2[200] = 8;
    assert_ne!(a, encode(Encoder::Hash, &w2));
  }

  #[test]
  fn onehot_row_has_an_intercept_and_two_hot_bits_per_placement() {
    let p = [Placement { column: 3, rotation: 1 }; PIECES_PER_STEP];
    let row = onehot_row(&p);
    assert_eq!(row.len(), ONEHOT_ROW_LEN);
    assert_eq!(row[0], 1.0);
    assert_eq!(row.iter().filter(|&&v| v == 1.0).count(), 1 + 2 * PIECES_PER_STEP);
    assert_eq!(row[1 + 3], 1.0);
    assert_eq!(row[1 + 10 + 1], 1.0);
    assert_eq!(row[1 + PLACEMENT_ONEHOT + 3], 1.0);
  }
}
