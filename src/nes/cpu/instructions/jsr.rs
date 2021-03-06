use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_jsr(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_var!(opcode);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    debug!("Ticking (processing instruction)...");
    self.tick();
    let return_address = self.program_counter.wrapping_sub(1);
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
    test_instruction!("JSR", Absolute, [0x0A, 0x00]{} => []{clock_counter: 6, program_counter: 10, stack_pointer: 0xFB}, |cpu: &mut CPU, _opcode: &Opcode| {
      // Write an RTS at the destination instruction.
      cpu.unclocked_write_u16(0x000A, 0x60)
    });
  }
}
