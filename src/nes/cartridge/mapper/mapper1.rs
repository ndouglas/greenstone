//! MMC1 (Mapper 1) implementation.
//!
//! Used by many games including Tetris, Legend of Zelda, Metroid, etc.
//!
//! Features:
//! - PRG ROM bank switching (16KB or 32KB modes)
//! - CHR ROM/RAM bank switching (4KB or 8KB modes)
//! - Mirroring control
//! - PRG RAM with optional write protection

use super::super::{Data, MirroringMode, Page, PageSize};
use crate::traits::Mappable;

use Page::*;
use PageSize::*;

/// MMC1 mapper state.
pub struct Mapper1 {
    data: Data,

    // Shift register for serial writes
    shift_register: u8,
    shift_count: u8,

    // Internal registers (set via serial interface)
    /// Control register: mirroring, PRG mode, CHR mode
    control: u8,
    /// CHR bank 0 (or full 8KB bank in 8KB mode)
    chr_bank_0: u8,
    /// CHR bank 1 (upper 4KB in 4KB mode)
    chr_bank_1: u8,
    /// PRG bank select + PRG RAM enable
    prg_bank: u8,
}

impl Mapper1 {
    pub fn new(data: Data) -> Mapper1 {
        Mapper1 {
            data,
            shift_register: 0,
            shift_count: 0,
            // Control defaults to 0x0C: PRG mode 3 (fix last bank at $C000)
            control: 0x0C,
            chr_bank_0: 0,
            chr_bank_1: 0,
            prg_bank: 0,
        }
    }

    /// Get mirroring mode from control register bits 0-1.
    fn get_mirroring(&self) -> MirroringMode {
        match self.control & 0x03 {
            0 => MirroringMode::SingleScreenLower,
            1 => MirroringMode::SingleScreenUpper,
            2 => MirroringMode::Vertical,
            3 => MirroringMode::Horizontal,
            _ => unreachable!(),
        }
    }

    /// Get PRG bank mode from control register bits 2-3.
    fn prg_mode(&self) -> u8 {
        (self.control >> 2) & 0x03
    }

    /// Get CHR bank mode from control register bit 4.
    /// false = 8KB mode, true = 4KB mode
    fn chr_mode_4kb(&self) -> bool {
        (self.control & 0x10) != 0
    }

    /// Handle write to mapper registers ($8000-$FFFF).
    fn write_register(&mut self, address: u16, value: u8) {
        // If bit 7 is set, reset the shift register
        if (value & 0x80) != 0 {
            self.shift_register = 0;
            self.shift_count = 0;
            // Also set PRG mode to 3 (fix last bank)
            self.control |= 0x0C;
            return;
        }

        // Shift in bit 0 of the value
        self.shift_register |= (value & 0x01) << self.shift_count;
        self.shift_count += 1;

        // After 5 writes, transfer to internal register
        if self.shift_count == 5 {
            let reg_value = self.shift_register;

            // Determine which register based on address
            match address {
                0x8000..=0x9FFF => {
                    // Control register
                    self.control = reg_value;
                }
                0xA000..=0xBFFF => {
                    // CHR bank 0
                    self.chr_bank_0 = reg_value;
                }
                0xC000..=0xDFFF => {
                    // CHR bank 1
                    self.chr_bank_1 = reg_value;
                }
                0xE000..=0xFFFF => {
                    // PRG bank
                    self.prg_bank = reg_value;
                }
                _ => {}
            }

            // Reset shift register
            self.shift_register = 0;
            self.shift_count = 0;
        }
    }

    /// Get the PRG ROM bank number for the given address.
    fn get_prg_bank(&self, address: u16) -> usize {
        let prg_bank = (self.prg_bank & 0x0F) as usize;
        let bank_count = self.data.prg_rom.get_page_count(SixteenKb);

        match self.prg_mode() {
            0 | 1 => {
                // 32KB mode: switch 32KB at $8000, ignore low bit of bank number
                let bank_32k = (prg_bank >> 1) * 2;
                if address < 0xC000 {
                    bank_32k % bank_count
                } else {
                    (bank_32k + 1) % bank_count
                }
            }
            2 => {
                // Fix first bank at $8000, switch 16KB bank at $C000
                if address < 0xC000 {
                    0
                } else {
                    prg_bank % bank_count
                }
            }
            3 => {
                // Fix last bank at $C000, switch 16KB bank at $8000
                if address < 0xC000 {
                    prg_bank % bank_count
                } else {
                    bank_count - 1
                }
            }
            _ => unreachable!(),
        }
    }

    /// Get the CHR bank number for the given address.
    fn get_chr_bank(&self, address: u16) -> usize {
        if self.chr_mode_4kb() {
            // 4KB mode
            if address < 0x1000 {
                self.chr_bank_0 as usize
            } else {
                self.chr_bank_1 as usize
            }
        } else {
            // 8KB mode: use chr_bank_0 with bit 0 cleared for lower, set for upper
            let bank_8k = (self.chr_bank_0 & 0x1E) as usize;
            if address < 0x1000 {
                bank_8k
            } else {
                bank_8k + 1
            }
        }
    }

    /// Check if PRG RAM is enabled.
    fn prg_ram_enabled(&self) -> bool {
        // Bit 4 of prg_bank: 0 = enabled, 1 = disabled
        (self.prg_bank & 0x10) == 0
    }
}

impl Mappable for Mapper1 {
    #[named]
    fn read_prg_u8(&self, address: u16) -> u8 {
        trace_enter!();
        trace_u16!(address);
        let result = match address {
            0x4020..=0x5FFF => {
                // Expansion ROM area - open bus
                0x00
            }
            0x6000..=0x7FFF => {
                // PRG RAM
                if self.prg_ram_enabled() {
                    self.data.prg_ram.read_u8(First(EightKb), address - 0x6000)
                } else {
                    0x00 // Open bus when disabled
                }
            }
            0x8000..=0xFFFF => {
                // PRG ROM
                let bank = self.get_prg_bank(address);
                let offset = if address < 0xC000 {
                    address - 0x8000
                } else {
                    address - 0xC000
                };
                self.data.prg_rom.read_u8(Number(bank, SixteenKb), offset)
            }
            _ => panic!("bad address: {}", format_u16!(address)),
        };
        trace_u8!(result);
        trace_exit!();
        result
    }

    #[named]
    fn write_prg_u8(&mut self, address: u16, value: u8) {
        trace_enter!();
        trace_u16!(address);
        trace_u8!(value);
        match address {
            0x4020..=0x5FFF => {
                // Expansion ROM area - ignore writes
            }
            0x6000..=0x7FFF => {
                // PRG RAM
                if self.prg_ram_enabled() {
                    self.data.prg_ram.write_u8(First(EightKb), address - 0x6000, value);
                }
            }
            0x8000..=0xFFFF => {
                // Mapper register write (serial interface)
                self.write_register(address, value);
            }
            _ => panic!("bad address: {}", format_u16!(address)),
        }
        trace_exit!();
    }

    #[named]
    fn read_chr_u8(&self, address: u16) -> u8 {
        trace_enter!();
        trace_u16!(address);
        let result = if self.data.header.chr_rom_pages == 0 {
            // CHR RAM
            self.data.chr_ram.read_u8(First(EightKb), address)
        } else {
            // CHR ROM with banking
            let bank = self.get_chr_bank(address);
            let offset = address & 0x0FFF; // 4KB offset within bank
            // Check if we have enough banks
            let bank_count = self.data.chr_rom.get_page_count(FourKb);
            let actual_bank = bank % bank_count;
            self.data.chr_rom.read_u8(Number(actual_bank, FourKb), offset)
        };
        trace_u8!(result);
        trace_exit!();
        result
    }

    #[named]
    fn write_chr_u8(&mut self, address: u16, value: u8) {
        trace_enter!();
        trace_u16!(address);
        trace_u8!(value);
        if self.data.header.chr_rom_pages == 0 {
            // CHR RAM - writable
            self.data.chr_ram.write_u8(First(EightKb), address, value);
        }
        // CHR ROM is not writable
        trace_exit!();
    }

    #[named]
    fn get_mirroring_mode(&self) -> MirroringMode {
        trace_enter!();
        let result = self.get_mirroring();
        trace_var!(result);
        trace_exit!();
        result
    }
}
