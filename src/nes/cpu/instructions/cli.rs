use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_cli(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    self.set_interrupt_disable_flag(false);
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
    test_instruction!("CLI", Implied, []{status: 0b11111111} => []{status: 0b11111011});
    test_instruction!("CLI", Implied, []{status: 0b11111011} => []{status: 0b11111011});
  }
}
