use super::super::*;

impl CPU {
  // JMP Cycle Information (from 6502_cpu.txt)
  //
  // Absolute addressing
  //
  //    #  address R/W description
  //   --- ------- --- -------------------------------------------------
  //    1    PC     R  fetch opcode, increment PC
  //    2    PC     R  fetch low address byte, increment PC
  //    3    PC     R  copy low address byte to PCL, fetch high address
  //                   byte to PCH
  //
  // Absolute indirect addressing
  //
  //       #   address  R/W description
  //      --- --------- --- ------------------------------------------
  //       1     PC      R  fetch opcode, increment PC
  //       2     PC      R  fetch pointer address low, increment PC
  //       3     PC      R  fetch pointer address high, increment PC
  //       4   pointer   R  fetch low address to latch
  //       5  pointer+1* R  fetch PCH, copy latch to PCL
  //
  //      Note: * The PCH will always be fetched from the same page
  //              than PCL, i.e. page boundary crossing is not handled.
  //
  #[inline]
  #[named]
  pub fn instruction_jmp(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_var!(opcode);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    self.program_counter = address;
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_jmp() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("JMP", Absolute, [0x0A, 0x00]{} => []{program_counter: 10});
    test_instruction!("JMP", Indirect, [0x03, 0x00, 0x0A, 0x00]{} => []{program_counter: 10});
    test_instruction!("JMP", Indirect, [0xFF, 0x01]{status: 0b10000000} => []{program_counter: 0x2211}, |cpu: &mut CPU, _opcode: &Opcode| {
      cpu.unclocked_write_u8(0x01FF, 0x11);
      cpu.unclocked_write_u8(0x0100, 0x22);
      cpu.unclocked_write_u8(0x0200, 0x33);
    });
  }
}
