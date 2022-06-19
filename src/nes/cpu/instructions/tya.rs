use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_tya(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_u8!(opcode.length);
    self.a = self.y;
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
  fn test_tya() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("TYA", Implied,  []{y: 1} => []{a: 1, y: 1, status: 0b00000000});
    test_instruction!("TYA", Implied,  []{y: 0} => []{a: 0, y: 0, status: 0b00000010});
    test_instruction!("TYA", Implied,  []{y: 128} => []{a: 128, y: 128, status: 0b10000000});
  }
}
