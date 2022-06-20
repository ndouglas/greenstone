use super::super::*;
use crate::traits::Addressable;
use crate::traits::Interruptible;

pub const STACK_BASE_ADDRESS: u16 = 0x0100;

impl CPU {
  #[inline]
  #[named]
  pub fn push_u8(&mut self, data: u8) {
    trace_enter!();
    trace_u8!(data);
    let address = STACK_BASE_ADDRESS + self.stack_pointer as u16;
    self.write_u8(address, data);
    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn pop_u8(&mut self) -> u8 {
    trace_enter!();
    self.stack_pointer = self.stack_pointer.wrapping_add(1);
    let address = STACK_BASE_ADDRESS + self.stack_pointer as u16;
    let result = self.read_u8(address);
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[inline]
  #[named]
  pub fn push_u16(&mut self, data: u16) {
    trace_enter!();
    trace_u16!(data);
    let bytes = data.to_le_bytes();
    self.push_u8(bytes[1]);
    self.push_u8(bytes[0]);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn pop_u16(&mut self) -> u16 {
    trace_enter!();
    let result = u16::from_le_bytes([self.pop_u8(), self.pop_u8()]);
    trace_u16!(result);
    trace_exit!();
    result
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_stack() {
    init();
    let mut cpu = CPU::new();
    cpu.handle_reset();
    assert_eq!(0x03, {
      cpu.push_u8(0x03);
      cpu.pop_u8()
    });
    assert_eq!(0x1234, {
      cpu.push_u16(0x1234);
      cpu.pop_u16()
    });
    assert_eq!(0x03, {
      cpu.push_u8(0x03);
      cpu.push_u8(0x04);
      assert_eq!(0x04, cpu.pop_u8());
      cpu.pop_u8()
    });
    assert_eq!(0x03, {
      cpu.push_u8(0x03);
      cpu.push_u16(0x1234);
      cpu.push_u8(0x04);
      assert_eq!(cpu.pop_u8(), 0x04);
      assert_eq!(cpu.pop_u16(), 0x1234);
      cpu.pop_u8()
    });
    assert_eq!(0x0305, {
      cpu.push_u8(0x04);
      cpu.push_u16(0x0305);
      cpu.push_u16(0x1234);
      cpu.push_u8(0x05);
      assert_eq!(cpu.pop_u8(), 0x05);
      assert_eq!(cpu.pop_u16(), 0x1234);
      let result = cpu.pop_u16();
      assert_eq!(cpu.pop_u8(), 0x04);
      result
    });
  }
}
