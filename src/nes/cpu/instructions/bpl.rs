use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_bpl(&mut self, opcode: &Opcode) {
    trace_enter!();
    self.branch_on_condition(opcode, !self.get_negative_flag());
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_bpl() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("BPL", Relative, [0x10]{status: 0b10000000} => []{program_counter: 2, clock_counter: 2});
    test_instruction!("BPL", Relative, [0x10]{status: 0b00000000} => []{program_counter: 18, clock_counter: 3});
    test_instruction!("BPL", Relative, [0x00]{status: 0b00000000} => []{program_counter: 0x0104, clock_counter: 4}, |cpu: &mut CPU<'_>, _opcode: &Opcode| {
      cpu.program_counter = 0x00FD;
      cpu.unclocked_write_u8(0x00FD, 0x10);
      cpu.unclocked_write_u16(0x00FE, 5);
    });
    test_instruction!("BPL", Relative, [0x00]{status: 0b00000000} => []{program_counter: 0x0101, clock_counter: 4}, |cpu: &mut CPU<'_>, _opcode: &Opcode| {
      cpu.program_counter = 0x00FD;
      cpu.unclocked_write_u8(0x00FD, 0x10);
      cpu.unclocked_write_u16(0x00FE, 2);
    });
    test_instruction!("BPL", Relative, [0x00]{status: 0b10000000} => []{program_counter: 0x00FF, clock_counter: 2}, |cpu: &mut CPU<'_>, _opcode: &Opcode| {
      cpu.program_counter = 0x00FD;
      cpu.unclocked_write_u8(0x00FD, 0x10);
      cpu.unclocked_write_u16(0x00FE, 5);
    });
  }
}
