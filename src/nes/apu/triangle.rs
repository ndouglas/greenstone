//! Triangle wave channel implementation.
//!
//! The triangle channel generates a triangle wave with no volume control.
//! It has a linear counter in addition to the length counter.

/// Triangle waveform sequence (32 steps)
const TRIANGLE_TABLE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
];

/// Triangle channel state.
pub struct Triangle {
    // Registers
    /// Linear counter load (lower 7 bits), control flag (bit 7)
    linear_control: u8,
    /// Timer low 8 bits
    timer_low: u8,
    /// Length counter load (upper 5 bits), timer high 3 bits (lower 3 bits)
    length_timer_high: u8,

    // Internal state
    /// Current position in triangle sequence (0-31)
    sequence_position: u8,
    /// Timer counter (counts down from period)
    timer_counter: u16,
    /// Timer period (11-bit)
    timer_period: u16,

    // Linear counter
    /// Linear counter value
    linear_counter: u8,
    /// Linear counter reload flag
    linear_reload: bool,

    // Length counter
    /// Length counter value
    pub length_counter: u8,
    /// Whether the channel is enabled
    pub enabled: bool,
}

impl Triangle {
    pub fn new() -> Self {
        Triangle {
            linear_control: 0,
            timer_low: 0,
            length_timer_high: 0,
            sequence_position: 0,
            timer_counter: 0,
            timer_period: 0,
            linear_counter: 0,
            linear_reload: false,
            length_counter: 0,
            enabled: false,
        }
    }

    /// Write to register 0 ($4008): linear counter
    pub fn write_linear_control(&mut self, value: u8) {
        self.linear_control = value;
    }

    /// Write to register 2 ($400A): timer low
    pub fn write_timer_low(&mut self, value: u8) {
        self.timer_low = value;
        self.update_timer_period();
    }

    /// Write to register 3 ($400B): length counter load, timer high
    pub fn write_length_timer_high(&mut self, value: u8, length_table: &[u8; 32]) {
        self.length_timer_high = value;
        self.update_timer_period();

        // Load length counter if channel is enabled
        if self.enabled {
            let length_index = (value >> 3) as usize;
            self.length_counter = length_table[length_index];
        }

        // Set linear counter reload flag
        self.linear_reload = true;
    }

    /// Update the 11-bit timer period from registers
    fn update_timer_period(&mut self) {
        self.timer_period =
            (self.timer_low as u16) | (((self.length_timer_high & 0x07) as u16) << 8);
    }

    /// Tick the timer (called every CPU cycle - triangle runs at CPU rate)
    pub fn tick_timer(&mut self) {
        if self.timer_counter == 0 {
            self.timer_counter = self.timer_period;

            // Only advance if both counters are non-zero
            if self.length_counter > 0 && self.linear_counter > 0 {
                self.sequence_position = (self.sequence_position + 1) & 0x1F;
            }
        } else {
            self.timer_counter -= 1;
        }
    }

    /// Clock the linear counter (called on quarter-frame)
    pub fn clock_linear_counter(&mut self) {
        if self.linear_reload {
            self.linear_counter = self.get_linear_counter_load();
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }

        // Clear reload flag if control flag is clear
        if !self.get_control_flag() {
            self.linear_reload = false;
        }
    }

    /// Clock the length counter (called on half-frame)
    pub fn clock_length_counter(&mut self) {
        // Length counter halt is same as control flag for triangle
        if !self.get_control_flag() && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    /// Get the current output sample (0-15)
    pub fn output(&self) -> u8 {
        // Silenced if length counter or linear counter is 0
        // But the sequencer still runs, just outputs the last value
        // Actually, for ultrasonic frequencies (period < 2), output is silenced
        if self.timer_period < 2 {
            return 0;
        }

        TRIANGLE_TABLE[self.sequence_position as usize]
    }

    // Helper methods

    fn get_linear_counter_load(&self) -> u8 {
        self.linear_control & 0x7F
    }

    fn get_control_flag(&self) -> bool {
        (self.linear_control & 0x80) != 0
    }

    /// Set enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }
}

impl Default for Triangle {
    fn default() -> Self {
        Self::new()
    }
}
