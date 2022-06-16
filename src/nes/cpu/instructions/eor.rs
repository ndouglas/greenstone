use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_eor(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let cycles = opcode.cycles;
    let mode = &opcode.mode;
    trace_var!(mode);
    let (address, additional_cycles) = self.get_operand_address(mode).unwrap();
    trace_u16!(address);
    trace_u8!(additional_cycles);
    let value = self.read_u8(address);
    trace_u8!(value);
    self.a = self.a ^ value;
    self.set_value_flags(self.a);
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
  fn test_eor() {
    init();
    test_instruction!("EOR", Immediate, [0b00001111]{a:0b01010101} => []{ a: 0b01011010, status: 0b00000000 });
    test_instruction!("EOR", Immediate, [0b10001111]{a:0b01010101} => []{ a: 0b11011010, status: 0b10000000 });
    test_instruction!("EOR", Immediate, [0xFF]{a:0xFF} => []{ a: 0x00, status: 0b00000010 });
    test_instruction!("EOR", ZeroPage, [0x02, 0xFF]{a: 0xF0} => []{a: 0x0F});
    test_instruction!("EOR", ZeroPageX, [0x02, 0x00, 0xFF]{x:1, a: 0xF0} => []{a: 0x0F});
    test_instruction!("EOR", Absolute, [0x04, 0x00, 0x00, 0xFF]{a:0xF0} => []{a: 0x0F});
    test_instruction!("EOR", AbsoluteX, [0x03, 0x00, 0x00, 0xFF]{x:1, a: 0xF0} => []{a: 0x0F});
    test_instruction!("EOR", AbsoluteY, [0x03, 0x00, 0x00, 0xFF]{y:1, a: 0xF0} => []{a: 0x0F});
    test_instruction!("EOR", IndirectX, [0x02, 0x00, 0x05, 0x00, 0xFF]{x:1, a: 0xF0} => []{a: 0x0F});
    test_instruction!("EOR", IndirectY, [0x02, 0x04, 0x00, 0x00, 0xFF]{y:1, a: 0xF0} => []{a: 0x0F});
  }
}
