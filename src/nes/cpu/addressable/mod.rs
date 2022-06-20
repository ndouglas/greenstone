use super::super::*;
use crate::traits::Addressable;

impl Addressable for CPU<'_> {

  #[named]
  #[inline]
  fn unclocked_read_u8(&mut self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let result = self.bus.unclocked_read_u8(address);
    trace_result!(result);
    result
  }

  #[named]
  #[inline]
  fn unclocked_write_u8(&mut self, address: u16, data: u8) {
    trace_enter!();
    trace_u16!(address);
    trace_u8!(data);
    self.bus.unclocked_write_u8(address, data);
    trace_exit!();
  }

  #[named]
  #[inline]
  fn read_u8(&mut self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    self.tick();
    let result = self.bus.read_u8(address);
    trace_result!(result);
    result
  }

  #[named]
  #[inline]
  fn write_u8(&mut self, address: u16, data: u8) {
    trace_enter!();
    trace_u16!(address);
    trace_u8!(data);
    self.tick();
    self.bus.write_u8(address, data);
    trace_exit!();
  }

  #[named]
  fn load(&mut self, program: Vec<u8>, start: u16) {
    trace_enter!();
    self.bus.load(program, start);
    trace_exit!();
  }

  #[named]
  #[inline]
  fn tick(&mut self) {
    trace_enter!();
    self.clock_counter = self.clock_counter.wrapping_add(1);
    debug!("Tick {}", self.clock_counter);
    self.bus.tick();
    trace_exit!();
  }

}
