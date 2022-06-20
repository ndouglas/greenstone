use super::*;
use crate::traits::mappable::MirroringMode;
use crate::traits::Mappable;

impl Mappable for Cartridge {
  #[named]
  #[inline]
  fn read_prg_u8(&self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let result = self.mapper.read_prg_u8(address);
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  fn write_prg_u8(&mut self, address: u16, data: u8) {
    trace_enter!();
    trace_u16!(address);
    trace_u8!(data);
    self.mapper.write_prg_u8(address, data);
    trace_exit!();
  }

  #[named]
  #[inline]
  fn read_chr_u8(&self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let result = self.mapper.read_chr_u8(address);
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  fn write_chr_u8(&mut self, address: u16, data: u8) {
    trace_enter!();
    trace_u16!(address);
    trace_u8!(data);
    self.mapper.write_chr_u8(address, data);
    trace_exit!();
  }

  #[named]
  #[inline]
  fn signal_scanline(&mut self) {
    trace_enter!();
    self.mapper.signal_scanline();
    trace_exit!();
  }

  #[named]
  #[inline]
  fn get_mirroring_mode(&self) -> MirroringMode {
    trace_enter!();
    let result = self.mapper.get_mirroring_mode();
    trace_var!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  fn get_irq_flag(&self) -> bool {
    trace_enter!();
    let result = self.mapper.get_irq_flag();
    trace_var!(result);
    trace_exit!();
    result
  }
}
