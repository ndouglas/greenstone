use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_nop(&mut self, opcode: &Opcode) {
    trace_enter!();
    let mode = &opcode.mode;
    trace_var!(mode);
    match opcode.code {
      // "DOP" or "Double No-Op" instructions
      0x04 | 0x44 | 0x64 | 0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 | 0x0C | 0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
        let address = self.get_operand_address(opcode, mode).unwrap();
        trace_u16!(address);
      }
      // "SKB" or "Skip Byte" instructions
      0x80 | 0x82 | 0x89 | 0xc2 | 0xe2 => {
        debug!("Incrementing program counter (SKB/Skip Byte No-Op)...");
        self.increment_program_counter();
      }
      _ => {}
    }
    debug!("Ticking (NOP)...");
    self.tick();
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_nop() {
    init();
    test_instruction!("NOP", Implied, []{} => []{});
  }
}
