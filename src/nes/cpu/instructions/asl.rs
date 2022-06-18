use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_asl(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_u8!(self.status);
    let mode = &opcode.mode;
    trace_var!(mode);
    if mode == &AddressingMode::Implied {
      return self.instruction_asl_0a(opcode);
    }
    let length = opcode.length;
    trace_u8!(length);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    let operand = self.read_u8(address);
    trace_u8!(operand);
    let output = operand << 1;
    trace_u8!(output);
    self.set_carry_flag(operand & NEGATIVE_FLAG != 0);
    self.tick();
    self.set_value_flags(output);
    self.write_u8(address, output);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn instruction_asl_0a(&mut self, _opcode: &Opcode) {
    trace_enter!();
    let operand = self.a;
    trace_u8!(operand);
    let output = operand << 1;
    trace_u8!(output);
    self.set_carry_flag(operand & NEGATIVE_FLAG != 0);
    self.set_value_flags(output);
    self.a = output;
    self.tick();
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_asl() {
    init();
    test_instruction!("ASL", ZeroPage,  [0x02, 0xFF]{status:1} => [0x02, 0xFE]{status: 0b10000001});
    test_instruction!("ASL", ZeroPage,  [0x02, 0xFF]{status:0} => [0x02, 0xFE]{status: 0b10000001});
    test_instruction!("ASL", ZeroPage,  [0x02, 0b10000000]{} => [0x02, 0x00]{status: 0b00000011});
    test_instruction!("ASL", ZeroPageX, [0x02, 0x00, 0x01]{x: 1} => [0x02, 0x00, 0x02]{});
    test_instruction!("ASL", Absolute,  [0x03, 0x00, 0x01]{} => [0x03, 0x00, 0x02]{});
    // test_instruction!("ASL", AbsoluteX, [0x03, 0x00, 0x00, 0x01]{x: 1} => [0x03, 0x00, 0x00, 0x02]{});
    test_instruction!("ASL", Implied, []{a: 1} => []{a: 2});
  }
}
