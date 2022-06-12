use super::super::*;

impl CPU<'_> {
  #[inline]
  pub fn instruction_clc(&mut self, _mode: &AddressingMode) -> bool {
    self.set_carry_flag(false);
    false
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_clc_implicit_clear_carry_flag() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0x18, //            CLC         ; Clear carry flag.
      0x00, //            BRK         ;
    ]);
    assert!(cpu.status & CARRY_FLAG == 0, "should clear the carry flag.");
  }

}
