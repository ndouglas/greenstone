use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_inc(&mut self, opcode: &Opcode) -> u8 {
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
    let output = operand.wrapping_add(1);
    trace_u8!(output);
    self.write_u8(address, output);
    self.set_value_flags(output);
    let result = cycles;
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
  fn test_inc() {
    init();
    test_instruction!("INC", ZeroPage,  [0x02, 255]{} => [0x02, 0]{status: 0b00000010});
    test_instruction!("INC", ZeroPage,  [0x02, 127]{} => [0x02, 128]{status: 0b10000000});
    test_instruction!("INC", ZeroPageX, [0x02, 0x00, 0x02]{x: 1} => [0x02, 0x00, 0x03]{});
    test_instruction!("INC", Absolute,  [0x03, 0x00, 0x02]{} => [0x03, 0x00, 0x03]{});
    test_instruction!("INC", AbsoluteX, [0x03, 0x00, 0x00, 0x02]{x: 1} => [0x03, 0x00, 0x00, 0x03]{});
  }
}
