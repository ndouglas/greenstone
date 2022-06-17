use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_rol(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let mode = &opcode.mode;
    trace_var!(mode);
    if mode == &AddressingMode::Implied {
      return self.instruction_rol_2a(opcode);
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
    let output = (operand << 1) | (self.status & CARRY_FLAG);
    trace_u8!(output);
    self.set_carry_flag(operand & NEGATIVE_FLAG != 0);
    self.set_value_flags(output);
    self.write_u8(address, output);
    let result = cycles;
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn instruction_rol_2a(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let operand = self.a;
    trace_u8!(operand);
    let cycles = opcode.cycles;
    trace_u8!(cycles);
    let output = (operand << 1) | (self.status & CARRY_FLAG);
    trace_u8!(output);
    self.set_carry_flag(operand & NEGATIVE_FLAG != 0);
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
  fn test_rol() {
    init();
    test_instruction!("ROL", ZeroPage,  [0x02, 0xFF]{status:0b00000001} => [0x02, 0xFF]{status: 0b10000001});
    test_instruction!("ROL", ZeroPage,  [0x02, 0xFF]{status:0b00000000} => [0x02, 0xFE]{status: 0b10000001});
    test_instruction!("ROL", ZeroPage,  [0x02, 0x80]{status:0b00000000} => [0x02, 0x00]{status: 0b00000011});
    test_instruction!("ROL", ZeroPageX, [0x02, 0x00, 0xFF]{status:0b00000001, x: 1} => [0x02, 0x00, 0xFF]{status: 0b10000001});
    test_instruction!("ROL", Absolute,  [0x03, 0x00, 0xFF]{status:0b00000001} => [0x03, 0x00, 0xFF]{status: 0b10000001});
    test_instruction!("ROL", AbsoluteX, [0x03, 0x00, 0x00, 0xFF]{status:0b00000001, x: 1} => [0x03, 0x00, 0x00, 0xFF]{status: 0b10000001});
  }
}
