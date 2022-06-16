use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_cpx(&mut self, opcode: &Opcode) -> u8 {
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
    self.set_value_flags(self.x.wrapping_sub(operand));
    self.set_carry_flag(self.x >= operand);
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
    test_instruction!("CPX", Immediate, [10]{x:10} => []{ status: 0b00000011 });
    test_instruction!("CPX", Immediate, [100]{x:10} => []{ status: 0b10000000 });
    test_instruction!("CPX", Immediate, [10]{x:100} => []{ status: 0b00000001 });
    test_instruction!("CPX", ZeroPage, [0x02, 10]{x: 10} => []{ status: 0b00000011 });
    test_instruction!("CPX", Absolute, [0x04, 0x00, 0x00, 10]{x:10} => []{ status: 0b00000011  });
  }
}
