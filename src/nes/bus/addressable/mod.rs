use super::*;
use crate::traits::Addressable;

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
    self.write_u16(PROGRAM_CONTROL_ADDRESS.try_into().unwrap(), start);
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
    debug!("Ticking PPU and checking results...");
    self.ppu.tick();
    debug!("Ticking PPU and checking results...");
    self.ppu.tick();
    debug!("Ticking PPU and checking results...");
    self.ppu.tick();
    // Tick APU once per CPU cycle
    self.apu.tick();
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
