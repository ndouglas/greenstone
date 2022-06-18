use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_cmp(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    debug!("Ticking (reading operand)...");
    let operand = self.read_u8(address);
    trace_u8!(operand);
    self.set_value_flags(self.a.wrapping_sub(operand));
    self.set_carry_flag(self.a >= operand);
    trace_var!(self.get_carry_flag());
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_cmp() {
    init();
    test_instruction!("CMP", Immediate, [10]{a:10} => []{ status: 0b00000011 });
    test_instruction!("CMP", Immediate, [100]{a:10} => []{ status: 0b10000000 });
    test_instruction!("CMP", Immediate, [10]{a:100} => []{ status: 0b00000001 });
    test_instruction!("CMP", ZeroPage, [0x02, 10]{a: 10} => []{ status: 0b00000011 });
    test_instruction!("CMP", ZeroPageX, [0x02, 0x00, 10]{x:1, a: 10} => []{ status: 0b00000011 });
    test_instruction!("CMP", Absolute, [0x04, 0x00, 0x00, 10]{a:10} => []{ status: 0b00000011  });
    test_instruction!("CMP", IndirectX, [0x02, 0x00, 0x05, 0x00, 10]{x:1, a: 10} => []{ status: 0b00000011 });
    test_instruction!("CMP", IndirectY, [0x02, 0x04, 0x00, 0x00, 10]{y:1, a: 10} => []{ status: 0b00000011 });
    test_instruction!("CMP", AbsoluteX, [0x03, 0x00, 0x00, 10]{x:1, a: 10} => []{ status: 0b00000011 });
    test_instruction!("CMP", AbsoluteY, [0x03, 0x00, 0x00, 10]{y:1, a: 10} => []{ status: 0b00000011 });
  }
}
