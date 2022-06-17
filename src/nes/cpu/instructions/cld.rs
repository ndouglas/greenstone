use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_cld(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let cycles = opcode.cycles;
    trace_u8!(cycles);
    self.set_decimal_flag(false);
    trace_var!(self.get_decimal_flag());
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
  fn test_cld() {
    init();
    test_instruction!("CLD", Implied, []{status: 0b11111111} => []{status: 0b11110111});
    test_instruction!("CLD", Implied, []{status: 0b11110111} => []{status: 0b11110111});
  }
}
