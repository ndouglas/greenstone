#[derive(Debug)]
pub struct ScrollRegister {
  pub scroll_x: u8,
  pub scroll_y: u8,
  pub is_latched: bool,
}

impl ScrollRegister {
  pub fn new() -> ScrollRegister {
    ScrollRegister {
      scroll_x: 0,
      scroll_y: 0,
      is_latched: false,
    }
  }

  #[named]
  #[inline]
  pub fn write_u8(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    if !self.is_latched {
      self.scroll_x = value;
    } else {
      self.scroll_y = value;
    }
    self.is_latched = !self.is_latched;
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn reset_latch(&mut self) {
    self.is_latched = false;
  }
}
