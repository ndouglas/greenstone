//! The fetches MMC2 (Punch-Out!!) depends on: tiles fetched but never drawn.

use super::*;
use crate::traits::Mappable;

/// A mapper 9 cartridge whose 4KB CHR bank `n` is filled with the byte `n`.
fn build_mmc2_cartridge() -> Rc<RefCell<Cartridge>> {
  let mut data = vec![0x4e, 0x45, 0x53, 0x1a, 0x02, 0x02, 0x90, 0x00, 0, 0, 0, 0, 0, 0, 0, 0];
  data.extend_from_slice(&[0u8; 2 * 0x4000]);
  for bank in 0..4u8 {
    data.extend(std::iter::repeat(bank).take(0x1000));
  }
  Rc::new(RefCell::new(Cartridge::new(&data)))
}

#[test]
fn the_two_tiles_past_the_visible_line_flip_the_mmc2_latch() {
  init();
  let mut ppu = PPU::new();
  let cart = build_mmc2_cartridge();
  ppu.vram.set_cartridge(cart.clone());
  cart.borrow_mut().write_prg_u8(0xD000, 1); // half 1 under FD: bank 1
  cart.borrow_mut().write_prg_u8(0xE000, 2); // half 1 under FE: bank 2
  cart.borrow_mut().write_prg_u8(0xF000, 0); // vertical mirroring
                                             // Row 0 of the left nametable draws tile 0; the right nametable's first
                                             // two tiles, fetched after the visible 32 but never drawn, are $FD.
  for c in 0..32 {
    ppu.vram.write_u8(0x2000 + c, 0x00);
    ppu.vram.write_u8(0x2400 + c, 0x00);
  }
  ppu.vram.write_u8(0x2400, 0xFD);
  ppu.vram.write_u8(0x2401, 0xFD);
  ppu.control_register.set_background_address_flag(true);
  ppu.mask_register.set_show_background_flag(true);
  ppu.mask_register.set_show_background_left_flag(true);
  assert_eq!(cart.borrow().read_chr_u8(0x1000), 2, "the latch powers on at FE");
  advance_ppu_to(&mut ppu, 0, 258);
  assert_eq!(cart.borrow().read_chr_u8(0x1000), 1, "fetching the undrawn $FD tiles set the latch to FD");
}
