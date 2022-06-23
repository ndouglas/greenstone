use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_sax(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_var!(opcode);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    let output = self.a & self.x;
    self.write_u8(address, output);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_sax() {
    init();
    test_instruction!("SAX", ZeroPage,  [0x02]{a: 0x66, x: 0x63} => [0x02, 0x62]{});
    test_instruction!("SAX", ZeroPageY, [0x02]{a: 0x66, x: 0x63, y:1} => [0x02, 0x00, 0x62]{});
    test_instruction!("SAX", Absolute,  [0x04, 0x00]{a:0x66, x:0x63} => [0x04, 0x00, 0x00, 0x62]{});
    test_instruction!("SAX", IndirectX, [0x02, 0x00, 0x05, 0x00, 0x00]{a: 0x65, x:0x01} => [0x02, 0x00, 0x05, 0x00, 0x01]{});
  }
}
