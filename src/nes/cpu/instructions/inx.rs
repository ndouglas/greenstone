use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_inx(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    self.set_carry_flag(false);
    self.x = self.x.wrapping_add(1);
    self.tick();
    trace_u8!(self.x);
    self.set_value_flags(self.x);
    trace_exit!();
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
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("INX", Implied,  []{x: 255} => []{x: 0, status: 0b00000010});
    test_instruction!("INX", Implied,  []{x: 127} => []{x: 128, status: 0b10000000});
  }
}
