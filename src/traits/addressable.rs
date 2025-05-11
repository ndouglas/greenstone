// A trait for addressable things: buses, memory, etc.
pub trait Addressable {
  fn unclocked_read_u8(&mut self, address: u16) -> u8;

  fn unclocked_write_u8(&mut self, address: u16, data: u8);

  fn load(&mut self, program: Vec<u8>, start: u16);

  fn tick(&mut self);

  #[named]
  #[inline]
  fn unclocked_read_u16(&mut self, address: u16) -> u16 {
    trace_enter!();
    let result = u16::from_le_bytes([self.unclocked_read_u8(address), self.unclocked_read_u8(address.wrapping_add(1))]);
    trace_u16!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  fn unclocked_write_u16(&mut self, address: u16, data: u16) {
    trace_enter!();
    let hi = (data >> 8) as u8;
    let lo = (data & 0xFF) as u8;
    self.unclocked_write_u8(address, lo);
    self.unclocked_write_u8(address.wrapping_add(1), hi);
    trace_exit!();
  }

  #[named]
  #[inline]
  fn read_u8(&mut self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    self.tick();
    let result = self.unclocked_read_u8(address);
    trace_result!(result);
    result
  }

  #[named]
  #[inline]
  fn read_u16(&mut self, address: u16) -> u16 {
    trace_enter!();
    let result = u16::from_le_bytes([self.read_u8(address), self.read_u8(address.wrapping_add(1))]);
    trace_u16!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  fn write_u8(&mut self, address: u16, data: u8) {
    trace_enter!();
    trace_u16!(address);
    trace_u16!(data);
    self.tick();
    self.unclocked_write_u8(address, data);
    trace_exit!();
  }

  #[named]
  #[inline]
  fn write_u16(&mut self, address: u16, data: u16) {
    let hi = (data >> 8) as u8;
    let lo = (data & 0xFF) as u8;
    self.write_u8(address, lo);
    self.write_u8(address.wrapping_add(1), hi);
  }

  /// Get the PPU framebuffer (256x240 RGB). Default returns empty slice.
  fn get_framebuffer(&self) -> &[u8] {
    &[]
  }

  /// Check if a new frame is ready and clear the flag. Default returns false.
  fn take_frame_ready(&mut self) -> bool {
    false
  }
}
