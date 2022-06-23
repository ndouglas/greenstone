use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_lax(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_var!(opcode);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    let operand = self.read_u8(address);
    trace_u8!(operand);
    self.a = operand;
    self.x = operand;
    trace_u8!(self.a);
    self.set_value_flags(operand);
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_lax() {
    init();
    test_instruction!("LAX", ZeroPage,  [0x02, 0x90]{} => []{ a: 0x90, x: 0x90 });
    test_instruction!("LAX", ZeroPageY, [0x02, 0, 0x90]{y:1} => []{ a: 0x90, x: 0x90 });
    test_instruction!("LAX", Absolute,  [0x04, 0, 0, 0x90]{} => []{ a: 0x90, x: 0x90 });
    test_instruction!("LAX", IndirectX, [0x02, 0, 0x05, 0, 0x90]{x:1} => []{ a: 0x90, x: 0x90 });
    test_instruction!("LAX", IndirectY, [0x02, 0x04, 0, 0, 0x90]{y:1} => []{ a: 0x90, x: 0x90 });
    test_instruction!("LAX", AbsoluteY, [0x03, 0, 0, 0x90]{y:1} => []{ a: 0x90, x: 0x90 });
  }
}
