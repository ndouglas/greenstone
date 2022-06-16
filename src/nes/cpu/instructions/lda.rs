use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_lda(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let cycles = opcode.cycles;
    trace_u8!(cycles);
    let mode = &opcode.mode;
    trace_var!(mode);
    let (address, additional_cycles) = self.get_operand_address(mode).unwrap();
    trace_u16!(address);
    trace_u8!(additional_cycles);
    let operand = self.read_u8(address);
    trace_u8!(operand);
    self.a = operand;
    trace_u8!(self.a);
    self.set_value_flags(operand);
    let mut result = cycles;
    if opcode.extra_cycle {
      result += additional_cycles;
    }
    trace_result!(result);
    result
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
    test_instruction!("LDA", Immediate, [0x00]{} => []{ a: 0x00, status: 0b00000010 });
    test_instruction!("LDA", Immediate, [0xFF]{} => []{ a: 0xFF, status: 0b10000000 });
    test_instruction!("LDA", Immediate, [0x20]{} => []{ a: 0x20, status: 0 });
    test_instruction!("LDA", ZeroPage,  [0x02, 0x90]{} => []{ a: 0x90 });
    test_instruction!("LDA", ZeroPageX, [0x02, 0, 0x90]{x:1} => []{ a: 0x90 });
    test_instruction!("LDA", Absolute,  [0x04, 0, 0, 0x90]{x:1} => []{ a: 0x90 });
    test_instruction!("LDA", AbsoluteX, [0x03, 0, 0, 0x90]{x:1} => []{ a: 0x90 });
    test_instruction!("LDA", AbsoluteY, [0x03, 0, 0, 0x90]{y:1} => []{ a: 0x90 });
    test_instruction!("LDA", IndirectX, [0x02, 0, 0x05, 0, 0x90]{x:1} => []{ a: 0x90 });
    test_instruction!("LDA", IndirectY, [0x02, 0x04, 0, 0, 0x90]{y:1} => []{ a: 0x90 });
  }
}
