//! The Greenstone emulator from Python: step frames with buttons held, read and write memory, look at the
//! picture, save and load moments.

use std::path::PathBuf;

use emu::nes::bus::{BUTTON_A, BUTTON_B, BUTTON_DOWN, BUTTON_LEFT, BUTTON_RIGHT, BUTTON_SELECT, BUTTON_START, BUTTON_UP};
use emu::nes::gym::series::ram::{Game, CASTLEVANIA, SMB1, SMB3};
use emu::nes::gym::NesEmulator;
use numpy::ndarray::Array;
use numpy::{IntoPyArray, PyArray1, PyArray3, PyReadonlyArray1};
use pyo3::exceptions::{PyOSError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

const RAM_BYTES: u16 = 2048;
const WIDTH: usize = 256;
const HEIGHT: usize = 240;
const GAMES: [(&str, &Game); 3] = [("smb1", &SMB1), ("smb3", &SMB3), ("castlevania", &CASTLEVANIA)];

/// Where the ROMs are: $GREENSTONE_ROMS, or this repo's roms2/.
fn roms() -> PathBuf {
  std::env::var_os("GREENSTONE_ROMS")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../roms2")))
}

fn read(path: &PathBuf) -> PyResult<Vec<u8>> {
  std::fs::read(path).map_err(|e| PyOSError::new_err(format!("read {}: {}", path.display(), e)))
}

/// A console. `frame` counts the frames stepped since it was made (or since the moment it last loaded).
#[pyclass(unsendable, module = "greenstone")]
struct NES {
  emu: NesEmulator,
  #[pyo3(get)]
  frame: u64,
}

impl NES {
  fn run(&mut self, buttons: u8) {
    self.emu.step_frame(buttons);
    self.frame += 1;
  }

  fn screen(&self) -> Vec<u8> {
    self.emu.framebuffer().to_vec()
  }
}

#[pymethods]
impl NES {
  /// Power on with the cartridge at `path`, at the very first frame (title screens and all).
  #[new]
  fn new(path: PathBuf) -> PyResult<Self> {
    Ok(NES {
      emu: NesEmulator::new(&read(&path)?),
      frame: 0,
    })
  }

  /// Run `frames` frames with `buttons` held (the button constants, or'd together).
  #[pyo3(signature = (buttons = 0, frames = 1))]
  fn step(&mut self, buttons: u8, frames: usize) {
    for _ in 0..frames {
      self.run(buttons);
    }
  }

  /// Run one frame per entry of `buttons`, returning the memory after each, shape (frames, 2048); with
  /// `pictures`, also the picture after each, shape (frames, 240, 256, 3).
  #[pyo3(signature = (buttons, pictures = false))]
  fn play<'py>(&mut self, py: Python<'py>, buttons: PyReadonlyArray1<'py, u8>, pictures: bool) -> PyResult<Bound<'py, PyAny>> {
    let buttons = buttons.as_array().to_vec();
    let n = buttons.len();
    let mut ram = Vec::with_capacity(n * RAM_BYTES as usize);
    let mut seen = Vec::with_capacity(if pictures { n * HEIGHT * WIDTH * 3 } else { 0 });
    for b in buttons {
      self.run(b);
      ram.extend(self.emu.ram_read_range(0, RAM_BYTES));
      if pictures {
        seen.extend_from_slice(self.emu.framebuffer());
      }
    }
    let ram = Array::from_shape_vec((n, RAM_BYTES as usize), ram)
      .expect("whole frames of memory")
      .into_pyarray(py);
    if !pictures {
      return Ok(ram.into_any());
    }
    let seen = Array::from_shape_vec((n, HEIGHT, WIDTH, 3), seen)
      .expect("whole pictures")
      .into_pyarray(py);
    Ok((ram, seen).into_pyobject(py)?.into_any())
  }

  /// All 2 KB of the console's memory, as it is now (a copy).
  #[getter]
  fn ram<'py>(&mut self, py: Python<'py>) -> Bound<'py, PyArray1<u8>> {
    self.emu.ram_read_range(0, RAM_BYTES).into_pyarray(py)
  }

  /// One byte of memory.
  fn peek(&mut self, address: u16) -> u8 {
    self.emu.ram_read(address)
  }

  /// Write one byte of memory.
  fn poke(&mut self, address: u16, value: u8) {
    self.emu.ram_write(address, value)
  }

  /// The picture the last frame drew, shape (240, 256, 3).
  fn picture<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray3<u8>> {
    Array::from_shape_vec((HEIGHT, WIDTH, 3), self.screen())
      .expect("a whole picture")
      .into_pyarray(py)
  }

  /// The sound made since the last call: mono, 44,100 samples a second.
  fn audio<'py>(&mut self, py: Python<'py>) -> Bound<'py, PyArray1<f32>> {
    self.emu.take_audio_samples().into_pyarray(py)
  }

  /// This moment, to come back to with `load`.
  fn save<'py>(&self, py: Python<'py>) -> Bound<'py, PyBytes> {
    let mut state = self.frame.to_le_bytes().to_vec();
    state.extend(self.emu.save_state());
    PyBytes::new(py, &state)
  }

  /// Go back to a moment from `save`.
  fn load(&mut self, state: &[u8]) -> PyResult<()> {
    if state.len() < 8 {
      return Err(PyValueError::new_err("not a saved moment"));
    }
    let (frame, rest) = state.split_at(8);
    self.emu.load_state(rest);
    self.frame = u64::from_le_bytes(frame.try_into().expect("8 bytes"));
    Ok(())
  }
}

/// A console at the first frame of play of `game` ("smb1", "smb3" or "castlevania"), its ROM checked to be
/// the one the series' addresses were verified on. ROMs come from `roms`, $GREENSTONE_ROMS, or roms2/.
#[pyfunction]
#[pyo3(signature = (game, roms = None))]
fn boot(game: &str, roms: Option<PathBuf>) -> PyResult<NES> {
  let (_, g) = GAMES
    .iter()
    .find(|(name, _)| *name == game)
    .ok_or_else(|| PyValueError::new_err(format!("no game {:?}; there's {:?}", game, GAMES.map(|(n, _)| n))))?;
  let file = PathBuf::from(g.rom);
  let path = roms.unwrap_or_else(self::roms).join(file.file_name().expect("a ROM file name"));
  let emu = g.boot(&read(&path)?).map_err(PyValueError::new_err)?;
  Ok(NES { emu, frame: 0 })
}

#[pymodule]
fn greenstone(m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_class::<NES>()?;
  m.add_function(wrap_pyfunction!(boot, m)?)?;
  for (name, bit) in [
    ("A", BUTTON_A),
    ("B", BUTTON_B),
    ("SELECT", BUTTON_SELECT),
    ("START", BUTTON_START),
    ("UP", BUTTON_UP),
    ("DOWN", BUTTON_DOWN),
    ("LEFT", BUTTON_LEFT),
    ("RIGHT", BUTTON_RIGHT),
  ] {
    m.add(name, bit)?;
  }
  m.add("GAMES", GAMES.map(|(n, _)| n).to_vec())?;
  Ok(())
}
