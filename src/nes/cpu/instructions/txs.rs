use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_txs(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_u8!(opcode.length);
    self.stack_pointer = self.x;
    self.tick();
    trace_u8!(self.stack_pointer);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_txs() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("TXS", Implied,  []{x: 1} => []{stack_pointer: 1, x: 1, status: 0b00000000});
    test_instruction!("TXS", Implied,  []{x: 255} => []{stack_pointer: 255, x: 255, status: 0b00000000});
  }
}
