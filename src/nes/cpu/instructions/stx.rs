use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_stx(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    trace_u8!(self.x);
    self.write_u8(address, self.x);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_stx() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("STX", ZeroPage,  [0x02]{x: 0x66} => [0x02, 0x66]{});
    test_instruction!("STX", ZeroPageY, [0x02]{x: 0x66, y:1} => [0x02, 0x00, 0x66]{});
    test_instruction!("STX", Absolute,  [0x04, 0x00]{x: 0x66} => [0x04, 0x00, 0x00, 0x66]{});
  }
}
