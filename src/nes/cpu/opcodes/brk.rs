use super::super::*;

impl CPU<'_> {
  #[inline]
  pub fn opcode_brk(&mut self, _mode: &AddressingMode) -> bool {
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
    cpu.interpret(vec![0x00]);
    assert!(cpu.halt, "BRK should halt.");
  }
}
