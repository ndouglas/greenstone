use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_sei(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    self.set_interrupt_disable_flag(true);
    self.tick();
    trace_var!(self.get_interrupt_disable_flag());
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_sei() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("SEI", Implied, []{status: 0b00000000} => []{status: 0b00000100});
    test_instruction!("SEI", Implied, []{status: 0b00000100} => []{status: 0b00000100});
  }
}
