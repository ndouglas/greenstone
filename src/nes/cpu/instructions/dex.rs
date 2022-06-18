use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_dex(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    self.set_carry_flag(false);
    self.x = self.x.wrapping_sub(1);
    trace_u8!(self.x);
    self.tick();
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
  fn test_dex() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("DEX", Implied,  []{x: 1} => []{x: 0, status: 0b00000010});
    test_instruction!("DEX", Implied,  []{x: 0} => []{x: 255, status: 0b10000000});
  }
}
