use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_tay(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    self.y = self.a;
    self.tick();
    trace_var!(self.y);
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
  fn test_tay() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("TAY", Implied,  []{a: 1} => []{a: 1, y: 1, status: 0b00000000});
    test_instruction!("TAY", Implied,  []{a: 0} => []{a: 0, y: 0, status: 0b00000010});
    test_instruction!("TAY", Implied,  []{a: 128} => []{a: 128, y: 128, status: 0b10000000});
  }
}
