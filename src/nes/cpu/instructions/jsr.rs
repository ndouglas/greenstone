use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_jsr(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    debug!("Ticking (processing instruction)...");
    self.tick();
    let return_address = self.program_counter.wrapping_add(1);
    trace_u16!(return_address);
    self.push_u16(return_address);
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
  fn test_jsr() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("JSR", Absolute, [0x0A, 0x00]{} => []{clock_counter: 6, program_counter: 10, stack_pointer: 0xFD}, |cpu: &mut CPU<'_>, _opcode: &Opcode| {
      // Write an RTS at the destination instruction.
      cpu.unclocked_write_u16(0x000A, 0x60)
    });
  }
}
