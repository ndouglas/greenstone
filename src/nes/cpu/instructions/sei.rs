use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_sei(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let cycles = opcode.cycles;
    trace_u8!(cycles);
    self.set_interrupt_disable_flag(true);
    trace_var!(self.get_interrupt_disable_flag());
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
  fn test_sei() {
    init();
    test_instruction!("SEI", Implied, []{status: 0b00000000} => []{status: 0b00000100});
    test_instruction!("SEI", Implied, []{status: 0b00000100} => []{status: 0b00000100});
  }
}