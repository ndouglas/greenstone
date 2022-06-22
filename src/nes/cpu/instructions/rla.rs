use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_rla(&mut self, opcode: &Opcode) {
    trace_enter!();
    let mode = &opcode.mode;
    trace_var!(mode);
    let length = opcode.length;
    trace_u8!(length);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    let operand = self.read_u8(address);
    trace_u8!(operand);
    let output = (operand << 1) | (self.status & CARRY_FLAG);
    trace_u8!(output);
    self.set_carry_flag(operand & NEGATIVE_FLAG != 0);
    self.write_u8(address, output);
    let answer = self.a & output;
    trace_u8!(answer);
    self.a = answer;
    self.set_value_flags(answer);
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
  fn test_rla() {
    init();
    test_instruction!("RLA", ZeroPage,  [0x02, 0x34]{status:0b00000001, a: 0x34} => [0x02, 0x69]{status: 0b00000000, a: 0x20});
    test_instruction!("RLA", ZeroPage,  [0x02, 0x34]{status:0b00000000, a: 0x34} => [0x02, 0x68]{status: 0b00000000, a: 0x20});
    test_instruction!("RLA", ZeroPage,  [0x02, 0x34]{status:0b00000000, a: 0x34} => [0x02, 0x68]{status: 0b00000000, a: 0x20});
    test_instruction!("RLA", ZeroPageX, [0x02, 0x00, 0x34]{status:0b00000001, x: 1, a: 0x34} => [0x02, 0x00, 0x69]{status: 0b00000000, a: 0x20});
    test_instruction!("RLA", Absolute,  [0x03, 0x00, 0x34]{status:0b00000001, a: 0x34} => [0x03, 0x00, 0x69]{status: 0b00000000, a: 0x20});
    test_instruction!("RLA", AbsoluteX, [0x03, 0x00, 0x00, 0x43]{status:0b00000001, x: 1, a: 0x34} => [0x03, 0x00, 0x00, 0x87]{status: 0b00000000, a: 0x04});
    test_instruction!("RLA", AbsoluteY, [0x03, 0x00, 0x00, 0x43]{status:0b00000001, y: 1, a: 0x34} => [0x03, 0x00, 0x00, 0x87]{status: 0b00000000, a: 0x04});
    test_instruction!("RLA", IndirectX, [0x02, 0x00, 0x05, 0x00, 0x43]{x:1, a: 0x34} => [0x02, 0x00, 0x05, 0x00, 0x86]{status: 0b00000000,a: 0x04});
    test_instruction!("RLA", IndirectY, [0x02, 0x04, 0x00, 0x00, 0x43]{y:1, a: 0x34} => [0x02, 0x04, 0x00, 0x00, 0x86]{status: 0b00000000,a: 0x04});
  }
}
