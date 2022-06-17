use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_tsx(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let cycles = opcode.cycles;
    trace_u8!(cycles);
    self.x = self.stack_pointer;
    trace_u8!(self.x);
    self.set_value_flags(self.x);
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
  fn test_tsx() {
    init();
    test_instruction!("TSX", Implied,  []{stack_pointer: 1} => []{stack_pointer: 1, x: 1, status: 0b00000000});
    test_instruction!("TSX", Implied,  []{stack_pointer: 0} => []{stack_pointer: 0, x: 0, status: 0b00000010});
    test_instruction!("TSX", Implied,  []{stack_pointer: 128} => []{stack_pointer: 128, x: 128, status: 0b10000000});
  }
}
