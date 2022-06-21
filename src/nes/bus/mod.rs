use std::cell::RefCell;
use std::rc::Rc;

use crate::cartridge::Cartridge;
use crate::traits::Mappable;

const PROGRAM_CONTROL_ADDRESS: u16 = 0xFFFC;
const MAX_ADDRESS: u16 = 0xFFFF;

const RAM_START_ADDRESS: u16 = 0x0000;
const RAM_ACTUAL_END_ADDRESS: u16 = 0x07FF;
const RAM_RANGE_END_ADDRESS: u16 = 0x1FFF;

const PPU_REGISTER_START_ADDRESS: u16 = 0x2000;
const PPU_REGISTER_ACTUAL_END_ADDRESS: u16 = 0x2007;
const PPU_REGISTER_RANGE_END_ADDRESS: u16 = 0x3FFF;

const CARTRIDGE_START_ADDRESS: u16 = 0x4018;
const CARTRIDGE_END_ADDRESS: u16 = MAX_ADDRESS;

pub mod addressable;
pub use addressable::*;

pub mod busable;
pub use busable::*;

pub mod interruptible;
pub use interruptible::*;

pub struct Bus {
  memory: [u8; (RAM_ACTUAL_END_ADDRESS + 1) as usize],
  cartridge: Option<Rc<RefCell<Cartridge>>>,
}

impl Bus {
  pub fn new() -> Bus {
    Bus {
      memory: [0; (RAM_ACTUAL_END_ADDRESS + 1) as usize],
      cartridge: None,
    }
  }

  #[named]
  pub fn inner_read_u8(&self, address: u16) -> u8 {
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
        todo!("Todo: PPU.");
        0x00
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
        todo!("Todo: PPU.");
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
    self.cartridge = Some(cartridge);
    trace_exit!();
  }
}
