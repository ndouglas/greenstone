use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_bvc(&mut self, opcode: &Opcode) {
    trace_enter!();
    self.branch_on_condition(opcode, !self.get_overflow_flag());
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_bvc() {
    init();
    test_instruction!("BVC", Relative, [0x10]{status: 0b01000000} => []{program_counter: 2, clock_counter: 2});
    test_instruction!("BVC", Relative, [0x10]{status: 0b00000000} => []{program_counter: 17, clock_counter: 3});
  }
}