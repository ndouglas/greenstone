use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_sed(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let cycles = opcode.cycles;
    trace_u8!(cycles);
    self.set_decimal_flag(true);
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
  fn test_sed() {
    init();
    test_instruction!("SED", Implied, []{status: 0b00000000} => []{status: 0b00001000});
    test_instruction!("SED", Implied, []{status: 0b00001000} => []{status: 0b00001000});
  }
}
