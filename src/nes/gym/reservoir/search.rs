//! Greedy search for the four window features that best predict the buttons
//! when sent as one-hot placements, which needs no Tetris to score.

use super::demo::{smb_in_cutscene, smb_start_state_at, smb_window, Demonstration, MACRO_FRAMES};
use super::encode::{onehot_row, BankEncoder, Encoder, Feature, Placement, BANK_BOARDS, ENEMY_DISTANCE, PIECES_PER_STEP};
use super::features::{BLOCK_ROWS, SMB_WINDOW_LEN, SMB_ZERO_PAGE_LEN};
use super::{buttons_to_targets, evaluate_folds, row_is_informative, smooth_a, Dataset, FoldMetrics};
use crate::nes::gym::NesEmulator;

/// Bin widths tried, as bits dropped: 4, 8, 16 and 32 values per column.
pub const SHIFTS: [u8; 4] = [2, 3, 4, 5];

/// SMB zero page: Mario's x page and x pixel. Not candidates, because on one
/// level his position predicts where the teacher jumped.
pub const POSITION_BYTES: [u16; 2] = [0x006D, 0x0086];
/// SMB zero page: the five enemy slots' x pixels, the only bytes worth
/// reading relative to Mario's x. Any other byte minus his x is his position
/// in disguise, and the search picked exactly that.
pub const ENEMY_X_BYTES: std::ops::Range<u16> = 0x0087..0x008C;

/// Every window byte but Mario's position, absolute, at every width; the
/// enemy x bytes also relative to Mario; and the nearest-enemy distance.
pub fn candidates() -> Vec<Feature> {
  let mut out = Vec::with_capacity((SMB_WINDOW_LEN + ENEMY_X_BYTES.len() + 1) * SHIFTS.len());
  for address in 0..SMB_WINDOW_LEN as u16 {
    if POSITION_BYTES.contains(&address) {
      continue;
    }
    for shift in SHIFTS {
      out.push(Feature {
        address,
        relative: false,
        shift,
      });
    }
    if ENEMY_X_BYTES.contains(&address) {
      for shift in SHIFTS {
        out.push(Feature {
          address,
          relative: true,
          shift,
        });
      }
    }
  }
  for shift in SHIFTS {
    out.push(Feature {
      address: ENEMY_DISTANCE,
      relative: false,
      shift,
    });
  }
  out
}

/// SMB zero page: the enemy slot arrays, five bytes each.
const ENEMY_ARRAYS: [u16; 5] = [0x000F, 0x0016, 0x006E, 0x0087, 0x00CF];
/// Terrain columns ahead of Mario in the near region; the rest are far.
const NEAR_COLUMNS: usize = 4;

/// Every window byte, absolute and relative to Mario's x, Mario's own
/// position bytes included, and the enemy distance: the pool before the
/// fix, where a byte minus Mario's x is his position in disguise. Only the
/// film uses it, to show the memoriser.
pub fn candidates_with_position() -> Vec<Feature> {
  let mut out = Vec::new();
  for address in 0..SMB_WINDOW_LEN as u16 {
    for relative in [false, true] {
      for shift in SHIFTS {
        out.push(Feature { address, relative, shift });
      }
    }
  }
  for shift in SHIFTS {
    out.push(Feature {
      address: ENEMY_DISTANCE,
      relative: false,
      shift,
    });
  }
  out
}

/// A part of the window, for a board of the bank to watch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
  /// The zero page less the enemy arrays and Mario's position.
  Mario,
  /// The enemy slot arrays, the enemy x bytes relative to Mario, and the nearest-enemy distance.
  Enemies,
  /// Terrain columns 0 to 3 ahead of Mario.
  Near,
  /// Terrain columns 4 to 7.
  Far,
}

impl Region {
  pub const ALL: [Region; 4] = [Region::Mario, Region::Enemies, Region::Near, Region::Far];

  pub fn name(self) -> &'static str {
    match self {
      Region::Mario => "mario",
      Region::Enemies => "enemies",
      Region::Near => "near",
      Region::Far => "far",
    }
  }

  /// Whether a feature of the pool belongs to this region.
  pub fn holds(self, f: Feature) -> bool {
    let enemy = f.address == ENEMY_DISTANCE || ENEMY_ARRAYS.iter().any(|&a| (a..a + 5).contains(&f.address));
    let terrain = if f.address == ENEMY_DISTANCE {
      None
    } else {
      (f.address as usize).checked_sub(SMB_ZERO_PAGE_LEN as usize)
    };
    match self {
      Region::Enemies => enemy,
      Region::Mario => !enemy && terrain.is_none(),
      Region::Near => terrain.map_or(false, |t| t / BLOCK_ROWS < NEAR_COLUMNS),
      Region::Far => terrain.map_or(false, |t| t / BLOCK_ROWS >= NEAR_COLUMNS),
    }
  }

  pub fn candidates(self) -> Vec<Feature> {
    candidates().into_iter().filter(|&f| self.holds(f)).collect()
  }
}

/// The informative windows and buttons of a demonstration, replayed on SMB alone.
pub fn collect_windows(smb_rom: &[u8], demo: &Demonstration) -> (Vec<Vec<u8>>, Vec<u8>) {
  let mut smb = NesEmulator::new(smb_rom);
  smb.load_state(&smb_start_state_at(smb_rom, demo.stage));
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

/// Leave-one-demonstration-out metrics of a one-hot readout on these features.
pub fn score(
  features: &[Feature],
  windows: &[Vec<Vec<u8>>],
  buttons: &[Vec<u8>],
  lambda: f64,
  logistic: bool,
  mask: u8,
) -> Result<FoldMetrics, String> {
  evaluate_folds(&datasets(features, windows, buttons), lambda, logistic, mask)
}

/// The best next pick from `pool` given `chosen`, by onset score across
/// threads: (score, feature, A specificity). Bytes already chosen are skipped.
fn best_pick(
  chosen: &[Feature],
  pool: &[Feature],
  windows: &[Vec<Vec<u8>>],
  buttons: &[Vec<u8>],
  lambda: f64,
  logistic: bool,
  mask: u8,
) -> (f64, Feature, f64) {
  let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
  let chunk = pool.len().div_ceil(threads).max(1);
  std::thread::scope(|s| {
    let handles: Vec<_> = pool
      .chunks(chunk)
      .map(|chunk| {
        s.spawn(move || {
          let mut best: Option<(f64, Feature, f64)> = None;
          for &f in chunk {
            if chosen.iter().any(|c| c.address == f.address && c.relative == f.relative) {
              continue;
            }
            let mut trial = chosen.to_vec();
            trial.push(f);
            let (onset, spec) = score(&trial, windows, buttons, lambda, logistic, mask)
              .map(|m| (m.onset_score(), m.a_specificity))
              .unwrap_or((0.0, 0.0));
            if best.map_or(true, |(b, _, _)| onset > b) {
              best = Some((onset, f, spec));
            }
          }
          best
        })
      })
      .collect();
    handles
      .into_iter()
      .filter_map(|h| h.join().expect("search worker panicked"))
      .fold(None, |acc: Option<(f64, Feature, f64)>, b| match acc {
        Some(a) if a.0 >= b.0 => Some(a),
        _ => Some(b),
      })
      .expect("at least one candidate")
  })
}

/// Greedy forward selection of `PIECES_PER_STEP` features by held-out onset
/// score (`FoldMetrics::onset_score`). `log` receives one line per pick.
pub fn search_features(
  windows: &[Vec<Vec<u8>>],
  buttons: &[Vec<u8>],
  lambda: f64,
  logistic: bool,
  mask: u8,
  log: &mut dyn FnMut(&str),
) -> [Feature; PIECES_PER_STEP] {
  search_features_in(&candidates(), windows, buttons, lambda, logistic, mask, log)
}

/// `search_features` over a given pool.
pub fn search_features_in(
  pool: &[Feature],
  windows: &[Vec<Vec<u8>>],
  buttons: &[Vec<u8>],
  lambda: f64,
  logistic: bool,
  mask: u8,
  log: &mut dyn FnMut(&str),
) -> [Feature; PIECES_PER_STEP] {
  let mut chosen: Vec<Feature> = Vec::new();
  for _ in 0..PIECES_PER_STEP {
    let best = best_pick(&chosen, pool, windows, buttons, lambda, logistic, mask);
    log(&format!(
      "pick {}: {}: held-out onsets {:.3}, A specificity {:.3}",
      chosen.len() + 1,
      best.1.describe(),
      best.0 - best.2,
      best.2
    ));
    chosen.push(best.1);
  }
  chosen.try_into().expect("PIECES_PER_STEP features")
}

/// Greedy forward selection of four features per region, jointly: each
/// round takes the best next feature from any region that still has room,
/// scored with everything picked so far as one one-hot row. Returns each
/// region's features in pick order, in `Region::ALL` order.
pub fn search_bank(
  windows: &[Vec<Vec<u8>>],
  buttons: &[Vec<u8>],
  lambda: f64,
  logistic: bool,
  mask: u8,
  log: &mut dyn FnMut(&str),
) -> [[Feature; PIECES_PER_STEP]; BANK_BOARDS] {
  let pools: Vec<Vec<Feature>> = Region::ALL.iter().map(|r| r.candidates()).collect();
  let mut boards: Vec<Vec<Feature>> = vec![Vec::new(); BANK_BOARDS];
  let mut chosen: Vec<Feature> = Vec::new();
  for round in 0..PIECES_PER_STEP * BANK_BOARDS {
    let pool: Vec<Feature> = pools
      .iter()
      .zip(&boards)
      .filter(|(_, board)| board.len() < PIECES_PER_STEP)
      .flat_map(|(pool, _)| pool.iter().copied())
      .collect();
    let best = best_pick(&chosen, &pool, windows, buttons, lambda, logistic, mask);
    let region = Region::ALL.iter().position(|r| r.holds(best.1)).expect("a pool feature has a region");
    log(&format!(
      "bank pick {}: {} ({}): held-out onsets {:.3}, A specificity {:.3}",
      round + 1,
      best.1.describe(),
      Region::ALL[region].name(),
      best.0 - best.2,
      best.2
    ));
    boards[region].push(best.1);
    chosen.push(best.1);
  }
  let boards: Vec<[Feature; PIECES_PER_STEP]> = boards.into_iter().map(|b| b.try_into().expect("four features per region")).collect();
  boards.try_into().expect("BANK_BOARDS boards")
}

/// The windows and smoothed buttons of every demonstration, for the searches.
pub fn demo_windows(smb_rom: &[u8], demos: &[Demonstration], horizon: usize) -> (Vec<Vec<Vec<u8>>>, Vec<Vec<u8>>) {
  demos
    .iter()
    .map(|d| {
      let (w, b) = collect_windows(smb_rom, d);
      (w, smooth_a(&b, horizon))
    })
    .unzip()
}

/// Search the encoder on the given demonstrations, against the smoothed A label.
pub fn search_encoder(
  smb_rom: &[u8],
  demos: &[Demonstration],
  lambda: f64,
  logistic: bool,
  mask: u8,
  horizon: usize,
  log: &mut dyn FnMut(&str),
) -> Encoder {
  let (windows, buttons) = demo_windows(smb_rom, demos, horizon);
  Encoder::Searched(search_features(&windows, &buttons, lambda, logistic, mask, log))
}

/// Search four features per region on the given demonstrations.
pub fn search_bank_encoder(
  smb_rom: &[u8],
  demos: &[Demonstration],
  lambda: f64,
  logistic: bool,
  mask: u8,
  horizon: usize,
  log: &mut dyn FnMut(&str),
) -> BankEncoder {
  let (windows, buttons) = demo_windows(smb_rom, demos, horizon);
  BankEncoder::Searched(search_bank(&windows, &buttons, lambda, logistic, mask, log))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_RIGHT};

  #[test]
  fn the_permissive_pool_admits_marios_position_and_the_default_does_not() {
    let wide = candidates_with_position();
    assert!(wide.iter().any(|f| f.address == 0x86 && !f.relative));
    assert!(wide.iter().any(|f| f.address == 0x6D));
    assert!(wide.iter().any(|f| f.address == 0x30 && f.relative), "relative reads of any byte");
    assert!(!candidates().iter().any(|f| f.address == 0x86 || f.address == 0x6D));
    assert!(wide.len() > candidates().len());
  }

  /// Three demonstrations where A is pressed exactly when byte 0x33 is 0x50 or more.
  fn synthetic() -> (Vec<Vec<Vec<u8>>>, Vec<Vec<u8>>) {
    let mut windows = Vec::new();
    let mut buttons = Vec::new();
    for demo in 0..3u32 {
      let mut ws = Vec::new();
      let mut bs = Vec::new();
      for i in 0..120u32 {
        let mut w = vec![0u8; SMB_WINDOW_LEN];
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

  /// Three demonstrations where a four-step press starts whenever byte 0x40
  /// is high, and byte 0x41 lags the press by one step. The lagging byte wins
  /// on accuracy (two errors per press to the trigger's three) and never sees
  /// an onset; the trigger byte sees every onset.
  fn trigger_and_lag() -> (Vec<Vec<Vec<u8>>>, Vec<Vec<u8>>) {
    let mut windows = Vec::new();
    let mut buttons = Vec::new();
    for demo in 0..3u32 {
      let mut ws = Vec::new();
      let mut bs = Vec::new();
      let mut hold = 0usize;
      let mut prev_a = false;
      for i in 0..120u32 {
        let mut w = vec![0u8; SMB_WINDOW_LEN];
        for (a, b) in w.iter_mut().enumerate() {
          *b = ((a as u32 * 31 + i * 17 + demo * 101) % 256) as u8;
        }
        let trigger = hold == 0 && (i + demo * 5) % 12 == 0;
        if trigger {
          hold = 4;
        }
        let a = hold > 0;
        w[0x40] = if trigger { 0xFF } else { 0 };
        w[0x41] = if prev_a { 0xFF } else { 0 };
        ws.push(w);
        bs.push(BUTTON_RIGHT | BUTTON_B | if a { BUTTON_A } else { 0 });
        prev_a = a;
        hold = hold.saturating_sub(1);
      }
      windows.push(ws);
      buttons.push(bs);
    }
    (windows, buttons)
  }

  #[test]
  fn the_search_is_scored_by_onsets_so_it_picks_the_trigger_over_the_lagging_byte() {
    let (windows, buttons) = trigger_and_lag();
    let mask = BUTTON_A | BUTTON_B | BUTTON_RIGHT;
    let feature = |address: u16| Feature {
      address,
      relative: false,
      shift: 4,
    };
    let trigger = score(&[feature(0x40)], &windows, &buttons, 1.0, false, mask).unwrap();
    let lag = score(&[feature(0x41)], &windows, &buttons, 1.0, false, mask).unwrap();
    assert!(lag.accuracy > trigger.accuracy, "lag {} trigger {}", lag.accuracy, trigger.accuracy);
    assert!(trigger.onset_recall > 0.9, "trigger onsets {}", trigger.onset_recall);
    assert!(lag.onset_recall < 0.1, "lag onsets {}", lag.onset_recall);
    assert!(lag.a_specificity < 0.95, "the lag byte presses A at every release: {}", lag.a_specificity);
    assert!(trigger.a_specificity > 0.95, "trigger specificity {}", trigger.a_specificity);
    assert!(trigger.onset_score() > lag.onset_score());
    let features = search_features(&windows, &buttons, 1.0, false, mask, &mut |_| {});
    assert_eq!(features[0].address, 0x40, "first pick {:?}", features[0]);
  }

  #[test]
  fn the_regions_partition_the_pool() {
    let all = candidates();
    let mut seen = 0;
    for region in Region::ALL {
      let c = region.candidates();
      assert!(!c.is_empty());
      assert!(c.iter().all(|f| all.contains(f)), "{:?} has a feature outside the pool", region);
      assert!(c.iter().all(|f| region.holds(*f)), "{:?} lists a feature it does not hold", region);
      seen += c.len();
    }
    assert_eq!(seen, all.len(), "every candidate is in exactly one region");
    assert!(Region::Enemies.holds(Feature {
      address: ENEMY_DISTANCE,
      relative: false,
      shift: 3
    }));
    assert!(Region::Enemies.holds(Feature {
      address: 0x87,
      relative: true,
      shift: 3
    }));
    assert!(Region::Mario.holds(Feature {
      address: 0x1D,
      relative: false,
      shift: 0
    }));
    assert!(Region::Near.holds(Feature {
      address: 256 + 3 * 13 + 10,
      relative: false,
      shift: 2
    }));
    assert!(Region::Far.holds(Feature {
      address: 256 + 4 * 13,
      relative: false,
      shift: 2
    }));
  }

  #[test]
  fn the_bank_search_fills_every_region_and_finds_the_trigger_in_its_region() {
    let (windows, buttons) = trigger_and_lag();
    let mask = BUTTON_A | BUTTON_B | BUTTON_RIGHT;
    let boards = search_bank(&windows, &buttons, 1.0, false, mask, &mut |_| {});
    for (region, board) in Region::ALL.iter().zip(boards.iter()) {
      assert!(board.iter().all(|f| region.holds(*f)), "{:?} got {:?}", region, board);
      let distinct: std::collections::HashSet<_> = board.iter().map(|f| (f.address, f.relative)).collect();
      assert_eq!(distinct.len(), PIECES_PER_STEP, "{:?} picked a byte twice", region);
    }
    assert_eq!(boards[0][0].address, 0x40, "the trigger byte is the Mario region's first pick");
  }

  #[test]
  fn candidates_skip_marios_position_and_read_only_enemy_x_relative_to_him() {
    let c = candidates();
    assert_eq!(c.len(), (SMB_WINDOW_LEN - 2 + 5 + 1) * 4);
    assert_eq!(c.iter().filter(|f| f.address == ENEMY_DISTANCE).count(), 4);
    assert!(c.iter().any(|f| f.address == 256 + 100), "terrain bytes are candidates");
    assert!(c.contains(&Feature {
      address: 0x87,
      relative: true,
      shift: 5
    }));
    assert!(!c.iter().any(|f| f.address == 0x86 || f.address == 0x6D), "Mario's x is not a candidate");
    assert!(
      !c.iter().any(|f| f.relative && !(0x87..0x8C).contains(&f.address)),
      "a terrain byte minus Mario's x is his position in disguise"
    );
  }

  #[test]
  fn greedy_search_finds_the_byte_that_decides_the_jump() {
    let (windows, buttons) = synthetic();
    let features = search_features(&windows, &buttons, 1.0, false, BUTTON_A | BUTTON_B | BUTTON_RIGHT, &mut |_| {});
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
      false,
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
      false,
      mask,
    )
    .unwrap();
    assert!(good.accuracy > noise.accuracy + 0.1, "good {} noise {}", good.accuracy, noise.accuracy);
  }
}
