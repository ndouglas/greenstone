use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_and(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_var!(opcode);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    let first = self.a;
    trace_u8!(first);
    debug!("Ticking (reading operand)...");
    let second = self.read_u8(address);
    trace_u8!(second);
    let answer = first & second;
    trace_u8!(answer);
    self.a = answer;
    self.set_value_flags(answer);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  fn test_and() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("AND", Immediate, [0b00001111]{a:0b01010101} => []{ a: 0b00000101, status: 0b00000000 });
    test_instruction!("AND", Immediate, [0b10001111]{a:0b11010101} => []{ a: 0b10000101, status: 0b10000000 });
    test_instruction!("AND", Immediate, [0x00]{a:0b11010101} => []{ a: 0x00, status: 0b00000010 });
    test_instruction!("AND", ZeroPage, [0x02, 0xFF]{a: 0xF0} => []{a: 0xF0});
    test_instruction!("AND", ZeroPageX, [0x02, 0x00, 0xFF]{x:1, a: 0xF0} => []{a: 0xF0});
    test_instruction!("AND", Absolute, [0x04, 0x00, 0x00, 0xFF]{a:0xF0} => []{a: 0xF0});
    test_instruction!("AND", IndirectX, [0x02, 0x00, 0x05, 0x00, 0xFF]{x:1, a: 0xF0} => []{a: 0xF0});
    test_instruction!("AND", IndirectY, [0x02, 0x04, 0x00, 0x00, 0xFF]{y:1, a: 0xF0} => []{a: 0xF0});
    test_instruction!("AND", AbsoluteX, [0x03, 0x00, 0x00, 0xFF]{x:1, a: 0xF0} => []{a: 0xF0});
    test_instruction!("AND", AbsoluteY, [0x03, 0x00, 0x00, 0xFF]{y:1, a: 0xF0} => []{a: 0xF0});
  }
}
