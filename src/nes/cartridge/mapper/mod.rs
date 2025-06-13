use crate::traits::mappable::MirroringMode;
use crate::traits::Mappable;

pub mod mapper0;
pub use mapper0::*;

pub mod mapper1;
pub use mapper1::*;

pub mod mapper2;
pub use mapper2::*;

pub mod mapper3;
pub use mapper3::*;

pub mod mapper4;
pub use mapper4::*;

pub mod mapper7;
pub use mapper7::*;

/// Enum-based mapper dispatch, replacing `Box<dyn Mappable>`.
/// This enables serde serialization of the entire cartridge state.
#[derive(Serialize, Deserialize)]
pub enum Mapper {
  Nrom(Mapper0),
  Mmc1(Mapper1),
  UxRom(Mapper2),
  CnRom(Mapper3),
  Mmc3(Mapper4),
  AxRom(Mapper7),
}

impl Mappable for Mapper {
  #[inline]
  fn read_prg_u8(&self, address: u16) -> u8 {
    match self {
      Mapper::Nrom(m) => m.read_prg_u8(address),
      Mapper::Mmc1(m) => m.read_prg_u8(address),
      Mapper::UxRom(m) => m.read_prg_u8(address),
      Mapper::CnRom(m) => m.read_prg_u8(address),
      Mapper::Mmc3(m) => m.read_prg_u8(address),
      Mapper::AxRom(m) => m.read_prg_u8(address),
    }
  }

  #[inline]
  fn write_prg_u8(&mut self, address: u16, data: u8) {
    match self {
      Mapper::Nrom(m) => m.write_prg_u8(address, data),
      Mapper::Mmc1(m) => m.write_prg_u8(address, data),
      Mapper::UxRom(m) => m.write_prg_u8(address, data),
      Mapper::CnRom(m) => m.write_prg_u8(address, data),
      Mapper::Mmc3(m) => m.write_prg_u8(address, data),
      Mapper::AxRom(m) => m.write_prg_u8(address, data),
    }
  }

  #[inline]
  fn read_chr_u8(&self, address: u16) -> u8 {
    match self {
      Mapper::Nrom(m) => m.read_chr_u8(address),
      Mapper::Mmc1(m) => m.read_chr_u8(address),
      Mapper::UxRom(m) => m.read_chr_u8(address),
      Mapper::CnRom(m) => m.read_chr_u8(address),
      Mapper::Mmc3(m) => m.read_chr_u8(address),
      Mapper::AxRom(m) => m.read_chr_u8(address),
    }
  }

  #[inline]
  fn write_chr_u8(&mut self, address: u16, data: u8) {
    match self {
      Mapper::Nrom(m) => m.write_chr_u8(address, data),
      Mapper::Mmc1(m) => m.write_chr_u8(address, data),
      Mapper::UxRom(m) => m.write_chr_u8(address, data),
      Mapper::CnRom(m) => m.write_chr_u8(address, data),
      Mapper::Mmc3(m) => m.write_chr_u8(address, data),
      Mapper::AxRom(m) => m.write_chr_u8(address, data),
    }
  }

  #[inline]
  fn signal_scanline(&mut self) {
    match self {
      Mapper::Nrom(m) => m.signal_scanline(),
      Mapper::Mmc1(m) => m.signal_scanline(),
      Mapper::UxRom(m) => m.signal_scanline(),
      Mapper::CnRom(m) => m.signal_scanline(),
      Mapper::Mmc3(m) => m.signal_scanline(),
      Mapper::AxRom(m) => m.signal_scanline(),
    }
  }

  #[inline]
  fn get_mirroring_mode(&self) -> MirroringMode {
    match self {
      Mapper::Nrom(m) => m.get_mirroring_mode(),
      Mapper::Mmc1(m) => m.get_mirroring_mode(),
      Mapper::UxRom(m) => m.get_mirroring_mode(),
      Mapper::CnRom(m) => m.get_mirroring_mode(),
      Mapper::Mmc3(m) => m.get_mirroring_mode(),
      Mapper::AxRom(m) => m.get_mirroring_mode(),
    }
  }

  #[inline]
  fn get_irq_flag(&self) -> bool {
    match self {
      Mapper::Nrom(m) => m.get_irq_flag(),
      Mapper::Mmc1(m) => m.get_irq_flag(),
      Mapper::UxRom(m) => m.get_irq_flag(),
      Mapper::CnRom(m) => m.get_irq_flag(),
      Mapper::Mmc3(m) => m.get_irq_flag(),
      Mapper::AxRom(m) => m.get_irq_flag(),
    }
  }
}
