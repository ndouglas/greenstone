#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub enum MirroringMode {
  Vertical,
  Horizontal,
  SingleScreenLower,
  SingleScreenUpper,
  None,
}

// A trait for mappers, specifically, or anything that uses them.
pub trait Mappable {
  fn read_prg_u8(&self, address: u16) -> u8;

  fn write_prg_u8(&mut self, address: u16, data: u8);

  fn read_chr_u8(&self, address: u16) -> u8;

  fn write_chr_u8(&mut self, address: u16, data: u8);

  fn signal_scanline(&mut self) {}

  fn get_mirroring_mode(&self) -> MirroringMode;

  fn get_irq_flag(&self) -> bool {
    false
  }
}
