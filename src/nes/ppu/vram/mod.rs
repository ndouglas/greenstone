use super::super::*;
use crate::traits::Mappable;
use crate::MirroringMode;

use std::cell::RefCell;
use std::rc::Rc;

pub const NAMETABLE_SIZE: u16 = 0x400;
pub const PALETTE_SIZE: u16 = 0x20;
pub const CARTRIDGE_CHR_START_ADDRESS: u16 = 0x0000;
pub const CARTRIDGE_CHR_END_ADDRESS: u16 = 0x1FFF;
pub const VRAM_START_ADDRESS: u16 = 0x2000;
pub const VRAM_END_ADDRESS: u16 = 0x3FFF;
pub const NAMETABLE_START_ADDRESS: u16 = VRAM_START_ADDRESS;
pub const NAMETABLE_END_ADDRESS: u16 = 0x3EFF;
pub const PALETTE_START_ADDRESS: u16 = 0x3F00;
pub const PALETTE_END_ADDRESS: u16 = VRAM_END_ADDRESS;
pub const VRAM_LENGTH: usize = 2048;

pub struct VRAM {
  nametables: Vec<u8>,
  palettes: Vec<u8>,
  cartridge: Option<Rc<RefCell<Cartridge>>>,
  read_buffer: u8,
}

impl VRAM {
  pub fn new() -> VRAM {
    VRAM {
      nametables: vec![0; 2 * NAMETABLE_SIZE as usize],
      palettes: vec![0; PALETTE_SIZE as usize],
      cartridge: None,
      read_buffer: 0,
    }
  }

  #[named]
  pub fn write_u8(&mut self, address: u16, value: u8) {
    trace_enter!();
    trace_u16!(address);
    trace_u8!(value);
    match address {
      CARTRIDGE_CHR_START_ADDRESS..=CARTRIDGE_CHR_END_ADDRESS => match self.cartridge {
        Some(ref cartridge) => cartridge.borrow_mut().write_chr_u8(address, value),
        None => panic!("Attempted to write data to CHR RAM of a cartridge, but no cartridge was loaded."),
      },
      NAMETABLE_START_ADDRESS..=NAMETABLE_END_ADDRESS => {
        let mirroring_mode = self.get_mirroring_mode();
        let mirrored_address = self.get_mirrored_nametable_address(&mirroring_mode, address) as usize;
        self.nametables[mirrored_address] = value;
      }
      PALETTE_START_ADDRESS..=PALETTE_END_ADDRESS => {
        let mirrored_address = self.get_mirrored_palette_address(address) as usize;
        self.palettes[mirrored_address] = value;
      }
      _ => (),
    }
    trace_exit!();
  }

  #[named]
  pub fn read_u8(&mut self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let result = match address {
      CARTRIDGE_CHR_START_ADDRESS..=CARTRIDGE_CHR_END_ADDRESS => match self.cartridge {
        Some(ref cartridge) => cartridge.borrow().read_chr_u8(address),
        None => panic!("Attempted to read data from CHR RAM of a cartridge, but no cartridge was loaded."),
      },
      NAMETABLE_START_ADDRESS..=NAMETABLE_END_ADDRESS => {
        let mirroring_mode = self.get_mirroring_mode();
        let mirrored_address = self.get_mirrored_nametable_address(&mirroring_mode, address) as usize;
        self.nametables[mirrored_address]
      }
      PALETTE_START_ADDRESS..=PALETTE_END_ADDRESS => {
        let mirrored_address = self.get_mirrored_palette_address(address) as usize;
        self.palettes[mirrored_address]
      }
      _ => 0x00,
    };
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  pub fn buffered_read_u8(&mut self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let result = match address {
      PALETTE_START_ADDRESS..=PALETTE_END_ADDRESS => {
        let read_value = self.read_u8(address);
        self.read_buffer = read_value;
        read_value
      }
      _ => {
        let buffered_value = self.read_buffer;
        self.read_buffer = self.read_u8(address);
        buffered_value
      }
    };
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn get_mirroring_mode(&self) -> MirroringMode {
    trace_enter!();
    let result;
    if let Some(ref cartridge) = self.cartridge {
      result = cartridge.borrow().get_mirroring_mode();
    } else {
      result = MirroringMode::None;
    }
    trace_exit!();
    result
  }

  #[named]
  pub fn get_mirrored_nametable_address(&self, mirroring_mode: &MirroringMode, address: u16) -> u16 {
    use MirroringMode::*;
    trace_enter!();
    trace_var!(mirroring_mode);
    trace_u16!(address);
    let result = match mirroring_mode {
      None => address - 0x2000,
      Horizontal => ((address / 2) & NAMETABLE_SIZE) + (address % NAMETABLE_SIZE),
      Vertical => address % (2 * NAMETABLE_SIZE),
    };
    trace_u16!(result);
    trace_exit!();
    result
  }

  #[named]
  pub fn get_mirrored_palette_address(&self, address: u16) -> u16 {
    trace_enter!();
    let address = address % PALETTE_SIZE;
    let result = match address {
      0x10 | 0x14 | 0x18 | 0x1C => address - 0x10,
      _ => address,
    };
    trace_u16!(result);
    trace_exit!();
    result
  }
}
