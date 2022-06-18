use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_ldy(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    let operand = self.read_u8(address);
    trace_u8!(operand);
    self.y = operand;
    trace_u8!(self.y);
    self.set_value_flags(operand);
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_ldy() {
    init();
    test_instruction!("LDY", Immediate, [0x00]{}                 => []{ y: 0x00, status: 0b00000010 });
    test_instruction!("LDY", Immediate, [0xFF]{}                 => []{ y: 0xFF, status: 0b10000000 });
    test_instruction!("LDY", Immediate, [0x20]{}                 => []{ y: 0x20, status: 0b00000000 });
    test_instruction!("LDY", ZeroPage,  [0x02, 0x90]{}           => []{ y: 0x90 });
    test_instruction!("LDY", ZeroPageX, [0x02, 0x00, 0x90]{x:1}     => []{ y: 0x90 });
    test_instruction!("LDY", Absolute,  [0x04, 0x00, 0x00, 0x90]{}     => []{ y: 0x90 });
    // test_instruction!("LDY", AbsoluteX, [0x03, 0x00, 0x00, 0x90]{x:1}  => []{ y: 0x90 });
  }
}
