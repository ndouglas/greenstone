use crate::nes::common::add_u8s;

use super::super::*;

impl CPU<'_> {
  #[inline]
  pub fn instruction_sbc(&mut self, mode: &AddressingMode) -> bool {
    let (address, additional_cycles) = self.get_operand_address(mode).unwrap();
    let minuend = self.a;
    println!("Minuend {} or {}", minuend, minuend as i8);
    println!("Data at Address {} or {}", self.read_u8(address), self.read_u8(address) as i8);
    let subtrahend = (self.read_u8(address) as i8).wrapping_neg() as u8;
    println!("Subtrahend {} or {}", subtrahend, subtrahend as i8);
    let carry = self.get_carry_flag();
    println!("Carry {}", carry);
    let (result, set_carry, set_overflow) = add_u8s(minuend, subtrahend, carry);
    self.a = result;
    self.set_carry_flag(set_carry);
    self.set_overflow_flag(set_overflow);
    self.set_value_flags(result);
    additional_cycles > 0
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_sbc_0xe1_indirectx_subtract_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x05, //        LDA #$05      ; A = 5
      0xAA, //              TAX           ; X = 5
      0xA9, 0x0F, //        LDA #$0F      ; A = 15
      0x85, 0x0A, //        STA $0A       ; $0A = 15
      0xA9, 0x58, //        LDA #$58      ; A = 88
      0x85, 0x0F, //        STA $0F       ; $0F = 88
      0xA9, 0x14, //        LDA #$14      ; A = 20
      0x38, //              SEC           ; Set carry flag.
      0xE1, 0x05, //        SBC ($05,X)   ; A -= ($05 + X)
      0x00, //              BRK           ;
    ]);
    assert_eq!(cpu.a as i8, -68i8);
    assert!(cpu.status & NEGATIVE_FLAG == NEGATIVE_FLAG, "should set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  fn test_sbc_0xe5_zeropage_subtract_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x05, //        LDA #$05      ; A = 5
      0x85, 0x0F, //        STA $0F       ; $0F = 5
      0xA9, 0x0A, //        LDA #$0A      ; A = 10
      0x38, //              SEC           ; Set carry flag.
      0xE5, 0x0F, //        SBC $0F       ; A -= $0F
      0x00, //              BRK           ;
    ]);
    assert_eq!(cpu.a as i8, 0x05 as i8);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == CARRY_FLAG, "should set the carry flag.");
  }

  #[test]
  fn test_sbc_0xe9_immediate_subtract_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x05, //        LDA #$05      ; A = 5
      0x38, //              SEC           ; Set carry flag.
      0xE9, 0x03, //        SBC #$03      ; A -= 3
      0x00, //              BRK           ;
    ]);
    assert_eq!(cpu.a as i8, 0x02 as i8);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == CARRY_FLAG, "should set the carry flag.");
  }

  #[test]
  fn test_sbc_0xed_absolute_subtract_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x03, //            LDA #$03        ; A = 3
      0x85, 0x05, //            STA #$05        ; $05 = 3
      0xA9, 0x01, //            LDA #$01        ; A = 1
      0x38, //                  SEC             ; Set carry flag.
      0xED, 0x05, 0x00, //      SBC $0005       ; A -= $0005
      0x00, //                  BRK             ;
    ]);
    assert_eq!(cpu.a as i8, -0x02 as i8);
    assert!(cpu.status & NEGATIVE_FLAG == NEGATIVE_FLAG, "should set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  fn test_sbc_0xf1_indirecty_subtract_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x0A, //        LDA #$0A          ; A = 10
      0x85, 0x03, //        STA #$03          ; $03 = 10
      0xA9, 0x73, //        LDA #$73          ; A = 115
      0x85, 0x0F, //        STA #$0F          ; $0F = 115
      0xA9, 0x05, //        LDA #$05          ; A = 5
      0xA8, //              TAY               ; Y = 5
      0xA9, 0x09, //        LDA #$09          ; A = 9
      0x38, //              SEC               ; Set carry flag.
      0xF1, 0x03, //        SBC ($03), Y      ; A -= ($03) + Y
      0x00, //              BRK               ;
    ]);
    assert_eq!(cpu.a as i8, -106 as i8);
    assert!(cpu.status & NEGATIVE_FLAG == NEGATIVE_FLAG, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  fn test_sbc_0xf5_zeropagex_subtract_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x0A, //          LDA #$0A          ; A = 10
      0x85, 0x03, //          STA #$03          ; $03 = 10
      0xA9, 0x73, //          LDA #$73          ; A = 115
      0x85, 0x08, //          STA #$08          ; $08 = 115
      0xA9, 0x05, //          LDA #$05          ; A = 5
      0xAA, //                TAX               ; X = 5
      0x38, //                SEC               ; Set carry flag.
      0xF5, 0x03, //          SBC $03, X        ; A -= $03 + X
      0x00, //                BRK               ;
    ]);
    assert_eq!(cpu.a as i8, -110 as i8);
    assert!(cpu.status & NEGATIVE_FLAG == NEGATIVE_FLAG, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  fn test_sbc_0xf9_absolutey_subtract_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x05, //          LDA #$05          ; A = 5
      0xA8, //                TAY               ; Y = 5
      0xA9, 0x73, //          LDA #$73          ; A = 115
      0x85, 0x08, //          STA #$08          ; $08 = 115
      0xA9, 0x05, //          LDA #$05          ; A = 5
      0x38, //                SEC               ; Set carry flag.
      0xF9, 0x03, 0x00, //    SBC $0003, Y      ; A -= $0003 + Y
      0x00, //                BRK               ;
    ]);
    assert_eq!(cpu.a as i8, -110 as i8);
    assert!(cpu.status & NEGATIVE_FLAG == NEGATIVE_FLAG, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  fn test_sbc_0xfd_absolutex_subtract_with_carry() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x05, //          LDA #$05          ; A = 5
      0xAA, //                TAX               ; X = 5
      0xA9, 0x73, //          LDA #$73          ; A = 115
      0x85, 0x08, //          STA #$08          ; $08 = 115
      0xA9, 0x05, //          LDA #$05          ; A = 5
      0x38, //                SEC               ; Set carry flag.
      0xFD, 0x03, 0x00, //    SBC $0003, X      ; A -= $0003 + X
      0x00, //                BRK               ;
    ]);
    assert_eq!(cpu.a as i8, -110 as i8);
    assert!(cpu.status & NEGATIVE_FLAG == NEGATIVE_FLAG, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

}