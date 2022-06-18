use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_txa(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    self.a = self.x;
    self.tick();
    trace_u8!(self.a);
    self.set_value_flags(self.a);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_txa() {
    init();
    test_instruction!("TXA", Implied,  []{x: 1} => []{a: 1, x: 1, status: 0b00000000});
    test_instruction!("TXA", Implied,  []{x: 0} => []{a: 0, x: 0, status: 0b00000010});
    test_instruction!("TXA", Implied,  []{x: 128} => []{a: 128, x: 128, status: 0b10000000});
  }
}
