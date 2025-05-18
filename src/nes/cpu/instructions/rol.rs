use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_rol(&mut self, opcode: &Opcode) {
    trace_enter!();
    let mode = &opcode.mode;
    trace_var!(mode);
    trace_var!(opcode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    let operand = self.read_u8(address);
    trace_u8!(operand);
    let output = (operand << 1) | (self.status & CARRY_FLAG);
    trace_u8!(output);
    self.set_carry_flag(operand & NEGATIVE_FLAG != 0);
    self.set_value_flags(output);
    // RMW dummy write - write original value before modified value
    self.write_u8(address, operand);
    self.write_u8(address, output);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn instruction_rol_2a(&mut self, _opcode: &Opcode) {
    trace_enter!();
    let operand = self.a;
    trace_u8!(operand);
    let output = (operand << 1) | (self.status & CARRY_FLAG);
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
  fn test_rol() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("ROL", ZeroPage,  [0x02, 0xFF]{status:0b00000001} => [0x02, 0xFF]{status: 0b10000001});
    test_instruction!("ROL", ZeroPage,  [0x02, 0xFF]{status:0b00000000} => [0x02, 0xFE]{status: 0b10000001});
    test_instruction!("ROL", ZeroPage,  [0x02, 0x80]{status:0b00000000} => [0x02, 0x00]{status: 0b00000011});
    test_instruction!("ROL", ZeroPageX, [0x02, 0x00, 0xFF]{status:0b00000001, x: 1} => [0x02, 0x00, 0xFF]{status: 0b10000001});
    test_instruction!("ROL", Absolute,  [0x03, 0x00, 0xFF]{status:0b00000001} => [0x03, 0x00, 0xFF]{status: 0b10000001});
    test_instruction!("ROL", AbsoluteX, [0x03, 0x00, 0x00, 0xFF]{status:0b00000001, x: 1} => [0x03, 0x00, 0x00, 0xFF]{status: 0b10000001});
  }
}
