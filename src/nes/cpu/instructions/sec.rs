use super::super::*;

impl CPU<'_> {
  #[inline]
  pub fn instruction_sec(&mut self, _mode: &AddressingMode) -> bool {
    self.set_carry_flag(true);
    false
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_sec_implicit_clear_carry_flag() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0x38, //            SEC         ; Set carry flag.
      0x00, //            BRK         ;
    ]);
    assert!(cpu.status & CARRY_FLAG == CARRY_FLAG, "should set the carry flag.");
  }

}
