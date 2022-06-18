use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_ora(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    let operand = self.read_u8(address);
    trace_u8!(operand);
    self.a = self.a | operand;
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
  fn test_ora() {
    init();
    test_instruction!("ORA", Immediate, [0b00001111]{a:0b01010101} => []{ a: 0b01011111, status: 0b00000000 });
    test_instruction!("ORA", Immediate, [0b10001111]{a:0b01010101} => []{ a: 0b11011111, status: 0b10000000 });
    test_instruction!("ORA", Immediate, [0x00]{a:0} => []{ a: 0x00, status: 0b00000010 });
    test_instruction!("ORA", ZeroPage, [0x02, 0xFF]{a: 0xF0} => []{a: 0xFF});
    // test_instruction!("ORA", ZeroPageX, [0x02, 0x00, 0xFF]{x:1, a: 0xF0} => []{a: 0xFF});
    // test_instruction!("ORA", Absolute, [0x04, 0x00, 0x00, 0xFF]{a:0xF0} => []{a: 0xFF});
    // test_instruction!("ORA", AbsoluteX, [0x03, 0x00, 0x00, 0xFF]{x:1, a: 0xF0} => []{a: 0xFF});
    // test_instruction!("ORA", AbsoluteY, [0x03, 0x00, 0x00, 0xFF]{y:1, a: 0xF0} => []{a: 0xFF});
    // test_instruction!("ORA", IndirectX, [0x02, 0x00, 0x05, 0x00, 0xFF]{x:1, a: 0xF0} => []{a: 0xFF});
    // test_instruction!("ORA", IndirectY, [0x02, 0x04, 0x00, 0x00, 0xFF]{y:1, a: 0xF0} => []{a: 0xFF});
  }
}
