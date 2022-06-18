use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_bne(&mut self, opcode: &Opcode) {
    trace_enter!();
    self.branch_on_condition(opcode, !self.get_zero_flag());
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_bne() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("BNE", Relative, [0x10]{status: 0b00000010} => []{program_counter: 2, clock_counter: 2});
    test_instruction!("BNE", Relative, [0x10]{status: 0b00000000} => []{program_counter: 17, clock_counter: 3});
  }
}
