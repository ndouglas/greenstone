use super::super::*;

pub fn should_overflow(a: u16, m: u16, r: u16) -> bool {
  ((!(a ^ m) & (a ^ r)) & 0x0080) > 0
}

impl CPU<'_> {
  #[inline]
  pub fn opcode_adc(&mut self, mode: &AddressingMode) -> bool {
    let (address, cycles) = self.get_operand_address(mode).unwrap();
    let addend = self.read_u8(address);
    let temp = self.a as u16 + addend as u16 + self.get_carry_flag() as u16;
    self.set_carry_flag(temp > 255);
    let overflowed = should_overflow(self.a as u16, addend as u16, temp);
    self.set_overflow_flag(overflowed);
    let value = temp as u8;
    self.a = value;
    self.set_value_flags(value);
    true
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_should_overflow() {
    let mut a = 0x05 as u16;
    let mut m = 0x02 as u16;
    let mut r = 0x80 as u16;
    assert!(should_overflow(a, m, r), "positive + positive = negative indicates overflow.");
    a = 0x82;
    m = 0x82;
    r = 0x03;
    assert!(should_overflow(a, m, r), "negative + negative = positive indicates overflow.");
    a = 0x00;
    m = 0x00;
    r = 0x00;
    assert!(!should_overflow(a, m, r), "zero + zero = zero indicates no overflow.");
    a = 0x0A;
    m = 0x0A;
    r = 0x14;
    assert!(!should_overflow(a, m, r), "positive + positive = positive indicates no overflow.");
    a = 0x8A;
    m = 0x8A;
    r = 0x84;
    assert!(!should_overflow(a, m, r), "negative + negative = negative indicates no overflow.");
    a = 0x0A;
    m = 0x8A;
    r = 0x14;
    assert!(!should_overflow(a, m, r), "positive + negative = positive indicates no overflow.");
    a = 0x3A;
    m = 0x8A;
    r = 0x84;
    assert!(!should_overflow(a, m, r), "positive + negative = negative indicates no overflow.");
  }

  #[test]
  fn test_adc_0x61_add_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xA9, 0x01, 0xAA, 0x61, 0x04, 0x00]);
    assert_eq!(cpu.a, 0x01);
    assert!(
      cpu.status & NEGATIVE_FLAG == 0,
      "ADC 0 + 1 should not set the negative flag."
    );
    assert!(cpu.status & CARRY_FLAG == 0, "ADC 0 + 1 should not set the carry flag.");
  }

//  Opcode::new(0x61, "ADC", 2, 6, AddressingMode::IndirectX, false, false, false, false),
//  Opcode::new(0x65, "ADC", 2, 3, AddressingMode::ZeroPage, false, false, false, false),
//  Opcode::new(0x69, "ADC", 2, 2, AddressingMode::Immediate, false, false, false, false),
//  Opcode::new(0x6D, "ADC", 3, 4, AddressingMode::Absolute, false, false, false, false),
//  Opcode::new(0x71, "ADC", 2, 5, AddressingMode::IndirectY, false, false, false, true),
//  Opcode::new(0x75, "ADC", 2, 4, AddressingMode::ZeroPageX, false, false, false, false),
//  Opcode::new(0x79, "ADC", 3, 4, AddressingMode::AbsoluteY, false, false, false, true),
//  Opcode::new(0x7D, "ADC", 3, 4, AddressingMode::AbsoluteX, false, false, false, true),


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
