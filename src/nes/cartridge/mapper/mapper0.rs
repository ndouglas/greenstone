use super::super::{Data, MirroringMode, Page, PageSize};
use crate::traits::Mappable;

use Page::*;
use PageSize::*;

pub struct Mapper0 {
  data: Data,
}

impl Mapper0 {
  pub fn new(data: Data) -> Mapper0 {
    Mapper0 { data }
  }
}

impl Mappable for Mapper0 {
  #[named]
  fn read_prg_u8(&self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let result = match address {
      0x6000..=0x7FFF => self.data.prg_ram.read_u8(First(EightKb), address - 0x6000),
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
      0x6000..=0x7FFF => self.data.prg_ram.write_u8(First(EightKb), address - 0x6000, value),
      _ => panic!("bad address: {}", format_u16!(address)),
    }
    trace_exit!();
  }

  #[named]
  fn read_chr_u8(&self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let result = if self.data.header.chr_rom_pages == 0 {
      self.data.chr_ram.read_u8(First(EightKb), address)
    } else {
      self.data.chr_rom.read_u8(First(EightKb), address)
    };
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  fn write_chr_u8(&mut self, address: u16, value: u8) {
    trace_enter!();
    trace_u16!(address);
    trace_u8!(value);
    if self.data.header.chr_rom_pages == 0 {
      self.data.chr_ram.write_u8(First(EightKb), address, value);
    }
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
