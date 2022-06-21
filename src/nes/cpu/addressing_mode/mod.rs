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

impl CPU {
  #[named]
  pub fn unclocked_get_operand_value(&mut self, mode: &AddressingMode, address: u16) -> Option<u8> {
    use AddressingMode::*;
    trace_enter!();
    trace!("Using addressing mode {}", mode);
    let result = match mode {
      Implied => None,
      _ => match self.unclocked_get_operand_address(mode, address) {
        Some(new_address) => Some(self.unclocked_read_u8(new_address)),
        None => None,
      },
    };
    trace_var!(result);
    trace_exit!();
    result
  }

  #[named]
  pub fn unclocked_get_operand_address(&mut self, mode: &AddressingMode, address: u16) -> Option<u16> {
    use AddressingMode::*;
    trace_enter!();
    trace!("Using addressing mode {}", mode);
    let result = match mode {
      Implied => None,
      Immediate => Some(address),
      Relative => Some(address),
      ZeroPage => Some(self.unclocked_read_u8(address) as u16),
      ZeroPageX => Some(self.unclocked_read_u8(address).wrapping_add(self.x) as u16),
      ZeroPageY => Some(self.unclocked_read_u8(address).wrapping_add(self.y) as u16),
      Absolute => Some(self.unclocked_read_u16(address)),
      AbsoluteX => Some(self.unclocked_read_u16(address).wrapping_add(self.x as u16)),
      AbsoluteY => Some(self.unclocked_read_u16(address).wrapping_add(self.y as u16)),
      Indirect => {
        let pointer = self.unclocked_read_u16(address);
        trace_u16!(pointer);
        let address;
        if pointer & 0x00FF == 0x00FF {
          // Buggy behavior.
          address = u16::from_le_bytes([self.unclocked_read_u8(pointer), self.unclocked_read_u8(pointer & 0xFF00)]);
        } else {
          // Normal behavior.
          address = self.unclocked_read_u16(pointer);
        }
        trace_u16!(address);
        Some(address)
      }
      IndirectX => {
        let base = self.unclocked_read_u8(address);
        let pointer = base.wrapping_add(self.x);
        let lo = self.unclocked_read_u8(pointer as u16);
        let hi = self.unclocked_read_u8(pointer.wrapping_add(1) as u16);
        Some((hi as u16) << 8 | (lo as u16))
      }
      IndirectY => {
        let base = self.unclocked_read_u8(address);
        let lo = self.unclocked_read_u8(base as u16);
        let hi = self.unclocked_read_u8(base.wrapping_add(1) as u16);
        let deref_base = (hi as u16) << 8 | (lo as u16);
        let deref = deref_base.wrapping_add(self.y as u16);
        Some(deref)
      }
    };
    trace_var!(result);
    trace_exit!();
    result
  }

  #[named]
  pub fn get_operand_address(&mut self, opcode: &Opcode, mode: &AddressingMode) -> Option<u16> {
    use AddressingMode::*;
    trace_enter!();
    trace!("Using addressing mode {}", mode);
    let result = match mode {
      // The Implied mode does not require additional data.
      //
      // Implied Instruction Cycle Information (from 6502_cpu.txt)
      //
      //  #  address R/W description
      // --- ------- --- -----------------------------------------------
      //  1    PC     R  fetch opcode, increment PC
      //  2    PC     R  read next instruction byte (and throw it away)
      //
      Implied => None,
      // The Immediate mode uses the subsequent byte of memory.
      //
      // Immediate Instruction Cycle Information (from 6502_cpu.txt)
      //
      //  #  address R/W description
      // --- ------- --- ------------------------------------------
      //  1    PC     R  fetch opcode, increment PC
      //  2    PC     R  fetch value, increment PC
      //
      Immediate => {
        let address = self.program_counter;
        self.increment_program_counter();
        trace_u16!(address);
        Some(address)
      }
      // Relative uses the subsequent byte as a signed byte offset in
      // memory.
      // Because the Relative mode is used only with branch instructions, and
      // because of how extra cycles are calculated for those instructions,
      // this will basically just repeat the Immediate mode code.
      //
      // Relative Instruction Cycle Information (from 6502_cpu.txt)
      //
      // Read instructions (LDA, LDX, LDY, EOR, AND, ORA, ADC, SBC, CMP, BIT, LAX, NOP)
      //
      //  #  address R/W description
      // --- ------- --- ------------------------------------------
      //  1    PC     R  fetch opcode, increment PC
      //  2    PC     R  fetch low byte of address, increment PC
      //  3    PC     R  fetch high byte of address, increment PC
      //  4  address  R  read from effective address
      //
      // Read-Modify-Write instructions (ASL, LSR, ROL, ROR, INC, DEC,
      //                      SLO, SRE, RLA, RRA, ISB, DCP)
      //
      //  #  address R/W description
      // --- ------- --- ------------------------------------------
      //  1    PC     R  fetch opcode, increment PC
      //  2    PC     R  fetch low byte of address, increment PC
      //  3    PC     R  fetch high byte of address, increment PC
      //  4  address  R  read from effective address
      //  5  address  W  write the value back to effective address,
      //                 and do the operation on it
      //  6  address  W  write the new value to effective address
      //
      // Write instructions (STA, STX, STY, SAX)
      //
      //  #  address R/W description
      // --- ------- --- ------------------------------------------
      //  1    PC     R  fetch opcode, increment PC
      //  2    PC     R  fetch low byte of address, increment PC
      //  3    PC     R  fetch high byte of address, increment PC
      //  4  address  W  write register to effective address
      //
      Relative => {
        let address = self.program_counter;
        self.increment_program_counter();
        trace_u16!(address);
        Some(address)
      }
      // Zero-Page uses an 8-bit value to form a 16-bit address in the
      // first page of memory.
      //
      // Zero-Page Instruction Cycle Information (from 6502_cpu.txt)
      //
      // Read instructions (LDA, LDX, LDY, EOR, AND, ORA, ADC, SBC, CMP, BIT, LAX, NOP)
      //
      //   #  address R/W description
      //  --- ------- --- ------------------------------------------
      //   1    PC     R  fetch opcode, increment PC
      //   2    PC     R  fetch address, increment PC
      //   3  address  R  read from effective address
      //
      // Read-Modify-Write instructions (ASL, LSR, ROL, ROR, INC, DEC,
      //                               SLO, SRE, RLA, RRA, ISB, DCP)
      //
      //   #  address R/W description
      //  --- ------- --- ------------------------------------------
      //   1     PC    R  fetch opcode, increment PC
      //   2     PC    R  fetch address, increment PC
      //   3   address R  read from effective address
      //   4   address W  write the value back to effective address,
      //                  and do the operation on it
      //   5   address W  write the new value to effective address
      //
      // Write instructions (STA, STX, STY, SAX)
      //
      //   #  address R/W description
      //  --- ------- --- ------------------------------------------
      //   1    PC     R  fetch opcode, increment PC
      //   2    PC     R  fetch address, increment PC
      //   3  address  W  write register to effective address
      //
      ZeroPage => {
        debug!("Ticking (reading operand address from zero page)...");
        let address = self.get_next_u8() as u16;
        trace_u16!(address);
        Some(address)
      }
      // Zero-Page, X-Indexed reads a byte and then adds an offset from
      // the X register.
      //
      // Zero-Page Indexed Instruction Information (from 6502_cpu.txt)
      //
      // Read instructions (LDA, LDX, LDY, EOR, AND, ORA, ADC, SBC, CMP, BIT,
      //                   LAX, NOP)
      //
      //   #   address  R/W description
      //  --- --------- --- ------------------------------------------
      //   1     PC      R  fetch opcode, increment PC
      //   2     PC      R  fetch address, increment PC
      //   3   address   R  read from address, add index register to it
      //   4  address+I* R  read from effective address
      //
      //  Notes: I denotes either index register (X or Y).
      //
      //         * The high byte of the effective address is always zero,
      //           i.e. page boundary crossings are not handled.
      //
      // Read-Modify-Write instructions (ASL, LSR, ROL, ROR, INC, DEC,
      //                                SLO, SRE, RLA, RRA, ISB, DCP)
      //
      //   #   address  R/W description
      //  --- --------- --- ---------------------------------------------
      //   1     PC      R  fetch opcode, increment PC
      //   2     PC      R  fetch address, increment PC
      //   3   address   R  read from address, add index register X to it
      //   4  address+X* R  read from effective address
      //   5  address+X* W  write the value back to effective address,
      //                    and do the operation on it
      //   6  address+X* W  write the new value to effective address
      //
      //  Note: * The high byte of the effective address is always zero,
      //          i.e. page boundary crossings are not handled.
      //
      // Write instructions (STA, STX, STY, SAX)
      //
      //   #   address  R/W description
      //  --- --------- --- -------------------------------------------
      //   1     PC      R  fetch opcode, increment PC
      //   2     PC      R  fetch address, increment PC
      //   3   address   R  read from address, add index register to it
      //   4  address+I* W  write to effective address
      //
      //  Notes: I denotes either index register (X or Y).
      //
      //         * The high byte of the effective address is always zero,
      //           i.e. page boundary crossings are not handled.
      //
      ZeroPageX => {
        debug!("Ticking (reading operand address from zero page)...");
        let base = self.get_next_u8();
        trace_u16!(base);
        let address = base.wrapping_add(self.x) as u16;
        trace_u16!(address);
        debug!("Ticking (ZeroPageX indexing)...");
        self.tick();
        Some(address)
      }
      // Zero-Page, Y-Indexed reads a byte and then adds an offset from
      // the Y register.
      //
      // Zero-Page Indexed Instruction Information (from 6502_cpu.txt)
      //
      // Read instructions (LDA, LDX, LDY, EOR, AND, ORA, ADC, SBC, CMP, BIT,
      //                   LAX, NOP)
      //
      //   #   address  R/W description
      //  --- --------- --- ------------------------------------------
      //   1     PC      R  fetch opcode, increment PC
      //   2     PC      R  fetch address, increment PC
      //   3   address   R  read from address, add index register to it
      //   4  address+I* R  read from effective address
      //
      //  Notes: I denotes either index register (X or Y).
      //
      //         * The high byte of the effective address is always zero,
      //           i.e. page boundary crossings are not handled.
      //
      // Read-Modify-Write instructions (ASL, LSR, ROL, ROR, INC, DEC,
      //                                SLO, SRE, RLA, RRA, ISB, DCP)
      //
      //   #   address  R/W description
      //  --- --------- --- ---------------------------------------------
      //   1     PC      R  fetch opcode, increment PC
      //   2     PC      R  fetch address, increment PC
      //   3   address   R  read from address, add index register X to it
      //   4  address+X* R  read from effective address
      //   5  address+X* W  write the value back to effective address,
      //                    and do the operation on it
      //   6  address+X* W  write the new value to effective address
      //
      //  Note: * The high byte of the effective address is always zero,
      //          i.e. page boundary crossings are not handled.
      //
      // Write instructions (STA, STX, STY, SAX)
      //
      //   #   address  R/W description
      //  --- --------- --- -------------------------------------------
      //   1     PC      R  fetch opcode, increment PC
      //   2     PC      R  fetch address, increment PC
      //   3   address   R  read from address, add index register to it
      //   4  address+I* W  write to effective address
      //
      //  Notes: I denotes either index register (X or Y).
      //
      //         * The high byte of the effective address is always zero,
      //           i.e. page boundary crossings are not handled.
      //
      ZeroPageY => {
        debug!("Ticking (reading operand address from zero page)...");
        let base = self.get_next_u8();
        trace_u16!(base);
        let address = base.wrapping_add(self.y) as u16;
        trace_u16!(address);
        debug!("Ticking (ZeroPageY indexing)...");
        self.tick();
        Some(address)
      }
      // Absolute builds a 16-bit address from two 8-bit reads.
      //
      // Absolute Instruction Cycle Information (from 6502_cpu.txt)
      //
      //    JMP
      //
      //    #  address R/W description
      //   --- ------- --- -------------------------------------------------
      //    1    PC     R  fetch opcode, increment PC
      //    2    PC     R  fetch low address byte, increment PC
      //    3    PC     R  copy low address byte to PCL, fetch high address
      //                   byte to PCH
      //
      // Read instructions (LDA, LDX, LDY, EOR, AND, ORA, ADC, SBC, CMP, BIT,
      //                    LAX, NOP)
      //
      //    #  address R/W description
      //   --- ------- --- ------------------------------------------
      //    1    PC     R  fetch opcode, increment PC
      //    2    PC     R  fetch low byte of address, increment PC
      //    3    PC     R  fetch high byte of address, increment PC
      //    4  address  R  read from effective address
      //
      // Read-Modify-Write instructions (ASL, LSR, ROL, ROR, INC, DEC,
      //                                 SLO, SRE, RLA, RRA, ISB, DCP)
      //
      //    #  address R/W description
      //   --- ------- --- ------------------------------------------
      //    1    PC     R  fetch opcode, increment PC
      //    2    PC     R  fetch low byte of address, increment PC
      //    3    PC     R  fetch high byte of address, increment PC
      //    4  address  R  read from effective address
      //    5  address  W  write the value back to effective address,
      //                   and do the operation on it
      //    6  address  W  write the new value to effective address
      //
      // Write instructions (STA, STX, STY, SAX)
      //
      //    #  address R/W description
      //   --- ------- --- ------------------------------------------
      //    1    PC     R  fetch opcode, increment PC
      //    2    PC     R  fetch low byte of address, increment PC
      //    3    PC     R  fetch high byte of address, increment PC
      //    4  address  W  write register to effective address
      //
      Absolute => {
        debug!("Ticking twice (reading 2-byte operand address)...");
        let address = self.get_next_u16();
        trace_u16!(address);
        Some(address)
      }
      // Absolute, X-Indexed builds a 16-bit address, then offsets it by
      // the contents of the X register.  If the resulting address is in
      // a different page, an additional clock cycle is required.
      //
      // Absolute, X-Indexed Instruction Cycle Information (from 6502_cpu.txt)
      //
      // Read instructions (LDA, LDX, LDY, EOR, AND, ORA, ADC, SBC, CMP, BIT,
      //                    LAX, LAE, SHS, NOP)
      //
      //    #   address  R/W description
      //   --- --------- --- ------------------------------------------
      //    1     PC      R  fetch opcode, increment PC
      //    2     PC      R  fetch low byte of address, increment PC
      //    3     PC      R  fetch high byte of address,
      //                     add index register to low address byte,
      //                     increment PC
      //    4  address+I* R  read from effective address,
      //                     fix the high byte of effective address
      //    5+ address+I  R  re-read from effective address
      //
      //   Notes: I denotes either index register (X or Y).
      //
      //          * The high byte of the effective address may be invalid
      //            at this time, i.e. it may be smaller by $100.
      //
      //          + This cycle will be executed only if the effective address
      //            was invalid during cycle #4, i.e. page boundary was crossed.
      //
      // Read-Modify-Write instructions (ASL, LSR, ROL, ROR, INC, DEC,
      //                                 SLO, SRE, RLA, RRA, ISB, DCP)
      //
      //    #   address  R/W description
      //   --- --------- --- ------------------------------------------
      //    1    PC       R  fetch opcode, increment PC
      //    2    PC       R  fetch low byte of address, increment PC
      //    3    PC       R  fetch high byte of address,
      //                     add index register X to low address byte,
      //                     increment PC
      //    4  address+X* R  read from effective address,
      //                     fix the high byte of effective address
      //    5  address+X  R  re-read from effective address
      //    6  address+X  W  write the value back to effective address,
      //                     and do the operation on it
      //    7  address+X  W  write the new value to effective address
      //
      //   Notes: * The high byte of the effective address may be invalid
      //            at this time, i.e. it may be smaller by $100.
      //
      // Write instructions (STA, STX, STY, SHA, SHX, SHY)
      //
      //    #   address  R/W description
      //   --- --------- --- ------------------------------------------
      //    1     PC      R  fetch opcode, increment PC
      //    2     PC      R  fetch low byte of address, increment PC
      //    3     PC      R  fetch high byte of address,
      //                     add index register to low address byte,
      //                     increment PC
      //    4  address+I* R  read from effective address,
      //                     fix the high byte of effective address
      //    5  address+I  W  write to effective address
      //
      //   Notes: I denotes either index register (X or Y).
      //
      //          * The high byte of the effective address may be invalid
      //            at this time, i.e. it may be smaller by $100. Because
      //            the processor cannot undo a write to an invalid
      //            address, it always reads from the address first.
      //
      AbsoluteX => {
        debug!("Ticking twice (reading 2-byte operand address)...");
        let base = self.get_next_u16();
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
      //
      // Absolute, Y-Indexed Instruction Cycle Information (from 6502_cpu.txt)
      //
      //
      // Absolute, X-Indexed Instruction Cycle Information (from 6502_cpu.txt)
      //
      // Read instructions (LDA, LDX, LDY, EOR, AND, ORA, ADC, SBC, CMP, BIT,
      //                    LAX, LAE, SHS, NOP)
      //
      //    #   address  R/W description
      //   --- --------- --- ------------------------------------------
      //    1     PC      R  fetch opcode, increment PC
      //    2     PC      R  fetch low byte of address, increment PC
      //    3     PC      R  fetch high byte of address,
      //                     add index register to low address byte,
      //                     increment PC
      //    4  address+I* R  read from effective address,
      //                     fix the high byte of effective address
      //    5+ address+I  R  re-read from effective address
      //
      //   Notes: I denotes either index register (X or Y).
      //
      //          * The high byte of the effective address may be invalid
      //            at this time, i.e. it may be smaller by $100.
      //
      //          + This cycle will be executed only if the effective address
      //            was invalid during cycle #4, i.e. page boundary was crossed.
      //
      // Read-Modify-Write instructions (ASL, LSR, ROL, ROR, INC, DEC,
      //                                 SLO, SRE, RLA, RRA, ISB, DCP)
      //
      //    #   address  R/W description
      //   --- --------- --- ------------------------------------------
      //    1    PC       R  fetch opcode, increment PC
      //    2    PC       R  fetch low byte of address, increment PC
      //    3    PC       R  fetch high byte of address,
      //                     add index register X to low address byte,
      //                     increment PC
      //    4  address+X* R  read from effective address,
      //                     fix the high byte of effective address
      //    5  address+X  R  re-read from effective address
      //    6  address+X  W  write the value back to effective address,
      //                     and do the operation on it
      //    7  address+X  W  write the new value to effective address
      //
      //   Notes: * The high byte of the effective address may be invalid
      //            at this time, i.e. it may be smaller by $100.
      //
      // Write instructions (STA, STX, STY, SHA, SHX, SHY)
      //
      //    #   address  R/W description
      //   --- --------- --- ------------------------------------------
      //    1     PC      R  fetch opcode, increment PC
      //    2     PC      R  fetch low byte of address, increment PC
      //    3     PC      R  fetch high byte of address,
      //                     add index register to low address byte,
      //                     increment PC
      //    4  address+I* R  read from effective address,
      //                     fix the high byte of effective address
      //    5  address+I  W  write to effective address
      //
      //   Notes: I denotes either index register (X or Y).
      //
      //          * The high byte of the effective address may be invalid
      //            at this time, i.e. it may be smaller by $100. Because
      //            the processor cannot undo a write to an invalid
      //            address, it always reads from the address first.
      //
      AbsoluteY => {
        debug!("Ticking twice (reading 2-byte operand address)...");
        let base = self.get_next_u16();
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
      //
      // Indirect Instruction Cycle Information (from 6502_cpu.txt)
      //
      //  #   address  R/W description
      // --- --------- --- ------------------------------------------
      //  1     PC      R  fetch opcode, increment PC
      //  2     PC      R  fetch pointer address low, increment PC
      //  3     PC      R  fetch pointer address high, increment PC
      //  4   pointer   R  fetch low address to latch
      //  5  pointer+1* R  fetch PCH, copy latch to PCL
      //
      // Note: * The PCH will always be fetched from the same page
      //         than PCL, i.e. page boundary crossing is not handled.
      //
      Indirect => {
        debug!("Ticking twice (reading 2-byte operand address)...");
        let pointer = self.get_next_u16();
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
      //
      // Indirect, X-Indexed Instruction Cycle Information (from 6502_cpu.txt)
      //
      // Read instructions (LDA, ORA, EOR, AND, ADC, CMP, SBC, LAX)
      //
      //    #    address   R/W description
      //   --- ----------- --- ------------------------------------------
      //    1      PC       R  fetch opcode, increment PC
      //    2      PC       R  fetch pointer address, increment PC
      //    3    pointer    R  read from the address, add X to it
      //    4   pointer+X   R  fetch effective address low
      //    5  pointer+X+1  R  fetch effective address high
      //    6    address    R  read from effective address
      //
      //   Note: The effective address is always fetched from zero page,
      //         i.e. the zero page boundary crossing is not handled.
      //
      // Read-Modify-Write instructions (SLO, SRE, RLA, RRA, ISB, DCP)
      //
      //    #    address   R/W description
      //   --- ----------- --- ------------------------------------------
      //    1      PC       R  fetch opcode, increment PC
      //    2      PC       R  fetch pointer address, increment PC
      //    3    pointer    R  read from the address, add X to it
      //    4   pointer+X   R  fetch effective address low
      //    5  pointer+X+1  R  fetch effective address high
      //    6    address    R  read from effective address
      //    7    address    W  write the value back to effective address,
      //                       and do the operation on it
      //    8    address    W  write the new value to effective address
      //
      //   Note: The effective address is always fetched from zero page,
      //         i.e. the zero page boundary crossing is not handled.
      //
      // Write instructions (STA, SAX)
      //
      //    #    address   R/W description
      //   --- ----------- --- ------------------------------------------
      //    1      PC       R  fetch opcode, increment PC
      //    2      PC       R  fetch pointer address, increment PC
      //    3    pointer    R  read from the address, add X to it
      //    4   pointer+X   R  fetch effective address low
      //    5  pointer+X+1  R  fetch effective address high
      //    6    address    W  write to effective address
      //
      //   Note: The effective address is always fetched from zero page,
      //         i.e. the zero page boundary crossing is not handled.
      //
      IndirectX => {
        debug!("Ticking (reading operand adress)...");
        let base = self.get_next_u8();
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
      //
      // Absolute, Y-Indexed Instruction Cycle Information (from 6502_cpu.txt)
      //
      // Read instructions (LDA, EOR, AND, ORA, ADC, SBC, CMP)
      //
      //    #    address   R/W description
      //   --- ----------- --- ------------------------------------------
      //    1      PC       R  fetch opcode, increment PC
      //    2      PC       R  fetch pointer address, increment PC
      //    3    pointer    R  fetch effective address low
      //    4   pointer+1   R  fetch effective address high,
      //                       add Y to low byte of effective address
      //    5   address+Y*  R  read from effective address,
      //                       fix high byte of effective address
      //    6+  address+Y   R  read from effective address
      //
      //   Notes: The effective address is always fetched from zero page,
      //          i.e. the zero page boundary crossing is not handled.
      //
      //          * The high byte of the effective address may be invalid
      //            at this time, i.e. it may be smaller by $100.
      //
      //          + This cycle will be executed only if the effective address
      //            was invalid during cycle #5, i.e. page boundary was crossed.
      //
      // Read-Modify-Write instructions (SLO, SRE, RLA, RRA, ISB, DCP)
      //
      //    #    address   R/W description
      //   --- ----------- --- ------------------------------------------
      //    1      PC       R  fetch opcode, increment PC
      //    2      PC       R  fetch pointer address, increment PC
      //    3    pointer    R  fetch effective address low
      //    4   pointer+1   R  fetch effective address high,
      //                       add Y to low byte of effective address
      //    5   address+Y*  R  read from effective address,
      //                       fix high byte of effective address
      //    6   address+Y   R  read from effective address
      //    7   address+Y   W  write the value back to effective address,
      //                       and do the operation on it
      //    8   address+Y   W  write the new value to effective address
      //
      //   Notes: The effective address is always fetched from zero page,
      //          i.e. the zero page boundary crossing is not handled.
      //
      //          * The high byte of the effective address may be invalid
      //            at this time, i.e. it may be smaller by $100.
      //
      // Write instructions (STA, SHA)
      //
      //    #    address   R/W description
      //   --- ----------- --- ------------------------------------------
      //    1      PC       R  fetch opcode, increment PC
      //    2      PC       R  fetch pointer address, increment PC
      //    3    pointer    R  fetch effective address low
      //    4   pointer+1   R  fetch effective address high,
      //                       add Y to low byte of effective address
      //    5   address+Y*  R  read from effective address,
      //                       fix high byte of effective address
      //    6   address+Y   W  write to effective address
      //
      //   Notes: The effective address is always fetched from zero page,
      //          i.e. the zero page boundary crossing is not handled.
      //
      //          * The high byte of the effective address may be invalid
      //            at this time, i.e. it may be smaller by $100.
      //
      IndirectY => {
        debug!("Ticking (reading operand address)...");
        let base = self.get_next_u8();
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
