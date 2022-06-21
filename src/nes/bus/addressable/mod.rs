use super::*;
use crate::traits::Addressable;

impl Addressable for Bus {
  #[named]
  #[inline]
  fn unclocked_read_u8(&self, address: u16) -> u8 {
    trace_enter!();
    let result = self.inner_read_u8(address);
    trace_result!(result);
    result
  }

  #[named]
  #[inline]
  fn unclocked_write_u8(&mut self, address: u16, data: u8) {
    trace_enter!();
    self.inner_write_u8(address, data);
    trace_exit!();
  }

  #[named]
  #[inline]
  fn load(&mut self, program: Vec<u8>, start: u16) {
    trace_enter!();
    let start_address = start as usize;
    self.memory[start_address..(start_address + program.len())].copy_from_slice(&program[..]);
    self.write_u16(PROGRAM_CONTROL_ADDRESS.try_into().unwrap(), start);
    trace_exit!();
  }

  #[named]
  #[inline]
  fn tick(&mut self) {}
}
