use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_cpy(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let cycles = opcode.cycles;
    let mode = &opcode.mode;
    trace_var!(mode);
    let (address, additional_cycles) = self.get_operand_address(mode).unwrap();
    trace_u16!(address);
    trace_u8!(additional_cycles);
    let operand = self.read_u8(address);
    trace_u8!(operand);
    self.set_value_flags(self.y.wrapping_sub(operand));
    self.set_carry_flag(self.y >= operand);
    trace_var!(self.get_carry_flag());
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
  fn test_cpx() {
    init();
    test_instruction!("CPY", Immediate, [10]{y:10} => []{ status: 0b00000011 });
    test_instruction!("CPY", Immediate, [100]{y:10} => []{ status: 0b10000000 });
    test_instruction!("CPY", Immediate, [10]{y:100} => []{ status: 0b00000001 });
    test_instruction!("CPY", ZeroPage, [0x02, 10]{y: 10} => []{ status: 0b00000011 });
    test_instruction!("CPY", Absolute, [0x04, 0x00, 0x00, 10]{y:10} => []{ status: 0b00000011  });
  }
}
