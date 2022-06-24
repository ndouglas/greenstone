use super::*;

pub mod registers;
pub use registers::*;

pub mod vram;
pub use vram::*;

pub const CONTROL_REGISTER_INDEX: u8 = 0;
pub const MASK_REGISTER_INDEX: u8 = 1;
pub const STATUS_REGISTER_INDEX: u8 = 2;
pub const OAM_ADDRESS_REGISTER_INDEX: u8 = 3;
pub const OAM_DATA_REGISTER_INDEX: u8 = 4;
pub const SCROLL_REGISTER_INDEX: u8 = 5;
pub const ADDRESS_REGISTER_INDEX: u8 = 6;
pub const DATA_REGISTER_INDEX: u8 = 7;

/// NES Picture-Processing Unit
pub struct PPU {
  /// Control Register.
  control_register: ControlRegister,
  /// Mask Register.
  mask_register: MaskRegister,
  /// Address Register.``
  address_register: AddressRegister,
  /// The PPU <-> CPU data bus, latching but constantly decaying...
  latching_bus: u8,
}

impl PPU {
  pub fn new() -> PPU {
    PPU {
      control_register: ControlRegister::new(),
      mask_register: MaskRegister::new(),
      address_register: AddressRegister::new(),
      latching_bus: 0x00,
    }
  }

  #[named]
  #[inline]
  pub fn read_u8(&mut self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let index = (address % 8) as u8;
    let result = match index {
      // By default, reads (including of write-only registers), return the
      // value on the decaying latching data bus.
      _ => self.latching_bus,
    };
    self.latching_bus = result;
    trace_result!(result);
    result
  }

  #[named]
  #[inline]
  pub fn write_u8(&mut self, address: u16, value: u8) {
    trace_enter!();
    trace_u16!(address);
    trace_u16!(value);
    let index = (address % 8) as u8;
    match index {
      CONTROL_REGISTER_INDEX => self.control_register.write_u8(value),
      MASK_REGISTER_INDEX => self.mask_register.write_u8(value),
      // Read-only!
      STATUS_REGISTER_INDEX => (),
      OAM_ADDRESS_REGISTER_INDEX => (),
      OAM_DATA_REGISTER_INDEX => (),
      SCROLL_REGISTER_INDEX => (),
      ADDRESS_REGISTER_INDEX => self.address_register.write_u8(value),
      DATA_REGISTER_INDEX => (),
      _ => (),
    };
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn tick(&mut self) {
    trace_enter!();
    trace_exit!();
  }
}
