use std::cmp::max;
use std::ops::Range;

use crate::traits::mappable::MirroringMode;

const PRG_ROM_PAGE_SIZE: usize = 0x4000;
const PRG_RAM_PAGE_SIZE: usize = 0x2000;
const CHR_ROM_PAGE_SIZE: usize = 0x2000;
const CHR_RAM_PAGE_SIZE: usize = 0x2000;
const MAGIC_STRING_LENGTH: usize = 4;
const MAGIC_STRING_BYTES: [u8; MAGIC_STRING_LENGTH] = [0x4e, 0x45, 0x53, 0x1a];
const HEADER_LENGTH: usize = 16;

#[named]
#[inline]
pub fn get_mapper_number(data: &[u8]) -> u8 {
  trace_enter!();
  let result = (data[6] >> 4) | (data[7] & 0xF0);
  trace_u8!(result);
  trace_exit!();
  result
}

#[named]
#[inline]
pub fn get_mirroring_mode(byte: u8) -> MirroringMode {
  use MirroringMode::*;
  trace_enter!();
  trace_u8!(byte);
  let result = match byte & 1 == 0 {
    true => Horizontal,
    false => Vertical,
  };
  trace_var!(result);
  trace_exit!();
  result
}

#[derive(Clone, Copy, Debug)]
pub struct Header {
  pub has_valid_magic_string: bool,
  pub prg_rom_pages: usize,
  pub chr_rom_pages: usize,
  pub prg_ram_pages: usize,
  pub mapper_number: u8,
  pub mirroring_mode: MirroringMode,
}

impl Header {
  pub fn new(data: &[u8]) -> Header {
    Header {
      has_valid_magic_string: data[0..MAGIC_STRING_LENGTH] == MAGIC_STRING_BYTES,
      prg_rom_pages: data[4] as usize,
      chr_rom_pages: data[5] as usize,
      prg_ram_pages: max(data[8] as usize, 1),
      mapper_number: get_mapper_number(data),
      mirroring_mode: get_mirroring_mode(data[6]),
    }
  }

  #[named]
  #[inline]
  pub fn get_prg_rom_range(&self) -> Range<usize> {
    trace_enter!();
    let result = HEADER_LENGTH..HEADER_LENGTH + self.get_prg_rom_size();
    trace_var!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn get_chr_rom_range(&self) -> Range<usize> {
    trace_enter!();
    let start = self.get_prg_rom_range().end;
    let result = start..start + self.get_chr_rom_size();
    trace_var!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn get_prg_rom_size(&self) -> usize {
    trace_enter!();
    let result = self.prg_rom_pages * PRG_ROM_PAGE_SIZE;
    trace_var!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn get_chr_rom_size(&self) -> usize {
    trace_enter!();
    let result = self.chr_rom_pages * CHR_ROM_PAGE_SIZE;
    trace_var!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn get_prg_ram_size(&self) -> usize {
    trace_enter!();
    let result = self.prg_ram_pages * PRG_RAM_PAGE_SIZE;
    trace_var!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn get_chr_ram_size(&self) -> usize {
    trace_enter!();
    let result;
    if self.chr_rom_pages > 0 {
      result = 0;
    } else {
      result = CHR_RAM_PAGE_SIZE;
    }
    trace_var!(result);
    trace_exit!();
    result
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  const TEST_HEADER: [u8; HEADER_LENGTH] = [
    0x4e, 0x45, 0x53, 0x1a, 0x10, 0x12, 0x11, 0x00, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  ];

  #[test]
  fn test_header() {
    init();
    // Test case comes from Starr Horne's rust-nes.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cartridge/cartridge_header.rs
    let header = Header::new(&TEST_HEADER);
    assert!(header.has_valid_magic_string);
    assert_eq!(MirroringMode::Vertical, header.mirroring_mode);
    assert_eq!(0x10, header.prg_rom_pages);
    assert_eq!(0x10 * PRG_ROM_PAGE_SIZE, header.get_prg_rom_size());
    assert_eq!(16..16 + 0x10 * PRG_ROM_PAGE_SIZE, header.get_prg_rom_range());
    assert_eq!(0x12, header.chr_rom_pages);
    assert_eq!(0x12 * CHR_ROM_PAGE_SIZE, header.get_chr_rom_size());
    assert_eq!(
      16 + 0x10 * PRG_ROM_PAGE_SIZE..16 + (0x10 * PRG_ROM_PAGE_SIZE) + (0x12 * CHR_ROM_PAGE_SIZE),
      header.get_chr_rom_range()
    );
    assert_eq!(0x13, header.prg_ram_pages);
    assert_eq!(0x13 * PRG_RAM_PAGE_SIZE, header.get_prg_ram_size());
    assert_eq!(0x01, header.mapper_number);
  }
}
