use super::super::*;

impl CPU<'_> {
  #[inline]
  pub fn opcode_lda(&mut self, mode: &AddressingMode) {
    let (address, cycles) = self.get_operand_address(mode).unwrap();
    let value = self.read_u8(address);
    self.a = value;
    self.set_value_flags(value);
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_lda_0xa9_load_data() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xA9, 0x05, 0x00]);
    assert_eq!(cpu.a, 0x05);
    assert!(
      cpu.status & NEGATIVE_FLAG == 0,
      "LDA #$05 should not set the negative flag."
    );
    assert!(cpu.status & CARRY_FLAG == 0, "LDA #$05 should not set the carry flag.");
  }

  #[test]
  fn test_lda_0xa9_sets_zero_flag() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xA9, 0x00, 0x00]);
    assert!(
      cpu.status & ZERO_FLAG == ZERO_FLAG,
      "LDA #$00 should set the zero flag."
    );
    cpu.interpret(vec![0xA9, 0x05, 0x00]);
    assert!(cpu.status & ZERO_FLAG == 0, "LDA #$05 should not set the zero flag.");
  }

  #[test]
  fn test_lda_0xa9_sets_negative_flag() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xA9, 0xFF, 0x00]);
    assert!(
      cpu.status & NEGATIVE_FLAG == NEGATIVE_FLAG,
      "LDA #$FF should set the negative flag."
    );
    cpu.interpret(vec![0xA9, 0x05, 0x00]);
    assert!(
      cpu.status & NEGATIVE_FLAG == 0,
      "LDA #$05 should not set the negative flag."
    );
  }

  #[test]
  fn test_lda_0xa9_from_zero_page() {
    let mut cpu = CPU::new();
    cpu.write_u8(0x10, 0x55);
    cpu.interpret(vec![0xA5, 0x10, 0x00]);
    assert_eq!(cpu.a, 0x55, "LDA $10 should retrieve $55 from address $0010.");
  }

  #[test]
  fn test_lda_0xad_from_absolute_address() {
    let mut cpu = CPU::new();
    cpu.write_u8(0x10, 0x55);
    cpu.interpret(vec![0xAD, 0x10, 0x00, 0x00]);
    assert_eq!(cpu.a, 0x55, "LDA $10 should retrieve $55 from address $0010.");
  }
}
