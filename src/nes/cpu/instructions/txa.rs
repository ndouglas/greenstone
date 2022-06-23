use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_txa(&mut self, _opcode: &Opcode) {
    trace_enter!();
    trace_var!(_opcode);
    self.a = self.x;
    self.tick();
    trace_u8!(self.a);
    self.set_value_flags(self.a);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_txa() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("TXA", Implied,  []{x: 1} => []{a: 1, x: 1, status: 0b00000000});
    test_instruction!("TXA", Implied,  []{x: 0} => []{a: 0, x: 0, status: 0b00000010});
    test_instruction!("TXA", Implied,  []{x: 128} => []{a: 128, x: 128, status: 0b10000000});
  }
}
