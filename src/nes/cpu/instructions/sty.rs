use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_sty(&mut self, opcode: &Opcode) -> u8 {
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
    trace_u8!(self.y);
    self.write_u8(address, self.y);
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
  fn test_sty() {
    init();
    test_instruction!("STY", ZeroPage,  [0x02]{y: 0x66} => [0x02, 0x66]{});
    test_instruction!("STY", ZeroPageX, [0x02]{y: 0x66, x:1} => [0x02, 0x00, 0x66]{});
    test_instruction!("STY", Absolute,  [0x04, 0x00]{y: 0x66} => [0x04, 0x00, 0x00, 0x66]{});
  }
}
