use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_rra(&mut self, opcode: &Opcode) {
    trace_enter!();
    let mode = &opcode.mode;
    trace_var!(mode);
    trace_var!(opcode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    let operand = self.read_u8(address);
    trace_u8!(operand);
    let output = (operand >> 1) | ((self.status & CARRY_FLAG) << 7);
    trace_u8!(output);
    self.set_carry_flag(operand & CARRY_FLAG != 0);
    // RMW dummy write - write original value before modified value
    self.write_u8(address, operand);
    self.write_u8(address, output);
    let (answer, set_carry, set_overflow) = add_u8s(self.a, output, self.get_carry_flag());
    trace_u8!(answer);
    self.a = answer;
    self.set_overflow_flag(set_overflow);
    self.set_carry_flag(set_carry);
    self.set_value_flags(answer);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_rra() {
    init();
    test_instruction!("RRA", ZeroPage,  [0x02, 0x34]{status: 0b00000001, a: 0x34} => [0x02, 0x9A]{status: 0b10000000, a: 0xCE});
    test_instruction!("RRA", ZeroPage,  [0x02, 0x34]{status: 0b00000000, a: 0x34} => [0x02, 0x1A]{status: 0b00000000, a: 0x4E});
    test_instruction!("RRA", ZeroPage,  [0x02, 0x34]{status: 0b00000000, a: 0x34} => [0x02, 0x1A]{status: 0b00000000, a: 0x4E});
    test_instruction!("RRA", ZeroPageX, [0x02, 0x00, 0x34]{status: 0b00000001, x: 1, a: 0x34} => [0x02, 0x00, 0x9A]{status: 0b10000000, a: 0xCE});
    test_instruction!("RRA", Absolute,  [0x03, 0x00, 0x34]{status: 0b00000001, a: 0x34} => [0x03, 0x00, 0x9A]{status: 0b10000000, a: 0xCE});
    test_instruction!("RRA", AbsoluteX, [0x03, 0x00, 0x00, 0x43]{status: 0b00000001, x: 1, a: 0x34} => [0x03, 0x00, 0x00, 0xA1]{status: 0b10000000, a: 0xD6});
    test_instruction!("RRA", AbsoluteY, [0x03, 0x00, 0x00, 0x43]{status: 0b00000001, y: 1, a: 0x34} => [0x03, 0x00, 0x00, 0xA1]{status: 0b10000000, a: 0xD6});
    test_instruction!("RRA", IndirectX, [0x02, 0x00, 0x05, 0x00, 0x43]{x: 1, a: 0x34} => [0x02, 0x00, 0x05, 0x00, 0x21]{status: 0b00000000, a: 0x56});
    test_instruction!("RRA", IndirectY, [0x02, 0x04, 0x00, 0x00, 0x43]{y: 1, a: 0x34} => [0x02, 0x04, 0x00, 0x00, 0x21]{status: 0b00000000, a: 0x56});
  }
}
