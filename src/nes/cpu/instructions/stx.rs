use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_stx(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let cycles = opcode.cycles;
    trace_u8!(cycles);
    let mode = &opcode.mode;
    trace_var!(mode);
    let (address, additional_cycles) = self.get_operand_address(mode).unwrap();
    trace_u16!(address);
    trace_var!(additional_cycles);
    trace_u8!(self.x);
    self.write_u8(address, self.x);
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
  fn test_stx() {
    test_instruction!("STX", ZeroPage,  [0x02]{x: 0x66} => [0x02, 0x66]{});
    test_instruction!("STX", ZeroPageY, [0x02]{x: 0x66, y:1} => [0x02, 0x00, 0x66]{});
    test_instruction!("STX", Absolute,  [0x04, 0x00]{x: 0x66} => [0x04, 0x00, 0x00, 0x66]{});
  }
}
