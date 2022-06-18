use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_bmi(&mut self, opcode: &Opcode) {
    trace_enter!();
    self.branch_on_condition(opcode, self.get_negative_flag());
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_bmi() {
    init();
    test_instruction!("BMI", Relative, [0x10]{status: 0b00000000} => []{program_counter: 2, clock_counter: 2});
    test_instruction!("BMI", Relative, [0x10]{status: 0b10000000} => []{program_counter: 17, clock_counter: 3});
    test_instruction!("BMI", Relative, [0x00]{status: 0b10000000} => []{program_counter: 0x0103, clock_counter: 4}, |cpu: &mut CPU<'_>| {
      cpu.program_counter = 0x00FD;
      cpu.unclocked_write_u8(0x00FD, 0x30);
      cpu.unclocked_write_u16(0x00FE, 5);
    });
    test_instruction!("BMI", Relative, [0x00]{status: 0b10000000} => []{program_counter: 0x0100, clock_counter: 4}, |cpu: &mut CPU<'_>| {
      cpu.program_counter = 0x00FD;
      cpu.unclocked_write_u8(0x00FD, 0x30);
      cpu.unclocked_write_u16(0x00FE, 2);
    });
    test_instruction!("BMI", Relative, [0x00]{status: 0b00000000} => []{program_counter: 0x00FF, clock_counter: 2}, |cpu: &mut CPU<'_>| {
      cpu.program_counter = 0x00FD;
      cpu.unclocked_write_u8(0x00FD, 0x30);
      cpu.unclocked_write_u16(0x00FE, 5);
    });
  }
}
