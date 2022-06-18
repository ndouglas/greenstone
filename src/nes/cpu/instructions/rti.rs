use super::super::*;

impl CPU<'_> {

  // RTI Cycle Information (from 6502_cpu.txt)
  //
  // #  address R/W description
  // --- ------- --- -----------------------------------------------
  //  1    PC     R  fetch opcode, increment PC
  //  2    PC     R  read next instruction byte (and throw it away)
  //  3  $0100,S  R  increment S
  //  4  $0100,S  R  pull P from stack, increment S
  //  5  $0100,S  R  pull PCL from stack, increment S
  //  6  $0100,S  R  pull PCH from stack

  #[inline]
  #[named]
  pub fn instruction_rti(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let mode = &opcode.mode;
    trace_var!(mode);
    debug!("Ticking (fetching, then discarding operand byte)...");
    self.tick();
    debug!("Ticking (adjusting stack pointer)...");
    self.tick();
    let mut new_status = self.pop_u8();
    trace_u8!(new_status);
    new_status = new_status & !BREAK_FLAG;
    new_status = new_status & !UNUSED_FLAG;
    self.status = new_status;
    trace_u8!(self.status);
    self.program_counter = self.pop_u16();
    trace_u8!(self.program_counter);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_rti() {
    init();
    test_instruction!("RTI", Implied, []{} => []{status: 0b11001111, program_counter: 0x1234, stack_pointer: 0xFF}, |cpu: &mut CPU<'_>, _opcode: &Opcode| {
      cpu.push_u16(0x1234);
      // RTI is not allowed to change the Unused flag.
      // RTI should clear the Break flag.
      cpu.push_u8(0b11011111);
    });
  }

}

