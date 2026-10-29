use super::*;
use crate::traits::Addressable;

/// CPU cycles a DMC sample fetch halts the CPU for (1-4 on hardware; the
/// common case is 4).
const DMC_STALL_CYCLES: u64 = 4;

impl Addressable for Bus {
  #[named]
  #[inline]
  fn unclocked_read_u8(&mut self, address: u16) -> u8 {
    trace_enter!();
    let result = self.inner_read_u8(address);
    trace_result!(result);
    result
  }

  #[named]
  #[inline]
  fn unclocked_write_u8(&mut self, address: u16, data: u8) {
    trace_enter!();
    self.inner_write_u8(address, data);
    trace_exit!();
  }

  #[named]
  #[inline]
  fn load(&mut self, program: Vec<u8>, start: u16) {
    trace_enter!();
    let start_address = start as usize;
    self.memory[start_address..(start_address + program.len())].copy_from_slice(&program[..]);
    self.write_u16(PROGRAM_CONTROL_ADDRESS, start);
    trace_exit!();
  }

  #[named]
  #[inline]
  fn tick(&mut self) {
    trace_enter!();
    self.clock_counter = self.clock_counter.wrapping_add(1);
    let cycles = self.clock_counter;
    trace_var!(cycles);
    // Tick PPU 3 times per CPU cycle (5.37 MHz vs 1.79 MHz)
    // After each PPU tick, check if the mapper needs a scanline signal.
    // MMC3 (and similar mappers) clock their scanline counter when A12 rises,
    // which happens at dot 260 of visible/pre-render scanlines while rendering.
    for _ in 0..3 {
      let prev_dot = self.ppu.dot;
      let prev_scanline = self.ppu.scanline;
      debug!("Ticking PPU and checking results...");
      self.ppu.tick();

      // Signal scanline to mapper at the right moment:
      // Dot 260 of scanlines 0-239 and pre-render (261) when rendering is enabled.
      let rendering_enabled = self.ppu.mask_register.get_show_background_flag() || self.ppu.mask_register.get_show_sprites_flag();
      if rendering_enabled && prev_dot == 260 && (prev_scanline < 240 || prev_scanline == 261) {
        if let Some(ref cartridge) = self.cartridge {
          cartridge.borrow_mut().signal_scanline();
        }
      }
    }
    // Tick APU once per CPU cycle
    self.apu.tick();
    // The DMC reads its samples from CPU memory, halting the CPU meanwhile;
    // the stalled cycles still clock the PPU and APU.
    if let Some(address) = self.apu.dmc_fetch_address() {
      let byte = self.inner_read_u8(address);
      self.apu.dmc_supply(byte);
      for _ in 0..DMC_STALL_CYCLES {
        self.tick();
      }
    }
    trace_exit!();
  }

  fn get_framebuffer(&self) -> &[u8] {
    self.ppu.get_framebuffer()
  }

  fn take_frame_ready(&mut self) -> bool {
    self.ppu.take_frame_ready()
  }

  fn take_audio_samples(&mut self) -> Vec<f32> {
    self.apu.take_samples()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// An NROM cartridge whose 16KB of PRG (mirrored at $C000) is all `fill`.
  fn nrom(fill: u8) -> Vec<u8> {
    let mut rom = b"NES\x1a\x01\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
    rom.extend(vec![fill; 16 * 1024]);
    rom.extend(vec![0; 8 * 1024]);
    rom
  }

  #[test]
  fn a_dmc_fetch_reads_the_cartridge_and_stalls_the_cpu() {
    let mut bus = Bus::new();
    bus.load_cartridge_data(&nrom(0xFF));
    bus.unclocked_write_u8(0x4013, 0x00);
    bus.unclocked_write_u8(0x4015, 0x10);
    assert_eq!(bus.unclocked_read_u8(0x4015) & 0x10, 0x10);
    let before = bus.clock_counter;
    bus.tick();
    // The one-byte sample is fetched, and the fetch took four more cycles.
    assert_eq!(bus.unclocked_read_u8(0x4015) & 0x10, 0);
    assert_eq!(bus.clock_counter - before, 1 + DMC_STALL_CYCLES);
  }

  #[test]
  fn without_a_sample_a_tick_is_one_cycle() {
    let mut bus = Bus::new();
    bus.load_cartridge_data(&nrom(0xFF));
    let before = bus.clock_counter;
    bus.tick();
    assert_eq!(bus.clock_counter - before, 1);
  }
}
