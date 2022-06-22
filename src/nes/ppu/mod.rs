pub struct PPU {}

impl PPU {
  pub fn new() -> PPU {
    PPU {}
  }

  #[named]
  #[inline]
  pub fn read_u8(&mut self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let result = 0x00;
    trace_result!(result);
    result
  }

  #[named]
  #[inline]
  pub fn write_u8(&mut self, address: u16, data: u8) {
    trace_enter!();
    trace_u16!(address);
    trace_u16!(data);
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn tick(&mut self) {
    trace_enter!();
    trace_exit!();
  }
}
