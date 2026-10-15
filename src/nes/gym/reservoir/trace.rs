//! Recording a demonstration and closed-loop episodes with frames, for the video.

use super::demo::{smb_cleared_at, smb_dead, smb_in_cutscene, smb_start_state_at, smb_window, smb_x, smb_y, Demonstration, Stage, MACRO_FRAMES};
use super::drive::{TETRIS_PLAYFIELD, TETRIS_PLAYFIELD_LEN};
use super::encode::{encode, Encoder};
use super::features::Condition;
use super::ridge::Readout;
use super::{collect, fit_readout, hidden_for, level_step, observe, outputs_to_buttons, Dataset, Hidden};
use crate::nes::gym::NesEmulator;
use std::io::Write;

/// Fit one condition's readout on every given demonstration, collecting one demonstration per thread.
pub fn fit_condition(
  condition: Condition,
  smb_rom: &[u8],
  tetris_rom: &[u8],
  demos: &[Demonstration],
  lambda: f64,
  logistic: bool,
  horizon: usize,
) -> Result<Readout, String> {
  let datasets: Vec<Dataset> = std::thread::scope(|s| {
    let handles: Vec<_> = demos
      .iter()
      .map(|demo| s.spawn(move || collect(condition, smb_rom, tetris_rom, demo, horizon)))
      .collect();
    handles
      .into_iter()
      .map(|h| h.join().expect("collect worker panicked"))
      .collect::<Result<Vec<_>, _>>()
  })?;
  let rows: Vec<Vec<f32>> = datasets.iter().flat_map(|d| d.rows.iter().cloned()).collect();
  let targets: Vec<Vec<f32>> = datasets.iter().flat_map(|d| d.targets.iter().cloned()).collect();
  fit_readout(&rows, &targets, lambda, logistic)
}

/// One demonstration step as the video sees it.
#[derive(Debug, Clone)]
pub struct StepRecord {
  pub x: i32,
  pub y: u8,
  pub buttons: u8,
  pub cutscene: bool,
  pub window: Vec<u8>,
  /// Tetris playfield after this step, 200 cells, empty when no reservoir is traced.
  pub board: Vec<u8>,
  /// The whole of Tetris's work RAM after this step, empty when no reservoir is traced.
  pub tetris_ram: Vec<u8>,
  pub enemies: Vec<Enemy>,
  /// The salient encoder's placements for this window, as (column, rotation).
  pub placements: Vec<(u8, u8)>,
  /// Each traced readout's raw outputs for this step, in the order given.
  pub outputs: Vec<Vec<f64>>,
}

/// Replay a demonstration, writing every SMB frame and one Tetris frame per
/// step (from the first reservoir condition traced, if any) to the sinks,
/// and return the per-step records.
pub fn replay(
  smb_rom: &[u8],
  tetris_rom: &[u8],
  demo: &Demonstration,
  readouts: &[(Condition, Readout)],
  smb_sink: &mut dyn Write,
  tetris_sink: &mut dyn Write,
) -> Result<Vec<StepRecord>, String> {
  let mut smb = NesEmulator::new(smb_rom);
  smb.load_state(&smb_start_state_at(smb_rom, demo.stage));
  smb.step_frames(0, demo.noops);
  let mut hiddens: Vec<Hidden> = readouts.iter().map(|(c, _)| hidden_for(*c, tetris_rom)).collect::<Result<_, _>>()?;
  let screen = hiddens.iter().position(|h| h.tetris().is_some());
  let mut steps = Vec::with_capacity(demo.buttons.len());
  for (step, &action) in demo.buttons.iter().enumerate() {
    let window = smb_window(&mut smb);
    let outputs: Vec<Vec<f64>> = readouts
      .iter()
      .zip(hiddens.iter_mut())
      .map(|((c, readout), hidden)| readout.predict(&observe(*c, &window, hidden, level_step(demo.noops, step))))
      .collect();
    let (board, tetris_ram) = match screen.and_then(|i| hiddens[i].tetris_mut()) {
      Some(r) => {
        tetris_sink.write_all(r.framebuffer()).map_err(|e| e.to_string())?;
        let ram = r.ram();
        (
          ram[TETRIS_PLAYFIELD as usize..(TETRIS_PLAYFIELD + TETRIS_PLAYFIELD_LEN) as usize].to_vec(),
          ram,
        )
      }
      None => (Vec::new(), Vec::new()),
    };
    let cutscene = smb_in_cutscene(&mut smb);
    steps.push(StepRecord {
      x: smb_x(&mut smb),
      y: smb_y(&mut smb),
      buttons: action,
      cutscene,
      enemies: enemies(&window),
      placements: encode(Encoder::Salient, &window).iter().map(|p| (p.column, p.rotation)).collect(),
      window,
      board,
      tetris_ram,
      outputs,
    });
    for _ in 0..MACRO_FRAMES {
      smb.step_frame(action);
      smb_sink.write_all(smb.framebuffer()).map_err(|e| e.to_string())?;
    }
  }
  Ok(steps)
}

/// One closed-loop step.
#[derive(Debug, Clone)]
pub struct EpisodeStep {
  pub x: i32,
  pub buttons: u8,
  pub dead: bool,
}

/// Run a readout closed-loop, writing every SMB frame to the sink.
pub fn trace_episode(
  condition: Condition,
  readout: &Readout,
  smb_rom: &[u8],
  tetris_rom: &[u8],
  stage: Stage,
  noops: usize,
  max_steps: usize,
  smb_sink: &mut dyn Write,
) -> Result<Vec<EpisodeStep>, String> {
  let mut smb = NesEmulator::new(smb_rom);
  smb.load_state(&smb_start_state_at(smb_rom, stage));
  smb.step_frames(0, noops);
  let mut hidden = hidden_for(condition, tetris_rom)?;
  let mut steps = Vec::new();
  for step in 0..max_steps {
    let window = smb_window(&mut smb);
    let buttons = outputs_to_buttons(&readout.predict(&observe(condition, &window, &mut hidden, level_step(noops, step))));
    for _ in 0..MACRO_FRAMES {
      smb.step_frame(buttons);
      smb_sink.write_all(smb.framebuffer()).map_err(|e| e.to_string())?;
    }
    let dead = smb_dead(&mut smb);
    steps.push(EpisodeStep {
      x: smb_x(&mut smb),
      buttons,
      dead,
    });
    if dead || smb_cleared_at(&mut smb, stage) {
      break;
    }
  }
  Ok(steps)
}

/// An enemy slot read from the SMB window: what it is and where it is in level x.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enemy {
  pub slot: u8,
  pub kind: u8,
  pub x: i32,
  /// Screen y, smaller is higher.
  pub y: u8,
}

/// SMB zero page: enemy slot active flags, types, x pages and x pixels.
const ENEMY_ACTIVE: usize = 0x000F;
const ENEMY_KIND: usize = 0x0016;
const ENEMY_X_PAGE: usize = 0x006E;
const ENEMY_X_PIXEL: usize = 0x0087;
const ENEMY_Y: usize = 0x00CF;
const ENEMY_SLOTS: usize = 5;

/// The active enemy slots in a window of at least 256 bytes.
pub fn enemies(window: &[u8]) -> Vec<Enemy> {
  (0..ENEMY_SLOTS)
    .filter(|&i| window[ENEMY_ACTIVE + i] != 0)
    .map(|i| Enemy {
      slot: i as u8,
      kind: window[ENEMY_KIND + i],
      x: window[ENEMY_X_PAGE + i] as i32 * 256 + window[ENEMY_X_PIXEL + i] as i32,
      y: window[ENEMY_Y + i],
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn enemies_reads_active_slots_only() {
    let mut window = vec![0u8; 256];
    window[ENEMY_ACTIVE + 1] = 1;
    window[ENEMY_KIND + 1] = 0x06;
    window[ENEMY_X_PAGE + 1] = 2;
    window[ENEMY_X_PIXEL + 1] = 0x10;
    window[ENEMY_KIND + 3] = 0x02;
    window[ENEMY_X_PIXEL + 3] = 0x99;
    window[ENEMY_Y + 1] = 0xB0;
    assert_eq!(
      enemies(&window),
      vec![Enemy {
        slot: 1,
        kind: 0x06,
        x: 528,
        y: 0xB0
      }]
    );
  }
}
