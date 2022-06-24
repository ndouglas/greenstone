// 7  bit  0
// ---- ----
// VSO. ....
// |||| ||||
// |||+-++++- PPU open bus. Returns stale PPU bus contents.
// ||+------- Sprite overflow. The intent was for this flag to be set
// ||         whenever more than eight sprites appear on a scanline, but a
// ||         hardware bug causes the actual behavior to be more complicated
// ||         and generate false positives as well as false negatives; see
// ||         PPU sprite evaluation. This flag is set during sprite
// ||         evaluation and cleared at dot 1 (the second dot) of the
// ||         pre-render line.
// |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
// |          a nonzero background pixel; cleared at dot 1 of the pre-render
// |          line.  Used for raster timing.
// +--------- Vertical blank has started (0: not in vblank; 1: in vblank).
//            Set at dot 1 of line 241 (the line *after* the post-render
//            line); cleared after reading $2002 and at dot 1 of the
//            pre-render line.
pub const VERTICAL_BLANK_FLAG: u8 = 0b1000_0000;
pub const SPRITE_ZERO_HIT_FLAG: u8 = 0b0100_0000;
pub const SPRITE_OVERFLOW_FLAG: u8 = 0b0010_0000;
pub const PPU_OPEN_BUS4_FLAG: u8 = 0b0001_0000;
pub const PPU_OPEN_BUS3_FLAG: u8 = 0b0000_1000;
pub const PPU_OPEN_BUS2_FLAG: u8 = 0b0000_0100;
pub const PPU_OPEN_BUS1_FLAG: u8 = 0b0000_0010;
pub const PPU_OPEN_BUS0_FLAG: u8 = 0b0000_0001;

#[derive(Debug)]
pub enum StatusFlags {
  VerticalBlank,
  SpriteZeroHit,
  SpriteOverflow,
}

pub struct StatusRegister {
  value: u8,
}

impl StatusRegister {
  pub fn new() -> StatusRegister {
    StatusRegister { value: 0 }
  }

  #[named]
  #[inline]
  pub fn read_u8(&self) -> u8 {
    trace_enter!();
    let result = self.value;
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn write_u8(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    self.value = value;
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn set_status_flag(&mut self, flag: StatusFlags, value: bool) {
    trace_enter!();
    trace_var!(flag);
    trace_var!(value);
    if value {
      self.value = self.value | (flag as u8);
    } else {
      self.value = self.value & !(flag as u8);
    }
    trace!("{}", format_ppu_status_register!(self.value));
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_status_flag(&self, flag: StatusFlags) -> bool {
    trace_enter!();
    let result = self.value & (flag as u8) > 0;
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_vertical_blank_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_status_flag(StatusFlags::VerticalBlank, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_vertical_blank_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_status_flag(StatusFlags::VerticalBlank);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_sprite_zero_hit_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_status_flag(StatusFlags::SpriteZeroHit, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_sprite_zero_hit_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_status_flag(StatusFlags::SpriteZeroHit);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_sprite_overflow_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_status_flag(StatusFlags::SpriteOverflow, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_sprite_overflow_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_status_flag(StatusFlags::SpriteOverflow);
    trace_result!(result);
    result
  }
}
