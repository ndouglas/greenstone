// 7  bit  0
// ---- ----
// BGRs bMmG
// |||| ||||
// |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
// |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
// |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
// |||| +---- 1: Show background
// |||+------ 1: Show sprites
// ||+------- Emphasize red
// |+-------- Emphasize green
// +--------- Emphasize blue

pub const EMPHASIZE_BLUE_FLAG: u8 = 0b1000_0000;
pub const EMPHASIZE_GREEN_FLAG: u8 = 0b0100_0000;
pub const EMPHASIZE_RED_FLAG: u8 = 0b0010_0000;
pub const SHOW_SPRITES_FLAG: u8 = 0b0001_0000;
pub const SHOW_BACKGROUND_FLAG: u8 = 0b0000_1000;
pub const SHOW_SPRITES_LEFT_FLAG: u8 = 0b0000_0100;
pub const SHOW_BACKGROUND_LEFT_FLAG: u8 = 0b0000_0010;
pub const GREYSCALE_FLAG: u8 = 0b0000_0001;

#[repr(u8)]
#[derive(Debug)]
pub enum MaskFlags {
  EmphasizeBlue = EMPHASIZE_BLUE_FLAG,
  EmphasizeGreen = EMPHASIZE_GREEN_FLAG,
  EmphasizeRed = EMPHASIZE_RED_FLAG,
  ShowSprites = SHOW_SPRITES_FLAG,
  ShowBackground = SHOW_BACKGROUND_FLAG,
  ShowSpritesLeft = SHOW_SPRITES_LEFT_FLAG,
  ShowBackgroundLeft = SHOW_BACKGROUND_LEFT_FLAG,
  Greyscale = GREYSCALE_FLAG,
}

// Color channel for emphasis - reserved for future use
#[allow(dead_code)]
pub enum ColorChannel {
  Red,
  Green,
  Blue,
}

pub struct MaskRegister {
  pub value: u8,
}

impl Default for MaskRegister {
  fn default() -> Self {
    Self::new()
  }
}

impl MaskRegister {
  pub fn new() -> MaskRegister {
    MaskRegister { value: 0 }
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
  pub fn set_mask_flag(&mut self, flag: MaskFlags, value: bool) {
    trace_enter!();
    trace_var!(flag);
    trace_var!(value);
    if value {
      self.value |= flag as u8;
    } else {
      self.value &= !(flag as u8);
    }
    trace!("{}", format_ppu_mask_register!(self.value));
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_mask_flag(&self, flag: MaskFlags) -> bool {
    trace_enter!();
    let result = self.value & (flag as u8) > 0;
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_emphasize_blue_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_mask_flag(MaskFlags::EmphasizeBlue, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_emphasize_blue_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_mask_flag(MaskFlags::EmphasizeBlue);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_emphasize_green_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_mask_flag(MaskFlags::EmphasizeGreen, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_emphasize_green_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_mask_flag(MaskFlags::EmphasizeGreen);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_emphasize_red_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_mask_flag(MaskFlags::EmphasizeRed, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_emphasize_red_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_mask_flag(MaskFlags::EmphasizeRed);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_show_sprites_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_mask_flag(MaskFlags::ShowSprites, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_show_sprites_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_mask_flag(MaskFlags::ShowSprites);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_show_background_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_mask_flag(MaskFlags::ShowBackground, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_show_background_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_mask_flag(MaskFlags::ShowBackground);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_show_sprites_left_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_mask_flag(MaskFlags::ShowSpritesLeft, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_show_sprites_left_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_mask_flag(MaskFlags::ShowSpritesLeft);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_show_background_left_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_mask_flag(MaskFlags::ShowBackgroundLeft, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_show_background_left_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_mask_flag(MaskFlags::ShowBackgroundLeft);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_greyscale_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_mask_flag(MaskFlags::Greyscale, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_greyscale_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_mask_flag(MaskFlags::Greyscale);
    trace_result!(result);
    result
  }
}
