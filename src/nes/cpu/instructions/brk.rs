use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_brk(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    self.increment_program_counter();
    self.r#break();
    self.set_interrupt_disable_flag(false);
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
