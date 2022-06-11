use super::super::*;

impl CPU<'_> {
  #[inline]
  pub fn opcode_tay(&mut self, _mode: &AddressingMode) -> bool {
    self.y = self.a;
    self.set_value_flags(self.y);
    false
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_tay_0xa8_transfer_data() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xA9, 0x05, 0xA8, 0x00]);
    assert_eq!(cpu.y, 0x05);
    assert!(
      cpu.status & NEGATIVE_FLAG == 0,
      "LDA #$05, TAY should not set the negative flag."
    );
    assert!(
      cpu.status & CARRY_FLAG == 0,
      "LDA #$05, TAY should not set the carry flag."
    );
  }

  #[test]
  fn test_tay_0xa8_sets_zero_flag() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xA9, 0x00, 0xA8, 0x00]);
    assert!(
      cpu.status & ZERO_FLAG == ZERO_FLAG,
      "LDA #$00, TAY should set the zero flag."
    );
    cpu.interpret(vec![0xa9, 0x05, 0x00]);
    assert!(
      cpu.status & ZERO_FLAG == 0,
      "LDA #$05, TAY should not set the zero flag."
    );
  }

  #[test]
  fn test_tay_0xa8_sets_negative_flag() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xA9, 0xFF, 0xA8, 0x00]);
    assert!(
      cpu.status & NEGATIVE_FLAG == NEGATIVE_FLAG,
      "LDA #$FF, TAY should set the negative flag."
    );
    cpu.interpret(vec![0xA9, 0x05, 0x00]);
    assert!(
      cpu.status & NEGATIVE_FLAG == 0,
      "LDA #$05, TAY should not set the negative flag."
    );
  }
}
