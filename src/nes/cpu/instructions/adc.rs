use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_adc(&mut self, opcode: &Opcode) -> u8 {
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
    let augend = self.a;
    trace_u8!(augend);
    let addend = self.read_u8(address);
    trace_u8!(addend);
    let carry = self.get_carry_flag();
    trace_var!(carry);
    let (answer, set_carry, set_overflow) = add_u8s(augend, addend, carry);
    trace_u8!(answer);
    trace_var!(set_carry);
    trace_var!(set_overflow);
    self.a = answer;
    self.set_carry_flag(set_carry);
    self.set_overflow_flag(set_overflow);
    self.set_value_flags(answer);
    let mut result = cycles;
    if opcode.extra_cycle {
      result = result.wrapping_add(additional_cycles);
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
  fn test_adc() {
    init();
    test_instruction!("ADC", Immediate, [3]{a:2, status:1} => []{ a: 6 });
    test_instruction!("ADC", Immediate, [255]{a:1, status:0x00} => []{ a: 0x00, status: 0b00000011 });
    test_instruction!("ADC", Immediate, [127]{a:1, status:0x00} => []{ a: 128, status: 0b11000000 });
    test_instruction!("ADC", Immediate, [200]{a:100} => []{ a: 44 });
    test_instruction!("ADC", ZeroPage, [0x02, 0x90]{a: 1} => []{ a: 0x91 });
    test_instruction!("ADC", ZeroPageX, [0x02, 0x00, 0x90]{x:1, a: 1} => []{ a: 0x91 });
    test_instruction!("ADC", Absolute, [0x04, 0x00, 0x00, 0x90]{a:1} => []{ a: 0x91 });
    test_instruction!("ADC", AbsoluteX, [0x03, 0x00, 0x00, 0x90]{x:1, a: 1} => []{ a: 0x91 });
    test_instruction!("ADC", AbsoluteY, [0x03, 0x00, 0x00, 0x90]{y:1, a: 1} => []{ a: 0x91 });
    test_instruction!("ADC", IndirectX, [0x02, 0x00, 0x05, 0x00, 0x90]{x:1, a: 1} => []{ a: 0x91 });
    test_instruction!("ADC", IndirectY, [0x02, 0x04, 0x00, 0x00, 0x90]{y:1, a: 1} => []{ a: 0x91 });
  }
}
