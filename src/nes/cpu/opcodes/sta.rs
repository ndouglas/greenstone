use super::super::*;

impl CPU<'_> {
  #[inline]
  pub fn opcode_sta(&mut self, mode: &AddressingMode) -> bool {
    let (address, _additional_cycles) = self.get_operand_address(mode).unwrap();
    self.write_u8(address, self.a);
    false
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_sta_0x85_zeropage_store_data() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x05, //    LDA #$05    ; A = 5
      0x85, 0x06, //    STA #$06    ; $0006 = 5
      0x00, //          BRK         ;
    ]);
    assert_eq!(cpu.a, 0x05);
    assert_eq!(cpu.read_u8(0x06), 0x05);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  fn test_sta_0x95_zeropagex_store_data() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0xA9, 0x05, //        LDA #$05        ; A = 5
      0x95, 0x07, //        STA $07         ; $0007 = 5
      0x00, //              BRK             ;
    ]);
    assert_eq!(cpu.x, 0x01);
    assert_eq!(cpu.a, 0x05);
    assert_eq!(cpu.read_u8(0x08), 0x05);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

//  Opcode::new(0x81, "STA", 2, 6, AddressingMode::IndirectX, false, false, false, false),
//  Opcode::new(0x85, "STA", 2, 3, AddressingMode::ZeroPage, false, false, false, false),
//  Opcode::new(0x8D, "STA", 3, 4, AddressingMode::Absolute, false, false, false, false),
//  Opcode::new(0x91, "STA", 2, 6, AddressingMode::IndirectY, false, false, false, false),
//  Opcode::new(0x95, "STA", 2, 4, AddressingMode::ZeroPageX, false, false, false, false),
//  Opcode::new(0x99, "STA", 3, 5, AddressingMode::AbsoluteY, false, false, false, false),
//  Opcode::new(0x9D, "STA", 3, 5, AddressingMode::AbsoluteX, false, false, false, false),

  #[test]
  fn test_sta_0x8d_absolute_store_data() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xA9, 0x05, 0x8D, 0x05, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(cpu.a, 0x05);
    assert_eq!(cpu.read_u8(0x05), 0x05);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  fn test_sta_0x9d_absolutex_store_data() {
    let mut cpu = CPU::new();
    cpu.interpret(vec![0xA9, 0x03, 0xAA, 0xA9, 0x05, 0x9D, 0x05, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(cpu.x, 0x03);
    assert_eq!(cpu.a, 0x05);
    assert_eq!(cpu.read_u8(0x08), 0x05);
    assert!(cpu.status & NEGATIVE_FLAG == 0, "should not set the negative flag.");
    assert!(cpu.status & CARRY_FLAG == 0, "should not set the carry flag.");
  }

  #[test]
  fn test_sta_0x99_absolutey_store_data() {
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
