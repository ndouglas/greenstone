use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_dey(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_var!(opcode);
    self.y = self.y.wrapping_sub(1);
    trace_u8!(self.y);
    self.tick();
    self.set_value_flags(self.y);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_dey() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("DEY", Implied,  []{y: 1} => []{y: 0, status: 0b00000010});
    test_instruction!("DEY", Implied,  []{y: 0} => []{y: 255, status: 0b10000000});
  }
}
