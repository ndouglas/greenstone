use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_ror(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    trace_u8!(self.status);
    let mode = &opcode.mode;
    trace_var!(mode);
    if mode == &AddressingMode::Implied {
      return self.instruction_ror_6a(opcode);
    }
    let length = opcode.length;
    trace_u8!(length);
    let cycles = opcode.cycles;
    trace_u8!(cycles);
    let (address, additional_cycles) = self.get_operand_address(mode).unwrap();
    trace_u16!(address);
    trace_u8!(additional_cycles);
    let operand = self.read_u8(address);
    trace_u8!(operand);
    let output = (operand >> 1) | ((self.status & CARRY_FLAG) << 7);
    trace_u8!(output);
    self.set_carry_flag(operand & CARRY_FLAG != 0);
    self.set_value_flags(output);
    self.write_u8(address, output);
    let result = cycles;
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn instruction_ror_6a(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let operand = self.a;
    trace_u8!(operand);
    let cycles = opcode.cycles;
    trace_u8!(cycles);
    let output = (operand >> 1) | ((self.status & CARRY_FLAG) << 7);
    trace_u8!(output);
    self.set_carry_flag(operand & CARRY_FLAG != 0);
    self.set_value_flags(output);
    self.a = output;
    let result = cycles;
    trace_result!(result);
    result
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_ror() {
    init();
    test_instruction!("ROR", ZeroPage,  [0x02, 0xFF]{status:0b00000001} => [0x02, 0xFF]{status: 0b10000001});
    test_instruction!("ROR", ZeroPage,  [0x02, 0xFF]{status:0b00000000} => [0x02, 0x7F]{status: 0b00000001});
    test_instruction!("ROR", ZeroPage,  [0x02, 0x01]{status:0b00000000} => [0x02, 0x00]{status: 0b00000011});
    test_instruction!("ROR", ZeroPageX,  [0x02, 0x00, 0x01]{status:0b00000000, x: 1} => [0x02, 0x00]{status: 0b00000011});
    test_instruction!("ROR", Absolute,  [0x03, 0x00, 0x01]{status:0b00000000} => [0x03, 0x00]{status: 0b00000011});
    test_instruction!("ROR", AbsoluteX,  [0x02, 0x00, 0x01]{status:0b00000000, x: 1} => [0x02, 0x00]{status: 0b00000011});
    test_instruction!("ROR", Implied, []{a: 2} => []{a: 1});
  }
}
