//! NES APU (Audio Processing Unit) implementation.
//!
//! The APU generates audio through 5 channels:
//! - 2 Pulse channels (square waves)
//! - 1 Triangle channel
//! - 1 Noise channel
//! - 1 DMC channel (delta modulation)
//!
//! Register map:
//! - $4000-$4003: Pulse 1
//! - $4004-$4007: Pulse 2
//! - $4008-$400B: Triangle
//! - $400C-$400F: Noise
//! - $4010-$4013: DMC
//! - $4015: Status (channel enable/status)
//! - $4017: Frame counter

pub mod noise;
pub mod pulse;
pub mod triangle;

use noise::Noise;
use pulse::Pulse;
use triangle::Triangle;

use function_name::named;
use log::trace;

/// Length counter lookup table.
/// Indexed by the upper 5 bits of the length counter load register.
#[rustfmt::skip]
pub const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20,  2, 40,  4, 80,  6, 160,  8, 60, 10, 14, 12, 26, 14,
    12,  16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

/// Frame sequencer step timings (in CPU cycles).
/// Mode 0 (4-step): steps at 7457, 14913, 22371, 29829, then reset at 29830
/// Mode 1 (5-step): steps at 7457, 14913, 22371, (nothing at 29829), 37281
pub const FRAME_STEP_CYCLES: [u32; 5] = [7457, 14913, 22371, 29829, 37281];

/// NES APU state.
pub struct APU {
    // Channels
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,

    // DMC registers ($4010-$4013) - stub for now
    dmc_flags_rate: u8,
    dmc_direct_load: u8,
    dmc_sample_address: u8,
    dmc_sample_length: u8,
    dmc_enabled: bool,

    // Frame counter register ($4017)
    frame_counter: u8,

    // Internal state
    /// CPU cycle counter for frame sequencer
    cycle_count: u32,
    /// Current step in frame sequence (0-4)
    frame_step: u8,
    /// Frame IRQ flag
    frame_irq_flag: bool,
    /// DMC IRQ flag
    dmc_irq_flag: bool,
    /// Pending frame counter reset (cycles until reset, 0 = no pending)
    reset_pending: u8,

    // Audio output
    /// Sample buffer for audio output
    sample_buffer: Vec<f32>,
    /// Fractional cycle accumulator for sample generation (16.16 fixed point)
    sample_accumulator: u32,
    /// Cycles per sample in 16.16 fixed point (CPU clock / sample rate)
    /// 1789773 / 44100 ≈ 40.584 = 0x28959A in 16.16 fixed point
    cycles_per_sample_fixed: u32,
    /// High-pass filter state for DC offset removal
    hp_prev_input: f32,
    hp_prev_output: f32,
    /// Low-pass filter state for smoothing
    lp_prev_output: f32,
}

impl APU {
    pub fn new() -> Self {
        // NTSC CPU clock is 1789773 Hz
        // For 44100 Hz sample rate: 1789773 / 44100 ≈ 40.584 cycles per sample
        // In 16.16 fixed point: 40.584 * 65536 = 2660218 = 0x289B7A
        let cycles_per_sample_fixed = (1789773u64 * 65536 / 44100) as u32;

        APU {
            pulse1: Pulse::new(1),
            pulse2: Pulse::new(2),
            triangle: Triangle::new(),
            noise: Noise::new(),

            dmc_flags_rate: 0,
            dmc_direct_load: 0,
            dmc_sample_address: 0,
            dmc_sample_length: 0,
            dmc_enabled: false,

            frame_counter: 0,

            cycle_count: 0,
            frame_step: 0,
            frame_irq_flag: false,
            dmc_irq_flag: false,
            reset_pending: 0,

            sample_buffer: Vec::with_capacity(1024),
            sample_accumulator: 0,
            cycles_per_sample_fixed,
            hp_prev_input: 0.0,
            hp_prev_output: 0.0,
            lp_prev_output: 0.0,
        }
    }

    /// Read an APU register.
    /// Only $4015 is readable, others return open bus.
    #[named]
    pub fn read_register(&mut self, address: u16) -> u8 {
        trace_enter!();
        let result = match address {
            0x4015 => {
                // Status register read
                let mut status = 0u8;
                if self.pulse1.length_counter > 0 {
                    status |= 0x01;
                }
                if self.pulse2.length_counter > 0 {
                    status |= 0x02;
                }
                if self.triangle.length_counter > 0 {
                    status |= 0x04;
                }
                if self.noise.length_counter > 0 {
                    status |= 0x08;
                }
                // TODO: DMC bytes remaining (bit 4)
                if self.frame_irq_flag {
                    status |= 0x40;
                }
                if self.dmc_irq_flag {
                    status |= 0x80;
                }
                // Reading $4015 clears the frame IRQ flag
                self.frame_irq_flag = false;
                status
            }
            _ => 0,
        };
        trace_u8!(result);
        trace_exit!();
        result
    }

    /// Write to an APU register.
    #[named]
    pub fn write_register(&mut self, address: u16, value: u8) {
        trace_enter!();
        trace_u8!(value);

        match address {
            // Pulse 1 ($4000-$4003)
            0x4000 => self.pulse1.write_duty_envelope(value),
            0x4001 => self.pulse1.write_sweep(value),
            0x4002 => self.pulse1.write_timer_low(value),
            0x4003 => self.pulse1.write_length_timer_high(value, &LENGTH_TABLE),

            // Pulse 2 ($4004-$4007)
            0x4004 => self.pulse2.write_duty_envelope(value),
            0x4005 => self.pulse2.write_sweep(value),
            0x4006 => self.pulse2.write_timer_low(value),
            0x4007 => self.pulse2.write_length_timer_high(value, &LENGTH_TABLE),

            // Triangle ($4008-$400B)
            0x4008 => self.triangle.write_linear_control(value),
            0x4009 => {} // Unused
            0x400A => self.triangle.write_timer_low(value),
            0x400B => self.triangle.write_length_timer_high(value, &LENGTH_TABLE),

            // Noise ($400C-$400F)
            0x400C => self.noise.write_envelope(value),
            0x400D => {} // Unused
            0x400E => self.noise.write_period(value),
            0x400F => self.noise.write_length(value, &LENGTH_TABLE),

            // DMC ($4010-$4013)
            0x4010 => self.dmc_flags_rate = value,
            0x4011 => self.dmc_direct_load = value,
            0x4012 => self.dmc_sample_address = value,
            0x4013 => self.dmc_sample_length = value,

            // Status ($4015)
            0x4015 => {
                self.pulse1.set_enabled((value & 0x01) != 0);
                self.pulse2.set_enabled((value & 0x02) != 0);
                self.triangle.set_enabled((value & 0x04) != 0);
                self.noise.set_enabled((value & 0x08) != 0);
                self.dmc_enabled = (value & 0x10) != 0;

                // Writing to $4015 clears the DMC IRQ flag
                self.dmc_irq_flag = false;
            }

            // Frame counter ($4017)
            0x4017 => {
                self.frame_counter = value;

                // If bit 6 is set, disable frame IRQ
                if (value & 0x40) != 0 {
                    self.frame_irq_flag = false;
                }

                // Frame counter reset happens after 3-4 cycles
                // Using 4 cycles as a reasonable approximation
                self.reset_pending = 4;

                // If mode 1 (bit 7 set), clock all units immediately
                // (this happens on the actual reset, handled in tick)
            }

            _ => {}
        }
        trace_exit!();
    }

    /// Tick the APU. Called once per CPU cycle.
    pub fn tick(&mut self) {
        // Handle pending frame counter reset
        if self.reset_pending > 0 {
            self.reset_pending -= 1;
            if self.reset_pending == 0 {
                self.cycle_count = 0;
                self.frame_step = 0;

                // If mode 1 (bit 7 set), clock all units immediately on reset
                if (self.frame_counter & 0x80) != 0 {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                }
            }
        }

        // Increment cycle counter
        self.cycle_count += 1;

        // Triangle timer runs at CPU rate
        self.triangle.tick_timer();

        // Pulse and noise timers run at half CPU rate (every 2 cycles)
        if self.cycle_count % 2 == 0 {
            self.pulse1.tick_timer();
            self.pulse2.tick_timer();
            self.noise.tick_timer();
        }

        // Frame sequencer
        let mode = (self.frame_counter & 0x80) != 0;
        let irq_inhibit = (self.frame_counter & 0x40) != 0;

        match self.frame_step {
            0 => {
                if self.cycle_count == FRAME_STEP_CYCLES[0] {
                    self.clock_quarter_frame();
                    self.frame_step = 1;
                }
            }
            1 => {
                if self.cycle_count == FRAME_STEP_CYCLES[1] {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                    self.frame_step = 2;
                }
            }
            2 => {
                if self.cycle_count == FRAME_STEP_CYCLES[2] {
                    self.clock_quarter_frame();
                    self.frame_step = 3;
                }
            }
            3 => {
                // In Mode 0, IRQ flag is set at cycle 29828 (1 cycle before length clock)
                if !mode && !irq_inhibit && self.cycle_count == FRAME_STEP_CYCLES[3] - 1 {
                    self.frame_irq_flag = true;
                }

                if self.cycle_count == FRAME_STEP_CYCLES[3] {
                    if !mode {
                        // Mode 0: 4-step sequence - length counters clock at 29829
                        self.clock_quarter_frame();
                        self.clock_half_frame();
                        // IRQ flag also set again at 29829 and 29830
                        if !irq_inhibit {
                            self.frame_irq_flag = true;
                        }
                        // Move to reset state (reset happens 1 cycle later at 29830)
                        self.frame_step = 5;
                    } else {
                        // Mode 1: continue to step 4
                        self.frame_step = 4;
                    }
                }
            }
            5 => {
                // Mode 0: reset happens 1 cycle after step 4 (at cycle 29830)
                // IRQ flag is also set again at 29830
                if !irq_inhibit {
                    self.frame_irq_flag = true;
                }
                self.cycle_count = 0;
                self.frame_step = 0;
            }
            4 => {
                if self.cycle_count == FRAME_STEP_CYCLES[4] {
                    // Mode 1: 5-step sequence - clock at 37281
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                    // Move to reset state (reset happens 1 cycle later at 37282)
                    self.frame_step = 6;
                }
            }
            6 => {
                // Mode 1: reset happens 1 cycle after step 5 (at cycle 37282)
                self.cycle_count = 0;
                self.frame_step = 0;
            }
            _ => {}
        }

        // Generate audio sample using fixed-point accumulator
        // Add one cycle (1.0 in 16.16 fixed point = 65536)
        self.sample_accumulator += 65536;
        while self.sample_accumulator >= self.cycles_per_sample_fixed {
            self.sample_accumulator -= self.cycles_per_sample_fixed;
            let raw_sample = self.mix_output();

            // Apply high-pass filter to remove DC offset (reduces clicks/pops)
            // First-order high-pass: y[n] = alpha * (y[n-1] + x[n] - x[n-1])
            // alpha = 0.996 gives ~90Hz cutoff at 44100Hz sample rate
            const HP_ALPHA: f32 = 0.996;
            let hp_out = HP_ALPHA * (self.hp_prev_output + raw_sample - self.hp_prev_input);
            self.hp_prev_input = raw_sample;
            self.hp_prev_output = hp_out;

            // Apply low-pass filter to smooth high frequencies (reduces aliasing artifacts)
            // First-order low-pass: y[n] = alpha * x[n] + (1 - alpha) * y[n-1]
            // alpha = 0.5 gives ~14kHz cutoff at 44100Hz sample rate
            const LP_ALPHA: f32 = 0.5;
            let lp_out = LP_ALPHA * hp_out + (1.0 - LP_ALPHA) * self.lp_prev_output;
            self.lp_prev_output = lp_out;

            self.sample_buffer.push(lp_out.clamp(-1.0, 1.0));
        }
    }

    /// Quarter-frame clock: envelope and triangle linear counter.
    fn clock_quarter_frame(&mut self) {
        self.pulse1.clock_envelope();
        self.pulse2.clock_envelope();
        self.triangle.clock_linear_counter();
        self.noise.clock_envelope();
    }

    /// Half-frame clock: length counters and sweep units.
    fn clock_half_frame(&mut self) {
        self.pulse1.clock_length_counter();
        self.pulse1.clock_sweep();
        self.pulse2.clock_length_counter();
        self.pulse2.clock_sweep();
        self.triangle.clock_length_counter();
        self.noise.clock_length_counter();
    }

    /// Mix channel outputs into a single audio sample.
    fn mix_output(&self) -> f32 {
        let pulse1 = self.pulse1.output() as f32;
        let pulse2 = self.pulse2.output() as f32;
        let triangle = self.triangle.output() as f32;
        let noise = self.noise.output() as f32;
        let dmc = self.dmc_direct_load as f32; // Stub: just use direct load value

        // NES APU mixer formulas (from nesdev wiki)
        // These produce a value roughly in the range 0.0 to 1.0

        let pulse_out = if pulse1 + pulse2 > 0.0 {
            95.88 / ((8128.0 / (pulse1 + pulse2)) + 100.0)
        } else {
            0.0
        };

        let tnd_sum = triangle / 8227.0 + noise / 12241.0 + dmc / 22638.0;
        let tnd_out = if tnd_sum > 0.0 {
            159.79 / ((1.0 / tnd_sum) + 100.0)
        } else {
            0.0
        };

        // Output 0.0-1.0 range directly - the high-pass filter will remove DC offset
        // and center the signal around 0. This avoids the -1.0 DC offset from silence.
        // Scale by 2.0 to use more of the output range (max output is ~0.5)
        (pulse_out + tnd_out) * 2.0
    }

    /// Take the accumulated audio samples and clear the buffer.
    pub fn take_samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.sample_buffer)
    }

    /// Get a reference to the sample buffer without clearing it.
    pub fn samples(&self) -> &[f32] {
        &self.sample_buffer
    }

    /// Check if APU is requesting an IRQ.
    pub fn irq_pending(&self) -> bool {
        self.frame_irq_flag || self.dmc_irq_flag
    }
}

impl Default for APU {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apu_new() {
        let apu = APU::new();
        assert!(!apu.pulse1.enabled);
        assert!(!apu.pulse2.enabled);
        assert!(!apu.triangle.enabled);
        assert!(!apu.noise.enabled);
        assert!(!apu.dmc_enabled);
    }

    #[test]
    fn test_status_register_write() {
        let mut apu = APU::new();

        // Enable all channels
        apu.write_register(0x4015, 0x1F);
        assert!(apu.pulse1.enabled);
        assert!(apu.pulse2.enabled);
        assert!(apu.triangle.enabled);
        assert!(apu.noise.enabled);
        assert!(apu.dmc_enabled);

        // Disable all channels
        apu.write_register(0x4015, 0x00);
        assert!(!apu.pulse1.enabled);
        assert!(!apu.pulse2.enabled);
        assert!(!apu.triangle.enabled);
        assert!(!apu.noise.enabled);
        assert!(!apu.dmc_enabled);
    }

    #[test]
    fn test_length_counter_load() {
        let mut apu = APU::new();

        // Enable pulse 1
        apu.write_register(0x4015, 0x01);

        // Write to $4003 with length index 0 (value 10)
        apu.write_register(0x4003, 0x00);
        assert_eq!(apu.pulse1.length_counter, 10);

        // Write with length index 1 (value 254)
        apu.write_register(0x4003, 0x08);
        assert_eq!(apu.pulse1.length_counter, 254);
    }

    #[test]
    fn test_length_counter_disabled_channel() {
        let mut apu = APU::new();

        // Don't enable pulse 1
        // Write to $4003 should not load length counter
        apu.write_register(0x4003, 0x08);
        assert_eq!(apu.pulse1.length_counter, 0);
    }

    #[test]
    fn test_status_read() {
        let mut apu = APU::new();

        // Enable channels and load length counters
        apu.write_register(0x4015, 0x0F);
        apu.write_register(0x4003, 0x08); // Pulse 1
        apu.write_register(0x4007, 0x08); // Pulse 2

        let status = apu.read_register(0x4015);
        assert_eq!(status & 0x03, 0x03); // Both pulse channels active
    }

    #[test]
    fn test_frame_irq_cleared_on_status_read() {
        let mut apu = APU::new();
        apu.frame_irq_flag = true;

        let status = apu.read_register(0x4015);
        assert_eq!(status & 0x40, 0x40); // IRQ flag was set in read
        assert!(!apu.frame_irq_flag); // But now cleared
    }

    #[test]
    fn test_pulse_output() {
        let mut apu = APU::new();

        // Enable pulse 1
        apu.write_register(0x4015, 0x01);

        // Set duty cycle, constant volume at max (15)
        apu.write_register(0x4000, 0x3F); // 12.5% duty, constant volume, vol=15

        // Set timer period
        apu.write_register(0x4002, 0x00); // Timer low
        apu.write_register(0x4003, 0x08); // Length counter load + timer high

        // The pulse should now be producing output
        // Tick enough times to ensure duty cycle advances
        for _ in 0..1000 {
            apu.tick();
        }

        // Check that we have samples
        assert!(!apu.sample_buffer.is_empty());
    }
}
