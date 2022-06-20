use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_sty(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    trace_u8!(self.y);
    self.write_u8(address, self.y);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_sty() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("STY", ZeroPage,  [0x02]{y: 0x66} => [0x02, 0x66]{});
    test_instruction!("STY", ZeroPageX, [0x02]{y: 0x66, x:1} => [0x02, 0x00, 0x66]{});
    test_instruction!("STY", Absolute,  [0x04, 0x00]{y: 0x66} => [0x04, 0x00, 0x00, 0x66]{});
  }
}
