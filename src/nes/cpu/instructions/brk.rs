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
  fn test_brk_0x00_halts() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0x00, //         BRK          ;
    ]);
    assert!(cpu.halt, "BRK should halt.");
  }
}
