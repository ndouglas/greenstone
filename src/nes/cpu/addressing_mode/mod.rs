use crate::traits::addressable::Addressable;

use super::CPU;

#[derive(Debug)]
pub enum AddressingMode {
  Implicit,
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
  pub fn get_operand_address(&self, mode: &AddressingMode) -> Option<(u16, u8)> {
    match mode {
      // The Implicit mode does not require additional data.
      AddressingMode::Implicit => None,
      // The Immediate mode uses the subsequent byte of memory.
      AddressingMode::Immediate => {
        let address = self.program_counter;
        let additional_cycles = 0;
        Some((address, additional_cycles))
      }
      // Relative uses the subsequent byte as a signed byte offset in
      // memory.
      AddressingMode::Relative => {
        let offset = self.read_u8(self.program_counter);
        let address = self.program_counter + (offset as u16);
        let additional_cycles = 0;
        Some((address, additional_cycles))
      }
      // Zero-Page uses an 8-bit value to form a 16-bit address in the
      // first page of memory.
      AddressingMode::ZeroPage => {
        let address = self.read_u8(self.program_counter) as u16;
        let additional_cycles = 0;
        Some((address, additional_cycles))
      }
      // Zero-Page, X-Indexed reads a byte and then adds an offset from
      // the X register.
      AddressingMode::ZeroPageX => {
        let base = self.read_u8(self.program_counter);
        let address = base.wrapping_add(self.x) as u16;
        let additional_cycles = 0;
        Some((address, additional_cycles))
      }
      // Zero-Page, Y-Indexed reads a byte and then adds an offset from
      // the Y register.
      AddressingMode::ZeroPageY => {
        let base = self.read_u8(self.program_counter);
        let address = base.wrapping_add(self.y) as u16;
        let additional_cycles = 0;
        Some((address, additional_cycles))
      }
      // Absolute builds a 16-bit address from two 8-bit reads.
      AddressingMode::Absolute => {
        let address = self.read_u16(self.program_counter);
        let additional_cycles = 0;
        Some((address, additional_cycles))
      }
      // Absolute, X-Indexed builds a 16-bit address, then offsets it by
      // the contents of the X register.  If the resulting address is in
      // a different page, an additional clock cycle is required.
      AddressingMode::AbsoluteX => {
        let base = self.read_u16(self.program_counter);
        let address = base.wrapping_add(self.x as u16);
        let mut additional_cycles = 0;
        if address & 0xFF00 != base & 0xFF00 {
          additional_cycles = 1;
        }
        Some((address, additional_cycles))
      }
      // Absolute, Y-Indexed builds a 16-bit address, then offsets it by
      // the contents of the Y register.  If the resulting address is in
      // a different page, an additional clock cycle is required.
      AddressingMode::AbsoluteY => {
        let base = self.read_u16(self.program_counter);
        let address = base.wrapping_add(self.y as u16);
        let mut additional_cycles = 0;
        if address & 0xFF00 != base & 0xFF00 {
          additional_cycles = 1;
        }
        Some((address, additional_cycles))
      }
      // Indirect builds a 16-bit address and reads the final address.
      // In other words, it's a pointer.  Unfortunately, it is buggy.
      // The low byte is read first because it's a Little Endian system,
      // but if this byte is 0xFF, the chip will not cross the page
      // boundary and instead will read the byte from the beginning of
      // the same page.
      AddressingMode::Indirect => {
        let pointer = self.read_u16(self.program_counter);
        let address;
        if pointer & 0x00FF == 0x00FF {
          // Buggy behavior.
          address = ((self.read_u8(pointer & 0xFF00) as u16) << 8) | self.read_u8(pointer) as u16;
        } else {
          // Normal behavior.
          address = ((self.read_u8(pointer + 1) as u16) << 8) | self.read_u8(pointer) as u16;
        }
        let additional_cycles = 0;
        Some((address, additional_cycles))
      }
      // Indirect, X-Indexed reads a byte to get a zero-page address,
      // offsets that by the X register, and then reads that to get a
      // 16-bit address.
      AddressingMode::IndirectX => {
        let base = self.read_u8(self.program_counter);
        let pointer: u8 = base.wrapping_add(self.x);
        let lo = self.read_u8(pointer as u16);
        let hi = self.read_u8(pointer.wrapping_add(1) as u16);
        let address = (hi as u16) << 8 | (lo as u16);
        let additional_cycles = 0;
        Some((address, additional_cycles))
      }
      // Indirect, Y-Indexed reads a byte to get a zero-page address,
      // reads that to get a 16-bit address, then offsets that by the
      // contents of the Y register to get a final address.  If the
      // offset causes the page to change, another cycle is incurred.
      AddressingMode::IndirectY => {
        let base = self.read_u8(self.program_counter);
        let lo = self.read_u8(base as u16);
        let hi = self.read_u8(base.wrapping_add(1) as u16);
        let deref_base = (hi as u16) << 8 | (lo as u16);
        let address = deref_base.wrapping_add(self.y as u16);
        let mut additional_cycles = 0;
        if address & 0xFF00 != deref_base & 0xFF00 {
          additional_cycles = 1;
        }
        Some((address, additional_cycles))
      }
    }
  }
}
