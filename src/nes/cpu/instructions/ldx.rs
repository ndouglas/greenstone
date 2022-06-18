use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_ldx(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    let operand = self.read_u8(address);
    trace_u8!(operand);
    self.x = operand;
    trace_u8!(self.x);
    self.set_value_flags(operand);
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_ldx() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("LDX", Immediate, [0x00]{}                 => []{ x: 0x00, status: 0b00000010 });
    test_instruction!("LDX", Immediate, [0xFF]{}                 => []{ x: 0xFF, status: 0b10000000 });
    test_instruction!("LDX", Immediate, [0x20]{}                 => []{ x: 0x20, status: 0b00000000 });
    test_instruction!("LDX", ZeroPage,  [0x02, 0x90]{}           => []{ x: 0x90 });
    test_instruction!("LDX", ZeroPageY, [0x02, 0x00, 0x90]{y:1}     => []{ x: 0x90 });
    test_instruction!("LDX", Absolute,  [0x04, 0x00, 0x00, 0x90]{}     => []{ x: 0x90 });
    test_instruction!("LDX", AbsoluteY, [0x03, 0x00, 0x00, 0x90]{y:1}  => []{ x: 0x90 });
  }
}
