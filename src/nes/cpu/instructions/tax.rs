use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_tax(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_u8!(opcode.length);
    self.x = self.a;
    self.tick();
    trace_u8!(self.x);
    self.set_value_flags(self.x);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_tax() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("TAX", Implied,  []{a: 1} => []{a: 1, x: 1, status: 0b00000000});
    test_instruction!("TAX", Implied,  []{a: 0} => []{a: 0, x: 0, status: 0b00000010});
    test_instruction!("TAX", Implied,  []{a: 128} => []{a: 128, x: 128, status: 0b10000000});
  }
}
