use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_sec(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let cycles = opcode.cycles;
    trace_u8!(cycles);
    self.set_carry_flag(true);
    trace_var!(self.get_carry_flag());
    let result = cycles;
    trace_result!(result);
    result
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_sec() {
    init();
    test_instruction!("SEC", Implied, []{status: 0b11111110} => []{status: 0b11111111});
    test_instruction!("SEC", Implied, []{status: 0b11111111} => []{status: 0b11111111});
  }
}
