/// Mapper 9 (MMC2)
///
/// PRG ROM: one switchable 8KB bank at $8000, the last three 8KB banks fixed at $A000-$FFFF.
/// CHR ROM: two 4KB halves, each switched between two banks by a latch that the
/// PPU sets by fetching tile $FD or $FE.
/// Games: Mike Tyson's Punch-Out!!, Punch-Out!!
use std::cell::Cell;

use super::super::{Data, MirroringMode, Page, PageSize};
use crate::traits::Mappable;

use Page::*;
use PageSize::*;

/// Latch values: which of a half's two banks is showing.
const FD: u8 = 0;
const FE: u8 = 1;

#[derive(Serialize, Deserialize)]
pub struct Mapper9 {
  data: Data,
  prg_bank: u8,
  /// `chr_banks[half][latch]`: the 4KB bank for each half under each latch value.
  chr_banks: [[u8; 2]; 2],
  /// Set by PPU reads, so they change behind `&self`.
  latches: [Cell<u8>; 2],
  horizontal: bool,
}

impl Mapper9 {
  pub fn new(data: Data) -> Mapper9 {
    Mapper9 {
      data,
      prg_bank: 0,
      chr_banks: [[0; 2]; 2],
      latches: [Cell::new(FE), Cell::new(FE)],
      horizontal: false,
    }
  }
}

impl Mappable for Mapper9 {
  #[named]
  fn read_prg_u8(&self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let count = self.data.prg_rom.get_page_count(EightKb);
    let result = match address {
      0x4020..=0x7FFF => 0x00,
      0x8000..=0x9FFF => self
        .data
        .prg_rom
        .read_u8(Number(self.prg_bank as usize % count, EightKb), address - 0x8000),
      0xA000..=0xFFFF => {
        let fixed = ((address - 0xA000) / 0x2000) as usize;
        self.data.prg_rom.read_u8(Number(count - 3 + fixed, EightKb), address & 0x1FFF)
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
      0x4020..=0x9FFF => {}
      0xA000..=0xAFFF => self.prg_bank = value & 0x0F,
      0xB000..=0xBFFF => self.chr_banks[0][FD as usize] = value & 0x1F,
      0xC000..=0xCFFF => self.chr_banks[0][FE as usize] = value & 0x1F,
      0xD000..=0xDFFF => self.chr_banks[1][FD as usize] = value & 0x1F,
      0xE000..=0xEFFF => self.chr_banks[1][FE as usize] = value & 0x1F,
      0xF000..=0xFFFF => self.horizontal = value & 1 == 1,
      _ => panic!("bad address: {}", format_u16!(address)),
    }
    trace_exit!();
  }

  #[named]
  fn read_chr_u8(&self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let half = ((address >> 12) & 1) as usize;
    let count = self.data.chr_rom.get_page_count(FourKb);
    let bank = self.chr_banks[half][self.latches[half].get() as usize] as usize % count;
    let result = self.data.chr_rom.read_u8(Number(bank, FourKb), address & 0x0FFF);
    match address {
      0x0FD8 => self.latches[0].set(FD),
      0x0FE8 => self.latches[0].set(FE),
      0x1FD8..=0x1FDF => self.latches[1].set(FD),
      0x1FE8..=0x1FEF => self.latches[1].set(FE),
      _ => {}
    }
    trace_u8!(result);
    trace_exit!();
    result
  }

  fn write_chr_u8(&mut self, _address: u16, _value: u8) {
    // CHR ROM: writes are ignored.
  }

  fn get_mirroring_mode(&self) -> MirroringMode {
    if self.horizontal {
      MirroringMode::Horizontal
    } else {
      MirroringMode::Vertical
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::nes::cartridge::Data;

  /// 8 KB PRG banks start with their bank number; 4 KB CHR banks start with theirs.
  fn build_mapper9(prg_8k: usize, chr_4k: usize) -> Mapper9 {
    let mut data = vec![
      0x4e,
      0x45,
      0x53,
      0x1a,
      (prg_8k / 2) as u8,
      (chr_4k / 2) as u8,
      0x90,
      0x00,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
      0,
    ];
    for bank in 0..prg_8k {
      data.extend((0..0x2000usize).map(|i| if i < 4 { bank as u8 } else { (i & 0xFF) as u8 }));
    }
    for bank in 0..chr_4k {
      data.extend((0..0x1000usize).map(|i| if i < 4 { bank as u8 } else { 0xA0 | (bank as u8 & 0x0F) }));
    }
    Mapper9::new(Data::new(&data))
  }

  #[test]
  fn prg_switches_8000_and_fixes_the_last_three_banks() {
    let mut m = build_mapper9(16, 32);
    assert_eq!(m.read_prg_u8(0x8000), 0);
    assert_eq!(m.read_prg_u8(0xA000), 13);
    assert_eq!(m.read_prg_u8(0xC000), 14);
    assert_eq!(m.read_prg_u8(0xE000), 15);
    m.write_prg_u8(0xA000, 5);
    assert_eq!(m.read_prg_u8(0x8000), 5);
    assert_eq!(m.read_prg_u8(0xA000), 13, "writing $A000 does not move the fixed banks");
  }

  #[test]
  fn chr_halves_follow_their_latches_which_start_at_fe() {
    let mut m = build_mapper9(16, 32);
    m.write_prg_u8(0xB000, 1); // half 0, FD
    m.write_prg_u8(0xC000, 2); // half 0, FE
    m.write_prg_u8(0xD000, 3); // half 1, FD
    m.write_prg_u8(0xE000, 4); // half 1, FE
    assert_eq!(m.read_chr_u8(0x0000), 2, "power-on latch 0 is FE");
    assert_eq!(m.read_chr_u8(0x1000), 4, "power-on latch 1 is FE");
  }

  #[test]
  fn reading_tile_fd_or_fe_flips_the_latch_after_the_read() {
    let mut m = build_mapper9(16, 32);
    m.write_prg_u8(0xB000, 1);
    m.write_prg_u8(0xC000, 2);
    m.write_prg_u8(0xD000, 3);
    m.write_prg_u8(0xE000, 4);
    assert_eq!(m.read_chr_u8(0x0FD8), 0xA2, "the triggering read still uses the FE bank");
    assert_eq!(m.read_chr_u8(0x0000), 1, "now FD");
    m.read_chr_u8(0x0FE8);
    assert_eq!(m.read_chr_u8(0x0000), 2, "back to FE");
    m.read_chr_u8(0x1FDC);
    assert_eq!(m.read_chr_u8(0x1000), 3, "half 1 flips anywhere in $1FD8-$1FDF");
    m.read_chr_u8(0x1FEF);
    assert_eq!(m.read_chr_u8(0x1000), 4);
    m.read_chr_u8(0x0FD9);
    assert_eq!(m.read_chr_u8(0x0000), 2, "half 0 flips only on exactly $0FD8");
  }

  #[test]
  fn f000_sets_mirroring() {
    let mut m = build_mapper9(16, 32);
    m.write_prg_u8(0xF000, 0);
    assert!(matches!(m.get_mirroring_mode(), MirroringMode::Vertical));
    m.write_prg_u8(0xF000, 1);
    assert!(matches!(m.get_mirroring_mode(), MirroringMode::Horizontal));
  }

  #[test]
  fn a_serde_round_trip_keeps_banks_and_latches() {
    let mut m = build_mapper9(16, 32);
    m.write_prg_u8(0xA000, 7);
    m.write_prg_u8(0xB000, 1);
    m.read_chr_u8(0x0FD8);
    let bytes = bincode::serialize(&m).unwrap();
    let r: Mapper9 = bincode::deserialize(&bytes).unwrap();
    assert_eq!(r.read_prg_u8(0x8000), 7);
    assert_eq!(r.read_chr_u8(0x0000), 1, "latch 0 restored as FD");
  }

  #[test]
  fn chr_rom_ignores_writes() {
    let mut m = build_mapper9(16, 32);
    m.write_chr_u8(0x0010, 0x55);
    assert_eq!(m.read_chr_u8(0x0010), 0xA0);
  }
}
