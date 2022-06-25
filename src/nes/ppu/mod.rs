pub mod address;
pub use address::*;

pub mod registers;
pub use registers::*;

pub mod vram;
pub use vram::*;

pub const CONTROL_REGISTER_INDEX: u8 = 0;
pub const MASK_REGISTER_INDEX: u8 = 1;
pub const STATUS_REGISTER_INDEX: u8 = 2;
pub const OAM_ADDRESS_REGISTER_INDEX: u8 = 3;
pub const OAM_DATA_REGISTER_INDEX: u8 = 4;
pub const SCROLL_REGISTER_INDEX: u8 = 5;
pub const ADDRESS_REGISTER_INDEX: u8 = 6;
pub const DATA_REGISTER_INDEX: u8 = 7;

pub const OAM_RAM_SIZE: u16 = 256;

/// NES Picture-Processing Unit
pub struct PPU {
  /// Control Register.
  pub control_register: ControlRegister,
  /// Mask Register.
  pub mask_register: MaskRegister,
  /// Status Register.
  pub status_register: StatusRegister,
  /// OAM Address Register.
  pub oam_address: u8,
  /// The PPU <-> CPU data bus, latching but constantly decaying...
  pub latching_bus: u8,
  /// Video RAM.
  pub vram: VRAM,
  /// OAM RAM.
  pub oam_ram: Vec<u8>,
  /// Whether we're forcing an NMI.
  pub force_nmi: bool,
  /// Whether we're currently latched.
  pub is_latched: bool,
  /// Loopy "t" address.
  pub t_address: Address,
  /// Loopy "v" address.
  pub v_address: Address,
  /// Fine "x" coordinate for scrolling, etc.
  pub fine_x: u8,
  /// Suppress vertical blank signals.
  pub suppress_vblanks: bool,
}

impl PPU {
  pub fn new() -> PPU {
    PPU {
      control_register: ControlRegister::new(),
      mask_register: MaskRegister::new(),
      status_register: StatusRegister::new(),
      oam_address: 0x00,
      latching_bus: 0x00,
      vram: VRAM::new(),
      oam_ram: vec![0; OAM_RAM_SIZE as usize],
      force_nmi: false,
      is_latched: false,
      t_address: Address(0),
      v_address: Address(0),
      fine_x: 0,
      suppress_vblanks: false,
    }
  }

  #[named]
  #[inline]
  pub fn read_register(&mut self, index: u8) -> u8 {
    trace_enter!();
    trace_u8!(index);
    let result = match index {
      STATUS_REGISTER_INDEX => self.read_status_register(),
      OAM_DATA_REGISTER_INDEX => self.read_oam_data_register(),
      DATA_REGISTER_INDEX => self.read_data_register(),
      // By default, reads (including of write-only registers), return the
      // value on the decaying latching data bus.
      _ => self.latching_bus,
    };
    self.latching_bus = result;
    trace_result!(result);
    result
  }

  #[named]
  #[inline]
  pub fn write_register(&mut self, index: u8, value: u8) {
    trace_enter!();
    trace_u16!(index);
    trace_u8!(value);
    self.latching_bus = value;
    match index {
      CONTROL_REGISTER_INDEX => self.write_control_register(value),
      MASK_REGISTER_INDEX => self.write_mask_register(value),
      // Read-only!
      STATUS_REGISTER_INDEX => (),
      OAM_ADDRESS_REGISTER_INDEX => self.write_oam_address_register(value),
      OAM_DATA_REGISTER_INDEX => self.write_oam_data_register(value),
      SCROLL_REGISTER_INDEX => self.write_scroll_register(value),
      ADDRESS_REGISTER_INDEX => self.write_address_register(value),
      DATA_REGISTER_INDEX => self.write_data_register(value),
      _ => (),
    };
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn should_force_nmi(&self, new_control: u8) -> bool {
    trace_enter!();
    trace_u8!(new_control);
    let new_control_generates_nmi = (new_control & GENERATE_NMI_FLAG) > 0;
    trace_var!(new_control_generates_nmi);
    let current_control_generates_nmi = self.control_register.get_generate_nmi_flag();
    trace_var!(current_control_generates_nmi);
    let result = !current_control_generates_nmi && new_control_generates_nmi;
    trace_var!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn write_control_register(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    if self.should_force_nmi(value) {
      self.force_nmi = true;
    }
    self.control_register.write_u8(value);
    self.t_address.set_nametable(self.control_register.get_nametable());
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn write_mask_register(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    self.mask_register.write_u8(value);
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn write_oam_address_register(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    self.oam_address = value;
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn write_oam_data_register(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    self.oam_ram[self.oam_address as usize] = value;
    self.oam_address = self.oam_address.wrapping_add(1);
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn read_oam_data_register(&mut self) -> u8 {
    trace_enter!();
    let result = self.oam_ram[self.oam_address as usize];
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn write_address_register(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    if self.is_latched {
      self.t_address.set_low_byte(value);
      self.v_address = self.t_address.clone();
    } else {
      self.t_address.set_high_byte(value);
    }
    trace_u16!(self.v_address.address());
    trace_u16!(self.t_address.address());
    self.is_latched = !self.is_latched;
    trace_var!(self.is_latched);
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn write_status_register(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    self.status_register.write_u8(value);
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn write_data_register(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    self.vram.write_u8(self.v_address.address(), value);
    self.v_address.increment(self.control_register.get_vram_address_increment() as u16);
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn write_scroll_register(&mut self, value: u8) {
    trace_enter!();
    trace_u8!(value);
    if self.is_latched {
      self.t_address.set_fine_y(value);
      self.t_address.set_coarse_y(value >> 3);
    } else {
      self.fine_x = value & 0b0000_0111;
      self.t_address.set_coarse_x(value >> 3);
    }
    trace_u16!(self.fine_x);
    trace_u16!(self.t_address.address());
    self.is_latched = !self.is_latched;
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn read_status_register(&mut self) -> u8 {
    trace_enter!();
    let mut result = self.status_register.read_u8();
    self.status_register.set_vertical_blank_flag(false);
    self.is_latched = false;
    self.suppress_vblanks = true;
    // Status register includes some garbage from the bus.
    result = result | (self.latching_bus & 0b0001_1111);
    trace_result!(result);
    result
  }

  #[named]
  #[inline]
  pub fn read_data_register(&mut self) -> u8 {
    trace_enter!();
    let address = self.v_address.address();
    self.v_address.increment(self.control_register.get_vram_address_increment() as u16);
    let result = self.vram.buffered_read_u8(address);
    trace_result!(result);
    result
  }

  #[named]
  #[inline]
  pub fn write_oam_dma(&mut self, data: &[u8; 256]) {
    for x in data.iter() {
      self.oam_ram[self.oam_address as usize] = *x;
      self.oam_address = self.oam_address.wrapping_add(1);
    }
  }

  #[named]
  #[inline]
  pub fn tick(&mut self) {
    trace_enter!();
    trace_exit!();
  }

  #[named]
  pub fn reset(&mut self) {
    trace_enter!();
    self.control_register = ControlRegister::new();
    self.mask_register = MaskRegister::new();
    self.status_register = StatusRegister::new();
    self.vram.reset();
    self.oam_ram = vec![0; OAM_RAM_SIZE as usize];
    trace_exit!();
  }
}

// The following test cases come from Starr Horne's `nes-rust`
// and Rafael Bagmanov's NES eBook
// See https://github.com/starrhorne/nes-rust/blob/master/src/ppu/registers.rs
// and
// https://github.com/bugzmanov/nes_ebook/blob/master/code/ch6.1/src/ppu/mod.rs
#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;
  use crate::Cartridge;
  use std::cell::RefCell;
  use std::rc::Rc;

  #[test]
  fn test_write_control_register() {
    init();
    let mut ppu = PPU::new();
    ppu.write_register(CONTROL_REGISTER_INDEX, 0b1010_1010);
    assert_eq!(ppu.control_register.read_u8(), 0b1010_1010);
  }

  #[test]
  fn test_write_mask_register() {
    init();
    let mut ppu = PPU::new();
    ppu.write_register(MASK_REGISTER_INDEX, 0b1010_1010);
    assert_eq!(ppu.mask_register.read_u8(), 0b1010_1010);
  }

  #[test]
  fn test_write_oam_address_register() {
    init();
    let mut ppu = PPU::new();
    ppu.write_register(OAM_ADDRESS_REGISTER_INDEX, 0xF0);
    assert_eq!(ppu.oam_address, 0xF0);
  }

  #[test]
  fn test_write_oam_data_register() {
    init();
    let mut ppu = PPU::new();
    ppu.oam_address = 5;
    ppu.write_register(OAM_DATA_REGISTER_INDEX, 0xF0);
    assert_eq!(ppu.oam_ram[5], 0xF0);
    assert_eq!(ppu.oam_address, 6);
  }

  #[test]
  fn test_read_status_register() {
    init();
    let mut ppu = PPU::new();
    ppu.is_latched = true;
    ppu.write_status_register(0b1110_0000);
    let status = ppu.read_register(STATUS_REGISTER_INDEX);
    assert_eq!(status, 0b1110_0000);
    assert_eq!(ppu.is_latched, false);
    assert_eq!(ppu.status_register.get_vertical_blank_flag(), false);
  }

  #[test]
  fn test_read_ghost_bits() {
    init();
    let mut ppu = PPU::new();
    ppu.write_register(STATUS_REGISTER_INDEX, 0b1111_1111);
    ppu.write_status_register(0b0000_0000);
    assert_eq!(ppu.read_register(STATUS_REGISTER_INDEX), 0b0001_1111);
    assert_eq!(ppu.read_register(CONTROL_REGISTER_INDEX), 0b0001_1111);
    assert_eq!(ppu.read_register(MASK_REGISTER_INDEX), 0b0001_1111);
    assert_eq!(ppu.read_register(OAM_ADDRESS_REGISTER_INDEX), 0b0001_1111);
    assert_eq!(ppu.read_register(SCROLL_REGISTER_INDEX), 0b0001_1111);
    assert_eq!(ppu.read_register(ADDRESS_REGISTER_INDEX), 0b0001_1111);
  }

  #[test]
  fn test_read_oam_data() {
    init();
    let mut ppu = PPU::new();
    ppu.oam_ram[5] = 0x0F;
    ppu.oam_address = 5;
    assert_eq!(ppu.read_register(OAM_DATA_REGISTER_INDEX), 0x0F);
    assert_eq!(ppu.oam_address, 5);
  }

  #[test]
  fn test_read_data_delayed() {
    init();
    let mut ppu = PPU::new();
    ppu.vram.write_u8(0x2001, 1);
    ppu.vram.write_u8(0x2002, 2);
    ppu.vram.write_u8(0x2003, 3);
    ppu.v_address.0 = 0x2001;
    ppu.read_register(DATA_REGISTER_INDEX);
    assert_eq!(ppu.read_register(DATA_REGISTER_INDEX), 1);
    assert_eq!(ppu.read_register(DATA_REGISTER_INDEX), 2);
    assert_eq!(ppu.read_register(DATA_REGISTER_INDEX), 3);
  }
  #[test]
  fn test_write_scroll_register() {
    init();
    let mut ppu = PPU::new();
    ppu.write_register(SCROLL_REGISTER_INDEX, 0b10101_010);
    assert_eq!(ppu.fine_x, 0b010);
    assert_eq!(ppu.t_address.coarse_x(), 0b10101);
    assert_eq!(ppu.is_latched, true);

    ppu.write_register(SCROLL_REGISTER_INDEX, 0b01010_101);
    assert_eq!(ppu.t_address.fine_y(), 0b101);
    assert_eq!(ppu.t_address.coarse_y(), 0b01010);
    assert_eq!(ppu.is_latched, false);
  }

  #[test]
  fn test_write_address_register() {
    init();
    let mut ppu = PPU::new();
    ppu.write_register(ADDRESS_REGISTER_INDEX, 0b11_101010);
    assert_eq!(ppu.t_address.high_byte(), 0b00_101010);
    assert_ne!(ppu.t_address, ppu.v_address);
    assert_eq!(ppu.is_latched, true);
    ppu.write_register(ADDRESS_REGISTER_INDEX, 0b1010_1010);
    assert_eq!(ppu.t_address.0, 0b0010_1010_1010_1010);
    assert_eq!(ppu.t_address, ppu.v_address);
    assert_eq!(ppu.is_latched, false);
  }

  #[test]
  fn test_write_data_register() {
    init();
    let mut ppu = PPU::new();
    ppu.v_address.0 = 0x2000;
    ppu.write_register(DATA_REGISTER_INDEX, 0xF0);
    assert_eq!(ppu.vram.read_u8(0x2000), 0xF0);
    assert_eq!(ppu.v_address.0, 0x2001);
    // Use vertical increment.
    ppu.write_control_register(0b0000_0100);
    ppu.write_register(DATA_REGISTER_INDEX, 0x0F);
    assert_eq!(ppu.vram.read_u8(0x2001), 0x0F);
    assert_eq!(ppu.v_address.0, 0x2001 + 32);
  }

  fn build_cartridge(vertical: bool) -> Rc<RefCell<Cartridge>> {
    init();
    let mut data = vec![
      0x4e, 0x45, 0x53, 0x1a, 0x02, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    if vertical {
      data[6] = 0x01;
    }
    data.extend_from_slice(&[0u8; 2 * 0x4000]);
    for i in 0..0x2000u16 {
      data.push(i as u8);
    }
    Rc::new(RefCell::new(Cartridge::new(&data)))
  }

  #[test]
  fn test_ppu_vram_writes() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(false);
    ppu.vram.set_cartridge(cartridge);
    ppu.write_address_register(0x23);
    ppu.write_address_register(0x05);
    ppu.write_data_register(0x66);
    assert_eq!(ppu.vram.read_u8(0x2305), 0x66);
  }

  #[test]
  fn test_ppu_vram_reads() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(false);
    ppu.vram.set_cartridge(cartridge);
    ppu.write_control_register(0x00);
    ppu.vram.write_u8(0x2305, 0x66);
    ppu.write_address_register(0x23);
    ppu.write_address_register(0x05);
    // Dummy read.
    ppu.read_data_register();
    assert_eq!(ppu.v_address.address(), 0x2306);
    assert_eq!(ppu.read_data_register(), 0x66);
  }

  #[test]
  fn test_ppu_vram_reads_cross_page() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(true);
    ppu.vram.set_cartridge(cartridge);
    ppu.write_control_register(0x00);
    ppu.vram.write_u8(0x21FF, 0x66);
    ppu.vram.write_u8(0x2200, 0x77);
    ppu.write_address_register(0x21);
    ppu.write_address_register(0xFF);
    assert_eq!(ppu.v_address.address(), 0x21FF);
    // Dummy read.
    ppu.read_data_register();
    assert_eq!(ppu.v_address.address(), 0x2200);
    assert_eq!(ppu.read_data_register(), 0x66);
    assert_eq!(ppu.read_data_register(), 0x77);
  }

  #[test]
  fn test_ppu_vram_reads_step_32() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(true);
    ppu.vram.set_cartridge(cartridge);
    ppu.write_control_register(0x00);
    ppu.control_register.set_vram_increment_size_flag(true);
    ppu.vram.write_u8(0x21FF, 0x66);
    ppu.vram.write_u8(0x21FF + 32, 0x77);
    ppu.vram.write_u8(0x21FF + 64, 0x88);
    ppu.write_address_register(0x21);
    ppu.write_address_register(0xFF);
    // Dummy read.
    ppu.read_data_register();
    assert_eq!(ppu.v_address.address(), 0x221F);
    assert_eq!(ppu.read_data_register(), 0x66);
    assert_eq!(ppu.read_data_register(), 0x77);
    assert_eq!(ppu.read_data_register(), 0x88);
  }

  // Horizontal: https://wiki.nesdev.com/w/index.php/Mirroring
  //   [0x2000 A ] [0x2400 a ]
  //   [0x2800 B ] [0x2C00 b ]
  #[test]
  fn test_vram_horizontal_mirror() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(false);
    ppu.vram.set_cartridge(cartridge);
    ppu.write_control_register(0x00);
    ppu.write_address_register(0x24);
    ppu.write_address_register(0x05);
    ppu.write_data_register(0x66);
    ppu.write_address_register(0x28);
    ppu.write_address_register(0x05);
    ppu.write_data_register(0x77);
    ppu.write_address_register(0x20);
    ppu.write_address_register(0x05);
    // Dummy read.
    ppu.read_data_register();
    assert_eq!(ppu.read_data_register(), 0x66);
    ppu.write_address_register(0x2C);
    ppu.write_address_register(0x05);
    // Dummy read.
    ppu.read_data_register();
    assert_eq!(ppu.read_data_register(), 0x77);
  }
  // Vertical: https://wiki.nesdev.com/w/index.php/Mirroring
  //   [0x2000 A ] [0x2400 B ]
  //   [0x2800 a ] [0x2C00 b ]
  #[test]
  fn test_vram_vertical_mirror() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(true);
    ppu.vram.set_cartridge(cartridge);
    ppu.write_control_register(0x00);
    ppu.write_address_register(0x20);
    ppu.write_address_register(0x05);
    ppu.write_data_register(0x66);
    ppu.write_address_register(0x2C);
    ppu.write_address_register(0x05);
    ppu.write_data_register(0x77);
    ppu.write_address_register(0x28);
    ppu.write_address_register(0x05);
    // Dummy read.
    ppu.read_data_register();
    assert_eq!(ppu.read_data_register(), 0x66);
    ppu.write_address_register(0x24);
    ppu.write_address_register(0x05);
    // Dummy read.
    ppu.read_data_register();
    assert_eq!(ppu.read_data_register(), 0x77);
  }

  #[test]
  fn test_read_status_resets_latch() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(true);
    ppu.vram.set_cartridge(cartridge);
    ppu.vram.write_u8(0x2305, 0x66);
    ppu.write_address_register(0x21);
    ppu.write_address_register(0x23);
    ppu.write_address_register(0x05);
    // Dummy read.
    ppu.read_data_register();
    assert_ne!(ppu.read_data_register(), 0x66);
    ppu.read_status_register();
    ppu.write_address_register(0x23);
    ppu.write_address_register(0x05);
    // Dummy read.
    ppu.read_data_register();
    assert_eq!(ppu.read_data_register(), 0x66);
  }

  #[test]
  fn test_ppu_vram_mirroring() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(true);
    ppu.vram.set_cartridge(cartridge);
    ppu.write_control_register(0x00);
    ppu.vram.write_u8(0x2305, 0x66);
    ppu.write_address_register(0x63);
    ppu.write_address_register(0x05);
    // Dummy read.
    ppu.read_data_register();
    assert_eq!(ppu.read_data_register(), 0x66);
  }

  #[test]
  fn test_read_status_resets_vblank() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(true);
    ppu.vram.set_cartridge(cartridge);
    ppu.status_register.set_vertical_blank_flag(true);
    let status = ppu.read_status_register();
    assert_eq!(status >> 7, 1);
    assert_eq!(ppu.read_status_register() >> 7, 0);
  }

  #[test]
  fn test_oam_read_write() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(true);
    ppu.vram.set_cartridge(cartridge);
    ppu.write_oam_address_register(0x10);
    ppu.write_oam_data_register(0x66);
    ppu.write_oam_data_register(0x77);
    ppu.write_oam_address_register(0x10);
    assert_eq!(ppu.read_oam_data_register(), 0x66);
    ppu.write_oam_address_register(0x11);
    assert_eq!(ppu.read_oam_data_register(), 0x77);
  }

  #[test]
  fn test_oam_dma() {
    init();
    let mut ppu = PPU::new();
    let cartridge = build_cartridge(true);
    ppu.vram.set_cartridge(cartridge);
    let mut data = [0x66; 256];
    data[0] = 0x77;
    data[255] = 0x88;
    ppu.write_oam_address_register(0x10);
    ppu.write_oam_dma(&data);
    ppu.write_oam_address_register(0x0F);
    assert_eq!(ppu.read_oam_data_register(), 0x88);
    ppu.write_oam_address_register(0x10);
    assert_eq!(ppu.read_oam_data_register(), 0x77);
    ppu.write_oam_address_register(0x11);
    assert_eq!(ppu.read_oam_data_register(), 0x66);
  }
}
