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
    assert!(
      should_overflow(a, m, r),
      "positive + positive = negative indicates overflow."
    );
    a = 0x82;
    m = 0x82;
    r = 0x03;
    assert!(
      should_overflow(a, m, r),
      "negative + negative = positive indicates overflow."
    );
    a = 0x00;
    m = 0x00;
    r = 0x00;
    assert!(!should_overflow(a, m, r), "zero + zero = zero indicates no overflow.");
    a = 0x0A;
    m = 0x0A;
    r = 0x14;
    assert!(
      !should_overflow(a, m, r),
      "positive + positive = positive indicates no overflow."
    );
    a = 0x8A;
    m = 0x8A;
    r = 0x84;
    assert!(
      !should_overflow(a, m, r),
      "negative + negative = negative indicates no overflow."
    );
    a = 0x0A;
    m = 0x8A;
    r = 0x14;
    assert!(
      !should_overflow(a, m, r),
      "positive + negative = positive indicates no overflow."
    );
    a = 0x3A;
    m = 0x8A;
    r = 0x84;
    assert!(
      !should_overflow(a, m, r),
      "positive + negative = negative indicates no overflow."
    );
  }

  #[test]
  fn test_adc_0x61_add_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x07, 0x85, 0x05, 0xA9, 0x05, 0xAA, 0x85, 0x0A, 0x61, 0x05, 0x00,
    ]);
    println!(
      "{} {} {} {} {} {} {} {}",
      cpu.read_u8(0),
      cpu.read_u8(1),
      cpu.read_u8(2),
      cpu.read_u8(3),
      cpu.read_u8(4),
      cpu.read_u8(5),
      cpu.read_u8(6),
      cpu.read_u8(7)
    );
    println!(
      "{} {} {} {} {} {} {} {}",
      cpu.read_u8(8),
      cpu.read_u8(9),
      cpu.read_u8(10),
      cpu.read_u8(11),
      cpu.read_u8(12),
      cpu.read_u8(13),
      cpu.read_u8(14),
      cpu.read_u8(15)
    );
    assert_eq!(cpu.x, 0x05);
    assert_eq!(cpu.a, 0x0C);
    assert!(
      cpu.status & NEGATIVE_FLAG == 0,
      "ADC 1 + 1 should not set the negative flag."
    );
    assert!(cpu.status & CARRY_FLAG == 0, "ADC 1 + 1 should not set the carry flag.");
  }

  #[test]
  fn test_adc_0x65_add_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xA9, 0x05, 0x85, 0x0F, 0x65, 0x0F, 0x00]);
    assert_eq!(cpu.a, 0x0A);
    assert!(
      cpu.status & NEGATIVE_FLAG == 0,
      "ADC 5 + 5 should not set the negative flag."
    );
    assert!(cpu.status & CARRY_FLAG == 0, "ADC 5 + 5 should not set the carry flag.");
  }

  //  Opcode::new(0x61, "ADC", 2, 6, AddressingMode::IndirectX, false, false, false, false),
  //  Opcode::new(0x65, "ADC", 2, 3, AddressingMode::ZeroPage, false, false, false, false),
  //  Opcode::new(0x69, "ADC", 2, 2, AddressingMode::Immediate, false, false, false, false),
  //  Opcode::new(0x6D, "ADC", 3, 4, AddressingMode::Absolute, false, false, false, false),
  //  Opcode::new(0x71, "ADC", 2, 5, AddressingMode::IndirectY, false, false, false, true),
  //  Opcode::new(0x75, "ADC", 2, 4, AddressingMode::ZeroPageX, false, false, false, false),
  //  Opcode::new(0x79, "ADC", 3, 4, AddressingMode::AbsoluteY, false, false, false, true),
  //  Opcode::new(0x7D, "ADC", 3, 4, AddressingMode::AbsoluteX, false, false, false, true),
}
