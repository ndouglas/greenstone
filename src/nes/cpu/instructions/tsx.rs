use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_tsx(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_u8!(opcode.length);
    self.x = self.stack_pointer;
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
  fn test_tsx() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("TSX", Implied,  []{stack_pointer: 1} => []{stack_pointer: 1, x: 1, status: 0b00000000});
    test_instruction!("TSX", Implied,  []{stack_pointer: 0} => []{stack_pointer: 0, x: 0, status: 0b00000010});
    test_instruction!("TSX", Implied,  []{stack_pointer: 128} => []{stack_pointer: 128, x: 128, status: 0b10000000});
  }
}
