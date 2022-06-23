use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_isb(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_var!(opcode);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    let operand = self.read_u8(address);
    trace_u8!(operand);
    let output = operand.wrapping_add(1);
    trace_u8!(output);
    self.tick();
    self.write_u8(address, output);
    let minuend = self.a;
    trace_u8!(minuend);
    let subtrahend = (output as i8).wrapping_neg().wrapping_sub(1) as u8;
    trace_u8!(subtrahend);
    let carry = self.get_carry_flag();
    trace_var!(carry);
    let (answer, set_carry, set_overflow) = add_u8s(minuend, subtrahend, carry);
    trace_u8!(answer);
    trace_var!(set_carry);
    trace_var!(set_overflow);
    self.a = answer;
    self.set_carry_flag(set_carry);
    self.set_overflow_flag(set_overflow);
    self.set_value_flags(answer);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_isb() {
    init();
    test_instruction!("ISB", ZeroPage,  [0x02, 255]{} => [0x02, 0]{status: 0b10000000});
    test_instruction!("ISB", ZeroPage,  [0x02, 127]{} => [0x02, 128]{status: 0b00000000});
    test_instruction!("ISB", ZeroPageX, [0x02, 0x00, 0x02]{x: 1} => [0x02, 0x00, 0x03]{});
    test_instruction!("ISB", Absolute,  [0x03, 0x00, 0x02]{} => [0x03, 0x00, 0x03]{});
    test_instruction!("ISB", AbsoluteX, [0x03, 0x00, 0x00, 0x02]{x: 1} => [0x03, 0x00, 0x00, 0x03]{});
    test_instruction!("ISB", AbsoluteY, [0x03, 0x00, 0x00, 0x02]{y: 1} => [0x03, 0x00, 0x00, 0x03]{});
    test_instruction!("ISB", IndirectX, [0x02, 0x00, 0x05, 0x00, 0x90]{x:1, a: 1} => []{ a: 0x6F });
    test_instruction!("ISB", IndirectY, [0x02, 0x04, 0x00, 0x00, 0x90]{y:1, a: 1} => []{ a: 0x6F });
  }
}
