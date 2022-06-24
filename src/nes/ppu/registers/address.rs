/// The PPU Address Register, or PPUADDR ($2006)
pub struct AddressRegister {
  /// This is a Big-Endian (!!!) address.
  value: (u8, u8),
  /// Whether we're pointing to the high byte or not at the moment.
  is_high_byte: bool,
}

impl AddressRegister {
  pub fn new() -> AddressRegister {
    AddressRegister {
      value: (0, 0),
      is_high_byte: true,
    }
  }

  #[named]
  #[inline]
  pub fn write_u16(&mut self, value: u16) {
    trace_enter!();
    trace_u16!(value);
    self.value.0 = (value >> 8) as u8;
    self.value.1 = (value & 0xFF) as u8;
    trace_var!(self.value);
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn read_u16(&self) -> u16 {
    trace_enter!();
    let result = ((self.value.0 as u16) << 8) | (self.value.1 as u16);
    trace_u16!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn write_u8(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    if self.is_high_byte {
      self.value.0 = value;
    } else {
      self.value.1 = value;
    }
    self.is_high_byte = !self.is_high_byte;
    trace_var!(self.is_high_byte);
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn increment(&mut self, offset: u8) {
    trace_enter!();
    trace_u8!(offset);
    let lo = self.value.1;
    self.value.1 = self.value.1.wrapping_add(offset);
    if lo > self.value.1 {
      self.value.0 = self.value.0.wrapping_add(1);
    }
    if self.value.0 > 0x3F {
      self.value.0 = self.value.0 & 0x3F;
    }
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn reset_latch(&mut self) {
    trace_enter!();
    self.is_high_byte = true;
    trace_exit!();
  }
}

#[cfg(test)]
mod test {

  use super::*;
  use crate::test::init;

  #[test]
  fn test_address_register() {
    init();
    let mut register = AddressRegister::new();
    register.write_u16(0x0203);
    assert_eq!(register.read_u16(), 0x0203);
    register.write_u8(0x04);
    assert_eq!(register.read_u16(), 0x0403);
    register.write_u8(0x05);
    assert_eq!(register.read_u16(), 0x0405);
  }
}
