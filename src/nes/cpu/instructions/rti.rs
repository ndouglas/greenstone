use super::super::*;

impl CPU {
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
    trace_var!(opcode);
    let mode = &opcode.mode;
    trace_var!(mode);
    debug!("Ticking (fetching, then discarding operand byte)...");
    self.tick();
    debug!("Ticking (adjusting stack pointer)...");
    self.tick();
    let new_status = (self.pop_u8() | UNUSED_FLAG) & !BREAK_FLAG;
    trace_u8!(new_status);
    self.status = new_status;
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
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("RTI", Implied, []{} => []{status: 0b11101111, program_counter: 0x1234, stack_pointer: 0xFD}, |cpu: &mut CPU, _opcode: &Opcode| {
      cpu.push_u16(0x1234);
      // RTI is not allowed to change the Unused flag.
      // RTI should clear the Break flag.
      cpu.push_u8(0b11011111);
    });
  }
}
