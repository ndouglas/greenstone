use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_clv(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    self.set_overflow_flag(false);
    debug!("Ticking (processing instruction)...");
    self.tick();
    trace_var!(self.get_overflow_flag());
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_clv() {
    init();
    test_instruction!("CLV", Implied, []{status: 0b11111111} => []{status: 0b10111111});
    test_instruction!("CLV", Implied, []{status: 0b10111111} => []{status: 0b10111111});
  }
}
