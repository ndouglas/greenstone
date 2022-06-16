use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_brk(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let cycles = opcode.cycles;
    trace_u8!(cycles);
    self.halt = true;
    trace_var!(self.halt);
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
  fn test_brk() {
    init();
    test_instruction!("BRK", Implied, [0x00, 0x00]{a:0x00, status: 0x00} => []{ a: 0x00});
  }
}
