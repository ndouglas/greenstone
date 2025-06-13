/// Mapper 7 (AxROM)
///
/// PRG ROM: 32KB switchable bank at $8000-$FFFF
/// CHR: 8KB RAM (no ROM)
/// Mirroring: Single-screen, switchable between lower and upper nametable
/// Games: Battletoads, Marble Madness, Wizards & Warriors, etc.
use super::super::{Data, MirroringMode, Page, PageSize};
use crate::traits::Mappable;

use Page::*;
use PageSize::*;

#[derive(Serialize, Deserialize)]
pub struct Mapper7 {
  data: Data,
  /// The selected 32KB PRG bank (bits 0-2 of written value).
  prg_bank: u8,
  /// Single-screen mirroring select (bit 4 of written value).
  /// false = lower nametable, true = upper nametable.
  vram_page: bool,
}

impl Mapper7 {
  pub fn new(data: Data) -> Mapper7 {
    Mapper7 {
      data,
      prg_bank: 0,
      vram_page: false,
    }
  }
}

impl Mappable for Mapper7 {
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
        // PRG RAM - AxROM doesn't typically have PRG RAM, return open bus
        0x00
      }
      0x8000..=0xFFFF => {
        // 32KB switchable bank
        // Each 32KB bank is composed of two consecutive 16KB banks.
        let bank_32k = self.prg_bank as usize;
        let bank_16k = bank_32k * 2;
        if address < 0xC000 {
          self.data.prg_rom.read_u8(Number(bank_16k, SixteenKb), address - 0x8000)
        } else {
          self.data.prg_rom.read_u8(Number(bank_16k + 1, SixteenKb), address - 0xC000)
        }
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
        // No PRG RAM on AxROM - ignore
      }
      0x8000..=0xFFFF => {
        // Bits 0-2: select 32KB PRG bank
        let bank_count = self.data.prg_rom.get_page_count(SixteenKb) / 2;
        self.prg_bank = (value & 0x07) % bank_count as u8;
        // Bit 4: select single-screen VRAM page
        self.vram_page = (value & 0x10) != 0;
      }
      _ => panic!("bad address: {}", format_u16!(address)),
    }
    trace_exit!();
  }

  #[named]
  fn read_chr_u8(&self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    // AxROM uses CHR RAM
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
    // AxROM uses CHR RAM
    self.data.chr_ram.write_u8(First(EightKb), address, value);
    trace_exit!();
  }

  #[named]
  fn get_mirroring_mode(&self) -> MirroringMode {
    trace_enter!();
    let result = if self.vram_page {
      MirroringMode::SingleScreenUpper
    } else {
      MirroringMode::SingleScreenLower
    };
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

  fn build_mapper7(prg_banks_32k: u8) -> Mapper7 {
    let prg_banks_16k = prg_banks_32k * 2;
    let mut data = vec![
      0x4e,
      0x45,
      0x53,
      0x1a,          // NES magic
      prg_banks_16k, // PRG ROM pages (16KB each)
      0x00,          // CHR ROM pages (0 = uses CHR RAM)
      0x70,          // Flags 6: mapper low nibble = 7
      0x00,          // Flags 7: mapper high nibble = 0
      0x00,
      0x00,
      0x00,
      0x00,
      0x00,
      0x00,
      0x00,
      0x00,
    ];

    // Fill PRG ROM - each 16KB bank starts with its bank number
    for bank in 0..prg_banks_16k {
      for i in 0..0x4000usize {
        if i < 4 {
          data.push(bank);
        } else {
          data.push((i & 0xFF) as u8);
        }
      }
    }

    let cart_data = Data::new(&data);
    Mapper7::new(cart_data)
  }

  #[test]
  fn test_initial_bank_is_zero() {
    init();
    let mapper = build_mapper7(4);
    // 32KB bank 0 = 16KB banks 0 and 1
    assert_eq!(mapper.read_prg_u8(0x8000), 0); // first 16KB bank
    assert_eq!(mapper.read_prg_u8(0xC000), 1); // second 16KB bank
  }

  #[test]
  fn test_bank_switching() {
    init();
    let mut mapper = build_mapper7(4);

    // Switch to 32KB bank 2 (= 16KB banks 4 and 5)
    mapper.write_prg_u8(0x8000, 2);
    assert_eq!(mapper.read_prg_u8(0x8000), 4);
    assert_eq!(mapper.read_prg_u8(0xC000), 5);

    // Switch to 32KB bank 3 (= 16KB banks 6 and 7)
    mapper.write_prg_u8(0x8000, 3);
    assert_eq!(mapper.read_prg_u8(0x8000), 6);
    assert_eq!(mapper.read_prg_u8(0xC000), 7);
  }

  #[test]
  fn test_mirroring_control() {
    init();
    let mut mapper = build_mapper7(4);

    // Default should be single-screen lower
    assert_eq!(mapper.get_mirroring_mode(), MirroringMode::SingleScreenLower);

    // Set bit 4 to select upper nametable
    mapper.write_prg_u8(0x8000, 0x10);
    assert_eq!(mapper.get_mirroring_mode(), MirroringMode::SingleScreenUpper);

    // Clear bit 4 to go back to lower
    mapper.write_prg_u8(0x8000, 0x00);
    assert_eq!(mapper.get_mirroring_mode(), MirroringMode::SingleScreenLower);
  }

  #[test]
  fn test_chr_ram_read_write() {
    init();
    let mut mapper = build_mapper7(2);

    // Write to CHR RAM
    for i in 0..0x2000u16 {
      mapper.write_chr_u8(i, (i & 0xFF) as u8);
    }

    // Read back
    for i in 0..0x2000u16 {
      assert_eq!(mapper.read_chr_u8(i), (i & 0xFF) as u8);
    }
  }

  #[test]
  fn test_bank_wraps() {
    init();
    let mut mapper = build_mapper7(4);

    // Writing bank 7 with only 4 32KB banks should wrap: 7 % 4 = 3
    mapper.write_prg_u8(0x8000, 7);
    assert_eq!(mapper.prg_bank, 3);
  }
}
