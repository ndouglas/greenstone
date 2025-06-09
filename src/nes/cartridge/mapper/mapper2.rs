use super::super::{Data, MirroringMode, Page, PageSize};
use crate::traits::Mappable;

use Page::*;
use PageSize::*;

/// Mapper 2 (UxROM)
///
/// PRG ROM: 16KB switchable bank at $8000-$BFFF, fixed last bank at $C000-$FFFF
/// CHR: 8KB RAM (no ROM)
/// Games: Mega Man, Castlevania, Contra, DuckTales, etc.
pub struct Mapper2 {
  data: Data,
  prg_bank: u8,
}

impl Mapper2 {
  pub fn new(data: Data) -> Mapper2 {
    Mapper2 { data, prg_bank: 0 }
  }
}

impl Mappable for Mapper2 {
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
        // PRG RAM - UxROM doesn't have PRG RAM, return open bus
        0x00
      }
      0x8000..=0xBFFF => {
        // Switchable 16KB bank
        self
          .data
          .prg_rom
          .read_u8(Number(self.prg_bank as usize, SixteenKb), address - 0x8000)
      }
      0xC000..=0xFFFF => {
        // Fixed to last 16KB bank
        self.data.prg_rom.read_u8(Last(SixteenKb), address - 0xC000)
      }
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
        // No PRG RAM on UxROM - ignore
      }
      0x8000..=0xFFFF => {
        // Bank select - low bits select the 16KB PRG bank at $8000-$BFFF
        self.prg_bank = value & 0x0F;
      }
      _ => panic!("bad address: {}", format_u16!(address)),
    }
    trace_exit!();
  }

  #[named]
  fn read_chr_u8(&self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    // UxROM uses CHR RAM
    let result = self.data.chr_ram.read_u8(First(EightKb), address);
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  fn write_chr_u8(&mut self, address: u16, value: u8) {
    trace_enter!();
    trace_u16!(address);
    trace_u8!(value);
    // UxROM uses CHR RAM
    self.data.chr_ram.write_u8(First(EightKb), address, value);
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

  fn build_mapper2(prg_banks: u8) -> Mapper2 {
    // Build a UxROM cartridge with specified number of 16KB PRG banks
    let mut data = vec![
      0x4e, 0x45, 0x53, 0x1a, // NES magic
      prg_banks,              // PRG ROM pages (16KB each)
      0x00,                   // CHR ROM pages (0 = uses CHR RAM)
      0x20,                   // Flags 6: mapper low nibble = 2
      0x00,                   // Flags 7: mapper high nibble = 0
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    // Fill PRG ROM with identifiable data
    // Each 16KB bank starts with its bank number repeated
    for bank in 0..prg_banks {
      for i in 0..0x4000usize {
        if i < 4 {
          data.push(bank);
        } else {
          data.push((i & 0xFF) as u8);
        }
      }
    }

    let cart_data = Data::new(&data);
    Mapper2::new(cart_data)
  }

  #[test]
  fn test_initial_bank_is_zero() {
    init();
    let mapper = build_mapper2(8);
    // First bank should be selected by default
    assert_eq!(mapper.read_prg_u8(0x8000), 0);
  }

  #[test]
  fn test_last_bank_fixed() {
    init();
    let mapper = build_mapper2(8);
    // $C000-$FFFF should always read from last bank (bank 7)
    assert_eq!(mapper.read_prg_u8(0xC000), 7);
  }

  #[test]
  fn test_bank_switching() {
    init();
    let mut mapper = build_mapper2(8);

    // Switch to bank 3
    mapper.write_prg_u8(0x8000, 3);
    assert_eq!(mapper.read_prg_u8(0x8000), 3);

    // Switch to bank 5
    mapper.write_prg_u8(0x8000, 5);
    assert_eq!(mapper.read_prg_u8(0x8000), 5);

    // Last bank should still be fixed
    assert_eq!(mapper.read_prg_u8(0xC000), 7);
  }

  #[test]
  fn test_bank_mask() {
    init();
    let mut mapper = build_mapper2(8);

    // Writing 0xFF should only use low 4 bits (0x0F)
    mapper.write_prg_u8(0x8000, 0xFF);
    assert_eq!(mapper.prg_bank, 0x0F);
  }

  #[test]
  fn test_chr_ram_read_write() {
    init();
    let mut mapper = build_mapper2(2);

    // Write to CHR RAM
    for i in 0..0x2000u16 {
      mapper.write_chr_u8(i, (i & 0xFF) as u8);
    }

    // Read back
    for i in 0..0x2000u16 {
      assert_eq!(mapper.read_chr_u8(i), (i & 0xFF) as u8);
    }
  }
}
