use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_dec(&mut self, opcode: &Opcode) {
    trace_enter!();
    self.decrement_u8(opcode);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_dec() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("DEC", ZeroPage,  [0x02, 0x00]{} => [0x02, 0xFF]{status: 0b10000000});
    test_instruction!("DEC", ZeroPage,  [0x02, 0x01]{} => [0x02, 0x00]{status: 0b00000010});
    test_instruction!("DEC", ZeroPageX, [0x02, 0x00, 0x02]{x: 1} => [0x02, 0x00, 0x01]{});
    test_instruction!("DEC", Absolute,  [0x03, 0x00, 0x02]{} => [0x03, 0x00, 0x01]{});
    test_instruction!("DEC", AbsoluteX, [0x03, 0x00, 0x00, 0x02]{x: 1} => [0x03, 0x00, 0x00, 0x01]{});
  }
}
