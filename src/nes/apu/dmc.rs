//! DMC (delta modulation) channel implementation.
//!
//! The DMC plays 1-bit delta-encoded samples fetched from CPU memory: each
//! bit moves a 7-bit output level up or down by 2. It can also be written
//! directly through $4011.

/// CPU cycles per output bit, indexed by the rate bits of $4010 (NTSC).
const DMC_RATE_TABLE: [u16; 16] = [428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54];

/// DMC channel state.
#[derive(Serialize, Deserialize)]
pub struct Dmc {
  // Registers
  /// IRQ enable (bit 7), loop (bit 6), rate index (bits 0-3)
  flags_rate: u8,
  /// Sample start address, $C000 + 64 * the $4012 value
  sample_address: u16,
  /// Sample length in bytes, 16 * the $4013 value + 1
  sample_length: u16,

  // Memory reader
  /// Address of the next byte to fetch
  current_address: u16,
  /// Bytes still to fetch; the channel is active while this is non-zero
  pub bytes_remaining: u16,
  /// The fetched byte waiting for the output unit
  sample_buffer: Option<u8>,

  // Timer
  timer_counter: u16,
  timer_period: u16,

  // Output unit
  shift_register: u8,
  bits_remaining: u8,
  silence: bool,
  /// 7-bit output level
  output_level: u8,

  /// Set when the last byte of a non-looping sample is fetched with IRQs on
  pub irq_flag: bool,
}

impl Dmc {
  pub fn new() -> Self {
    Dmc {
      flags_rate: 0,
      sample_address: 0xC000,
      sample_length: 1,
      current_address: 0xC000,
      bytes_remaining: 0,
      sample_buffer: None,
      timer_counter: 0,
      timer_period: DMC_RATE_TABLE[0],
      shift_register: 0,
      bits_remaining: 8,
      silence: true,
      output_level: 0,
      irq_flag: false,
    }
  }

  /// Write to register 0 ($4010): IRQ enable, loop, rate
  pub fn write_flags_rate(&mut self, value: u8) {
    self.flags_rate = value;
    self.timer_period = DMC_RATE_TABLE[(value & 0x0F) as usize];
    if value & 0x80 == 0 {
      self.irq_flag = false;
    }
  }

  /// Write to register 1 ($4011): direct load of the output level
  pub fn write_direct_load(&mut self, value: u8) {
    self.output_level = value & 0x7F;
  }

  /// Write to register 2 ($4012): sample address
  pub fn write_address(&mut self, value: u8) {
    self.sample_address = 0xC000 + value as u16 * 64;
  }

  /// Write to register 3 ($4013): sample length
  pub fn write_length(&mut self, value: u8) {
    self.sample_length = value as u16 * 16 + 1;
  }

  /// Bit 4 of $4015: stop the sample, or start it if none is playing.
  pub fn set_enabled(&mut self, enabled: bool) {
    if !enabled {
      self.bytes_remaining = 0;
    } else if self.bytes_remaining == 0 {
      self.restart();
    }
  }

  fn restart(&mut self) {
    self.current_address = self.sample_address;
    self.bytes_remaining = self.sample_length;
  }

  /// The address the memory reader wants read, if its buffer is empty.
  pub fn fetch_address(&self) -> Option<u16> {
    (self.sample_buffer.is_none() && self.bytes_remaining > 0).then_some(self.current_address)
  }

  /// The byte read from `fetch_address`.
  pub fn supply(&mut self, byte: u8) {
    self.sample_buffer = Some(byte);
    self.current_address = if self.current_address == 0xFFFF {
      0x8000
    } else {
      self.current_address + 1
    };
    self.bytes_remaining -= 1;
    if self.bytes_remaining == 0 {
      if self.flags_rate & 0x40 != 0 {
        self.restart();
      } else if self.flags_rate & 0x80 != 0 {
        self.irq_flag = true;
      }
    }
  }

  /// Tick the timer (called every CPU cycle).
  pub fn tick_timer(&mut self) {
    if self.timer_counter > 0 {
      self.timer_counter -= 1;
      return;
    }
    self.timer_counter = self.timer_period - 1;
    if !self.silence {
      if self.shift_register & 1 == 1 {
        if self.output_level <= 125 {
          self.output_level += 2;
        }
      } else if self.output_level >= 2 {
        self.output_level -= 2;
      }
    }
    self.shift_register >>= 1;
    self.bits_remaining -= 1;
    if self.bits_remaining == 0 {
      self.bits_remaining = 8;
      match self.sample_buffer.take() {
        Some(byte) => {
          self.shift_register = byte;
          self.silence = false;
        }
        None => self.silence = true,
      }
    }
  }

  /// Current output level (0-127).
  pub fn output(&self) -> u8 {
    self.output_level
  }
}

impl Default for Dmc {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Feed the channel from `memory` (indexed from $C000) until `cycles` pass.
  fn run(dmc: &mut Dmc, memory: &[u8], cycles: usize) {
    for _ in 0..cycles {
      if let Some(address) = dmc.fetch_address() {
        dmc.supply(memory[(address - 0xC000) as usize]);
      }
      dmc.tick_timer();
    }
  }

  #[test]
  fn address_and_length_registers_scale_as_the_hardware_does() {
    let mut dmc = Dmc::new();
    dmc.write_address(0x01);
    dmc.write_length(0x01);
    dmc.set_enabled(true);
    assert_eq!(dmc.fetch_address(), Some(0xC040));
    assert_eq!(dmc.bytes_remaining, 17);
  }

  #[test]
  fn nothing_is_fetched_until_the_channel_is_enabled() {
    let mut dmc = Dmc::new();
    dmc.write_length(0x01);
    assert_eq!(dmc.fetch_address(), None);
    dmc.set_enabled(true);
    assert!(dmc.fetch_address().is_some());
    dmc.set_enabled(false);
    assert_eq!(dmc.fetch_address(), None);
    assert_eq!(dmc.bytes_remaining, 0);
  }

  #[test]
  fn a_fetch_advances_the_address_and_wraps_to_8000() {
    let mut dmc = Dmc::new();
    dmc.write_address(0xFF);
    dmc.write_length(0xFF);
    dmc.set_enabled(true);
    assert_eq!(dmc.fetch_address(), Some(0xFFC0));
    for _ in 0..0x3F {
      dmc.supply(0);
      dmc.sample_buffer = None;
    }
    assert_eq!(dmc.fetch_address(), Some(0xFFFF));
    dmc.supply(0);
    dmc.sample_buffer = None;
    assert_eq!(dmc.fetch_address(), Some(0x8000));
  }

  #[test]
  fn one_bits_raise_the_level_and_zero_bits_lower_it() {
    // Rate 15 is 54 CPU cycles a bit.
    let mut dmc = Dmc::new();
    dmc.write_flags_rate(0x0F);
    dmc.write_direct_load(64);
    dmc.write_length(0x00);
    dmc.set_enabled(true);
    let memory = [0xFFu8; 64];
    // The first byte loads after the silent 8 bits already in the unit.
    run(&mut dmc, &memory, 54 * 16 + 1);
    assert_eq!(dmc.output(), 64 + 16);

    let mut dmc = Dmc::new();
    dmc.write_flags_rate(0x0F);
    dmc.write_direct_load(64);
    dmc.write_length(0x00);
    dmc.set_enabled(true);
    run(&mut dmc, &[0x00u8; 64], 54 * 16 + 1);
    assert_eq!(dmc.output(), 64 - 16);
  }

  #[test]
  fn the_level_stays_within_seven_bits() {
    let mut dmc = Dmc::new();
    dmc.write_flags_rate(0x0F);
    dmc.write_direct_load(0xFF);
    assert_eq!(dmc.output(), 127);
    dmc.write_length(0x00);
    dmc.set_enabled(true);
    run(&mut dmc, &[0xFFu8; 64], 54 * 16 + 1);
    assert_eq!(dmc.output(), 127);
    dmc.write_direct_load(1);
    dmc.write_flags_rate(0x0F);
    dmc.set_enabled(true);
    run(&mut dmc, &[0x00u8; 64], 54 * 16 + 1);
    assert_eq!(dmc.output(), 1);
  }

  #[test]
  fn the_last_byte_raises_the_irq_unless_the_sample_loops() {
    let mut dmc = Dmc::new();
    dmc.write_flags_rate(0x80);
    dmc.write_length(0x00);
    dmc.set_enabled(true);
    dmc.supply(0);
    assert_eq!(dmc.bytes_remaining, 0);
    assert!(dmc.irq_flag);

    let mut dmc = Dmc::new();
    dmc.write_flags_rate(0xC0);
    dmc.write_length(0x00);
    dmc.set_enabled(true);
    dmc.supply(0);
    assert_eq!(dmc.bytes_remaining, 1);
    assert!(!dmc.irq_flag);
  }

  #[test]
  fn clearing_the_irq_enable_clears_the_flag() {
    let mut dmc = Dmc::new();
    dmc.write_flags_rate(0x80);
    dmc.write_length(0x00);
    dmc.set_enabled(true);
    dmc.supply(0);
    assert!(dmc.irq_flag);
    dmc.write_flags_rate(0x00);
    assert!(!dmc.irq_flag);
  }

  #[test]
  fn the_buffer_is_only_refilled_once_it_empties() {
    let mut dmc = Dmc::new();
    dmc.write_length(0x01);
    dmc.set_enabled(true);
    dmc.supply(0xAA);
    assert_eq!(dmc.fetch_address(), None);
  }
}
