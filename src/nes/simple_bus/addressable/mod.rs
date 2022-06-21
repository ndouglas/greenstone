use super::*;
use crate::traits::Addressable;

impl Addressable for SimpleBus {
  #[named]
  fn unclocked_read_u8(&self, address: u16) -> u8 {
    trace_enter!();
    let result = self.memory[address as usize];
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  fn unclocked_write_u8(&mut self, address: u16, data: u8) {
    trace_enter!();
    self.memory[address as usize] = data;
    trace_exit!();
  }

  #[named]
  fn load(&mut self, program: Vec<u8>, start: u16) {
    trace_enter!();
    let start_address = start as usize;
    self.memory[start_address..(start_address + program.len())].copy_from_slice(&program[..]);
    trace_exit!();
  }

  #[named]
  fn tick(&mut self) {
    trace_enter!();
    trace_exit!();
  }
}
