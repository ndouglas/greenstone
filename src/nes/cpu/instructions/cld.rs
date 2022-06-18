use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_cld(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    self.set_decimal_flag(false);
    self.tick();
    trace_var!(self.get_decimal_flag());
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_cld() {
    init();
    test_instruction!("CLD", Implied, []{status: 0b11111111} => []{status: 0b11110111});
    test_instruction!("CLD", Implied, []{status: 0b11110111} => []{status: 0b11110111});
  }
}
