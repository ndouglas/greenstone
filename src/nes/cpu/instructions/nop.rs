use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_nop(&mut self, _opcode: &Opcode) {
    trace_enter!();
    debug!("Ticking (NOP)...");
    self.tick();
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_nop() {
    init();
    test_instruction!("NOP", Implied, []{} => []{});
  }
}
