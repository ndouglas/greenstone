use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_lda(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_var!(opcode);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    let operand = self.read_u8(address);
    trace_u8!(operand);
    self.a = operand;
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
  fn test_lda() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("LDA", Immediate, [0x00]{} => []{ a: 0x00, status: 0b00000010 });
    test_instruction!("LDA", Immediate, [0xFF]{} => []{ a: 0xFF, status: 0b10000000 });
    test_instruction!("LDA", Immediate, [0x20]{} => []{ a: 0x20, status: 0 });
    test_instruction!("LDA", ZeroPage,  [0x02, 0x90]{} => []{ a: 0x90 });
    test_instruction!("LDA", ZeroPageX, [0x02, 0, 0x90]{x:1} => []{ a: 0x90 });
    test_instruction!("LDA", Absolute,  [0x04, 0, 0, 0x90]{x:1} => []{ a: 0x90 });
    test_instruction!("LDA", IndirectX, [0x02, 0, 0x05, 0, 0x90]{x:1} => []{ a: 0x90 });
    test_instruction!("LDA", IndirectY, [0x02, 0x04, 0, 0, 0x90]{y:1} => []{ a: 0x90 });
    test_instruction!("LDA", AbsoluteX, [0x03, 0, 0, 0x90]{x:1} => []{ a: 0x90 });
    test_instruction!("LDA", AbsoluteY, [0x03, 0, 0, 0x90]{y:1} => []{ a: 0x90 });
  }
}
