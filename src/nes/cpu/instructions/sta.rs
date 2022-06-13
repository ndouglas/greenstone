use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_sta(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let mode = &opcode.mode;
    trace_var!(mode);
    let (address, additional_cycles) = self.get_operand_address(mode).unwrap();
    trace_u16!(address);
    trace_var!(additional_cycles);
    trace_u8!(self.a);
    self.write_u8(address, self.a);
    let result = 0;
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
  fn test_sta_0x85_zeropage_store_data() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x05, //          LDA #$05    ; A = 5
      0x85, 0x06, //          STA #$06    ; $0006 = 5
      0x00, //                BRK         ;
    ]);
    assert_eq!(cpu.a, 0x05);
    assert_eq!(cpu.read_u8(0x06), 0x05);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  #[named]
  fn test_sta_0x95_zeropagex_store_data() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x05, //        LDA #$05        ; A = 5
      0xAA, //              TAX             ; X = 5
      0xA9, 0x02, //        LDA #$02        ; A = 2
      0x95, 0x05, //        STA $05,X       ; $000A = 5
      0x00, //              BRK             ;
    ]);
    assert_eq!(cpu.a, 0x02);
    assert_eq!(cpu.read_u8(0x0A), 0x02);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  //  Opcode::new(0x81, "STA", 2, 6, AddressingMode::IndirectX, false, false, false, false), (z, X)
  //  Opcode::new(0x85, "STA", 2, 3, AddressingMode::ZeroPage, false, false, false, false), z
  //  Opcode::new(0x8D, "STA", 3, 4, AddressingMode::Absolute, false, false, false, false),  a
  //  Opcode::new(0x91, "STA", 2, 6, AddressingMode::IndirectY, false, false, false, false), (z), Y
  //  Opcode::new(0x95, "STA", 2, 4, AddressingMode::ZeroPageX, false, false, false, false), z, X
  //  Opcode::new(0x99, "STA", 3, 5, AddressingMode::AbsoluteY, false, false, false, false), a, Y
  //  Opcode::new(0x9D, "STA", 3, 5, AddressingMode::AbsoluteX, false, false, false, false), a, X

  #[test]
  #[named]
  fn test_sta_0x8d_absolute_store_data() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xA9, 0x05, 0x8D, 0x05, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(cpu.a, 0x05);
    assert_eq!(cpu.read_u8(0x05), 0x05);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  #[named]
  fn test_sta_0x9d_absolutex_store_data() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xA9, 0x03, 0xAA, 0xA9, 0x05, 0x9D, 0x05, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(cpu.x, 0x03);
    assert_eq!(cpu.a, 0x05);
    assert_eq!(cpu.read_u8(0x08), 0x05);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  #[named]
  fn test_sta_0x99_absolutey_store_data() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xA9, 0x03, 0xA8, 0xA9, 0x05, 0x99, 0x05, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(cpu.y, 0x03);
    assert_eq!(cpu.a, 0x05);
    assert_eq!(cpu.read_u8(0x08), 0x05);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  //    Opcode::new(0x99, "STA", 3, 5, AddressingMode::AbsoluteY),
  //    Opcode::new(0x81, "STA", 2, 6, AddressingMode::IndirectX),
  //    Opcode::new(0x91, "STA", 2, 6, AddressingMode::IndirectY),
}
