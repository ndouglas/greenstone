//! NES Gym Environment for Reinforcement Learning.
//!
//! Provides an OpenAI Gym-style interface to the NES emulator,
//! enabling headless emulation with save/load state support.
//!
//! # Example
//!
//! ```no_run
//! use greenstone::gym::NesEmulator;
//! use greenstone::bus::{BUTTON_A, BUTTON_RIGHT};
//!
//! let rom = std::fs::read("game.nes").unwrap();
//! let mut emu = NesEmulator::new(&rom);
//!
//! loop {
//!     let _result = emu.step_frame(BUTTON_A | BUTTON_RIGHT);
//!     let _pixels = emu.framebuffer(); // 256x240 RGB
//!     let _score_byte = emu.ram_read(0x07E2);
//! }
//! ```

pub mod action;
pub mod env;
pub mod preprocess;
pub mod prospector;
pub mod reservoir;
pub mod reward;

pub use action::ActionSpace;
pub use env::{NesEnv, NesEnvConfig, StepInfo};
pub use preprocess::{FrameStack, OBS_HEIGHT, OBS_PIXELS, OBS_WIDTH};
pub use reward::{ClippedReward, CompositeReward, NullReward, RewardExtractor, SingleByteReward, SuperMarioBrosReward};

use crate::nes::bus::Bus;
use crate::nes::cpu::CPU;
use crate::traits::Addressable;
use crate::traits::Interruptible;

/// Result of a single emulation step (one CPU instruction).
#[derive(Debug, Clone)]
pub struct StepResult {
  /// Whether the PPU just completed a new frame.
  pub frame_ready: bool,
  /// Current CPU cycle count.
  pub clock_counter: u64,
}

/// Result of running the emulator until the next frame.
#[derive(Debug, Clone)]
pub struct FrameResult {
  /// Number of CPU instructions executed this frame.
  pub instructions: u64,
  /// Current CPU cycle count.
  pub clock_counter: u64,
}

/// Serializable NES state snapshot for save/load.
/// Captures the entire mutable emulator state.
pub type NesState = Vec<u8>;

/// Headless NES emulator wrapper for RL training.
///
/// Wraps a CPU + Bus without any display/audio dependencies.
/// Provides a clean step/reset/observe interface suitable for
/// reinforcement learning environments.
pub struct NesEmulator {
  /// The raw ROM data, kept for reset.
  rom_data: Vec<u8>,
  /// The CPU (which owns the Bus via Box<dyn Busable>).
  cpu: CPU,
}

impl NesEmulator {
  /// Create a new headless NES emulator from ROM data.
  pub fn new(rom_data: &[u8]) -> Self {
    let mut bus = Bus::new();
    bus.load_cartridge_data(rom_data);
    let mut cpu = CPU::new_with_bus(Box::new(bus));
    cpu.handle_reset();
    NesEmulator {
      rom_data: rom_data.to_vec(),
      cpu,
    }
  }

  /// Reset the emulator to initial power-on state.
  pub fn reset(&mut self) {
    let mut bus = Bus::new();
    bus.load_cartridge_data(&self.rom_data);
    self.cpu = CPU::new_with_bus(Box::new(bus));
    self.cpu.handle_reset();
  }

  /// Execute a single CPU instruction.
  /// Returns whether a new frame became ready.
  pub fn step_instruction(&mut self) -> StepResult {
    self.cpu.clock();
    let frame_ready = self.cpu.take_frame_ready();
    StepResult {
      frame_ready,
      clock_counter: self.cpu.clock_counter,
    }
  }

  /// Run the emulator for exactly one frame (until frame_ready fires).
  /// Sets the controller state for the duration of the frame.
  ///
  /// This is the primary interface for RL: call once per "step" in the
  /// environment, passing the agent's chosen button combination.
  pub fn step_frame(&mut self, buttons: u8) -> FrameResult {
    self.cpu.set_controller1(buttons);
    let mut instructions: u64 = 0;
    loop {
      self.cpu.clock();
      instructions += 1;
      if self.cpu.take_frame_ready() {
        break;
      }
    }
    FrameResult {
      instructions,
      clock_counter: self.cpu.clock_counter,
    }
  }

  /// Run the emulator for N frames, applying the same button state.
  /// Useful for frame-skipping (e.g., repeat action for 4 frames).
  pub fn step_frames(&mut self, buttons: u8, count: usize) -> FrameResult {
    self.cpu.set_controller1(buttons);
    let mut total_instructions: u64 = 0;
    for _ in 0..count {
      loop {
        self.cpu.clock();
        total_instructions += 1;
        if self.cpu.take_frame_ready() {
          break;
        }
      }
    }
    FrameResult {
      instructions: total_instructions,
      clock_counter: self.cpu.clock_counter,
    }
  }

  /// Get the current framebuffer (256×240 pixels, RGB24 format).
  /// Returns 256 * 240 * 3 = 184,320 bytes.
  pub fn framebuffer(&self) -> &[u8] {
    self.cpu.get_framebuffer()
  }

  /// Downsample the framebuffer to 84×84 grayscale (standard Atari DQN input).
  /// Uses simple bilinear-ish averaging for speed.
  pub fn framebuffer_grayscale_84x84(&self) -> Vec<u8> {
    let fb = self.framebuffer();
    let src_w: usize = 256;
    let src_h: usize = 240;
    let dst_w: usize = 84;
    let dst_h: usize = 84;
    let mut out = vec![0u8; dst_w * dst_h];

    for dy in 0..dst_h {
      for dx in 0..dst_w {
        // Map destination pixel to source region
        let sx_start = dx * src_w / dst_w;
        let sx_end = ((dx + 1) * src_w / dst_w).min(src_w);
        let sy_start = dy * src_h / dst_h;
        let sy_end = ((dy + 1) * src_h / dst_h).min(src_h);

        let mut sum: u32 = 0;
        let mut count: u32 = 0;
        for sy in sy_start..sy_end {
          for sx in sx_start..sx_end {
            let idx = (sy * src_w + sx) * 3;
            if idx + 2 < fb.len() {
              // ITU-R BT.601 luminance
              let r = fb[idx] as u32;
              let g = fb[idx + 1] as u32;
              let b = fb[idx + 2] as u32;
              sum += (r * 299 + g * 587 + b * 114) / 1000;
              count += 1;
            }
          }
        }
        out[dy * dst_w + dx] = if count > 0 { (sum / count) as u8 } else { 0 };
      }
    }
    out
  }

  /// Read a byte from CPU address space (RAM, PPU registers, etc.).
  /// Uses unclocked reads (no side effects on PPU/APU timing).
  pub fn ram_read(&mut self, address: u16) -> u8 {
    self.cpu.unclocked_read_u8(address)
  }

  /// Write a byte to CPU address space.
  /// Uses unclocked writes (no side effects on PPU/APU timing).
  pub fn ram_write(&mut self, address: u16, value: u8) {
    self.cpu.unclocked_write_u8(address, value);
  }

  /// Read a range of RAM bytes. Useful for extracting game state.
  pub fn ram_read_range(&mut self, start: u16, length: u16) -> Vec<u8> {
    (0..length).map(|i| self.cpu.unclocked_read_u8(start.wrapping_add(i))).collect()
  }

  /// Save the complete emulator state as a binary blob.
  /// Can be restored later with `load_state()`.
  pub fn save_state(&self) -> NesState {
    let bus = self.get_bus();
    // Serialize CPU registers + Bus state
    let cpu_state = CpuSnapshot {
      a: self.cpu.a,
      x: self.cpu.x,
      y: self.cpu.y,
      status: self.cpu.status,
      stack_pointer: self.cpu.stack_pointer,
      program_counter: self.cpu.program_counter,
      clock_counter: self.cpu.clock_counter,
    };
    let snapshot = EmulatorSnapshot {
      cpu: cpu_state,
      bus: bincode::serialize(bus).expect("Failed to serialize Bus state"),
    };
    bincode::serialize(&snapshot).expect("Failed to serialize emulator state")
  }

  /// Restore emulator state from a previously saved binary blob.
  pub fn load_state(&mut self, state: &[u8]) {
    let snapshot: EmulatorSnapshot = bincode::deserialize(state).expect("Failed to deserialize emulator state");

    // Restore CPU registers
    self.cpu.a = snapshot.cpu.a;
    self.cpu.x = snapshot.cpu.x;
    self.cpu.y = snapshot.cpu.y;
    self.cpu.status = snapshot.cpu.status;
    self.cpu.stack_pointer = snapshot.cpu.stack_pointer;
    self.cpu.program_counter = snapshot.cpu.program_counter;
    self.cpu.clock_counter = snapshot.cpu.clock_counter;

    // Restore Bus state
    let mut bus: Bus = bincode::deserialize(&snapshot.bus).expect("Failed to deserialize Bus state");
    // Reconnect the shared Rc<RefCell<Cartridge>> between Bus and VRAM
    bus.reconnect_cartridge();
    self.cpu.bus = Box::new(bus);
  }

  /// Get the current CPU cycle count.
  pub fn clock_counter(&self) -> u64 {
    self.cpu.clock_counter
  }

  /// Get a reference to the underlying CPU (for advanced usage).
  pub fn cpu(&self) -> &CPU {
    &self.cpu
  }

  /// Get a mutable reference to the underlying CPU.
  pub fn cpu_mut(&mut self) -> &mut CPU {
    &mut self.cpu
  }

  /// Internal helper: downcast Box<dyn Busable> to &Bus.
  fn get_bus(&self) -> &Bus {
    self.cpu.bus.as_any().downcast_ref::<Bus>().expect("Bus downcast failed")
  }

  /// Internal helper: downcast Box<dyn Busable> to &mut Bus.
  #[allow(dead_code)]
  fn get_bus_mut(&mut self) -> &mut Bus {
    self.cpu.bus.as_any_mut().downcast_mut::<Bus>().expect("Bus downcast failed")
  }
}

/// Snapshot of CPU register state (no trait objects, trivially serializable).
#[derive(Serialize, Deserialize, Clone)]
struct CpuSnapshot {
  a: u8,
  x: u8,
  y: u8,
  status: u8,
  stack_pointer: u8,
  program_counter: u16,
  clock_counter: u64,
}

/// Complete emulator snapshot: CPU registers + serialized Bus.
#[derive(Serialize, Deserialize, Clone)]
struct EmulatorSnapshot {
  cpu: CpuSnapshot,
  /// Bus serialized as bincode bytes (includes PPU, APU, cartridge, RAM).
  bus: Vec<u8>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test::init;

  /// Build a minimal test ROM (NROM mapper 0).
  /// This creates a ROM that immediately loops (JMP $8000).
  fn build_test_rom() -> Vec<u8> {
    let mut data = vec![
      // iNES header (16 bytes)
      0x4e, 0x45, 0x53, 0x1a, // Magic "NES\x1a"
      0x02, // 2 PRG ROM pages (32KB)
      0x01, // 1 CHR ROM page (8KB)
      0x00, // Flags 6: mapper 0, horizontal mirroring
      0x00, // Flags 7
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Rest of header
    ];
    // PRG ROM (32KB)
    data.resize(16 + 0x8000, 0x00);
    // Write JMP $8000 at the start of PRG ROM
    data[16] = 0x4C; // JMP absolute
    data[17] = 0x00; // low byte of $8000
    data[18] = 0x80; // high byte of $8000

    // Write reset vector at $FFFC (offset 0x7FFC in PRG data)
    data[16 + 0x7FFC] = 0x00; // low byte -> $8000
    data[16 + 0x7FFD] = 0x80; // high byte

    // CHR ROM (8KB)
    data.resize(16 + 0x8000 + 0x2000, 0x00);
    data
  }

  #[test]
  fn test_gym_create_and_reset() {
    init();
    let rom = build_test_rom();
    let mut emu = NesEmulator::new(&rom);
    assert_eq!(emu.cpu.program_counter, 0x8000);

    // Run a few instructions
    for _ in 0..10 {
      emu.step_instruction();
    }

    // Reset and verify we're back at the start
    emu.reset();
    assert_eq!(emu.cpu.program_counter, 0x8000);
  }

  #[test]
  fn test_gym_step_frame() {
    init();
    let rom = build_test_rom();
    let mut emu = NesEmulator::new(&rom);

    // Step one frame
    let result = emu.step_frame(0);
    assert!(result.instructions > 0);
    assert!(result.clock_counter > 0);
  }

  #[test]
  fn test_gym_framebuffer() {
    init();
    let rom = build_test_rom();
    let mut emu = NesEmulator::new(&rom);

    // Step a frame to populate the framebuffer
    emu.step_frame(0);

    let fb = emu.framebuffer();
    assert_eq!(fb.len(), 256 * 240 * 3);
  }

  #[test]
  fn test_gym_grayscale_downsampling() {
    init();
    let rom = build_test_rom();
    let mut emu = NesEmulator::new(&rom);
    emu.step_frame(0);

    let gray = emu.framebuffer_grayscale_84x84();
    assert_eq!(gray.len(), 84 * 84);
  }

  #[test]
  fn test_gym_ram_access() {
    init();
    let rom = build_test_rom();
    let mut emu = NesEmulator::new(&rom);

    // Write to RAM and read back
    emu.ram_write(0x0200, 0x42);
    assert_eq!(emu.ram_read(0x0200), 0x42);

    // Read range
    emu.ram_write(0x0201, 0x43);
    emu.ram_write(0x0202, 0x44);
    let range = emu.ram_read_range(0x0200, 3);
    assert_eq!(range, vec![0x42, 0x43, 0x44]);
  }

  #[test]
  fn test_gym_save_load_state() {
    init();
    let rom = build_test_rom();
    let mut emu = NesEmulator::new(&rom);

    // Run some frames to build up state
    emu.step_frame(0);
    emu.step_frame(0);

    // Write a marker to RAM
    emu.ram_write(0x0300, 0xAB);

    // Save state
    let state = emu.save_state();
    let saved_pc = emu.cpu.program_counter;
    let saved_clock = emu.cpu.clock_counter;

    // Run more frames (state diverges)
    emu.step_frame(0);
    emu.step_frame(0);
    emu.ram_write(0x0300, 0xFF);

    // Verify state diverged
    assert_ne!(emu.cpu.clock_counter, saved_clock);
    assert_eq!(emu.ram_read(0x0300), 0xFF);

    // Load state
    emu.load_state(&state);

    // Verify state was restored
    assert_eq!(emu.cpu.program_counter, saved_pc);
    assert_eq!(emu.cpu.clock_counter, saved_clock);
    assert_eq!(emu.ram_read(0x0300), 0xAB);
  }

  #[test]
  fn test_gym_step_multiple_frames() {
    init();
    let rom = build_test_rom();
    let mut emu = NesEmulator::new(&rom);

    // Step 4 frames (frame skip)
    let result = emu.step_frames(0, 4);
    assert!(result.instructions > 0);

    // Verify framebuffer is populated
    let fb = emu.framebuffer();
    assert_eq!(fb.len(), 256 * 240 * 3);
  }

  #[test]
  fn test_gym_controller_input() {
    init();
    let rom = build_test_rom();
    let mut emu = NesEmulator::new(&rom);

    // Step with A button pressed
    let _result = emu.step_frame(crate::bus::BUTTON_A);

    // Step with multiple buttons
    let _result = emu.step_frame(crate::bus::BUTTON_A | crate::bus::BUTTON_RIGHT);

    // Should not crash
  }
}
