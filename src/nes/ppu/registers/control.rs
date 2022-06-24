// 7  bit  0
// ---- ----
// VPHB SINN
// |||| ||||
// |||| ||++- Base nametable address
// |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
// |||| |+--- VRAM address increment per CPU read/write of PPUDATA
// |||| |     (0: add 1, going across; 1: add 32, going down)
// |||| +---- Sprite pattern table address for 8x8 sprites
// ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
// |||+------ Background pattern table address (0: $0000; 1: $1000)
// ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
// |+-------- PPU master/slave select
// |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
// +--------- Generate an NMI at the start of the
//            vertical blanking interval (0: off; 1: on)
pub const GENERATE_NMI_FLAG: u8 = 0b1000_0000;
pub const PPU_ROLE_SELECT_FLAG: u8 = 0b0100_0000;
pub const SPRITE_SIZE_FLAG: u8 = 0b0010_0000;
pub const BACKGROUND_ADDRESS_FLAG: u8 = 0b0001_0000;
pub const SPRITE_ADDRESS_FLAG: u8 = 0b0000_1000;
pub const VRAM_INCREMENT_FLAG: u8 = 0b0000_0100;
pub const NAMETABLE_2_FLAG: u8 = 0b0000_0010;
pub const NAMETABLE_1_FLAG: u8 = 0b0000_0001;

#[repr(u8)]
#[derive(Debug)]
pub enum ControlFlags {
  GenerateNmi = GENERATE_NMI_FLAG,
  RoleSelect = PPU_ROLE_SELECT_FLAG,
  SpriteSize = SPRITE_SIZE_FLAG,
  BackgroundAddress = BACKGROUND_ADDRESS_FLAG,
  SpriteAddress = SPRITE_ADDRESS_FLAG,
  VramIncrementSize = VRAM_INCREMENT_FLAG,
  Nametable2 = NAMETABLE_2_FLAG,
  Nametable1 = NAMETABLE_1_FLAG,
}

/// The PPU Control Register, or PPUCTRL ($2006)
pub struct ControlRegister {
  pub value: u8,
}

impl ControlRegister {
  pub fn new() -> ControlRegister {
    ControlRegister { value: 0 }
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
  pub fn set_control_flag(&mut self, flag: ControlFlags, value: bool) {
    trace_enter!();
    trace_var!(flag);
    trace_var!(value);
    if value {
      self.value = self.value | (flag as u8);
    } else {
      self.value = self.value & !(flag as u8);
    }
    trace!("{}", format_ppu_control_register!(self.value));
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_control_flag(&self, flag: ControlFlags) -> bool {
    trace_enter!();
    let result = self.value & (flag as u8) > 0;
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_generate_nmi_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_control_flag(ControlFlags::GenerateNmi, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_generate_nmi_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_control_flag(ControlFlags::GenerateNmi);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_role_select_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_control_flag(ControlFlags::RoleSelect, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_role_select_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_control_flag(ControlFlags::RoleSelect);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_sprite_size_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_control_flag(ControlFlags::SpriteSize, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_sprite_size_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_control_flag(ControlFlags::SpriteSize);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_background_address_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_control_flag(ControlFlags::BackgroundAddress, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_background_address_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_control_flag(ControlFlags::BackgroundAddress);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_sprite_address_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_control_flag(ControlFlags::SpriteAddress, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_sprite_address_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_control_flag(ControlFlags::SpriteAddress);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_vram_increment_size_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_control_flag(ControlFlags::VramIncrementSize, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_vram_increment_size_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_control_flag(ControlFlags::VramIncrementSize);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_nametable2_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_control_flag(ControlFlags::Nametable2, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_nametable2_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_control_flag(ControlFlags::Nametable2);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_nametable1_flag(&mut self, value: bool) {
    trace_enter!();
    trace_var!(value);
    self.set_control_flag(ControlFlags::Nametable1, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_nametable1_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_control_flag(ControlFlags::Nametable1);
    trace_result!(result);
    result
  }

  #[named]
  #[inline]
  pub fn get_vram_address_increment(&self) -> u8 {
    trace_enter!();
    let result;
    if self.get_control_flag(ControlFlags::VramIncrementSize) {
      result = 32;
    } else {
      result = 1;
    }
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn get_base_nametable_address(&self) -> u16 {
    trace_enter!();
    let result = match self.value & 0x03 {
      0 => 0x2000,
      1 => 0x2400,
      2 => 0x2800,
      3 => 0x2C00,
      _ => panic!("Impossible!"),
    };
    trace_u16!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn get_sprite_pattern_address(&self) -> u16 {
    trace_enter!();
    let result;
    if self.get_sprite_address_flag() {
      result = 0x1000;
    } else {
      result = 0x0000;
    }
    trace_u16!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn get_background_pattern_address(&self) -> u16 {
    trace_enter!();
    let result;
    if self.get_background_address_flag() {
      result = 0x1000;
    } else {
      result = 0x0000;
    }
    trace_u16!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn get_sprite_size(&self) -> u8 {
    trace_enter!();
    let result;
    if self.get_sprite_size_flag() {
      result = 16;
    } else {
      result = 8;
    }
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn get_role(&self) -> u8 {
    trace_enter!();
    let result = self.get_role_select_flag() as u8;
    trace_u8!(result);
    trace_exit!();
    result
  }
}
