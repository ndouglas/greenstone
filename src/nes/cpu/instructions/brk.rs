use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_brk(&mut self, _opcode: &Opcode) {
    trace_enter!();
    // BRK is a 2-byte instruction (opcode + signature byte)
    // The signature byte is skipped but PC after BRK points past it
    self.increment_program_counter();
    self.handle_break();
    // Note: handle_break() already sets the interrupt disable flag to true
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_brk() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("BRK", Implied, [0x00, 0x00]{a:0x00, status: 0x00} => []{ a: 0x00});
  }
}
