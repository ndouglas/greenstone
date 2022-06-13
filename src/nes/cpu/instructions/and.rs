use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_and(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let cycles = opcode.cycles;
    trace_u8!(cycles);
    let mode = &opcode.mode;
    trace_var!(mode);
    let (address, additional_cycles) = self.get_operand_address(mode).unwrap();
    trace_u16!(address);
    trace_u8!(additional_cycles);
    let first = self.a;
    trace_u8!(first);
    let second = self.read_u8(address);
    trace_u8!(second);
    let answer = first & second;
    trace_u8!(answer);
    self.a = answer;
    self.set_value_flags(answer);
    let mut result = cycles;
    if opcode.extra_cycle {
      result += additional_cycles;
    }
    trace_result!(result);
    result
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_and_0x21_indirectx_bitwise_and() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x07, //        LDA #$07      ; A = 7
      0x85, 0x05, //        STA $05       ; $05 = 7
      0xA9, 0x05, //        LDA #$05      ; A = 5
      0xAA, //              TAX           ; X = 5
      0xA9, 0x0F, //        LDA #$0F      ; A = 15
      0x85, 0x0A, //        STA $0A       ; $0A = 15
      0xA9, 0x58, //        LDA #$58      ; A = 88
      0x85, 0x0F, //        STA $0F       ; $0F = 88
      0xA9, 0x0D, //        LDA #$0D      ; A = 13
      0x21, 0x05, //        AND ($05,X)   ; A = A & ($05 + X)
      0x00, //              BRK           ;
    ]);
    assert_eq!(cpu.a, 0x8);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }


  #[test]
  #[named]
  fn test_adc_0x65_zeropage_add_with_carry() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x05, //        LDA #$05      ; A = 5
      0x85, 0x0F, //        STA $0F       ; $0F = 5
      0x65, 0x0F, //        ADC $0F       ; A += $0F
      0x00, //              BRK           ;
    ]);
    assert_eq!(cpu.a, 0x0A);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  #[named]
  fn test_adc_0x69_immediate_add_with_carry() {
    init();
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
  #[named]
  fn test_adc_0x6d_absolute_add_with_carry() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x03, //            LDA #$03        ; A = 3
      0x85, 0x05, //            STA $05         ; $05 = 3
      0xA9, 0x01, //            LDA #$01        ; A = 1
      0x6D, 0x05, 0x00, //      ADC $0005       ; A += $0005
      0x00, //                  BRK             ;
    ]);
    assert_eq!(cpu.a, 0x04);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  #[named]
  fn test_adc_0x71_indirecty_add_with_carry() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x0A, //        LDA #$0A          ; A = 10
      0x85, 0x03, //        STA $03           ; $03 = 10
      0xA9, 0x73, //        LDA #$73          ; A = 115
      0x85, 0x0F, //        STA $0F           ; $0F = 115
      0xA9, 0x05, //        LDA #$05          ; A = 5
      0xA8, //              TAY               ; Y = 5
      0xA9, 0x09, //        LDA #$09          ; A = 9
      0x71, 0x03, //        ADC ($03),Y       ; A += ($03) + Y
      0x00, //              BRK               ;
    ]);
    assert_eq!(cpu.a, 0x7C);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  #[named]
  fn test_adc_0x75_zeropagex_add_with_carry() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x0A, //          LDA #$0A          ; A = 10
      0x85, 0x03, //          STA $03           ; $03 = 10
      0xA9, 0x73, //          LDA #$73          ; A = 115
      0x85, 0x08, //          STA $08           ; $08 = 115
      0xA9, 0x05, //          LDA #$05          ; A = 5
      0xAA, //                TAX               ; X = 5
      0x75, 0x03, //          ADC $03,X         ; A += $03 + X
      0x00, //                BRK               ;
    ]);
    assert_eq!(cpu.a, 0x78);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  #[named]
  fn test_adc_0x79_absolutey_add_with_carry() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x05, //          LDA #$05          ; A = 5
      0xA8, //                TAY               ; Y = 5
      0xA9, 0x73, //          LDA #$73          ; A = 115
      0x85, 0x08, //          STA $08           ; $08 = 115
      0xA9, 0x05, //          LDA #$05          ; A = 5
      0x79, 0x03, 0x00, //    ADC $0003,Y       ; A += $0003 + Y
      0x00, //                BRK               ;
    ]);
    assert_eq!(cpu.a, 0x78);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }
//  Opcode::new(0x21, "AND", 2, 6, AddressingMode::IndirectX, false, false, false, false),
//  Opcode::new(0x25, "AND", 2, 3, AddressingMode::ZeroPage, false, false, false, false),
//  Opcode::new(0x29, "AND", 2, 2, AddressingMode::Immediate, false, false, false, false),
//  Opcode::new(0x2D, "AND", 3, 4, AddressingMode::Absolute, false, false, false, false),
//  Opcode::new(0x31, "AND", 2, 5, AddressingMode::IndirectY, false, false, false, true),
//  Opcode::new(0x35, "AND", 2, 4, AddressingMode::ZeroPageX, false, false, false, false),
//  Opcode::new(0x39, "AND", 3, 4, AddressingMode::AbsoluteY, false, false, false, true),


  #[test]
  #[named]
  fn test_and_0x3d_absolutex_bitwise_and() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x05, //          LDA #$05          ; A = 5
      0xAA, //                TAX               ; X = 5
      0xA9, 0x73, //          LDA #$73          ; A = 115
      0x85, 0x08, //          STA $08           ; $08 = 115
      0xA9, 0x05, //          LDA #$05          ; A = 5
      0x3D, 0x03, 0x00, //    AND $0003,X       ; A = A & $0003 + X
      0x00, //                BRK               ;
    ]);
    assert_eq!(cpu.a, 0x01);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }


}
