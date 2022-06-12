use super::super::*;

impl CPU<'_> {
  #[inline]
  pub fn instruction_brk(&mut self, _mode: &AddressingMode) -> bool {
    self.halt = true;
    false
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_brk_0x00_halts() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0x00, //         BRK          ;
    ]);
    assert!(cpu.halt, "BRK should halt.");
  }
}
