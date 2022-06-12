use super::super::*;

#[inline]
fn should_overflow(a: u16, m: u16, r: u16) -> bool {
  ((!(a ^ m) & (a ^ r)) & 0x0080) > 0
}

impl CPU<'_> {
  #[inline]
  pub fn instruction_adc(&mut self, mode: &AddressingMode) -> bool {
    let (address, additional_cycles) = self.get_operand_address(mode).unwrap();
    let addend = self.read_u8(address);
    let temp = self.a as u16 + addend as u16 + self.get_carry_flag() as u16;
    self.set_carry_flag(temp > 255);
    let overflowed = should_overflow(self.a as u16, addend as u16, temp);
    self.set_overflow_flag(overflowed);
    let value = temp as u8;
    self.a = value;
    self.set_value_flags(value);
    additional_cycles > 0
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
  fn test_adc_0x61_indirectx_add_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x07, //        LDA #$07      ; A = 7
      0x85, 0x05, //        STA #$05      ; $05 = 7
      0xA9, 0x05, //        LDA #$05      ; A = 5
      0xAA, //              TAX           ; X = 5
      0xA9, 0x0F, //        LDA #$0F      ; A = 15
      0x85, 0x0A, //        STA #$0A      ; $0A = 15
      0xA9, 0x58, //        LDA #$58      ; A = 88
      0x85, 0x0F, //        STA #$0F      ; $0F = 88
      0xA9, 0x0D, //        LDA #$0D      ; A = 13
      0x61, 0x05, //        ADC ($05, X)  ; A += ($05 + X)
      0x00, //              BRK           ;
    ]);
    assert_eq!(cpu.a, 0x65);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  fn test_adc_0x65_zeropage_add_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x05, //        LDA #$05      ; A = 5
      0x85, 0x0F, //        STA #$0F      ; $0F = 5
      0x65, 0x0F, //        ADC $0F       ; A += $0F
      0x00, //              BRK           ;
    ]);
    assert_eq!(cpu.a, 0x0A);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  fn test_adc_0x69_immediate_add_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x05, //        LDA #$05      ; A = 5
      0x69, 0x03, //        ADC #$03      ; A += 3
      0x00, //              BRK           ;
    ]);
    assert_eq!(cpu.a, 0x08);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  fn test_adc_0x6d_absolute_add_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x03, //            LDA #$03        ; A = 3
      0x85, 0x05, //            STA #$05        ; $05 = 3
      0xA9, 0x01, //            LDA #$01        ; A = 1
      0x6D, 0x05, 0x00, //      ADC $0005       ; A += $0005
      0x00, //                  BRK             ;
    ]);
    assert_eq!(cpu.a, 0x04);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  fn test_adc_0x71_indirecty_add_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x0A, //        LDA #$0A          ; A = 10
      0x85, 0x03, //        STA #$03          ; $03 = 10
      0xA9, 0x73, //        LDA #$73          ; A = 115
      0x85, 0x0F, //        STA #$0F          ; $0F = 115
      0xA9, 0x05, //        LDA #$05          ; A = 5
      0xA8, //              TAY               ; Y = 5
      0xA9, 0x09, //        LDA #$09          ; A = 9
      0x71, 0x03, //        ADC ($03), Y      ; A += ($03) + Y
      0x00, //              BRK               ;
    ]);
    assert_eq!(cpu.a, 0x7C);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  fn test_adc_0x75_zeropagex_add_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x0A, //          LDA #$0A          ; A = 10
      0x85, 0x03, //          STA #$03          ; $03 = 10
      0xA9, 0x73, //          LDA #$73          ; A = 115
      0x85, 0x08, //          STA #$08          ; $08 = 115
      0xA9, 0x05, //          LDA #$05          ; A = 5
      0xAA, //                TAX               ; X = 5
      0x75, 0x03, //          ADC $03, X        ; A += $03 + X
      0x00, //                BRK               ;
    ]);
    assert_eq!(cpu.a, 0x78);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  fn test_adc_0x79_absolutey_add_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x05, //          LDA #$05          ; A = 5
      0xA8, //                TAY               ; Y = 5
      0xA9, 0x73, //          LDA #$73          ; A = 115
      0x85, 0x08, //          STA #$08          ; $08 = 115
      0xA9, 0x05, //          LDA #$05          ; A = 5
      0x79, 0x03, 0x00, //    ADC $0003, Y      ; A += $0003 + Y
      0x00, //                BRK               ;
    ]);
    assert_eq!(cpu.a, 0x78);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  fn test_adc_0x7d_absolutex_add_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x05, //          LDA #$05          ; A = 5
      0xAA, //                TAX               ; X = 5
      0xA9, 0x73, //          LDA #$73          ; A = 115
      0x85, 0x08, //          STA #$08          ; $08 = 115
      0xA9, 0x05, //          LDA #$05          ; A = 5
      0x7D, 0x03, 0x00, //    ADC $0003, X      ; A += $0003 + X
      0x00, //                BRK               ;
    ]);
    assert_eq!(cpu.a, 0x78);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }
}
