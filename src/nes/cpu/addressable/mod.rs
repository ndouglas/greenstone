use super::super::*;
use crate::traits::Addressable;

impl Addressable for CPU<'_> {
  #[named]
  fn read_u8(&mut self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    self.tick();
    let result = self.unclocked_read_u8(address);
    trace_result!(result);
    result
  }

  #[named]
  fn write_u8(&mut self, address: u16, data: u8) {
    trace_enter!();
    trace_u16!(address);
    trace_u16!(data);
    self.tick();
    self.unclocked_write_u8(address, data);
    trace_exit!();
  }

  #[named]
  fn load(&mut self, program: Vec<u8>, start: u16) {
    trace_enter!();
    self.bus.load(program, start);
    trace_exit!();
  }

  #[named]
  fn tick(&mut self) {
    trace_enter!();
    self.clock_counter = self.clock_counter.wrapping_add(1);
    debug!("Tick {}", self.clock_counter);
    self.bus.tick();
    trace_exit!();
  }
}
