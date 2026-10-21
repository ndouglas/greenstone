use crate::traits::mappable::MirroringMode;

pub mod data;
pub use data::*;

pub mod header;
pub use header::*;

mod mappable;

pub mod mapper;
pub use mapper::*;

pub mod pager;
pub use pager::*;

#[derive(Serialize, Deserialize)]
pub struct Cartridge {
  mapper: Mapper,
}

impl Cartridge {
  pub fn new(data: &[u8]) -> Cartridge {
    let data = Data::new(data);
    let mapper = match data.header.mapper_number {
      0 => Mapper::Nrom(Mapper0::new(data)),
      1 => Mapper::Mmc1(Mapper1::new(data)),
      2 => Mapper::UxRom(Mapper2::new(data)),
      3 => Mapper::CnRom(Mapper3::new(data)),
      4 => Mapper::Mmc3(Mapper4::new(data)),
      7 => Mapper::AxRom(Mapper7::new(data)),
      9 => Mapper::Mmc2(Mapper9::new(data)),
      number => panic!("Mapper {number} not implemented"),
    };
    Cartridge { mapper }
  }
}

// These test cases are based on Starr Horne's `nes-rust`.
// See https://github.com/starrhorne/nes-rust/blob/master/src/cartridge/mapper.rs
#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;
  use crate::traits::Mappable;

  fn build_cartridge(chr_ram: bool) -> Cartridge {
    let mut data = vec![
      0x4e, 0x45, 0x53, 0x1a, 0x02, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    data[5] = !chr_ram as u8;
    for i in 0..0x8000u16 / 2 {
      data.push((i >> 8) as u8);
      data.push(i as u8);
    }
    if !chr_ram {
      for i in 0..0x2000u16 / 2 {
        data.push((i >> 8) as u8);
        data.push(i as u8);
      }
    }
    Cartridge::new(&data)
  }

  #[test]
  fn test_read_prg_rom() {
    init();
    let cartridge = build_cartridge(false);
    for i in 0..0x8000u16 {
      if i % 2 == 0 {
        assert_eq!(cartridge.read_prg_u8(0x8000 + i), ((i / 2) >> 8) as u8);
      } else {
        assert_eq!(cartridge.read_prg_u8(0x8000 + i), (i / 2) as u8);
      }
    }
  }

  #[test]
  fn test_prg_ram() {
    init();
    let mut cartridge = build_cartridge(false);
    for i in 0x6000u16..0x7000u16 {
      cartridge.write_prg_u8(i, i as u8);
      assert_eq!(cartridge.read_prg_u8(i), i as u8);
    }
  }

  #[test]
  fn test_read_chr_rom() {
    init();
    let cartridge = build_cartridge(false);
    for i in 0..0x2000u16 {
      if i % 2 == 0 {
        assert_eq!(cartridge.read_chr_u8(i), ((i / 2) >> 8) as u8);
      } else {
        assert_eq!(cartridge.read_chr_u8(i), (i / 2) as u8);
      }
    }
  }

  #[test]
  fn test_chr_ram() {
    init();
    let mut cartridge = build_cartridge(true);
    for i in 0..0x2000u16 {
      cartridge.write_chr_u8(i, i as u8);
      assert_eq!(cartridge.read_chr_u8(i), i as u8);
    }
  }
}
