use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_rts(&mut self, _opcode: &Opcode) {
    trace_enter!();
    debug!("Ticking (processing instruction)...");
    self.tick();
    debug!("Ticking (processing instruction)...");
    self.tick();
    // Address stored on the stack is (address - 1).
    let address = self.pop_u16().wrapping_add(1);
    trace_u16!(address);
    self.program_counter = address;
    debug!("Ticking (processing instruction)...");
    self.tick();
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
    test_instruction!("JSR", Absolute, [0x0A, 0x00]{} => []{clock_counter: 6, program_counter: 10, stack_pointer: 0xFD}, |cpu: &mut CPU<'_>, _opcode: &Opcode| {
      // Write an RTS at the destination instruction.
      cpu.unclocked_write_u16(0x000A, 0x60)
    });
  }
}
