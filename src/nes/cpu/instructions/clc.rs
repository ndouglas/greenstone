use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_clc(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    self.set_carry_flag(false);
    debug!("Ticking (processing instruction)...");
    self.tick();
    trace_var!(self.get_carry_flag());
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_clc() {
    init();
    test_instruction!("CLC", Implied, []{status: 0b11111111} => []{status: 0b11111110});
    test_instruction!("CLC", Implied, []{status: 0b11111110} => []{status: 0b11111110});
  }
}
