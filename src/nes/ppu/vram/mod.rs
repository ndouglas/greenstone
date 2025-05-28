use super::super::*;
use crate::traits::Mappable;
use crate::MirroringMode;

use std::cell::RefCell;
use std::rc::Rc;

pub const NAMETABLE_SIZE: u16 = 0x400;
pub const PALETTE_SIZE: u16 = 0x20;
pub const CARTRIDGE_CHR_START_ADDRESS: u16 = 0x0000;
pub const CARTRIDGE_CHR_END_ADDRESS: u16 = 0x1FFF;
pub const VRAM_START_ADDRESS: u16 = 0x2000;
pub const VRAM_END_ADDRESS: u16 = 0x3FFF;
pub const NAMETABLE_START_ADDRESS: u16 = VRAM_START_ADDRESS;
pub const NAMETABLE_END_ADDRESS: u16 = 0x3EFF;
pub const PALETTE_START_ADDRESS: u16 = 0x3F00;
pub const PALETTE_END_ADDRESS: u16 = VRAM_END_ADDRESS;
pub const VRAM_LENGTH: usize = 2048;

pub struct VRAM {
  pub nametables: Vec<u8>,
  pub palettes: Vec<u8>,
  pub cartridge: Option<Rc<RefCell<Cartridge>>>,
  pub read_buffer: u8,
}

impl VRAM {
  pub fn new() -> VRAM {
    VRAM {
      nametables: vec![0; 2 * NAMETABLE_SIZE as usize],
      palettes: vec![0; PALETTE_SIZE as usize],
      cartridge: None,
      read_buffer: 0,
    }
  }

  #[named]
  pub fn set_cartridge(&mut self, cartridge: Rc<RefCell<Cartridge>>) {
    trace_enter!();
    self.cartridge = Some(cartridge);
    trace_exit!();
  }

  #[named]
  pub fn write_u8(&mut self, address: u16, value: u8) {
    trace_enter!();
    trace_u16!(address);
    trace_u8!(value);
    match address {
      CARTRIDGE_CHR_START_ADDRESS..=CARTRIDGE_CHR_END_ADDRESS => match self.cartridge {
        Some(ref cartridge) => cartridge.borrow_mut().write_chr_u8(address, value),
        None => panic!("Attempted to write data to CHR RAM of a cartridge, but no cartridge was loaded."),
      },
      NAMETABLE_START_ADDRESS..=NAMETABLE_END_ADDRESS => {
        let mirroring_mode = self.get_mirroring_mode();
        let mirrored_address = self.get_mirrored_nametable_address(mirroring_mode, address) as usize;
        self.nametables[mirrored_address] = value;
      }
      PALETTE_START_ADDRESS..=PALETTE_END_ADDRESS => {
        let mirrored_address = self.get_mirrored_palette_address(address) as usize;
        self.palettes[mirrored_address] = value;
      }
      _ => (),
    }
    trace_exit!();
  }

  #[named]
  pub fn read_u8(&mut self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let result = match address {
      CARTRIDGE_CHR_START_ADDRESS..=CARTRIDGE_CHR_END_ADDRESS => match self.cartridge {
        Some(ref cartridge) => cartridge.borrow().read_chr_u8(address),
        None => panic!("Attempted to read data from CHR RAM of a cartridge, but no cartridge was loaded."),
      },
      NAMETABLE_START_ADDRESS..=NAMETABLE_END_ADDRESS => {
        let mirroring_mode = self.get_mirroring_mode();
        let mirrored_address = self.get_mirrored_nametable_address(mirroring_mode, address) as usize;
        self.nametables[mirrored_address]
      }
      PALETTE_START_ADDRESS..=PALETTE_END_ADDRESS => {
        let mirrored_address = self.get_mirrored_palette_address(address) as usize;
        self.palettes[mirrored_address]
      }
      _ => 0x00,
    };
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  pub fn buffered_read_u8(&mut self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let result = match address {
      PALETTE_START_ADDRESS..=PALETTE_END_ADDRESS => {
        let read_value = self.read_u8(address);
        self.read_buffer = read_value;
        read_value
      }
      _ => {
        let buffered_value = self.read_buffer;
        self.read_buffer = self.read_u8(address);
        buffered_value
      }
    };
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn get_mirroring_mode(&self) -> MirroringMode {
    trace_enter!();
    let result;
    if let Some(ref cartridge) = self.cartridge {
      result = cartridge.borrow().get_mirroring_mode();
    } else {
      result = MirroringMode::None;
    }
    trace_exit!();
    result
  }

  // Adapted from code in Starr Horne's `rust-nes`.
  #[named]
  pub fn get_mirrored_nametable_address(&self, mirroring_mode: MirroringMode, address: u16) -> u16 {
    use MirroringMode::*;
    trace_enter!();
    trace_var!(mirroring_mode);
    trace_u16!(address);
    let result = match mirroring_mode {
      None => address - 0x2000,
      Horizontal => ((address / 2) & NAMETABLE_SIZE) + (address % NAMETABLE_SIZE),
      Vertical => address % (2 * NAMETABLE_SIZE),
      SingleScreenLower => address % NAMETABLE_SIZE,
      SingleScreenUpper => (address % NAMETABLE_SIZE) + NAMETABLE_SIZE,
    };
    trace_u16!(result);
    trace_exit!();
    result
  }

  // Adapted from code in Starr Horne's `rust-nes`.
  #[named]
  pub fn get_mirrored_palette_address(&self, address: u16) -> u16 {
    trace_enter!();
    let address = address % PALETTE_SIZE;
    let result = match address {
      0x10 | 0x14 | 0x18 | 0x1C => address - 0x10,
      _ => address,
    };
    trace_u16!(result);
    trace_exit!();
    result
  }

  #[named]
  pub fn reset(&mut self) {
    trace_enter!();
    self.nametables = vec![0; 2 * NAMETABLE_SIZE as usize];
    self.palettes = vec![0; PALETTE_SIZE as usize];
    self.read_buffer = 0;
    // Note: Don't clear cartridge reference - it's still needed for CHR ROM access
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  // These test cases come from Starr Horne's `rust-nes`.
  // See https://github.com/starrhorne/nes-rust/blob/master/src/ppu/vram.rs

  #[test]
  fn test_read_u8_nametable() {
    init();
    let mut vram = VRAM::new();
    vram.nametables[0x201] = 0x11;
    assert_eq!(vram.read_u8(0x2201), 0x11);
    assert_eq!(vram.read_u8(0x2200), 0x00);
  }

  #[test]
  fn test_write_u8_nametable() {
    init();
    let mut vram = VRAM::new();
    vram.write_u8(0x2201, 0x11);
    assert_eq!(vram.nametables[0x201], 0x11);
    assert_eq!(vram.nametables[0x200], 0x00);
  }

  #[test]
  fn test_read_u8_palette() {
    init();
    let mut vram = VRAM::new();
    vram.palettes[0x09] = 0x22;
    vram.palettes[0x00] = 0x33;
    assert_eq!(vram.read_u8(0x3F09), 0x22);
    assert_eq!(vram.read_u8(0x3F00), 0x33);
    assert_eq!(vram.read_u8(0x3F11), 0x00);
  }

  #[test]
  fn test_write_u8_palette() {
    init();
    let mut vram = VRAM::new();
    vram.write_u8(0x3F09, 0x11);
    assert_eq!(vram.palettes[0x09], 0x11);
  }

  fn build_cartridge() -> Rc<RefCell<Cartridge>> {
    init();
    let mut data = vec![
      0x4e, 0x45, 0x53, 0x1a, 0x02, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    data.extend_from_slice(&[0u8; 2 * 0x4000]);
    for i in 0..0x2000u16 {
      data.push(i as u8);
    }
    Rc::new(RefCell::new(Cartridge::new(&data)))
  }

  #[test]
  fn test_read_u8_cartridge() {
    init();
    let mut vram = VRAM::new();
    vram.set_cartridge(build_cartridge());
    assert_eq!(vram.read_u8(0), 0);
    assert_eq!(vram.read_u8(10), 10);
    assert_eq!(vram.read_u8(20), 20);
  }

  #[test]
  fn test_buffered_read_u8() {
    init();
    let mut vram = VRAM::new();
    vram.nametables[0x201] = 0x11;
    vram.nametables[0x202] = 0x12;
    assert_eq!(vram.buffered_read_u8(0x2201), 0);
    assert_eq!(vram.buffered_read_u8(0x2202), 0x11);
    assert_eq!(vram.buffered_read_u8(0x2203), 0x12);
    assert_eq!(vram.buffered_read_u8(0x2204), 0);
  }

  #[test]
  fn test_mirror_nametable_horizontally() {
    use MirroringMode::*;
    init();
    let vram = VRAM::new();
    // Nametable 1 - starting at 0x2000
    assert_eq!(vram.get_mirrored_nametable_address(Horizontal, 0x2001), 1);
    assert_eq!(vram.get_mirrored_nametable_address(Horizontal, 0x2201), 0x201);
    assert_eq!(vram.get_mirrored_nametable_address(Horizontal, 0x2401), 1);
    assert_eq!(vram.get_mirrored_nametable_address(Horizontal, 0x2601), 0x201);

    // Nametable 1 - mirrored at 0x3000
    assert_eq!(vram.get_mirrored_nametable_address(Horizontal, 0x3001), 1);
    assert_eq!(vram.get_mirrored_nametable_address(Horizontal, 0x3201), 0x201);
    assert_eq!(vram.get_mirrored_nametable_address(Horizontal, 0x3401), 1);
    assert_eq!(vram.get_mirrored_nametable_address(Horizontal, 0x3601), 0x201);

    // Nametable 2 - starting at 0x2800
    assert_eq!(vram.get_mirrored_nametable_address(Horizontal, 0x2801), 0x401);
    assert_eq!(vram.get_mirrored_nametable_address(Horizontal, 0x2A01), 0x601);
    assert_eq!(vram.get_mirrored_nametable_address(Horizontal, 0x2C01), 0x401);
    assert_eq!(vram.get_mirrored_nametable_address(Horizontal, 0x2E01), 0x601);

    // Nametable 2 - mirrored at 0x3800
    assert_eq!(vram.get_mirrored_nametable_address(Horizontal, 0x3801), 0x401);
    assert_eq!(vram.get_mirrored_nametable_address(Horizontal, 0x3A01), 0x601);
    assert_eq!(vram.get_mirrored_nametable_address(Horizontal, 0x3C01), 0x401);
    assert_eq!(vram.get_mirrored_nametable_address(Horizontal, 0x3E01), 0x601);
  }

  #[test]
  fn test_mirror_nametable_vertically() {
    use MirroringMode::*;
    init();
    let vram = VRAM::new();
    // Nametable 1 - starting at 0x2000
    assert_eq!(vram.get_mirrored_nametable_address(Vertical, 0x2001), 1);
    assert_eq!(vram.get_mirrored_nametable_address(Vertical, 0x2201), 0x201);
    assert_eq!(vram.get_mirrored_nametable_address(Vertical, 0x2801), 1);
    assert_eq!(vram.get_mirrored_nametable_address(Vertical, 0x2A01), 0x201);

    // Nametable 1 - mirrored at 0x3000
    assert_eq!(vram.get_mirrored_nametable_address(Vertical, 0x3001), 1);
    assert_eq!(vram.get_mirrored_nametable_address(Vertical, 0x3201), 0x201);
    assert_eq!(vram.get_mirrored_nametable_address(Vertical, 0x3801), 1);
    assert_eq!(vram.get_mirrored_nametable_address(Vertical, 0x3A01), 0x201);

    // Nametable 2 - starting at 0x2400
    assert_eq!(vram.get_mirrored_nametable_address(Vertical, 0x2401), 0x401);
    assert_eq!(vram.get_mirrored_nametable_address(Vertical, 0x2601), 0x601);
    assert_eq!(vram.get_mirrored_nametable_address(Vertical, 0x2C01), 0x401);
    assert_eq!(vram.get_mirrored_nametable_address(Vertical, 0x2E01), 0x601);

    // Nametable 2 - mirrored at 0x3800
    assert_eq!(vram.get_mirrored_nametable_address(Vertical, 0x3401), 0x401);
    assert_eq!(vram.get_mirrored_nametable_address(Vertical, 0x3601), 0x601);
    assert_eq!(vram.get_mirrored_nametable_address(Vertical, 0x3C01), 0x401);
    assert_eq!(vram.get_mirrored_nametable_address(Vertical, 0x3E01), 0x601);
  }

  #[test]
  fn test_mirror_palette() {
    init();
    let vram = VRAM::new();
    assert_eq!(vram.get_mirrored_palette_address(0x3F01), 1);
    assert_eq!(vram.get_mirrored_palette_address(0x3F21), 1);
    assert_eq!(vram.get_mirrored_palette_address(0x3F41), 1);
    assert_eq!(vram.get_mirrored_palette_address(0x3F11), 0x11);
    // Test mirroring of 0x10
    assert_eq!(vram.get_mirrored_palette_address(0x3F10), 0);
    assert_eq!(vram.get_mirrored_palette_address(0x3F30), 0);
    // Test mirroring of 0x14
    assert_eq!(vram.get_mirrored_palette_address(0x3F14), 4);
    assert_eq!(vram.get_mirrored_palette_address(0x3F34), 4);
    // Test mirroring of 0x18
    assert_eq!(vram.get_mirrored_palette_address(0x3F18), 8);
    assert_eq!(vram.get_mirrored_palette_address(0x3F38), 8);
    // Test mirroring of 0x1c
    assert_eq!(vram.get_mirrored_palette_address(0x3F1C), 0x0C);
    assert_eq!(vram.get_mirrored_palette_address(0x3F3C), 0x0C);
  }
}
