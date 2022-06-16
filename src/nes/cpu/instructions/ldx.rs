use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_ldx(&mut self, opcode: &Opcode) -> u8 {
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
    let value = self.read_u8(address);
    trace_u8!(value);
    self.x = value;
    trace_u8!(self.x);
    self.set_value_flags(value);
    let mut result = cycles;
    if opcode.extra_cycle {
      result += additional_cycles;
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
  #[named]
  fn test_ldx() {
    init();
    test_instruction!("LDX", Immediate, [0x00]{}                 => []{ x: 0x00, status: 0b00000010 });
    test_instruction!("LDX", Immediate, [0xFF]{}                 => []{ x: 0xFF, status: 0b10000000 });
    test_instruction!("LDX", Immediate, [0x20]{}                 => []{ x: 0x20, status: 0b00000000 });
    test_instruction!("LDX", ZeroPage,  [0x02, 0x90]{}           => []{ x: 0x90 });
    test_instruction!("LDX", ZeroPageY, [0x02, 0x00, 0x90]{y:1}     => []{ x: 0x90 });
    test_instruction!("LDX", Absolute,  [0x04, 0x00, 0x00, 0x90]{}     => []{ x: 0x90 });
    test_instruction!("LDX", AbsoluteY, [0x03, 0x00, 0x00, 0x90]{y:1}  => []{ x: 0x90 });
  }
}
