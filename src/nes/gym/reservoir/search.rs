//! Greedy search for the four window features that best predict the buttons
//! when sent as one-hot placements, which needs no Tetris to score.

use super::demo::{smb_in_cutscene, smb_start_state, smb_window, Demonstration, MACRO_FRAMES};
use super::encode::{onehot_row, Encoder, Feature, Placement, PIECES_PER_STEP};
use super::{buttons_to_targets, evaluate_folds, row_is_informative, Dataset};
use crate::nes::gym::NesEmulator;

/// Bin widths tried, as bits dropped: 4, 8, 16 and 32 values per column.
pub const SHIFTS: [u8; 4] = [2, 3, 4, 5];

/// Every window byte, absolute and relative to Mario's x, at every width.
pub fn candidates() -> Vec<Feature> {
  let mut out = Vec::with_capacity(256 * 2 * SHIFTS.len());
  for address in 0..=255u8 {
    for relative in [false, true] {
      for shift in SHIFTS {
        out.push(Feature { address, relative, shift });
      }
    }
  }
  out
}

/// The informative windows and buttons of a demonstration, replayed on SMB alone.
pub fn collect_windows(smb_rom: &[u8], demo: &Demonstration) -> (Vec<Vec<u8>>, Vec<u8>) {
  let mut smb = NesEmulator::new(smb_rom);
  smb.load_state(&smb_start_state(smb_rom));
  smb.step_frames(0, demo.noops);
  let mut windows = Vec::new();
  let mut buttons = Vec::new();
  for &action in &demo.buttons {
    let window = smb_window(&mut smb);
    if row_is_informative(smb_in_cutscene(&mut smb)) {
      windows.push(window);
      buttons.push(action);
    }
    smb.step_frames(action, MACRO_FRAMES);
  }
  (windows, buttons)
}

fn datasets(features: &[Feature], windows: &[Vec<Vec<u8>>], buttons: &[Vec<u8>]) -> Vec<Dataset> {
  windows
    .iter()
    .zip(buttons)
    .map(|(ws, bs)| Dataset {
      rows: ws
        .iter()
        .map(|w| {
          let placements: Vec<Placement> = features.iter().map(|f| f.placement(w)).collect();
          onehot_row(&placements)
        })
        .collect(),
      targets: bs.iter().map(|&b| buttons_to_targets(b)).collect(),
      buttons: bs.clone(),
      resets: 0,
    })
    .collect()
}

/// Leave-one-demonstration-out bit accuracy of a one-hot readout on these features.
pub fn score(features: &[Feature], windows: &[Vec<Vec<u8>>], buttons: &[Vec<u8>], lambda: f64, mask: u8) -> Result<f64, String> {
  evaluate_folds(&datasets(features, windows, buttons), lambda, mask).map(|m| m.accuracy)
}

/// Greedy forward selection of `PIECES_PER_STEP` features by held-out accuracy.
/// `log` receives one line per pick.
pub fn search_features(
  windows: &[Vec<Vec<u8>>],
  buttons: &[Vec<u8>],
  lambda: f64,
  mask: u8,
  log: &mut dyn FnMut(&str),
) -> [Feature; PIECES_PER_STEP] {
  let pool = candidates();
  let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
  let mut chosen: Vec<Feature> = Vec::new();
  for _ in 0..PIECES_PER_STEP {
    let chunk = pool.len().div_ceil(threads);
    let best: (f64, Feature) = std::thread::scope(|s| {
      let handles: Vec<_> = pool
        .chunks(chunk)
        .map(|chunk| {
          let chosen = &chosen;
          s.spawn(move || {
            let mut best: Option<(f64, Feature)> = None;
            for &f in chunk {
              if chosen.iter().any(|c| c.address == f.address && c.relative == f.relative) {
                continue;
              }
              let mut trial = chosen.clone();
              trial.push(f);
              let acc = score(&trial, windows, buttons, lambda, mask).unwrap_or(0.0);
              if best.map_or(true, |(b, _)| acc > b) {
                best = Some((acc, f));
              }
            }
            best
          })
        })
        .collect();
      handles
        .into_iter()
        .filter_map(|h| h.join().expect("search worker panicked"))
        .fold(None, |acc: Option<(f64, Feature)>, b| match acc {
          Some(a) if a.0 >= b.0 => Some(a),
          _ => Some(b),
        })
        .expect("at least one candidate")
    });
    log(&format!(
      "pick {}: ${:02X}{} width {}: held-out accuracy {:.3}",
      chosen.len() + 1,
      best.1.address,
      if best.1.relative { " - mario x" } else { "" },
      1 << best.1.shift,
      best.0
    ));
    chosen.push(best.1);
  }
  chosen.try_into().expect("PIECES_PER_STEP features")
}

/// Search the encoder on the given demonstrations.
pub fn search_encoder(smb_rom: &[u8], demos: &[Demonstration], lambda: f64, mask: u8, log: &mut dyn FnMut(&str)) -> Encoder {
  let (windows, buttons): (Vec<_>, Vec<_>) = demos.iter().map(|d| collect_windows(smb_rom, d)).unzip();
  Encoder::Searched(search_features(&windows, &buttons, lambda, mask, log))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_RIGHT};

  /// Three demonstrations where A is pressed exactly when byte 0x33 is 0x50 or more.
  fn synthetic() -> (Vec<Vec<Vec<u8>>>, Vec<Vec<u8>>) {
    let mut windows = Vec::new();
    let mut buttons = Vec::new();
    for demo in 0..3u32 {
      let mut ws = Vec::new();
      let mut bs = Vec::new();
      for i in 0..120u32 {
        let mut w = vec![0u8; 256];
        for (a, b) in w.iter_mut().enumerate() {
          *b = ((a as u32 * 31 + i * 17 + demo * 101) % 256) as u8;
        }
        w[0x33] = ((i * 7919 + demo * 13) % 256) as u8;
        let a = if w[0x33] >= 0x50 { BUTTON_A } else { 0 };
        ws.push(w);
        bs.push(BUTTON_RIGHT | BUTTON_B | a);
      }
      windows.push(ws);
      buttons.push(bs);
    }
    (windows, buttons)
  }

  #[test]
  fn candidates_cover_every_address_both_ways_at_four_widths() {
    let c = candidates();
    assert_eq!(c.len(), 256 * 2 * 4);
    assert!(c.contains(&Feature {
      address: 0x87,
      relative: true,
      shift: 5
    }));
  }

  #[test]
  fn greedy_search_finds_the_byte_that_decides_the_jump() {
    let (windows, buttons) = synthetic();
    let features = search_features(&windows, &buttons, 1.0, BUTTON_A | BUTTON_B | BUTTON_RIGHT, &mut |_| {});
    assert_eq!(features.len(), PIECES_PER_STEP);
    assert_eq!(features[0].address, 0x33, "first pick {:?}", features[0]);
    assert!(!features[0].relative);
    let distinct: std::collections::HashSet<_> = features.iter().collect();
    assert_eq!(distinct.len(), PIECES_PER_STEP, "no feature is picked twice");
  }

  #[test]
  fn fold_score_of_the_deciding_byte_beats_a_noise_byte() {
    let (windows, buttons) = synthetic();
    let mask = BUTTON_A | BUTTON_B | BUTTON_RIGHT;
    let good = score(
      &[Feature {
        address: 0x33,
        relative: false,
        shift: 4,
      }],
      &windows,
      &buttons,
      1.0,
      mask,
    )
    .unwrap();
    let noise = score(
      &[Feature {
        address: 0x44,
        relative: false,
        shift: 4,
      }],
      &windows,
      &buttons,
      1.0,
      mask,
    )
    .unwrap();
    assert!(good > noise + 0.1, "good {} noise {}", good, noise);
  }
}
