//! Noise channel implementation.
//!
//! The noise channel generates pseudo-random noise using a linear feedback
//! shift register (LFSR). It can produce either "white noise" or a more
//! metallic/buzzing sound depending on the mode.

/// Noise period lookup table (NTSC)
const NOISE_PERIOD_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

/// Noise channel state.
pub struct Noise {
    // Registers
    /// Envelope loop/length counter halt, constant volume, volume/envelope period
    envelope_reg: u8,
    /// Mode flag (bit 7), period index (bits 0-3)
    period_reg: u8,
    /// Length counter load (upper 5 bits)
    length_reg: u8,

    // Internal state
    /// 15-bit linear feedback shift register
    shift_register: u16,
    /// Timer counter
    timer_counter: u16,
    /// Timer period
    timer_period: u16,

    // Envelope
    /// Envelope start flag
    envelope_start: bool,
    /// Envelope divider counter
    envelope_divider: u8,
    /// Envelope decay level (0-15)
    envelope_decay: u8,

    // Length counter
    /// Length counter value
    pub length_counter: u8,
    /// Whether the channel is enabled
    pub enabled: bool,
}

impl Noise {
    pub fn new() -> Self {
        Noise {
            envelope_reg: 0,
            period_reg: 0,
            length_reg: 0,
            shift_register: 1, // Must be non-zero, typically initialized to 1
            timer_counter: 0,
            timer_period: 0,
            envelope_start: false,
            envelope_divider: 0,
            envelope_decay: 0,
            length_counter: 0,
            enabled: false,
        }
    }

    /// Write to register 0 ($400C): envelope
    pub fn write_envelope(&mut self, value: u8) {
        self.envelope_reg = value;
    }

    /// Write to register 2 ($400E): mode and period
    pub fn write_period(&mut self, value: u8) {
        self.period_reg = value;
        self.timer_period = NOISE_PERIOD_TABLE[(value & 0x0F) as usize];
    }

    /// Write to register 3 ($400F): length counter load
    pub fn write_length(&mut self, value: u8, length_table: &[u8; 32]) {
        self.length_reg = value;

        // Load length counter if channel is enabled
        if self.enabled {
            let length_index = (value >> 3) as usize;
            self.length_counter = length_table[length_index];
        }

        // Restart envelope
        self.envelope_start = true;
    }

    /// Tick the timer (called every APU cycle)
    pub fn tick_timer(&mut self) {
        if self.timer_counter == 0 {
            self.timer_counter = self.timer_period;
            self.clock_shift_register();
        } else {
            self.timer_counter -= 1;
        }
    }

    /// Clock the LFSR
    fn clock_shift_register(&mut self) {
        // Get the feedback bit
        let mode = (self.period_reg & 0x80) != 0;
        let feedback_bit = if mode {
            // Mode 1: XOR bit 0 and bit 6
            ((self.shift_register & 0x01) ^ ((self.shift_register >> 6) & 0x01)) as u16
        } else {
            // Mode 0: XOR bit 0 and bit 1
            ((self.shift_register & 0x01) ^ ((self.shift_register >> 1) & 0x01)) as u16
        };

        // Shift right and insert feedback at bit 14
        self.shift_register = (self.shift_register >> 1) | (feedback_bit << 14);
    }

    /// Clock the envelope (called on quarter-frame)
    pub fn clock_envelope(&mut self) {
        if self.envelope_start {
            self.envelope_start = false;
            self.envelope_decay = 15;
            self.envelope_divider = self.get_envelope_period();
        } else if self.envelope_divider > 0 {
            self.envelope_divider -= 1;
        } else {
            self.envelope_divider = self.get_envelope_period();
            if self.envelope_decay > 0 {
                self.envelope_decay -= 1;
            } else if self.get_envelope_loop() {
                self.envelope_decay = 15;
            }
        }
    }

    /// Clock the length counter (called on half-frame)
    pub fn clock_length_counter(&mut self) {
        if !self.get_length_counter_halt() && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    /// Get the current output sample (0-15)
    pub fn output(&self) -> u8 {
        // Silenced if length counter is 0
        if self.length_counter == 0 {
            return 0;
        }
        // Silenced if timer period is 0 (not properly initialized)
        // Minimum valid period from table is 4
        if self.timer_period == 0 {
            return 0;
        }
        // Silenced when shift register bit 0 is set
        if (self.shift_register & 0x01) != 0 {
            return 0;
        }

        // Return envelope or constant volume
        if self.get_constant_volume() {
            self.get_volume()
        } else {
            self.envelope_decay
        }
    }

    // Helper methods

    fn get_envelope_period(&self) -> u8 {
        self.envelope_reg & 0x0F
    }

    fn get_envelope_loop(&self) -> bool {
        (self.envelope_reg & 0x20) != 0
    }

    fn get_length_counter_halt(&self) -> bool {
        (self.envelope_reg & 0x20) != 0
    }

    fn get_constant_volume(&self) -> bool {
        (self.envelope_reg & 0x10) != 0
    }

    fn get_volume(&self) -> u8 {
        self.envelope_reg & 0x0F
    }

    /// Set enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }
}

impl Default for Noise {
    fn default() -> Self {
        Self::new()
    }
}
