const PROGRAM_CONTROL_ADDRESS: u16 = 0xFFFC;
const MAX_ADDRESS: u16 = 0xFFFF;

const RAM_START_ADDRESS: u16 = 0x0000;
const RAM_ACTUAL_MAX_ADDRESS: u16 = 0x07FF;
const RAM_RANGE_MAX_ADDRESS: u16 = 0x1FFF;

const PPU_REGISTER_START_ADDRESS: u16 = 0x2000;
const PPU_REGISTER_ACTUAL_MAX_ADDRESS: u16 = 0x2007;
const PPU_REGISTER_RANGE_MAX_ADDRESS: u16 = 0x3FFF;

pub mod addressable;
pub use addressable::*;

pub mod busable;
pub use busable::*;

pub mod interruptible;
pub use interruptible::*;

pub struct Bus {
  memory: [u8; (RAM_ACTUAL_MAX_ADDRESS + 1) as usize],
}

impl Bus {
  pub fn new() -> Bus {
    Bus {
      memory: [0; (RAM_ACTUAL_MAX_ADDRESS + 1) as usize],
    }
  }

  #[named]
  pub fn inner_read_u8(&mut self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let result = match address {
      RAM_START_ADDRESS..=RAM_RANGE_MAX_ADDRESS => {
        let actual_address = address & RAM_ACTUAL_MAX_ADDRESS;
        trace_u16!(actual_address);
        self.memory[actual_address as usize]
      }
      PPU_REGISTER_START_ADDRESS..=PPU_REGISTER_RANGE_MAX_ADDRESS => {
        let actual_address = address & PPU_REGISTER_ACTUAL_MAX_ADDRESS;
        trace_u16!(actual_address);
        todo!("Todo: PPU.");
        0x00
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
  pub fn inner_write_u8(&mut self, address: u16, data: u8) {
    trace_enter!();
    trace_u16!(address);
    match address {
      RAM_START_ADDRESS..=RAM_RANGE_MAX_ADDRESS => {
        let actual_address = address & RAM_ACTUAL_MAX_ADDRESS;
        trace_u16!(actual_address);
        self.memory[actual_address as usize] = data;
      }
      PPU_REGISTER_START_ADDRESS..=PPU_REGISTER_RANGE_MAX_ADDRESS => {
        let actual_address = address & PPU_REGISTER_ACTUAL_MAX_ADDRESS;
        trace_u16!(actual_address);
        todo!("Todo: PPU.");
      }
      _ => {
        warn!("Ignoring out-of-bounds memory write at address {}", format_u16!(address));
      }
    }
    trace_exit!();
  }
}
