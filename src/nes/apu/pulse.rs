//! Pulse (square wave) channel implementation.
//!
//! The NES has two pulse channels that generate square waves with
//! configurable duty cycles (12.5%, 25%, 50%, 75%).

/// Duty cycle waveform patterns.
/// Each pattern is 8 steps, output is 0 or 1.
const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0], // 12.5%
    [0, 1, 1, 0, 0, 0, 0, 0], // 25%
    [0, 1, 1, 1, 1, 0, 0, 0], // 50%
    [1, 0, 0, 1, 1, 1, 1, 1], // 75% (inverted 25%)
];

/// Pulse channel state.
pub struct Pulse {
    /// Channel number (1 or 2) - affects sweep unit behavior
    channel: u8,

    // Registers
    /// Duty cycle (0-3), envelope loop/length counter halt, constant volume flag, volume/envelope
    duty_envelope: u8,
    /// Sweep unit: enabled, period, negate, shift
    sweep: u8,
    /// Timer low 8 bits
    timer_low: u8,
    /// Length counter load (upper 5 bits), timer high 3 bits (lower 3 bits)
    length_timer_high: u8,

    // Internal state
    /// Current position in duty cycle (0-7)
    duty_position: u8,
    /// Timer counter (counts down from period)
    timer_counter: u16,
    /// Timer period (11-bit, from timer_low and timer_high)
    timer_period: u16,

    // Envelope
    /// Envelope start flag
    envelope_start: bool,
    /// Envelope divider counter
    envelope_divider: u8,
    /// Envelope decay level (0-15)
    envelope_decay: u8,

    // Sweep
    /// Sweep reload flag
    sweep_reload: bool,
    /// Sweep divider counter
    sweep_divider: u8,
    /// Target period for muting calculation
    sweep_target_period: u16,
    /// Whether channel is muted due to sweep
    sweep_muting: bool,

    // Length counter
    /// Length counter value
    pub length_counter: u8,
    /// Whether the channel is enabled
    pub enabled: bool,
}

impl Pulse {
    pub fn new(channel: u8) -> Self {
        Pulse {
            channel,
            duty_envelope: 0,
            sweep: 0,
            timer_low: 0,
            length_timer_high: 0,
            duty_position: 0,
            timer_counter: 0,
            timer_period: 0,
            envelope_start: false,
            envelope_divider: 0,
            envelope_decay: 0,
            sweep_reload: false,
            sweep_divider: 0,
            sweep_target_period: 0,
            sweep_muting: false,
            length_counter: 0,
            enabled: false,
        }
    }

    /// Write to register 0 ($4000/$4004): duty, envelope
    pub fn write_duty_envelope(&mut self, value: u8) {
        self.duty_envelope = value;
    }

    /// Write to register 1 ($4001/$4005): sweep
    pub fn write_sweep(&mut self, value: u8) {
        self.sweep = value;
        self.sweep_reload = true;
    }

    /// Write to register 2 ($4002/$4006): timer low
    pub fn write_timer_low(&mut self, value: u8) {
        self.timer_low = value;
        self.update_timer_period();
    }

    /// Write to register 3 ($4003/$4007): length counter load, timer high
    pub fn write_length_timer_high(&mut self, value: u8, length_table: &[u8; 32]) {
        self.length_timer_high = value;
        self.update_timer_period();

        // Load length counter if channel is enabled
        if self.enabled {
            let length_index = (value >> 3) as usize;
            self.length_counter = length_table[length_index];
        }

        // Restart envelope
        self.envelope_start = true;

        // Reset duty position
        self.duty_position = 0;
    }

    /// Update the 11-bit timer period from registers
    fn update_timer_period(&mut self) {
        self.timer_period =
            (self.timer_low as u16) | (((self.length_timer_high & 0x07) as u16) << 8);
        self.calculate_sweep_target();
    }

    /// Calculate the sweep target period and muting status
    fn calculate_sweep_target(&mut self) {
        let shift = self.sweep & 0x07;
        let change = self.timer_period >> shift;

        let negate = (self.sweep & 0x08) != 0;
        if negate {
            // Pulse 1 uses one's complement (subtract change + 1)
            // Pulse 2 uses two's complement (subtract change)
            if self.channel == 1 {
                self.sweep_target_period = self.timer_period.wrapping_sub(change).wrapping_sub(1);
            } else {
                self.sweep_target_period = self.timer_period.wrapping_sub(change);
            }
        } else {
            self.sweep_target_period = self.timer_period.wrapping_add(change);
        }

        // Muting occurs if period < 8 or target period > 0x7FF
        self.sweep_muting = self.timer_period < 8 || self.sweep_target_period > 0x7FF;
    }

    /// Tick the timer (called every APU cycle, which is every 2 CPU cycles)
    pub fn tick_timer(&mut self) {
        if self.timer_counter == 0 {
            self.timer_counter = self.timer_period;
            // Advance duty position
            self.duty_position = (self.duty_position + 1) & 0x07;
        } else {
            self.timer_counter -= 1;
        }
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

    /// Clock the sweep unit (called on half-frame)
    pub fn clock_sweep(&mut self) {
        // Decrement divider, reload if needed
        let should_update = self.sweep_divider == 0 && self.is_sweep_enabled() && !self.sweep_muting;

        if self.sweep_divider == 0 || self.sweep_reload {
            self.sweep_divider = self.get_sweep_period();
            self.sweep_reload = false;

            if should_update && self.get_sweep_shift() > 0 {
                self.timer_period = self.sweep_target_period;
                self.calculate_sweep_target();
            }
        } else {
            self.sweep_divider -= 1;
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
        // Channel is silenced if:
        // - Length counter is 0
        // - Sweep is muting
        // - Duty cycle output is 0
        if self.length_counter == 0 {
            return 0;
        }
        if self.sweep_muting {
            return 0;
        }

        let duty = (self.duty_envelope >> 6) & 0x03;
        if DUTY_TABLE[duty as usize][self.duty_position as usize] == 0 {
            return 0;
        }

        // Return envelope or constant volume
        if self.get_constant_volume() {
            self.get_volume()
        } else {
            self.envelope_decay
        }
    }

    // Helper methods for register bits

    fn get_envelope_period(&self) -> u8 {
        self.duty_envelope & 0x0F
    }

    fn get_envelope_loop(&self) -> bool {
        (self.duty_envelope & 0x20) != 0
    }

    fn get_length_counter_halt(&self) -> bool {
        // Same bit as envelope loop
        (self.duty_envelope & 0x20) != 0
    }

    fn get_constant_volume(&self) -> bool {
        (self.duty_envelope & 0x10) != 0
    }

    fn get_volume(&self) -> u8 {
        self.duty_envelope & 0x0F
    }

    fn is_sweep_enabled(&self) -> bool {
        (self.sweep & 0x80) != 0
    }

    fn get_sweep_period(&self) -> u8 {
        (self.sweep >> 4) & 0x07
    }

    fn get_sweep_shift(&self) -> u8 {
        self.sweep & 0x07
    }

    /// Set enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }
}
