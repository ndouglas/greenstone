use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_and(&mut self, opcode: &Opcode) -> u8 {
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
    let first = self.a;
    trace_u8!(first);
    let second = self.read_u8(address);
    trace_u8!(second);
    let answer = first & second;
    trace_u8!(answer);
    self.a = answer;
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
  fn test_and() {
    init();
    test_instruction!("AND", Immediate, [0b00001111]{a:0b01010101} => []{ a: 0b00000101, status: 0b00000000 });
    test_instruction!("AND", Immediate, [0b10001111]{a:0b11010101} => []{ a: 0b10000101, status: 0b10000000 });
    test_instruction!("AND", Immediate, [0x00]{a:0b11010101} => []{ a: 0x00, status: 0b00000010 });
    test_instruction!("AND", ZeroPage, [0x02, 0xFF]{a: 0xF0} => []{a: 0xF0});
    test_instruction!("AND", ZeroPageX, [0x02, 0x00, 0xFF]{x:1, a: 0xF0} => []{a: 0xF0});
    test_instruction!("AND", Absolute, [0x04, 0x00, 0x00, 0xFF]{a:0xF0} => []{a: 0xF0});
    test_instruction!("AND", AbsoluteX, [0x03, 0x00, 0x00, 0xFF]{x:1, a: 0xF0} => []{a: 0xF0});
    test_instruction!("AND", AbsoluteY, [0x03, 0x00, 0x00, 0xFF]{y:1, a: 0xF0} => []{a: 0xF0});
    test_instruction!("AND", IndirectX, [0x02, 0x00, 0x05, 0x00, 0xFF]{x:1, a: 0xF0} => []{a: 0xF0});
    test_instruction!("AND", IndirectY, [0x02, 0x04, 0x00, 0x00, 0xFF]{y:1, a: 0xF0} => []{a: 0xF0});
  }
}
