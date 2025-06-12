/// Mapper 3 (CNROM)
///
/// PRG ROM: 16KB or 32KB, not switchable
/// CHR ROM: 8KB switchable banks
/// Games: Arkanoid, Solomon's Key, Gradius, Paperboy, etc.
use super::super::{Data, MirroringMode, Page, PageSize};
use crate::traits::Mappable;

use Page::*;
use PageSize::*;

pub struct Mapper3 {
  data: Data,
  chr_bank: u8,
}

impl Mapper3 {
  pub fn new(data: Data) -> Mapper3 {
    Mapper3 { data, chr_bank: 0 }
  }
}

impl Mappable for Mapper3 {
  #[named]
  fn read_prg_u8(&self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let result = match address {
      0x4020..=0x5FFF => {
        // Expansion ROM area - open bus
        0x00
      }
      0x6000..=0x7FFF => {
        // PRG RAM - CNROM doesn't typically have PRG RAM, return open bus
        0x00
      }
      0x8000..=0xBFFF => self.data.prg_rom.read_u8(First(SixteenKb), address - 0x8000),
      0xC000..=0xFFFF => self.data.prg_rom.read_u8(Last(SixteenKb), address - 0xC000),
      _ => panic!("bad address: {}", format_u16!(address)),
    };
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  fn write_prg_u8(&mut self, address: u16, value: u8) {
    trace_enter!();
    trace_u16!(address);
    trace_u8!(value);
    match address {
      0x4020..=0x5FFF => {
        // Expansion ROM area - ignore
      }
      0x6000..=0x7FFF => {
        // No PRG RAM on CNROM - ignore
      }
      0x8000..=0xFFFF => {
        // Bank select - low bits select the 8KB CHR bank
        // CNROM typically uses only 2 bits, but some variants use more.
        // Mask to the number of available banks.
        let bank_count = self.data.chr_rom.get_page_count(EightKb);
        self.chr_bank = value & (bank_count as u8 - 1);
      }
      _ => panic!("bad address: {}", format_u16!(address)),
    }
    trace_exit!();
  }

  #[named]
  fn read_chr_u8(&self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let result = self.data.chr_rom.read_u8(Number(self.chr_bank as usize, EightKb), address);
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  fn write_chr_u8(&mut self, address: u16, _value: u8) {
    trace_enter!();
    trace_u16!(address);
    // CHR ROM is not writable on CNROM
    trace_exit!();
  }

  #[named]
  fn get_mirroring_mode(&self) -> MirroringMode {
    trace_enter!();
    let result = self.data.header.mirroring_mode;
    trace_var!(result);
    trace_exit!();
    result
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::nes::cartridge::Data;
  use crate::test::init;

  fn build_mapper3(prg_banks: u8, chr_banks: u8) -> Mapper3 {
    let mut data = vec![
      0x4e, 0x45, 0x53, 0x1a,      // NES magic
      prg_banks, // PRG ROM pages (16KB each)
      chr_banks, // CHR ROM pages (8KB each)
      0x30,      // Flags 6: mapper low nibble = 3
      0x00,      // Flags 7: mapper high nibble = 0
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    // Fill PRG ROM
    for bank in 0..prg_banks {
      for i in 0..0x4000usize {
        if i < 4 {
          data.push(bank);
        } else {
          data.push((i & 0xFF) as u8);
        }
      }
    }

    // Fill CHR ROM - each 8KB bank starts with its bank number repeated
    for bank in 0..chr_banks {
      for i in 0..0x2000usize {
        if i < 4 {
          data.push(bank);
        } else {
          data.push((i & 0xFF) as u8);
        }
      }
    }

    let cart_data = Data::new(&data);
    Mapper3::new(cart_data)
  }

  #[test]
  fn test_initial_chr_bank_is_zero() {
    init();
    let mapper = build_mapper3(2, 4);
    // First CHR bank should be selected by default
    assert_eq!(mapper.read_chr_u8(0x0000), 0);
  }

  #[test]
  fn test_chr_bank_switching() {
    init();
    let mut mapper = build_mapper3(2, 4);

    // Switch to CHR bank 2
    mapper.write_prg_u8(0x8000, 2);
    assert_eq!(mapper.read_chr_u8(0x0000), 2);

    // Switch to CHR bank 3
    mapper.write_prg_u8(0x8000, 3);
    assert_eq!(mapper.read_chr_u8(0x0000), 3);

    // Switch back to bank 0
    mapper.write_prg_u8(0x8000, 0);
    assert_eq!(mapper.read_chr_u8(0x0000), 0);
  }

  #[test]
  fn test_chr_bank_mask() {
    init();
    let mut mapper = build_mapper3(2, 4);

    // Writing a value larger than bank count should be masked
    // 4 banks = mask 0x03, so 0xFF & 0x03 = 3
    mapper.write_prg_u8(0x8000, 0xFF);
    assert_eq!(mapper.chr_bank, 3);
  }

  #[test]
  fn test_prg_rom_not_switchable() {
    init();
    let mapper = build_mapper3(2, 4);

    // $8000-$BFFF should read from first 16KB bank
    assert_eq!(mapper.read_prg_u8(0x8000), 0);

    // $C000-$FFFF should read from last 16KB bank
    assert_eq!(mapper.read_prg_u8(0xC000), 1);
  }

  #[test]
  fn test_chr_rom_not_writable() {
    init();
    let mut mapper = build_mapper3(2, 4);

    let original = mapper.read_chr_u8(0x0000);
    mapper.write_chr_u8(0x0000, 0xFF);
    // Value should not have changed - CHR ROM is read-only
    assert_eq!(mapper.read_chr_u8(0x0000), original);
  }
}
