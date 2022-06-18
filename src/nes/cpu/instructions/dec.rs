use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_dec(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    let operand = self.read_u8(address);
    trace_u8!(operand);
    let output = operand.wrapping_sub(1);
    trace_u8!(output);
    self.tick();
    self.write_u8(address, output);
    self.set_value_flags(output);
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
    test_instruction!("DEC", ZeroPage,  [0x02, 0x00]{} => [0x02, 0xFF]{status: 0b10000000});
    test_instruction!("DEC", ZeroPage,  [0x02, 0x01]{} => [0x02, 0x00]{status: 0b00000010});
    test_instruction!("DEC", ZeroPageX, [0x02, 0x00, 0x02]{x: 1} => [0x02, 0x00, 0x01]{});
    test_instruction!("DEC", Absolute,  [0x03, 0x00, 0x02]{} => [0x03, 0x00, 0x01]{});
    // test_instruction!("DEC", AbsoluteX, [0x03, 0x00, 0x00, 0x02]{x: 1} => [0x03, 0x00, 0x00, 0x01]{});
  }
}
