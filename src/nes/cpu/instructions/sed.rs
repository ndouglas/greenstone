use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_sed(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_var!(opcode);
    self.set_decimal_flag(true);
    self.tick();
    trace_var!(self.get_decimal_flag());
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_sed() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("SED", Implied, []{status: 0b00000000} => []{status: 0b00001000});
    test_instruction!("SED", Implied, []{status: 0b00001000} => []{status: 0b00001000});
  }
}
