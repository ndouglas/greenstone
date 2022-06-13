use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_tay(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let cycles = opcode.cycles;
    trace_u8!(cycles);
    self.y = self.a;
    trace_var!(self.y);
    self.set_value_flags(self.y);
    let result = cycles;
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
  fn test_tay_0xa8_transfer_data() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x05, //      LDA #$05    ; A = 5
      0xA8, //            TAY         ; Y = 5
      0x00, //            BRK         ;
    ]);
    assert_eq!(cpu.y, 0x05);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  #[named]
  fn test_tay_0xa8_sets_zero_flag() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x00, //      LDA #$00    ; A = 0
      0xA8, //            TAY         ; Y = 0
      0x00, //            BRK         ;
    ]);
    assert!(cpu.status & ZERO_FLAG == ZERO_FLAG, "should set the zero flag.");
    cpu.interpret(vec![
      0xA9, 0x05, //      LDA #$05    ; A = 5
      0xA8, //            TAY         ; Y = 5
      0x00, //            BRK         ;
    ]);
    assert!(cpu.status & ZERO_FLAG == 0, "should not set the zero flag.");
  }

  #[test]
  #[named]
  fn test_tay_0xa8_sets_negative_flag() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0xFF, //      LDA #$FF    ; A = -128
      0xA8, //            TAY         ; Y = -128
      0x00, //            BRK         ;
    ]);
    assert!(cpu.status & NEGATIVE_FLAG == NEGATIVE_FLAG, "should set the negative flag.");
    cpu.interpret(vec![
      0xA9, 0x05, //      LDA #$05    ; A = 5
      0xA8, //            TAY         ; Y = 5
      0x00, //            BRK         ;
    ]);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
  }
}
