use std::fmt;

use crate::traits::addressable::Addressable;

use super::opcode::Opcode;
use super::CPU;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AddressingMode {
  Implied,
  Immediate,
  Relative,
  ZeroPage,
  ZeroPageX,
  ZeroPageY,
  Absolute,
  AbsoluteX,
  AbsoluteY,
  Indirect,
  IndirectX,
  IndirectY,
}

impl CPU<'_> {
  #[named]
  pub fn get_operand_address(&mut self, opcode: &Opcode, mode: &AddressingMode) -> Option<u16> {
    trace_enter!();
    trace!("Using addressing mode {}", mode);
    let result = match mode {
      // The Implied mode does not require additional data.
      AddressingMode::Implied => None,
      // The Immediate mode uses the subsequent byte of memory.
      AddressingMode::Immediate => {
        let address = self.program_counter;
        trace_u16!(address);
        Some(address)
      }
      // Relative uses the subsequent byte as a signed byte offset in
      // memory.
      // Because the Relative mode is used only with branch instructions, and
      // because of how extra cycles are calculated for those instructions,
      // this will basically just repeat the Immediate mode code.
      AddressingMode::Relative => {
        let address = self.program_counter;
        trace_u16!(address);
        Some(address)
      }
      // Zero-Page uses an 8-bit value to form a 16-bit address in the
      // first page of memory.
      AddressingMode::ZeroPage => {
        debug!("Ticking (reading operand address from zero page)...");
        let address = self.read_u8(self.program_counter) as u16;
        trace_u16!(address);
        Some(address)
      }
      // Zero-Page, X-Indexed reads a byte and then adds an offset from
      // the X register.
      AddressingMode::ZeroPageX => {
        debug!("Ticking (reading operand address from zero page)...");
        let base = self.read_u8(self.program_counter);
        trace_u16!(base);
        let address = base.wrapping_add(self.x) as u16;
        trace_u16!(address);
        debug!("Ticking (ZeroPageX indexing)...");
        self.tick();
        Some(address)
      }
      // Zero-Page, Y-Indexed reads a byte and then adds an offset from
      // the Y register.
      AddressingMode::ZeroPageY => {
        debug!("Ticking (reading operand address from zero page)...");
        let base = self.read_u8(self.program_counter);
        trace_u16!(base);
        let address = base.wrapping_add(self.y) as u16;
        trace_u16!(address);
        debug!("Ticking (ZeroPageY indexing)...");
        self.tick();
        Some(address)
      }
      // Absolute builds a 16-bit address from two 8-bit reads.
      AddressingMode::Absolute => {
        debug!("Ticking twice (reading 2-byte operand address)...");
        let address = self.read_u16(self.program_counter);
        trace_u16!(address);
        Some(address)
      }
      // Absolute, X-Indexed builds a 16-bit address, then offsets it by
      // the contents of the X register.  If the resulting address is in
      // a different page, an additional clock cycle is required.
      AddressingMode::AbsoluteX => {
        debug!("Ticking twice (reading 2-byte operand address)...");
        let base = self.read_u16(self.program_counter);
        trace_u16!(base);
        let address = base.wrapping_add(self.x as u16);
        trace_u16!(address);
        if opcode.extra_cycle {
          debug!("Ticking (extra cycle due to opcode)...");
          self.tick();
        } else if address & 0xFF00 != base & 0xFF00 {
          debug!("Ticking (extra cycle due to crossing page boundary)...");
          self.tick();
        }
        Some(address)
      }
      // Absolute, Y-Indexed builds a 16-bit address, then offsets it by
      // the contents of the Y register.  If the resulting address is in
      // a different page, an additional clock cycle is required.
      AddressingMode::AbsoluteY => {
        debug!("Ticking twice (reading 2-byte operand address)...");
        let base = self.read_u16(self.program_counter);
        trace_u16!(base);
        let address = base.wrapping_add(self.y as u16);
        trace_u16!(address);
        if opcode.extra_cycle {
          debug!("Ticking (extra cycle due to opcode)...");
          self.tick();
        } else if address & 0xFF00 != base & 0xFF00 {
          debug!("Ticking (extra cycle due to crossing page boundary)...");
          self.tick();
        }
        Some(address)
      }
      // Indirect builds a 16-bit address and reads the final address.
      // In other words, it's a pointer.  Unfortunately, it is buggy.
      // The low byte is read first because it's a Little Endian system,
      // but if this byte is 0xFF, the chip will not cross the page
      // boundary and instead will read the byte from the beginning of
      // the same page.
      AddressingMode::Indirect => {
        debug!("Ticking twice (reading 2-byte operand address)...");
        let pointer = self.read_u16(self.program_counter);
        trace_u16!(pointer);
        let address;
        if pointer & 0x00FF == 0x00FF {
          // Buggy behavior.
          debug!("Ticking twice (reading 2-byte address via buggy Indirect behavior)...");
          address = u16::from_le_bytes([self.read_u8(pointer), self.read_u8(pointer & 0xFF00)]);
        } else {
          // Normal behavior.
          debug!("Ticking twice (reading 2-byte address via non-buggy Indirect behavior)...");
          address = self.read_u16(pointer);
        }
        trace_u16!(address);
        Some(address)
      }
      // Indirect, X-Indexed reads a byte to get a zero-page address,
      // offsets that by the X register, and then reads that to get a
      // 16-bit address.
      AddressingMode::IndirectX => {
        debug!("Ticking (reading operand adress)...");
        let base = self.read_u8(self.program_counter);
        trace_u8!(base);
        let pointer: u8 = base.wrapping_add(self.x);
        trace_u8!(pointer);
        self.tick();
        debug!("Ticking (reading low byte of 2-byte operand address)...");
        let lo = self.read_u8(pointer as u16);
        trace_u8!(lo);
        debug!("Ticking (reading high byte of 2-byte operand address)...");
        let hi = self.read_u8(pointer.wrapping_add(1) as u16);
        trace_u8!(hi);
        let address = (hi as u16) << 8 | (lo as u16);
        trace_u16!(address);
        Some(address)
      }
      // Indirect, Y-Indexed reads a byte to get a zero-page address,
      // reads that to get a 16-bit address, then offsets that by the
      // contents of the Y register to get a final address.  If the
      // offset causes the page to change, another cycle is incurred.
      AddressingMode::IndirectY => {
        debug!("Ticking (reading operand address)...");
        let base = self.read_u8(self.program_counter);
        trace_u8!(base);
        debug!("Ticking (reading low byte of 2-byte operand address)...");
        let lo = self.read_u8(base as u16);
        trace_u8!(lo);
        debug!("Ticking (reading high byte of 2-byte operand address)...");
        let hi = self.read_u8(base.wrapping_add(1) as u16);
        trace_u8!(hi);
        let deref_base = (hi as u16) << 8 | (lo as u16);
        trace_u16!(deref_base);
        let address = deref_base.wrapping_add(self.y as u16);
        trace_u16!(address);
        if opcode.extra_cycle {
          debug!("Ticking (extra cycle due to opcode)...");
          self.tick();
        } else if address & 0xFF00 != deref_base & 0xFF00 {
          debug!("Ticking (extra cycle due to crossing page boundary)...");
          self.tick();
        }
        Some(address)
      }
    };
    trace_result!(result);
    result
  }
}

impl fmt::Display for AddressingMode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}
