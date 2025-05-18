use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_slo(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_u8!(self.status);
    let mode = &opcode.mode;
    trace_var!(mode);
    trace_var!(opcode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    debug!("Ticking (reading operand)...");
    let operand = self.read_u8(address);
    trace_u8!(operand);
    let output = operand << 1;
    trace_u8!(output);
    self.set_carry_flag(operand & NEGATIVE_FLAG != 0);
    self.set_value_flags(output);
    // RMW dummy write - write original value before modified value
    self.write_u8(address, operand);
    self.write_u8(address, output);
    self.a = self.a | output;
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
  fn test_slo() {
    init();
    test_instruction!("SLO", ZeroPage,  [0x02, 0xFF]{status:1, a:0xEA} => [0x02, 0xFE]{status: 0b10000001, a: 0xFE});
    test_instruction!("SLO", ZeroPage,  [0x02, 0xFF]{status:0, a:0xEA} => [0x02, 0xFE]{status: 0b10000001, a: 0xFE});
    test_instruction!("SLO", ZeroPage,  [0x02, 0xEA]{a:0xEA} => [0x02, 0xD4]{status: 0b10000001, a: 0xFE});
    test_instruction!("SLO", ZeroPageX, [0x02, 0x00, 0x01]{a:0xEA, x: 1} => [0x02, 0x00, 0x02]{status: 0b10000000, a: 0xEA});
    test_instruction!("SLO", Absolute,  [0x03, 0x00, 0x01]{a:0xEA} => [0x03, 0x00, 0x02]{status: 0b10000000, a: 0xEA});
    test_instruction!("SLO", AbsoluteX, [0x03, 0x00, 0x00, 0x01]{a:0xEA, x: 1} => [0x03, 0x00, 0x00, 0x02]{status: 0b10000000, a: 0xEA});
    test_instruction!("SLO", AbsoluteY, [0x03, 0x00, 0x00, 0x01]{a:0xEA, y: 1} => [0x03, 0x00, 0x00, 0x02]{status: 0b10000000, a: 0xEA});
    test_instruction!("SLO", IndirectX, [0x02, 0x00, 0x05, 0x00, 0xFF]{x:1, a:0xEA} => [0x02, 0x00, 0x05, 0x00, 0xFE]{status: 0b10000001, a: 0xFE});
    test_instruction!("SLO", IndirectY, [0x02, 0x04, 0x00, 0x00, 0xFF]{y:1, a:0xEA} => [0x02, 0x04, 0x00, 0x00, 0xFE]{status: 0b10000001, a: 0xFE});
  }
}
