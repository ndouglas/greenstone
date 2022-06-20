use super::Header;
use super::Pager;

// This implementation comes directly from Starr Horne's rust-nes.
// See https://github.com/starrhorne/nes-rust/blob/master/src/cartridge/cartridge_data.rs
pub struct Data {
  pub header: Header,
  pub prg_rom: Pager,
  pub prg_ram: Pager,
  pub chr_rom: Pager,
  pub chr_ram: Pager,
}

impl Data {
  pub fn new(data: &[u8]) -> Data {
    let header = Header::new(data);

    let prg_rom_range = header.get_prg_rom_range();
    let prg_rom_data = data[prg_rom_range].to_vec();

    let chr_rom_range = header.get_chr_rom_range();
    let chr_rom_data = data[chr_rom_range].to_vec();

    let prg_ram_size = header.get_prg_ram_size();
    let prg_ram_data = vec![0u8; prg_ram_size];

    let chr_ram_size = header.get_chr_ram_size();
    let chr_ram_data = vec![0u8; chr_ram_size];

    Data {
      header,
      prg_rom: Pager::new(prg_rom_data),
      chr_rom: Pager::new(chr_rom_data),
      prg_ram: Pager::new(prg_ram_data),
      chr_ram: Pager::new(chr_ram_data),
    }
  }
}
