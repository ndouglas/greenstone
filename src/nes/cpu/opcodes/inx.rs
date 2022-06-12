use super::super::*;

impl CPU<'_> {
  #[inline]
  pub fn opcode_inx(&mut self, _mode: &AddressingMode) -> bool {
    self.x = self.x.wrapping_add(1);
    self.set_value_flags(self.x);
    false
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_inx_0xe8_adding_data() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0xC0, //            LDA, #$C0     ; A = -64
      0xAA, //                  TAX           ; X = -64
      0xE8, //                  INX           ; X += 1
      0x00, //                  BRK           ;
    ]);
    assert_eq!(cpu.a, 0xC0, "should set A to $C0.");
    assert_eq!(cpu.x, 0xC1, "should set X to $C1");
    assert!(cpu.status & NEGATIVE_FLAG == NEGATIVE_FLAG, "should set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  fn test_inx_0xe8_overflow() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0xFF, //            LDA #$FF        ; A = 255
      0xAA, //                  TAX             ; X = 255
      0xE8, //                  INX             ; X += 1
      0xE8, //                  INX             ; X += 1
      0x00, //                  BRK             ;
    ]);
    assert_eq!(cpu.x, 1);
    assert!(cpu.status & OVERFLOW_FLAG == 0, "should not set the overflow flag.");
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }
}
