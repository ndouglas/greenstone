use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_jmp(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    self.program_counter = address;
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_jmp() {
    init();
    test_instruction!("JMP", Absolute, [0x0A, 0x00]{} => []{program_counter: 10});
    test_instruction!("JMP", Indirect, [0x03, 0x00, 0x0A, 0x00]{} => []{program_counter: 10});
    test_instruction!("JMP", Indirect, [0xFF, 0x01]{status: 0b10000000} => []{program_counter: 0x2211}, |cpu: &mut CPU<'_>, _opcode: &Opcode| {
      cpu.unclocked_write_u8(0x01FF, 0x11);
      cpu.unclocked_write_u8(0x0100, 0x22);
      cpu.unclocked_write_u8(0x0200, 0x33);
    });
  }
}

