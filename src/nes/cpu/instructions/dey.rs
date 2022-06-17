use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_dey(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let cycles = opcode.cycles;
    trace_u8!(cycles);
    self.set_carry_flag(false);
    self.y = self.y.wrapping_sub(1);
    trace_u8!(self.y);
    self.set_value_flags(self.y);
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
  fn test_dey() {
    init();
    test_instruction!("DEY", Implied,  []{y: 1} => []{y: 0, status: 0b00000010});
    test_instruction!("DEY", Implied,  []{y: 0} => []{y: 255, status: 0b10000000});
  }
}
