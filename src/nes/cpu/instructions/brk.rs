use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_brk(&mut self, _mode: &AddressingMode) -> bool {
    trace_enter!();
    self.halt = true;
    trace_var!(self.halt);
    trace_result!(false);
    false
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
