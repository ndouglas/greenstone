use crate::traits::addressable::Addressable;

use super::CPU;
use super::opcode::Opcode;

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
      AddressingMode::Relative => {
        let offset = self.read_u8(self.program_counter);
        trace_u8!(offset);
        let address = self.program_counter.wrapping_add(offset as u16);
        trace_u16!(address);
        Some(address)
      }
      // Zero-Page uses an 8-bit value to form a 16-bit address in the
      // first page of memory.
      AddressingMode::ZeroPage => {
        let address = self.read_u8(self.program_counter) as u16;
        trace_u16!(address);
        Some(address)
      }
      // Zero-Page, X-Indexed reads a byte and then adds an offset from
      // the X register.
      AddressingMode::ZeroPageX => {
        let base = self.read_u8(self.program_counter);
        trace_u16!(base);
        let address = base.wrapping_add(self.x) as u16;
        trace_u16!(address);
        self.tick();
        Some(address)
      }
      // Zero-Page, Y-Indexed reads a byte and then adds an offset from
      // the Y register.
      AddressingMode::ZeroPageY => {
        let base = self.read_u8(self.program_counter);
        trace_u16!(base);
        let address = base.wrapping_add(self.y) as u16;
        trace_u16!(address);
        self.tick();
        Some(address)
      }
      // Absolute builds a 16-bit address from two 8-bit reads.
      AddressingMode::Absolute => {
        let address = self.read_u16(self.program_counter);
        trace_u16!(address);
        Some(address)
      }
      // Absolute, X-Indexed builds a 16-bit address, then offsets it by
      // the contents of the X register.  If the resulting address is in
      // a different page, an additional clock cycle is required.
      AddressingMode::AbsoluteX => {
        let base = self.read_u16(self.program_counter);
        trace_u16!(base);
        let address = base.wrapping_add(self.x as u16);
        trace_u16!(address);
        if address & 0xFF00 != base & 0xFF00 {
          self.tick();
        }
        Some(address)
      }
      // Absolute, Y-Indexed builds a 16-bit address, then offsets it by
      // the contents of the Y register.  If the resulting address is in
      // a different page, an additional clock cycle is required.
      AddressingMode::AbsoluteY => {
        let base = self.read_u16(self.program_counter);
        trace_u16!(base);
        let address = base.wrapping_add(self.y as u16);
        trace_u16!(address);
        if address & 0xFF00 != base & 0xFF00 {
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
        let pointer = self.read_u16(self.program_counter);
        trace_u16!(pointer);
        let address;
        if pointer & 0x00FF == 0x00FF {
          // Buggy behavior.
          address = u16::from_le_bytes([self.read_u8(pointer & 0xFF00), self.read_u8(pointer)]);
        } else {
          // Normal behavior.
          address = u16::from_le_bytes([self.read_u8(pointer.wrapping_add(1)), self.read_u8(pointer)]);
        }
        trace_u16!(address);
        Some(address)
      }
      // Indirect, X-Indexed reads a byte to get a zero-page address,
      // offsets that by the X register, and then reads that to get a
      // 16-bit address.
      AddressingMode::IndirectX => {
        let base = self.read_u8(self.program_counter);
        trace_u8!(base);
        let pointer: u8 = base.wrapping_add(self.x);
        trace_u8!(pointer);
        self.tick();
        let lo = self.read_u8(pointer as u16);
        trace_u8!(lo);
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
        let base = self.read_u8(self.program_counter);
        trace_u8!(base);
        let lo = self.read_u8(base as u16);
        trace_u8!(lo);
        let hi = self.read_u8(base.wrapping_add(1) as u16);
        trace_u8!(hi);
        let deref_base = (hi as u16) << 8 | (lo as u16);
        trace_u16!(deref_base);
        let address = deref_base.wrapping_add(self.y as u16);
        trace_u16!(address);
        if address & 0xFF00 != deref_base & 0xFF00 {
          self.tick();
        }
        Some(address)
      }
    };
    trace_result!(result);
    result
  }
}
