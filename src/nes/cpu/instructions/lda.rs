use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_lda(&mut self, mode: &AddressingMode) -> bool {
    trace_enter!();
    let (address, additional_cycles) = self.get_operand_address(mode).unwrap();
    trace_u16!(address);
    trace_u8!(additional_cycles);
    let value = self.read_u8(address);
    trace_u8!(value);
    self.a = value;
    trace_u8!(self.a);
    self.set_value_flags(value);
    let force_additional_cycle = additional_cycles > 0;
    trace_result!(force_additional_cycle);
    force_additional_cycle
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  //  Opcode::new(0xA1, "LDA", 2, 6, AddressingMode::IndirectX, false, false, false, false), (z, X)
  //  Opcode::new(0xA5, "LDA", 2, 3, AddressingMode::ZeroPage, false, false, false, false), z
  //  Opcode::new(0xA9, "LDA", 2, 2, AddressingMode::Immediate, false, false, false, false), #n
  //  Opcode::new(0xAD, "LDA", 3, 4, AddressingMode::Absolute, false, false, false, false), a
  //  Opcode::new(0xB1, "LDA", 2, 5, AddressingMode::IndirectY, false, false, false, true), (z), Y
  //  Opcode::new(0xB5, "LDA", 2, 4, AddressingMode::ZeroPageX, false, false, false, false), z, X
  //  Opcode::new(0xB9, "LDA", 3, 4, AddressingMode::AbsoluteY, false, false, false, true), a, Y
  //  Opcode::new(0xBD, "LDA", 3, 4, AddressingMode::AbsoluteX, false, false, false, true), a, X

  #[test]
  #[named]
  fn test_lda_0xa9_immediate_load_data() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x05, //        LDA #$05      ; A = 5
      0x00, //              BRK           ;
    ]);
    assert_eq!(cpu.a, 0x05);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  #[named]
  fn test_lda_0xa9_immediate_sets_zero_flag() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x00, //        LDA #$00      ; A = 0
      0x00, //              BRK           ;
    ]);
    assert!(cpu.status & ZERO_FLAG == ZERO_FLAG, "should set the zero flag.");
    cpu.interpret(vec![
      0xA9, 0x05, //        LDA #$05      ; A = 5
      0x00, //              BRK           ;
    ]);
    assert!(cpu.status & ZERO_FLAG == 0, "should not set the zero flag.");
  }

  #[test]
  #[named]
  fn test_lda_0xa9_immediate_sets_negative_flag() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0xFF, //        LDA #$FF      ; A = 255
      0x00, //              BRK           ;
    ]);
    assert!(cpu.status & NEGATIVE_FLAG == NEGATIVE_FLAG, "should set the negative flag.");
    cpu.interpret(vec![
      0xA9, 0x05, //        LDA #$05      ; A = 5
      0x00, //              BRK           ;
    ]);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
  }

  #[test]
  #[named]
  fn test_lda_0xa5_zeropage_load_data() {
    init();
    let mut cpu = CPU::new();
    cpu.write_u8(0x10, 0x55);
    cpu.interpret(vec![
      0xA5, 0x10, //        LDA $10       ; A = 85
      0x00, //              BRK           ;
    ]);
    assert_eq!(cpu.a, 0x55, "should retrieve 85 from address $0010.");
  }

  #[test]
  #[named]
  fn test_lda_0xad_absolute_load_data() {
    init();
    let mut cpu = CPU::new();
    cpu.write_u8(0x10, 0x55);
    cpu.interpret(vec![
      0xAD, 0x10, 0x00, //      LDA $0010     ; A = 85
      0x00, //                  BRK           ;
    ]);
    assert_eq!(cpu.a, 0x55, "should retrieve 85 from address $0010.");
  }
}
