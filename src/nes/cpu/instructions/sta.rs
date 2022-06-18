use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_sta(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    trace_u8!(self.a);
    self.write_u8(address, self.a);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_sta() {
    init();
    test_instruction!("STA", ZeroPage,  [0x02]{a: 0x66} => [0x02, 0x66]{});
    test_instruction!("STA", ZeroPageX, [0x02]{a: 0x66, x:1} => [0x02, 0x00, 0x66]{});
    test_instruction!("STA", Absolute,  [0x04, 0x00]{a:0x66} => [0x04, 0x00, 0x00, 0x66]{});
    // test_instruction!("STA", AbsoluteX, [0x03, 0x00]{a:0x66, x:1} => [0x03, 0x00, 0x00, 0x66]{});
    // test_instruction!("STA", AbsoluteY, [0x03, 0x00]{a:0x66, y:1} => [0x03, 0x00, 0x00, 0x66]{});
    // test_instruction!("STA", IndirectX, [0x02, 0x00, 0x05, 0x00, 0x00]{a: 0x66, x:1} => [0x02, 0x00, 0x05, 0x00, 0x66]{});
    // test_instruction!("STA", IndirectY, [0x02, 0x04, 0x00, 0x00, 0x00]{a: 0x66, y:1} => [0x02, 0x04, 0x00, 0x00, 0x66]{});
  }
}
