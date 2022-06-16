use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_inx(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let cycles = opcode.cycles;
    trace_u8!(cycles);
    self.set_carry_flag(false);
    self.x = self.x.wrapping_add(1);
    trace_u8!(self.x);
    self.set_value_flags(self.x);
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
  fn test_inx() {
    init();
    test_instruction!("INX", Implied,  []{x: 255} => []{x: 0, status: 0b00000010});
    test_instruction!("INX", Implied,  []{x: 127} => []{x: 128, status: 0b10000000});
  }
}
