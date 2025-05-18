use std::cell::RefCell;
use std::rc::Rc;

use crate::cartridge::Cartridge;
use crate::traits::Mappable;

use super::ppu::PPU;

const PROGRAM_CONTROL_ADDRESS: u16 = 0xFFFC;
const MAX_ADDRESS: u16 = 0xFFFF;

const RAM_START_ADDRESS: u16 = 0x0000;
const RAM_ACTUAL_END_ADDRESS: u16 = 0x07FF;
const RAM_RANGE_END_ADDRESS: u16 = 0x1FFF;

const PPU_REGISTER_START_ADDRESS: u16 = 0x2000;
const PPU_REGISTER_ACTUAL_END_ADDRESS: u16 = 0x2007;
const PPU_REGISTER_RANGE_END_ADDRESS: u16 = 0x3FFF;

const APU_IO_START_ADDRESS: u16 = 0x4000;
const APU_IO_END_ADDRESS: u16 = 0x4017;
const OAM_DMA_ADDRESS: u16 = 0x4014;

const CARTRIDGE_START_ADDRESS: u16 = 0x4018;
const CARTRIDGE_END_ADDRESS: u16 = MAX_ADDRESS;

pub mod addressable;
pub use addressable::*;

pub mod busable;
pub use busable::*;

pub mod interruptible;
pub use interruptible::*;

/// NES Controller button bits (Active-HIGH for internal state)
pub const BUTTON_A: u8 = 0b0000_0001;
pub const BUTTON_B: u8 = 0b0000_0010;
pub const BUTTON_SELECT: u8 = 0b0000_0100;
pub const BUTTON_START: u8 = 0b0000_1000;
pub const BUTTON_UP: u8 = 0b0001_0000;
pub const BUTTON_DOWN: u8 = 0b0010_0000;
pub const BUTTON_LEFT: u8 = 0b0100_0000;
pub const BUTTON_RIGHT: u8 = 0b1000_0000;

pub struct Bus {
  memory: [u8; (RAM_ACTUAL_END_ADDRESS + 1) as usize],
  cartridge: Option<Rc<RefCell<Cartridge>>>,
  clock_counter: u64,
  ppu: PPU,
  /// Controller 1 button state (bits: A, B, Select, Start, Up, Down, Left, Right)
  controller1_state: u8,
  /// Controller 2 button state
  controller2_state: u8,
  /// Controller 1 shift register (latched state being read out)
  controller1_shift: u8,
  /// Controller 2 shift register
  controller2_shift: u8,
  /// Controller strobe state (when high, continuously reload shift registers)
  controller_strobe: bool,
  /// APU registers storage ($4000-$4017 excluding controller ports)
  /// Used to return last written value for registers that don't have full read behavior
  apu_registers: [u8; 24],
}

impl Bus {
  pub fn new() -> Bus {
    Bus {
      memory: [0; (RAM_ACTUAL_END_ADDRESS + 1) as usize],
      cartridge: None,
      clock_counter: 0,
      ppu: PPU::new(),
      controller1_state: 0,
      controller2_state: 0,
      controller1_shift: 0,
      controller2_shift: 0,
      controller_strobe: false,
      apu_registers: [0xFF; 24],
    }
  }

  /// Set the button state for controller 1.
  /// Use the BUTTON_* constants to set individual buttons.
  pub fn set_controller1(&mut self, state: u8) {
    self.controller1_state = state;
    // If strobe is high, immediately update shift register
    if self.controller_strobe {
      self.controller1_shift = state;
    }
  }

  /// Set the button state for controller 2.
  pub fn set_controller2(&mut self, state: u8) {
    self.controller2_state = state;
    if self.controller_strobe {
      self.controller2_shift = state;
    }
  }

  /// Press a button on controller 1.
  pub fn press_button1(&mut self, button: u8) {
    self.controller1_state |= button;
    if self.controller_strobe {
      self.controller1_shift = self.controller1_state;
    }
  }

  /// Release a button on controller 1.
  pub fn release_button1(&mut self, button: u8) {
    self.controller1_state &= !button;
    if self.controller_strobe {
      self.controller1_shift = self.controller1_state;
    }
  }

  #[named]
  pub fn inner_read_u8(&mut self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let result = match address {
      RAM_START_ADDRESS..=RAM_RANGE_END_ADDRESS => {
        let actual_address = address & RAM_ACTUAL_END_ADDRESS;
        trace_u16!(actual_address);
        self.memory[actual_address as usize]
      }
      PPU_REGISTER_START_ADDRESS..=PPU_REGISTER_RANGE_END_ADDRESS => {
        let actual_address = address & PPU_REGISTER_ACTUAL_END_ADDRESS;
        trace_u16!(actual_address);
        let index = (actual_address % 8) as u8;
        self.ppu.read_register(index)
      }
      APU_IO_START_ADDRESS..=APU_IO_END_ADDRESS => {
        match address {
          0x4016 => {
            // Controller 1 read - return bit 0 of shift register, then shift
            // Upper bits return open bus (typically last value on bus)
            let result = (self.controller1_shift & 1) | 0x40;
            // Shift right (next read gets next button)
            // After all 8 buttons are read, subsequent reads return 1
            self.controller1_shift = (self.controller1_shift >> 1) | 0x80;
            result
          }
          0x4017 => {
            // Controller 2 read
            let result = (self.controller2_shift & 1) | 0x40;
            self.controller2_shift = (self.controller2_shift >> 1) | 0x80;
            result
          }
          0x4015 => {
            // APU status - TODO: implement proper APU status
            // For now return last written value (needed for nestest)
            self.apu_registers[(address - APU_IO_START_ADDRESS) as usize]
          }
          _ => {
            // Other APU registers - return last written value
            self.apu_registers[(address - APU_IO_START_ADDRESS) as usize]
          }
        }
      }
      CARTRIDGE_START_ADDRESS..=CARTRIDGE_END_ADDRESS => {
        if let Some(ref cartridge) = self.cartridge {
          cartridge.borrow().read_prg_u8(address)
        } else {
          (address >> 8) as u8
        }
      }
      _ => {
        warn!("Ignoring out-of-bounds memory read at address {}", format_u16!(address));
        0x00
      }
    };
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  pub fn inner_write_u8(&mut self, address: u16, value: u8) {
    trace_enter!();
    trace_u16!(address);
    trace_u8!(value);
    match address {
      RAM_START_ADDRESS..=RAM_RANGE_END_ADDRESS => {
        let actual_address = address & RAM_ACTUAL_END_ADDRESS;
        trace_u16!(actual_address);
        self.memory[actual_address as usize] = value;
      }
      PPU_REGISTER_START_ADDRESS..=PPU_REGISTER_RANGE_END_ADDRESS => {
        let actual_address = address & PPU_REGISTER_ACTUAL_END_ADDRESS;
        trace_u16!(actual_address);
        let index = (actual_address % 8) as u8;
        self.ppu.write_register(index, value);
      }
      APU_IO_START_ADDRESS..=APU_IO_END_ADDRESS => {
        // Store all APU register writes for later read-back
        self.apu_registers[(address - APU_IO_START_ADDRESS) as usize] = value;

        if address == OAM_DMA_ADDRESS {
          // OAM DMA: copy 256 bytes from CPU page XX00-XXFF to OAM
          let source_page = (value as u16) << 8;
          let mut oam_data = [0u8; 256];
          for i in 0..256u16 {
            let src_addr = source_page | i;
            oam_data[i as usize] = self.inner_read_u8(src_addr);
          }
          self.ppu.write_oam_dma(&oam_data);
          // Note: Real OAM DMA takes 513-514 CPU cycles
          // TODO: Add cycle-accurate DMA timing
        } else if address == 0x4016 {
          // Controller strobe
          let new_strobe = (value & 1) != 0;
          // When strobe goes from high to low, latch current button state
          if self.controller_strobe && !new_strobe {
            self.controller1_shift = self.controller1_state;
            self.controller2_shift = self.controller2_state;
          }
          self.controller_strobe = new_strobe;
          // While strobe is high, continuously reload
          if self.controller_strobe {
            self.controller1_shift = self.controller1_state;
            self.controller2_shift = self.controller2_state;
          }
        }
        // TODO: Implement proper APU behavior (0x4000-0x4013, 0x4015, 0x4017)
      }
      CARTRIDGE_START_ADDRESS..=CARTRIDGE_END_ADDRESS => {
        if let Some(ref cartridge) = self.cartridge {
          cartridge.borrow_mut().write_prg_u8(address, value);
        }
      }
      _ => {
        warn!("Ignoring out-of-bounds memory write at address {}", format_u16!(address));
      }
    }
    trace_exit!();
  }

  #[named]
  pub fn load_cartridge_data(&mut self, data: &[u8]) {
    trace_enter!();
    info!("Loading cartridge from data...");
    let cartridge = Rc::new(RefCell::new(Cartridge::new(data)));
    self.ppu.vram.set_cartridge(cartridge.clone());
    self.cartridge = Some(cartridge);
    trace_exit!();
  }

  /// Get a reference to the PPU's framebuffer.
  pub fn get_framebuffer(&self) -> &[u8] {
    self.ppu.get_framebuffer()
  }

  /// Check if a new frame is ready and clear the flag.
  pub fn take_frame_ready(&mut self) -> bool {
    self.ppu.take_frame_ready()
  }

  /// Get debug info about PPU state
  #[cfg(test)]
  pub fn get_ppu_debug_info(&self) -> String {
    format!(
      "PPU Debug:\n  CTRL: 0x{:02X}\n  MASK: 0x{:02X}\n  STATUS: 0x{:02X}\n  show_bg: {}\n  show_sprites: {}\n  scanline: {}\n  dot: {}",
      self.ppu.control_register.read_u8(),
      self.ppu.mask_register.read_u8(),
      self.ppu.status_register.read_u8(),
      self.ppu.mask_register.get_show_background_flag(),
      self.ppu.mask_register.get_show_sprites_flag(),
      self.ppu.scanline,
      self.ppu.dot
    )
  }
}
