use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_iny(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    self.set_carry_flag(false);
    self.y = self.y.wrapping_add(1);
    self.tick();
    trace_u8!(self.y);
    self.set_value_flags(self.y);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_iny() {
    init();
    test_instruction!("INY", Implied,  []{y: 255} => []{y: 0, status: 0b00000010});
    test_instruction!("INY", Implied,  []{y: 127} => []{y: 128, status: 0b10000000});
  }
}
