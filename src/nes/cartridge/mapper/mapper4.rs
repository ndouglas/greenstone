/// Mapper 4 (MMC3 / TxROM)
///
/// The most widely-used NES mapper, found in ~24% of all licensed NES games.
///
/// Features:
/// - PRG ROM: Two switchable 8KB banks + two fixed 8KB banks
/// - CHR ROM/RAM: Six switchable banks (2x 2KB + 4x 1KB, or inverted)
/// - Mirroring control (horizontal/vertical)
/// - PRG RAM (8KB at $6000-$7FFF) with optional write protection
/// - Scanline counter with IRQ generation
///
/// Games: Super Mario Bros 2/3, Mega Man 3-6, Kirby's Adventure,
///        Batman, Double Dragon II/III, Ninja Gaiden II/III, etc.
use super::super::{Data, MirroringMode, Page, PageSize};
use crate::traits::Mappable;

use Page::*;
use PageSize::*;

#[derive(Serialize, Deserialize)]
pub struct Mapper4 {
  data: Data,

  // Bank select register ($8000-$9FFE, even)
  /// Which bank register to update (0-7) on the next write to $8001.
  bank_select: u8,
  /// PRG ROM bank mode: false = $8000 switchable/$C000 fixed,
  ///                     true  = $C000 switchable/$8000 fixed.
  prg_bank_mode: bool,
  /// CHR A12 inversion: false = 2KB banks at $0000, 1KB at $1000,
  ///                     true  = 2KB banks at $1000, 1KB at $0000.
  chr_inversion: bool,

  // Bank registers (written via $8001)
  /// R0-R7 bank register values.
  bank_registers: [u8; 8],

  // Mirroring ($A000-$BFFE, even)
  mirroring_mode: MirroringMode,

  // PRG RAM protect ($A001-$BFFF, odd) — simplified
  prg_ram_enabled: bool,

  // Scanline counter
  /// IRQ latch value (reload value for counter).
  irq_latch: u8,
  /// IRQ counter (counts down on each scanline signal).
  irq_counter: u8,
  /// Whether the counter should be reloaded on the next signal.
  irq_reload: bool,
  /// Whether IRQ generation is enabled.
  irq_enabled: bool,
  /// Whether an IRQ is currently pending.
  irq_pending: bool,
}

impl Mapper4 {
  pub fn new(data: Data) -> Mapper4 {
    let mirroring_mode = data.header.mirroring_mode;
    Mapper4 {
      data,
      bank_select: 0,
      prg_bank_mode: false,
      chr_inversion: false,
      bank_registers: [0; 8],
      mirroring_mode,
      prg_ram_enabled: true,
      irq_latch: 0,
      irq_counter: 0,
      irq_reload: false,
      irq_enabled: false,
      irq_pending: false,
    }
  }

  /// Get the 8KB PRG bank number for the given CPU address.
  fn get_prg_bank(&self, address: u16) -> usize {
    let bank_count = self.data.prg_rom.get_page_count(EightKb);
    let second_last = bank_count - 2;
    let last = bank_count - 1;

    let bank = match address {
      0x8000..=0x9FFF => {
        if self.prg_bank_mode {
          second_last // Fixed to second-last bank
        } else {
          (self.bank_registers[6] & 0x3F) as usize // Switchable (R6)
        }
      }
      0xA000..=0xBFFF => (self.bank_registers[7] & 0x3F) as usize, // Always switchable (R7)
      0xC000..=0xDFFF => {
        if self.prg_bank_mode {
          (self.bank_registers[6] & 0x3F) as usize // Switchable (R6)
        } else {
          second_last // Fixed to second-last bank
        }
      }
      0xE000..=0xFFFF => last, // Always fixed to last bank
      _ => unreachable!(),
    };

    bank % bank_count
  }

  /// Get the 1KB CHR bank number for the given PPU address (0x0000-0x1FFF).
  fn get_chr_bank(&self, address: u16) -> usize {
    // The 8KB CHR space is divided into 8 x 1KB slots.
    // R0 and R1 select 2KB banks (lower bit ignored), R2-R5 select 1KB banks.
    //
    // Without CHR inversion (chr_inversion = false):
    //   $0000-$07FF: R0 (2KB) -> 1KB slots 0,1
    //   $0800-$0FFF: R1 (2KB) -> 1KB slots 2,3
    //   $1000-$13FF: R2 (1KB) -> 1KB slot 4
    //   $1400-$17FF: R3 (1KB) -> 1KB slot 5
    //   $1800-$1BFF: R4 (1KB) -> 1KB slot 6
    //   $1C00-$1FFF: R5 (1KB) -> 1KB slot 7
    //
    // With CHR inversion (chr_inversion = true):
    //   $0000-$03FF: R2 (1KB) -> 1KB slot 0
    //   $0400-$07FF: R3 (1KB) -> 1KB slot 1
    //   $0800-$0BFF: R4 (1KB) -> 1KB slot 2
    //   $0C00-$0FFF: R5 (1KB) -> 1KB slot 3
    //   $1000-$17FF: R0 (2KB) -> 1KB slots 4,5
    //   $1800-$1FFF: R1 (2KB) -> 1KB slots 6,7

    let slot = (address >> 10) as usize; // 0-7 (1KB slot index)

    let bank = if !self.chr_inversion {
      match slot {
        0 => (self.bank_registers[0] & 0xFE) as usize,     // R0, low 1KB
        1 => (self.bank_registers[0] & 0xFE) as usize + 1, // R0, high 1KB
        2 => (self.bank_registers[1] & 0xFE) as usize,     // R1, low 1KB
        3 => (self.bank_registers[1] & 0xFE) as usize + 1, // R1, high 1KB
        4 => self.bank_registers[2] as usize,              // R2
        5 => self.bank_registers[3] as usize,              // R3
        6 => self.bank_registers[4] as usize,              // R4
        7 => self.bank_registers[5] as usize,              // R5
        _ => unreachable!(),
      }
    } else {
      match slot {
        0 => self.bank_registers[2] as usize,              // R2
        1 => self.bank_registers[3] as usize,              // R3
        2 => self.bank_registers[4] as usize,              // R4
        3 => self.bank_registers[5] as usize,              // R5
        4 => (self.bank_registers[0] & 0xFE) as usize,     // R0, low 1KB
        5 => (self.bank_registers[0] & 0xFE) as usize + 1, // R0, high 1KB
        6 => (self.bank_registers[1] & 0xFE) as usize,     // R1, low 1KB
        7 => (self.bank_registers[1] & 0xFE) as usize + 1, // R1, high 1KB
        _ => unreachable!(),
      }
    };

    if self.data.header.chr_rom_pages > 0 {
      let bank_count = self.data.chr_rom.get_page_count(OneKb);
      bank % bank_count
    } else {
      bank % 8 // CHR RAM: 8KB = 8 x 1KB
    }
  }
}

impl Mappable for Mapper4 {
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
        if self.prg_ram_enabled {
          self.data.prg_ram.read_u8(First(EightKb), address - 0x6000)
        } else {
          0x00 // Open bus when disabled
        }
      }
      0x8000..=0xFFFF => {
        // PRG ROM with bank switching
        let bank = self.get_prg_bank(address);
        let offset = address & 0x1FFF; // 8KB offset within bank
        self.data.prg_rom.read_u8(Number(bank, EightKb), offset)
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
        // Expansion ROM area - ignore
      }
      0x6000..=0x7FFF => {
        // PRG RAM
        if self.prg_ram_enabled {
          self.data.prg_ram.write_u8(First(EightKb), address - 0x6000, value);
        }
      }
      0x8000..=0x9FFF => {
        if address & 0x0001 == 0 {
          // Bank select ($8000-$9FFE, even)
          self.bank_select = value & 0x07;
          self.prg_bank_mode = (value & 0x40) != 0;
          self.chr_inversion = (value & 0x80) != 0;
        } else {
          // Bank data ($8001-$9FFF, odd)
          self.bank_registers[self.bank_select as usize] = value;
        }
      }
      0xA000..=0xBFFF => {
        if address & 0x0001 == 0 {
          // Mirroring ($A000-$BFFE, even)
          self.mirroring_mode = if (value & 0x01) == 0 {
            MirroringMode::Vertical
          } else {
            MirroringMode::Horizontal
          };
        } else {
          // PRG RAM protect ($A001-$BFFF, odd)
          self.prg_ram_enabled = (value & 0x80) != 0;
          // Bit 6 = write protection (not fully emulated)
        }
      }
      0xC000..=0xDFFF => {
        if address & 0x0001 == 0 {
          // IRQ latch ($C000-$DFFE, even)
          self.irq_latch = value;
        } else {
          // IRQ reload ($C001-$DFFF, odd)
          self.irq_counter = 0;
          self.irq_reload = true;
        }
      }
      0xE000..=0xFFFF => {
        if address & 0x0001 == 0 {
          // IRQ disable ($E000-$FFFE, even)
          self.irq_enabled = false;
          self.irq_pending = false; // Acknowledge any pending IRQ
        } else {
          // IRQ enable ($E001-$FFFF, odd)
          self.irq_enabled = true;
        }
      }
      _ => panic!("bad address: {}", format_u16!(address)),
    }
    trace_exit!();
  }

  #[named]
  fn read_chr_u8(&self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    let bank = self.get_chr_bank(address);
    let offset = address & 0x03FF; // 1KB offset within bank
    let result = if self.data.header.chr_rom_pages == 0 {
      // CHR RAM
      self.data.chr_ram.read_u8(Number(bank, OneKb), offset)
    } else {
      // CHR ROM
      self.data.chr_rom.read_u8(Number(bank, OneKb), offset)
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
      // CHR RAM is writable
      let bank = self.get_chr_bank(address);
      let offset = address & 0x03FF;
      self.data.chr_ram.write_u8(Number(bank, OneKb), offset, value);
    }
    // CHR ROM is not writable
    trace_exit!();
  }

  #[named]
  fn signal_scanline(&mut self) {
    trace_enter!();
    if self.irq_counter == 0 || self.irq_reload {
      self.irq_counter = self.irq_latch;
      self.irq_reload = false;
    } else {
      self.irq_counter -= 1;
    }

    if self.irq_counter == 0 && self.irq_enabled {
      self.irq_pending = true;
    }
    trace_exit!();
  }

  #[named]
  fn get_mirroring_mode(&self) -> MirroringMode {
    trace_enter!();
    let result = self.mirroring_mode;
    trace_var!(result);
    trace_exit!();
    result
  }

  #[named]
  fn get_irq_flag(&self) -> bool {
    trace_enter!();
    let result = self.irq_pending;
    trace_var!(result);
    trace_exit!();
    result
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::nes::cartridge::Data;
  use crate::test::init;

  /// Build a Mapper4 cartridge with the given number of 8KB PRG banks
  /// and 1KB CHR banks.
  fn build_mapper4(prg_banks_8k: u8, chr_banks_1k: u8) -> Mapper4 {
    // iNES header counts PRG in 16KB pages and CHR in 8KB pages
    let prg_pages_16k = prg_banks_8k / 2;
    let chr_pages_8k = chr_banks_1k / 8;
    let mut data = vec![
      0x4e,
      0x45,
      0x53,
      0x1a,          // NES magic
      prg_pages_16k, // PRG ROM pages (16KB each)
      chr_pages_8k,  // CHR ROM pages (8KB each)
      0x40,          // Flags 6: mapper low nibble = 4
      0x00,          // Flags 7: mapper high nibble = 0
      0x00,
      0x00,
      0x00,
      0x00,
      0x00,
      0x00,
      0x00,
      0x00,
    ];

    // Fill PRG ROM - each 8KB bank starts with its bank number
    for bank in 0..prg_banks_8k {
      for i in 0..0x2000usize {
        if i < 4 {
          data.push(bank);
        } else {
          data.push((i & 0xFF) as u8);
        }
      }
    }

    // Fill CHR ROM - each 1KB bank starts with its bank number
    for bank in 0..chr_banks_1k {
      for i in 0..0x400usize {
        if i < 4 {
          data.push(bank);
        } else {
          data.push((i & 0xFF) as u8);
        }
      }
    }

    let cart_data = Data::new(&data);
    Mapper4::new(cart_data)
  }

  // ---- PRG banking tests ----

  #[test]
  fn test_initial_prg_layout() {
    init();
    let mapper = build_mapper4(16, 32); // 16 x 8KB PRG, 32 x 1KB CHR

    // Default: R6=0, R7=0, prg_bank_mode=false
    // $8000-$9FFF = R6 (bank 0)
    assert_eq!(mapper.read_prg_u8(0x8000), 0);
    // $A000-$BFFF = R7 (bank 0)
    assert_eq!(mapper.read_prg_u8(0xA000), 0);
    // $C000-$DFFF = second-to-last bank (bank 14)
    assert_eq!(mapper.read_prg_u8(0xC000), 14);
    // $E000-$FFFF = last bank (bank 15)
    assert_eq!(mapper.read_prg_u8(0xE000), 15);
  }

  #[test]
  fn test_prg_bank_switching() {
    init();
    let mut mapper = build_mapper4(16, 32);

    // Select R6 register
    mapper.write_prg_u8(0x8000, 6);
    // Set R6 = 5
    mapper.write_prg_u8(0x8001, 5);

    // $8000 should now read from bank 5
    assert_eq!(mapper.read_prg_u8(0x8000), 5);

    // Select R7 register
    mapper.write_prg_u8(0x8000, 7);
    // Set R7 = 3
    mapper.write_prg_u8(0x8001, 3);

    // $A000 should now read from bank 3
    assert_eq!(mapper.read_prg_u8(0xA000), 3);

    // Fixed banks should be unchanged
    assert_eq!(mapper.read_prg_u8(0xC000), 14);
    assert_eq!(mapper.read_prg_u8(0xE000), 15);
  }

  #[test]
  fn test_prg_bank_mode_swap() {
    init();
    let mut mapper = build_mapper4(16, 32);

    // Set R6 = 5
    mapper.write_prg_u8(0x8000, 6);
    mapper.write_prg_u8(0x8001, 5);

    // In mode 0: $8000=R6(5), $C000=fixed(14)
    assert_eq!(mapper.read_prg_u8(0x8000), 5);
    assert_eq!(mapper.read_prg_u8(0xC000), 14);

    // Switch to mode 1 (bit 6 set): $8000=fixed(14), $C000=R6(5)
    mapper.write_prg_u8(0x8000, 0x46); // bank_select=6, prg_bank_mode=true
    assert_eq!(mapper.read_prg_u8(0x8000), 14);
    assert_eq!(mapper.read_prg_u8(0xC000), 5);

    // $E000 always fixed to last
    assert_eq!(mapper.read_prg_u8(0xE000), 15);
  }

  // ---- CHR banking tests ----

  #[test]
  fn test_initial_chr_layout() {
    init();
    let mapper = build_mapper4(4, 32);

    // Default: all bank registers = 0, chr_inversion = false
    // $0000-$07FF = R0 (2KB bank 0) -> 1KB banks 0,1
    assert_eq!(mapper.read_chr_u8(0x0000), 0); // slot 0 = bank 0
    assert_eq!(mapper.read_chr_u8(0x0400), 1); // slot 1 = bank 1
  }

  #[test]
  fn test_chr_2kb_bank_switching() {
    init();
    let mut mapper = build_mapper4(4, 32);

    // Select R0 and set to bank 4 (2KB, so bit 0 ignored -> maps to 1KB banks 4,5)
    mapper.write_prg_u8(0x8000, 0); // select R0
    mapper.write_prg_u8(0x8001, 4); // R0 = 4

    // $0000-$03FF = 1KB bank 4
    assert_eq!(mapper.read_chr_u8(0x0000), 4);
    // $0400-$07FF = 1KB bank 5
    assert_eq!(mapper.read_chr_u8(0x0400), 5);
  }

  #[test]
  fn test_chr_1kb_bank_switching() {
    init();
    let mut mapper = build_mapper4(4, 32);

    // Select R2 and set to bank 10
    mapper.write_prg_u8(0x8000, 2); // select R2
    mapper.write_prg_u8(0x8001, 10); // R2 = 10

    // Without inversion, R2 maps to $1000-$13FF
    assert_eq!(mapper.read_chr_u8(0x1000), 10);
  }

  #[test]
  fn test_chr_inversion() {
    init();
    let mut mapper = build_mapper4(4, 32);

    // Set R0 = 4, R2 = 10
    mapper.write_prg_u8(0x8000, 0);
    mapper.write_prg_u8(0x8001, 4); // R0 = 4
    mapper.write_prg_u8(0x8000, 2);
    mapper.write_prg_u8(0x8001, 10); // R2 = 10

    // Without inversion: $0000 = R0(4), $1000 = R2(10)
    assert_eq!(mapper.read_chr_u8(0x0000), 4);
    assert_eq!(mapper.read_chr_u8(0x1000), 10);

    // Enable CHR inversion (bit 7 set)
    mapper.write_prg_u8(0x8000, 0x80); // chr_inversion=true, select R0

    // With inversion: $0000 = R2(10), $1000 = R0(4)
    assert_eq!(mapper.read_chr_u8(0x0000), 10);
    assert_eq!(mapper.read_chr_u8(0x1000), 4);
  }

  // ---- Mirroring tests ----

  #[test]
  fn test_mirroring_control() {
    init();
    let mut mapper = build_mapper4(4, 32);

    // Set vertical mirroring
    mapper.write_prg_u8(0xA000, 0x00);
    assert_eq!(mapper.get_mirroring_mode(), MirroringMode::Vertical);

    // Set horizontal mirroring
    mapper.write_prg_u8(0xA000, 0x01);
    assert_eq!(mapper.get_mirroring_mode(), MirroringMode::Horizontal);
  }

  // ---- PRG RAM tests ----

  #[test]
  fn test_prg_ram() {
    init();
    let mut mapper = build_mapper4(4, 32);

    // PRG RAM enabled by default
    mapper.write_prg_u8(0x6000, 0x42);
    assert_eq!(mapper.read_prg_u8(0x6000), 0x42);

    // Disable PRG RAM (clear bit 7 of $A001)
    mapper.write_prg_u8(0xA001, 0x00);
    assert_eq!(mapper.read_prg_u8(0x6000), 0x00); // Open bus

    // Re-enable PRG RAM
    mapper.write_prg_u8(0xA001, 0x80);
    // Value should still be there
    assert_eq!(mapper.read_prg_u8(0x6000), 0x42);
  }

  // ---- IRQ / scanline counter tests ----

  #[test]
  fn test_irq_basic() {
    init();
    let mut mapper = build_mapper4(4, 32);

    // No IRQ initially
    assert!(!mapper.get_irq_flag());

    // Set latch to 3, enable IRQs
    mapper.write_prg_u8(0xC000, 3); // IRQ latch = 3
    mapper.write_prg_u8(0xC001, 0); // IRQ reload
    mapper.write_prg_u8(0xE001, 0); // IRQ enable

    // Counter loaded to 3 on first signal
    mapper.signal_scanline(); // counter: 0 -> 3 (reload)
    assert!(!mapper.get_irq_flag());

    mapper.signal_scanline(); // counter: 3 -> 2
    assert!(!mapper.get_irq_flag());

    mapper.signal_scanline(); // counter: 2 -> 1
    assert!(!mapper.get_irq_flag());

    mapper.signal_scanline(); // counter: 1 -> 0, IRQ fires!
    assert!(mapper.get_irq_flag());
  }

  #[test]
  fn test_irq_disable_clears_pending() {
    init();
    let mut mapper = build_mapper4(4, 32);

    // Fire an IRQ
    mapper.write_prg_u8(0xC000, 1); // latch = 1
    mapper.write_prg_u8(0xC001, 0); // reload
    mapper.write_prg_u8(0xE001, 0); // enable

    mapper.signal_scanline(); // counter: 0 -> 1 (reload)
    mapper.signal_scanline(); // counter: 1 -> 0, IRQ fires
    assert!(mapper.get_irq_flag());

    // Disable IRQ — should acknowledge/clear pending
    mapper.write_prg_u8(0xE000, 0);
    assert!(!mapper.get_irq_flag());
  }

  #[test]
  fn test_irq_not_fired_when_disabled() {
    init();
    let mut mapper = build_mapper4(4, 32);

    // Set up counter but don't enable IRQs
    mapper.write_prg_u8(0xC000, 1); // latch = 1
    mapper.write_prg_u8(0xC001, 0); // reload

    mapper.signal_scanline(); // reload
    mapper.signal_scanline(); // counter hits 0, but IRQ disabled
    assert!(!mapper.get_irq_flag());
  }

  #[test]
  fn test_irq_counter_reload() {
    init();
    let mut mapper = build_mapper4(4, 32);

    // Set latch to 2, enable
    mapper.write_prg_u8(0xC000, 2);
    mapper.write_prg_u8(0xC001, 0);
    mapper.write_prg_u8(0xE001, 0);

    mapper.signal_scanline(); // reload -> 2
    mapper.signal_scanline(); // 2 -> 1
    mapper.signal_scanline(); // 1 -> 0, IRQ fires

    // Writing to $C001 forces a reload
    mapper.write_prg_u8(0xE000, 0); // disable + acknowledge
    mapper.write_prg_u8(0xC001, 0); // reload flag
    mapper.write_prg_u8(0xE001, 0); // re-enable

    mapper.signal_scanline(); // reload -> 2
    assert!(!mapper.get_irq_flag());
    mapper.signal_scanline(); // 2 -> 1
    mapper.signal_scanline(); // 1 -> 0, IRQ fires again
    assert!(mapper.get_irq_flag());
  }
}
