use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_sre(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_u8!(self.status);
    let mode = &opcode.mode;
    trace_var!(mode);
    trace_var!(opcode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    let operand = self.read_u8(address);
    trace_u8!(operand);
    let output = operand >> 1;
    trace_u8!(output);
    self.tick();
    self.write_u8(address, output);
    let answer = self.a ^ output;
    trace_u8!(answer);
    self.set_value_flags(answer);
    self.set_carry_flag(operand & CARRY_FLAG != 0);
    self.a = answer;
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_sre() {
    init();
    test_instruction!("SRE", ZeroPage,  [0x02, 0xEA]{status:1, a: 0x34} => [0x02, 0x75]{status: 0b00000000, a: 0x41});
    test_instruction!("SRE", ZeroPage,  [0x02, 0xEA]{status:0, a: 0x34} => [0x02, 0x75]{status: 0b00000000, a: 0x41});
    test_instruction!("SRE", ZeroPageX, [0x02, 0x00, 0xEA]{x: 1, a: 0x34} => [0x02, 0x00, 0x75]{status: 0b00000000, a: 0x41});
    test_instruction!("SRE", Absolute,  [0x03, 0x00, 0xEA]{a: 0x34} => [0x03, 0x00, 0x75]{status: 0b00000000, a: 0x41});
    test_instruction!("SRE", AbsoluteX, [0x03, 0x00, 0x00, 0xEA]{x: 1, a: 0x34} => [0x03, 0x00, 0x00, 0x75]{status: 0b00000000, a: 0x41});
    test_instruction!("SRE", AbsoluteY, [0x03, 0x00, 0x00, 0xEA]{y: 1, a: 0x34} => [0x03, 0x00, 0x00, 0x75]{status: 0b00000000, a: 0x41});
    test_instruction!("SRE", IndirectX, [0x02, 0x00, 0x05, 0x00, 0xEA]{x:1, a:0x34} => [0x02, 0x00, 0x05, 0x00, 0x75]{status: 0b00000000, a: 0x41});
    test_instruction!("SRE", IndirectY, [0x02, 0x04, 0x00, 0x00, 0xEA]{y:1, a:0x34} => [0x02, 0x04, 0x00, 0x00, 0x75]{status: 0b00000000, a: 0x41});
  }
}
