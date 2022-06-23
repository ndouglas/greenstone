use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_cli(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_var!(opcode);
    self.set_interrupt_disable_flag(false);
    debug!("Ticking (processing instruction)...");
    self.tick();
    trace_var!(self.get_interrupt_disable_flag());
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_cli() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("CLI", Implied, []{status: 0b11111111} => []{status: 0b11111011});
    test_instruction!("CLI", Implied, []{status: 0b11111011} => []{status: 0b11111011});
  }
}
