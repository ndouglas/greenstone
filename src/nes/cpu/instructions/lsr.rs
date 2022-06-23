use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_lsr(&mut self, opcode: &Opcode) {
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
    self.set_carry_flag(operand & CARRY_FLAG != 0);
    self.tick();
    self.set_value_flags(output);
    self.write_u8(address, output);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn instruction_lsr_4a(&mut self, _opcode: &Opcode) {
    trace_enter!();
    let operand = self.a;
    trace_u8!(operand);
    let output = operand >> 1;
    trace_u8!(output);
    self.set_carry_flag(operand & CARRY_FLAG != 0);
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
  fn test_lsr() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("LSR", Implied, []{a: 2} => []{a: 1});
    test_instruction!("LSR", ZeroPage,  [0x02, 0x01]{status:1} => [0x02, 0x00]{status: 0b00000011});
    test_instruction!("LSR", ZeroPage,  [0x02, 0x01]{status:0} => [0x02, 0x00]{status: 0b00000011});
    test_instruction!("LSR", ZeroPageX, [0x02, 0x00, 0x02]{x: 1} => [0x02, 0x00, 0x01]{});
    test_instruction!("LSR", Absolute,  [0x03, 0x00, 0x02]{} => [0x03, 0x00, 0x01]{});
    test_instruction!("LSR", AbsoluteX, [0x03, 0x00, 0x00, 0x02]{x: 1} => [0x03, 0x00, 0x00, 0x01]{});
  }
}
