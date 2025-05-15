use super::AddressingMode;
use super::AddressingMode::*;
use super::CPU;
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Clone, Copy, Derivative)]
#[derivative(Debug)]
pub struct Opcode {
  pub code: u8,
  pub mnemonic: &'static str,
  pub length: u8,
  pub cycles: u8,
  pub mode: AddressingMode,
  pub status_mask: u8,
  pub lockup: bool,
  pub unofficial: bool,
  pub unusual: bool,
  pub extra_cycle: bool,
  #[derivative(Debug = "ignore")]
  pub function: fn(&mut CPU, &Opcode) -> (),
}

impl Opcode {
  #[named]
  fn new(
    code: u8,
    mnemonic: &'static str,
    length: u8,
    cycles: u8,
    function: fn(&mut CPU, &Opcode) -> (),
    mode: AddressingMode,
    status_mask: u8,
    lockup: bool,
    unofficial: bool,
    unusual: bool,
    extra_cycle: bool,
  ) -> Self {
    Opcode {
      code,
      mnemonic,
      length,
      cycles,
      function,
      mode,
      status_mask,
      lockup,
      unofficial,
      unusual,
      extra_cycle,
    }
  }
}

lazy_static! {
  pub static ref OPCODE_VECTOR: Vec<Opcode> = vec![
    Opcode::new(0x00, "BRK", 1, 7, CPU::instruction_brk, Implied, 0b00010000, false, false, false, true),
    Opcode::new(0x01, "ORA", 2, 6, CPU::instruction_ora, IndirectX, 0b10000010, false, false, false, true),
    Opcode::new(0x02, "JAM", 1, 2, CPU::instruction_jam, Implied, 0b00000000, true, true, false, true),
    Opcode::new(0x03, "SLO", 2, 8, CPU::instruction_slo, IndirectX, 0b10000011, false, true, false, true),
    Opcode::new(0x04, "NOP", 2, 3, CPU::instruction_nop, ZeroPage, 0b00000000, false, true, false, true),
    Opcode::new(0x05, "ORA", 2, 3, CPU::instruction_ora, ZeroPage, 0b10000010, false, false, false, true),
    Opcode::new(0x06, "ASL", 2, 5, CPU::instruction_asl, ZeroPage, 0b10000011, false, false, false, true),
    Opcode::new(0x07, "SLO", 2, 5, CPU::instruction_slo, ZeroPage, 0b10000011, false, true, false, true),
    Opcode::new(0x08, "PHP", 1, 3, CPU::instruction_php, Implied, 0b00000000, false, false, false, true),
    Opcode::new(0x09, "ORA", 2, 2, CPU::instruction_ora, Immediate, 0b10000010, false, false, false, true),
    Opcode::new(0x0A, "ASL", 1, 2, CPU::instruction_asl_0a, Implied, 0b10000011, false, false, false, true),
    Opcode::new(0x0B, "ANC", 2, 2, CPU::instruction_wtf, Implied, 0b00000000, false, true, true, true),
    Opcode::new(0x0C, "NOP", 3, 4, CPU::instruction_nop, Absolute, 0b00000000, false, true, false, true),
    Opcode::new(0x0D, "ORA", 3, 4, CPU::instruction_ora, Absolute, 0b10000010, false, false, false, true),
    Opcode::new(0x0E, "ASL", 3, 6, CPU::instruction_asl, Absolute, 0b10000011, false, false, false, true),
    Opcode::new(0x0F, "SLO", 3, 6, CPU::instruction_slo, Absolute, 0b10000011, false, true, false, true),
    Opcode::new(0x10, "BPL", 2, 2, CPU::instruction_bpl, Relative, 0b00000000, false, false, false, true),
    Opcode::new(0x11, "ORA", 2, 5, CPU::instruction_ora, IndirectY, 0b10000010, false, false, false, false),
    Opcode::new(0x12, "JAM", 1, 2, CPU::instruction_jam, Implied, 0b00000000, true, true, false, true),
    Opcode::new(0x13, "SLO", 2, 8, CPU::instruction_slo, IndirectY, 0b10000011, false, true, false, true),
    Opcode::new(0x14, "NOP", 2, 4, CPU::instruction_nop, ZeroPageX, 0b00000000, false, true, false, true),
    Opcode::new(0x15, "ORA", 2, 4, CPU::instruction_ora, ZeroPageX, 0b10000010, false, false, false, true),
    Opcode::new(0x16, "ASL", 2, 6, CPU::instruction_asl, ZeroPageX, 0b10000011, false, false, false, true),
    Opcode::new(0x17, "SLO", 2, 6, CPU::instruction_slo, ZeroPageX, 0b10000011, false, true, false, true),
    Opcode::new(0x18, "CLC", 1, 2, CPU::instruction_clc, Implied, 0b00000001, false, false, false, true),
    Opcode::new(0x19, "ORA", 3, 4, CPU::instruction_ora, AbsoluteY, 0b10000010, false, false, false, false),
    Opcode::new(0x1A, "NOP", 1, 2, CPU::instruction_nop, Implied, 0b00000000, false, true, false, true),
    Opcode::new(0x1B, "SLO", 3, 7, CPU::instruction_slo, AbsoluteY, 0b10000011, false, true, true, true),
    Opcode::new(0x1C, "NOP", 3, 4, CPU::instruction_nop, AbsoluteX, 0b00000000, false, true, false, false),
    Opcode::new(0x1D, "ORA", 3, 4, CPU::instruction_ora, AbsoluteX, 0b10000010, false, false, false, false),
    Opcode::new(0x1E, "ASL", 3, 7, CPU::instruction_asl, AbsoluteX, 0b10000011, false, false, false, true),
    Opcode::new(0x1F, "SLO", 3, 7, CPU::instruction_slo, AbsoluteX, 0b10000011, false, true, false, true),
    Opcode::new(0x20, "JSR", 3, 6, CPU::instruction_jsr, Absolute, 0b00000000, false, false, false, true),
    Opcode::new(0x21, "AND", 2, 6, CPU::instruction_and, IndirectX, 0b10000010, false, false, false, true),
    Opcode::new(0x22, "JAM", 1, 2, CPU::instruction_jam, Implied, 0b00000000, true, true, false, true),
    Opcode::new(0x23, "RLA", 2, 8, CPU::instruction_rla, IndirectX, 0b10000011, false, true, false, true),
    Opcode::new(0x24, "BIT", 2, 3, CPU::instruction_bit, ZeroPage, 0b11000010, false, false, false, true),
    Opcode::new(0x25, "AND", 2, 3, CPU::instruction_and, ZeroPage, 0b10000010, false, false, false, true),
    Opcode::new(0x26, "ROL", 2, 5, CPU::instruction_rol, ZeroPage, 0b10000011, false, false, false, true),
    Opcode::new(0x27, "RLA", 2, 5, CPU::instruction_rla, ZeroPage, 0b10000011, false, true, false, true),
    Opcode::new(0x28, "PLP", 1, 4, CPU::instruction_plp, Implied, 0b11111111, false, false, false, true),
    Opcode::new(0x29, "AND", 2, 2, CPU::instruction_and, Immediate, 0b10000010, false, false, false, true),
    Opcode::new(0x2A, "ROL", 1, 2, CPU::instruction_rol_2a, Implied, 0b10000011, false, false, false, true),
    Opcode::new(0x2B, "ANC", 2, 2, CPU::instruction_wtf, Implied, 0b00000000, false, true, true, true),
    Opcode::new(0x2C, "BIT", 3, 4, CPU::instruction_bit, Absolute, 0b11000010, false, false, false, true),
    Opcode::new(0x2D, "AND", 3, 4, CPU::instruction_and, Absolute, 0b10000010, false, false, false, true),
    Opcode::new(0x2E, "ROL", 3, 6, CPU::instruction_rol, Absolute, 0b10000011, false, false, false, true),
    Opcode::new(0x2F, "RLA", 3, 6, CPU::instruction_rla, Absolute, 0b10000011, false, true, false, true),
    Opcode::new(0x30, "BMI", 2, 2, CPU::instruction_bmi, Relative, 0b00000000, false, false, false, true),
    Opcode::new(0x31, "AND", 2, 5, CPU::instruction_and, IndirectY, 0b10000010, false, false, false, false),
    Opcode::new(0x32, "JAM", 1, 2, CPU::instruction_jam, Implied, 0b00000000, true, true, false, true),
    Opcode::new(0x33, "RLA", 2, 8, CPU::instruction_rla, IndirectY, 0b10000011, false, true, false, true),
    Opcode::new(0x34, "NOP", 2, 4, CPU::instruction_nop, ZeroPageX, 0b00000000, false, true, false, true),
    Opcode::new(0x35, "AND", 2, 4, CPU::instruction_and, ZeroPageX, 0b10000010, false, false, false, true),
    Opcode::new(0x36, "ROL", 2, 6, CPU::instruction_rol, ZeroPageX, 0b10000011, false, false, false, true),
    Opcode::new(0x37, "RLA", 2, 6, CPU::instruction_rla, ZeroPageX, 0b10000011, false, true, false, true),
    Opcode::new(0x38, "SEC", 1, 2, CPU::instruction_sec, Implied, 0b00000001, false, false, false, true),
    Opcode::new(0x39, "AND", 3, 4, CPU::instruction_and, AbsoluteY, 0b10000010, false, false, false, false),
    Opcode::new(0x3A, "NOP", 1, 2, CPU::instruction_nop, Implied, 0b00000000, false, true, false, true),
    Opcode::new(0x3B, "RLA", 3, 7, CPU::instruction_rla, AbsoluteY, 0b10000011, false, true, false, true),
    Opcode::new(0x3C, "NOP", 3, 4, CPU::instruction_nop, AbsoluteX, 0b00000000, false, true, false, false),
    Opcode::new(0x3D, "AND", 3, 4, CPU::instruction_and, AbsoluteX, 0b10000010, false, false, false, false),
    Opcode::new(0x3E, "ROL", 3, 7, CPU::instruction_rol, AbsoluteX, 0b10000011, false, false, false, true),
    Opcode::new(0x3F, "RLA", 3, 7, CPU::instruction_rla, AbsoluteX, 0b10000011, false, true, false, true),
    Opcode::new(0x40, "RTI", 1, 6, CPU::instruction_rti, Implied, 0b11111111, false, false, false, true),
    Opcode::new(0x41, "EOR", 2, 6, CPU::instruction_eor, IndirectX, 0b10000010, false, false, false, true),
    Opcode::new(0x42, "JAM", 1, 2, CPU::instruction_jam, Implied, 0b00000000, true, true, false, true),
    Opcode::new(0x43, "SRE", 2, 8, CPU::instruction_sre, IndirectX, 0b10000011, false, true, false, true),
    Opcode::new(0x44, "NOP", 2, 3, CPU::instruction_nop, ZeroPage, 0b00000000, false, true, false, true),
    Opcode::new(0x45, "EOR", 2, 3, CPU::instruction_eor, ZeroPage, 0b10000010, false, false, false, true),
    Opcode::new(0x46, "LSR", 2, 5, CPU::instruction_lsr, ZeroPage, 0b10000011, false, false, false, true),
    Opcode::new(0x47, "SRE", 2, 5, CPU::instruction_sre, ZeroPage, 0b10000011, false, true, false, true),
    Opcode::new(0x48, "PHA", 1, 3, CPU::instruction_pha, Implied, 0b00000000, false, false, false, true),
    Opcode::new(0x49, "EOR", 2, 2, CPU::instruction_eor, Immediate, 0b10000010, false, false, false, true),
    Opcode::new(0x4A, "LSR", 1, 2, CPU::instruction_lsr_4a, Implied, 0b10000011, false, false, false, true),
    Opcode::new(0x4B, "ASR", 2, 2, CPU::instruction_wtf, Implied, 0b00000000, false, true, true, true),
    Opcode::new(0x4C, "JMP", 3, 3, CPU::instruction_jmp, Absolute, 0b00000000, false, false, false, true),
    Opcode::new(0x4D, "EOR", 3, 4, CPU::instruction_eor, Absolute, 0b10000010, false, false, false, true),
    Opcode::new(0x4E, "LSR", 3, 6, CPU::instruction_lsr, Absolute, 0b10000011, false, false, false, true),
    Opcode::new(0x4F, "SRE", 3, 6, CPU::instruction_sre, Absolute, 0b10000011, false, true, false, true),
    Opcode::new(0x50, "BVC", 2, 2, CPU::instruction_bvc, Relative, 0b00000000, false, false, false, true),
    Opcode::new(0x51, "EOR", 2, 5, CPU::instruction_eor, IndirectY, 0b00000000, false, false, false, false),
    Opcode::new(0x52, "JAM", 1, 2, CPU::instruction_jam, Implied, 0b00000000, true, true, false, true),
    Opcode::new(0x53, "SRE", 2, 8, CPU::instruction_sre, IndirectY, 0b10000011, false, true, false, true),
    Opcode::new(0x54, "NOP", 2, 4, CPU::instruction_nop, ZeroPageX, 0b00000000, false, true, false, true),
    Opcode::new(0x55, "EOR", 2, 4, CPU::instruction_eor, ZeroPageX, 0b10000010, false, false, false, true),
    Opcode::new(0x56, "LSR", 2, 6, CPU::instruction_lsr, ZeroPageX, 0b10000011, false, false, false, true),
    Opcode::new(0x57, "SRE", 2, 6, CPU::instruction_sre, ZeroPageX, 0b10000011, false, true, false, true),
    Opcode::new(0x58, "CLI", 1, 2, CPU::instruction_cli, Implied, 0b00000100, false, false, false, true),
    Opcode::new(0x59, "EOR", 3, 4, CPU::instruction_eor, AbsoluteY, 0b10000010, false, false, false, false),
    Opcode::new(0x5A, "NOP", 1, 2, CPU::instruction_nop, Implied, 0b00000000, false, true, false, true),
    Opcode::new(0x5B, "SRE", 3, 7, CPU::instruction_sre, AbsoluteY, 0b10000011, false, true, false, true),
    Opcode::new(0x5C, "NOP", 3, 4, CPU::instruction_nop, AbsoluteX, 0b00000000, false, true, false, false),
    Opcode::new(0x5D, "EOR", 3, 4, CPU::instruction_eor, AbsoluteX, 0b10000010, false, false, false, false),
    Opcode::new(0x5E, "LSR", 3, 7, CPU::instruction_lsr, AbsoluteX, 0b10000011, false, false, false, true),
    Opcode::new(0x5F, "SRE", 3, 7, CPU::instruction_sre, AbsoluteX, 0b10000011, false, true, false, true),
    Opcode::new(0x60, "RTS", 1, 6, CPU::instruction_rts, Implied, 0b00000000, false, false, false, true),
    Opcode::new(0x61, "ADC", 2, 6, CPU::instruction_adc, IndirectX, 0b11000011, false, false, false, true),
    Opcode::new(0x62, "JAM", 1, 2, CPU::instruction_jam, Implied, 0b00000000, true, true, false, true),
    Opcode::new(0x63, "RRA", 2, 8, CPU::instruction_rra, IndirectX, 0b11000011, false, true, false, true),
    Opcode::new(0x64, "NOP", 2, 3, CPU::instruction_nop, ZeroPage, 0b00000000, false, true, false, true),
    Opcode::new(0x65, "ADC", 2, 3, CPU::instruction_adc, ZeroPage, 0b11000011, false, false, false, true),
    Opcode::new(0x66, "ROR", 2, 5, CPU::instruction_ror, ZeroPage, 0b10000011, false, false, false, true),
    Opcode::new(0x67, "RRA", 2, 5, CPU::instruction_rra, ZeroPage, 0b11000011, false, true, false, true),
    Opcode::new(0x68, "PLA", 1, 4, CPU::instruction_pla, Implied, 0b10000010, false, false, false, true),
    Opcode::new(0x69, "ADC", 2, 2, CPU::instruction_adc, Immediate, 0b11000011, false, false, false, true),
    Opcode::new(0x6A, "ROR", 1, 2, CPU::instruction_ror_6a, Implied, 0b10000011, false, false, false, true),
    Opcode::new(0x6B, "ARR", 2, 2, CPU::instruction_wtf, Implied, 0b00000000, false, true, true, true),
    Opcode::new(0x6C, "JMP", 3, 5, CPU::instruction_jmp, Indirect, 0b00000000, false, false, false, true),
    Opcode::new(0x6D, "ADC", 3, 4, CPU::instruction_adc, Absolute, 0b11000011, false, false, false, true),
    Opcode::new(0x6E, "ROR", 3, 6, CPU::instruction_ror, Absolute, 0b10000011, false, false, false, true),
    Opcode::new(0x6F, "RRA", 3, 6, CPU::instruction_rra, Absolute, 0b11000011, false, true, false, true),
    Opcode::new(0x70, "BVS", 2, 2, CPU::instruction_bvs, Relative, 0b00000000, false, false, false, true),
    Opcode::new(0x71, "ADC", 2, 5, CPU::instruction_adc, IndirectY, 0b11000011, false, false, false, false),
    Opcode::new(0x72, "JAM", 1, 2, CPU::instruction_jam, Implied, 0b00000000, true, true, false, true),
    Opcode::new(0x73, "RRA", 2, 8, CPU::instruction_rra, IndirectY, 0b11000011, false, true, false, true),
    Opcode::new(0x74, "NOP", 2, 4, CPU::instruction_nop, ZeroPageX, 0b00000000, false, true, false, true),
    Opcode::new(0x75, "ADC", 2, 4, CPU::instruction_adc, ZeroPageX, 0b11000011, false, false, false, true),
    Opcode::new(0x76, "ROR", 2, 6, CPU::instruction_ror, ZeroPageX, 0b10000011, false, false, false, true),
    Opcode::new(0x77, "RRA", 2, 6, CPU::instruction_rra, ZeroPageX, 0b11000011, false, true, false, true),
    Opcode::new(0x78, "SEI", 1, 2, CPU::instruction_sei, Implied, 0b00000100, false, false, false, true),
    Opcode::new(0x79, "ADC", 3, 4, CPU::instruction_adc, AbsoluteY, 0b11000011, false, false, false, false),
    Opcode::new(0x7A, "NOP", 1, 2, CPU::instruction_nop, Implied, 0b00000000, false, true, false, true),
    Opcode::new(0x7B, "RRA", 3, 7, CPU::instruction_rra, AbsoluteY, 0b11000011, false, true, true, true),
    Opcode::new(0x7C, "NOP", 3, 4, CPU::instruction_nop, AbsoluteX, 0b00000000, false, true, false, false),
    Opcode::new(0x7D, "ADC", 3, 4, CPU::instruction_adc, AbsoluteX, 0b11000011, false, false, false, false),
    Opcode::new(0x7E, "ROR", 3, 7, CPU::instruction_ror, AbsoluteX, 0b10000011, false, false, false, true),
    Opcode::new(0x7F, "RRA", 3, 7, CPU::instruction_rra, AbsoluteX, 0b11000011, false, true, false, true),
    Opcode::new(0x80, "NOP", 2, 2, CPU::instruction_nop, Immediate, 0b00000000, false, true, false, true),
    Opcode::new(0x81, "STA", 2, 6, CPU::instruction_sta, IndirectX, 0b00000000, false, false, false, true),
    Opcode::new(0x82, "NOP", 2, 2, CPU::instruction_nop, Immediate, 0b00000000, true, true, false, true),
    Opcode::new(0x83, "SAX", 2, 6, CPU::instruction_sax, IndirectX, 0b00000000, false, true, false, true),
    Opcode::new(0x84, "STY", 2, 3, CPU::instruction_sty, ZeroPage, 0b00000000, false, false, false, true),
    Opcode::new(0x85, "STA", 2, 3, CPU::instruction_sta, ZeroPage, 0b00000000, false, false, false, true),
    Opcode::new(0x86, "STX", 2, 3, CPU::instruction_stx, ZeroPage, 0b00000000, false, false, false, true),
    Opcode::new(0x87, "SAX", 2, 3, CPU::instruction_sax, ZeroPage, 0b00000000, false, true, false, true),
    Opcode::new(0x88, "DEY", 1, 2, CPU::instruction_dey, Implied, 0b10000010, false, false, false, true),
    Opcode::new(0x89, "NOP", 2, 2, CPU::instruction_nop, Immediate, 0b00000000, false, true, false, true),
    Opcode::new(0x8A, "TXA", 1, 2, CPU::instruction_txa, Implied, 0b10000010, false, false, false, true),
    Opcode::new(0x8B, "ANE", 2, 2, CPU::instruction_wtf, Implied, 0b00000000, false, true, true, true),
    Opcode::new(0x8C, "STY", 3, 4, CPU::instruction_sty, Absolute, 0b00000000, false, false, false, true),
    Opcode::new(0x8D, "STA", 3, 4, CPU::instruction_sta, Absolute, 0b00000000, false, false, false, true),
    Opcode::new(0x8E, "STX", 3, 4, CPU::instruction_stx, Absolute, 0b00000000, false, false, false, true),
    Opcode::new(0x8F, "SAX", 3, 4, CPU::instruction_sax, Absolute, 0b00000000, false, true, false, true),
    Opcode::new(0x90, "BCC", 2, 2, CPU::instruction_bcc, Relative, 0b00000000, false, false, false, true),
    Opcode::new(0x91, "STA", 2, 6, CPU::instruction_sta, IndirectY, 0b00000000, false, false, false, true),
    Opcode::new(0x92, "JAM", 1, 2, CPU::instruction_jam, Implied, 0b00000000, true, true, false, true),
    Opcode::new(0x93, "SHA", 2, 6, CPU::instruction_wtf, Implied, 0b00000000, false, true, true, true),
    Opcode::new(0x94, "STY", 2, 4, CPU::instruction_sty, ZeroPageX, 0b00000000, false, false, false, true),
    Opcode::new(0x95, "STA", 2, 4, CPU::instruction_sta, ZeroPageX, 0b00000000, false, false, false, true),
    Opcode::new(0x96, "STX", 2, 4, CPU::instruction_stx, ZeroPageY, 0b00000000, false, false, false, true),
    Opcode::new(0x97, "SAX", 2, 4, CPU::instruction_sax, ZeroPageY, 0b00000000, false, true, false, true),
    Opcode::new(0x98, "TYA", 1, 2, CPU::instruction_tya, Implied, 0b10000010, false, false, false, true),
    Opcode::new(0x99, "STA", 3, 5, CPU::instruction_sta, AbsoluteY, 0b00000000, false, false, false, true),
    Opcode::new(0x9A, "TXS", 1, 2, CPU::instruction_txs, Implied, 0b00000000, false, false, false, true),
    Opcode::new(0x9B, "SHS", 3, 5, CPU::instruction_wtf, Implied, 0b00000000, false, true, true, true),
    Opcode::new(0x9C, "SHY", 3, 5, CPU::instruction_wtf, Implied, 0b00000000, false, true, true, true),
    Opcode::new(0x9D, "STA", 3, 5, CPU::instruction_sta, AbsoluteX, 0b00000000, false, false, false, true),
    Opcode::new(0x9E, "SHX", 3, 5, CPU::instruction_wtf, Implied, 0b00000000, false, true, true, true),
    Opcode::new(0x9F, "SHA", 3, 5, CPU::instruction_wtf, Implied, 0b00000000, false, true, true, true),
    Opcode::new(0xA0, "LDY", 2, 2, CPU::instruction_ldy, Immediate, 0b10000010, false, false, false, true),
    Opcode::new(0xA1, "LDA", 2, 6, CPU::instruction_lda, IndirectX, 0b10000010, false, false, false, true),
    Opcode::new(0xA2, "LDX", 2, 2, CPU::instruction_ldx, Immediate, 0b10000010, false, false, false, true),
    Opcode::new(0xA3, "LAX", 2, 6, CPU::instruction_lax, IndirectX, 0b11000000, false, true, false, true),
    Opcode::new(0xA4, "LDY", 2, 3, CPU::instruction_ldy, ZeroPage, 0b10000010, false, false, false, true),
    Opcode::new(0xA5, "LDA", 2, 3, CPU::instruction_lda, ZeroPage, 0b10000010, false, false, false, true),
    Opcode::new(0xA6, "LDX", 2, 3, CPU::instruction_ldx, ZeroPage, 0b10000010, false, false, false, true),
    Opcode::new(0xA7, "LAX", 2, 3, CPU::instruction_lax, ZeroPage, 0b11000000, false, true, false, true),
    Opcode::new(0xA8, "TAY", 1, 2, CPU::instruction_tay, Implied, 0b10000010, false, false, false, true),
    Opcode::new(0xA9, "LDA", 2, 2, CPU::instruction_lda, Immediate, 0b10000010, false, false, false, true),
    Opcode::new(0xAA, "TAX", 1, 2, CPU::instruction_tax, Implied, 0b10000010, false, false, false, true),
    Opcode::new(0xAB, "LXA", 2, 2, CPU::instruction_wtf, Implied, 0b00000000, false, true, true, true),
    Opcode::new(0xAC, "LDY", 3, 4, CPU::instruction_ldy, Absolute, 0b10000010, false, false, false, true),
    Opcode::new(0xAD, "LDA", 3, 4, CPU::instruction_lda, Absolute, 0b10000010, false, false, false, true),
    Opcode::new(0xAE, "LDX", 3, 4, CPU::instruction_ldx, Absolute, 0b10000010, false, false, false, true),
    Opcode::new(0xAF, "LAX", 3, 4, CPU::instruction_lax, Absolute, 0b11000000, false, true, false, true),
    Opcode::new(0xB0, "BCS", 2, 2, CPU::instruction_bcs, Relative, 0b00000000, false, false, false, true),
    Opcode::new(0xB1, "LDA", 2, 5, CPU::instruction_lda, IndirectY, 0b10000010, false, false, false, false),
    Opcode::new(0xB2, "JAM", 1, 2, CPU::instruction_jam, Implied, 0b00000000, true, true, false, true),
    Opcode::new(0xB3, "LAX", 2, 5, CPU::instruction_lax, IndirectY, 0b11000000, false, true, false, false),
    Opcode::new(0xB4, "LDY", 2, 4, CPU::instruction_ldy, ZeroPageX, 0b10000010, false, false, false, true),
    Opcode::new(0xB5, "LDA", 2, 4, CPU::instruction_lda, ZeroPageX, 0b10000010, false, false, false, true),
    Opcode::new(0xB6, "LDX", 2, 4, CPU::instruction_ldx, ZeroPageY, 0b10000010, false, false, false, true),
    Opcode::new(0xB7, "LAX", 2, 4, CPU::instruction_lax, ZeroPageY, 0b11000000, false, true, false, true),
    Opcode::new(0xB8, "CLV", 1, 2, CPU::instruction_clv, Implied, 0b01000000, false, false, false, true),
    Opcode::new(0xB9, "LDA", 3, 4, CPU::instruction_lda, AbsoluteY, 0b10000010, false, false, false, false),
    Opcode::new(0xBA, "TSX", 1, 2, CPU::instruction_tsx, Implied, 0b10000010, false, false, false, true),
    Opcode::new(0xBB, "LAS", 3, 4, CPU::instruction_wtf, Implied, 0b00000000, false, true, true, false),
    Opcode::new(0xBC, "LDY", 3, 4, CPU::instruction_ldy, AbsoluteX, 0b10000010, false, false, false, false),
    Opcode::new(0xBD, "LDA", 3, 4, CPU::instruction_lda, AbsoluteX, 0b10000010, false, false, false, false),
    Opcode::new(0xBE, "LDX", 3, 4, CPU::instruction_ldx, AbsoluteY, 0b10000010, false, false, false, false),
    Opcode::new(0xBF, "LAX", 3, 4, CPU::instruction_lax, AbsoluteY, 0b11000000, false, true, false, false),
    Opcode::new(0xC0, "CPY", 2, 2, CPU::instruction_cpy, Immediate, 0b10000011, false, false, false, true),
    Opcode::new(0xC1, "CMP", 2, 6, CPU::instruction_cmp, IndirectX, 0b10000011, false, false, false, true),
    Opcode::new(0xC2, "NOP", 2, 2, CPU::instruction_nop, Immediate, 0b00000000, true, true, false, true),
    Opcode::new(0xC3, "DCP", 2, 8, CPU::instruction_dcp, IndirectX, 0b00000000, false, true, false, true),
    Opcode::new(0xC4, "CPY", 2, 3, CPU::instruction_cpy, ZeroPage, 0b10000011, false, false, false, true),
    Opcode::new(0xC5, "CMP", 2, 3, CPU::instruction_cmp, ZeroPage, 0b10000011, false, false, false, true),
    Opcode::new(0xC6, "DEC", 2, 5, CPU::instruction_dec, ZeroPage, 0b10000010, false, false, false, true),
    Opcode::new(0xC7, "DCP", 2, 5, CPU::instruction_dcp, ZeroPage, 0b00000000, false, true, false, true),
    Opcode::new(0xC8, "INY", 1, 2, CPU::instruction_iny, Implied, 0b10000010, false, false, false, true),
    Opcode::new(0xC9, "CMP", 2, 2, CPU::instruction_cmp, Immediate, 0b10000011, false, false, false, true),
    Opcode::new(0xCA, "DEX", 1, 2, CPU::instruction_dex, Implied, 0b10000010, false, false, false, true),
    Opcode::new(0xCB, "SBX", 2, 2, CPU::instruction_wtf, Implied, 0b00000000, false, true, true, true),
    Opcode::new(0xCC, "CPY", 3, 4, CPU::instruction_cpy, Absolute, 0b10000011, false, false, false, true),
    Opcode::new(0xCD, "CMP", 3, 4, CPU::instruction_cmp, Absolute, 0b10000011, false, false, false, true),
    Opcode::new(0xCE, "DEC", 3, 6, CPU::instruction_dec, Absolute, 0b10000010, false, false, false, true),
    Opcode::new(0xCF, "DCP", 3, 6, CPU::instruction_dcp, Absolute, 0b00000000, false, true, false, true),
    Opcode::new(0xD0, "BNE", 2, 2, CPU::instruction_bne, Relative, 0b00000000, false, false, false, true),
    Opcode::new(0xD1, "CMP", 2, 5, CPU::instruction_cmp, IndirectY, 0b10000011, false, false, false, false),
    Opcode::new(0xD2, "JAM", 1, 2, CPU::instruction_jam, Implied, 0b00000000, true, true, false, true),
    Opcode::new(0xD3, "DCP", 2, 8, CPU::instruction_dcp, IndirectY, 0b00000000, false, true, false, true),
    Opcode::new(0xD4, "NOP", 2, 4, CPU::instruction_nop, ZeroPageX, 0b00000000, false, true, false, true),
    Opcode::new(0xD5, "CMP", 2, 4, CPU::instruction_cmp, ZeroPageX, 0b10000011, false, false, false, true),
    Opcode::new(0xD6, "DEC", 2, 6, CPU::instruction_dec, ZeroPageX, 0b10000010, false, false, false, true),
    Opcode::new(0xD7, "DCP", 2, 6, CPU::instruction_dcp, ZeroPageX, 0b00000000, false, true, false, true),
    Opcode::new(0xD8, "CLD", 1, 2, CPU::instruction_cld, Implied, 0b00001000, false, false, false, true),
    Opcode::new(0xD9, "CMP", 3, 4, CPU::instruction_cmp, AbsoluteY, 0b10000011, false, false, false, false),
    Opcode::new(0xDA, "NOP", 1, 2, CPU::instruction_nop, Implied, 0b00000000, false, true, false, true),
    Opcode::new(0xDB, "DCP", 3, 7, CPU::instruction_dcp, AbsoluteY, 0b00000000, false, true, false, true),
    Opcode::new(0xDC, "NOP", 3, 4, CPU::instruction_nop, AbsoluteX, 0b00000000, false, true, false, false),
    Opcode::new(0xDD, "CMP", 3, 4, CPU::instruction_cmp, AbsoluteX, 0b10000011, false, false, false, false),
    Opcode::new(0xDE, "DEC", 3, 7, CPU::instruction_dec, AbsoluteX, 0b10000010, false, false, false, true),
    Opcode::new(0xDF, "DCP", 3, 7, CPU::instruction_dcp, AbsoluteX, 0b00000000, false, true, false, true),
    Opcode::new(0xE0, "CPX", 2, 2, CPU::instruction_cpx, Immediate, 0b10000011, false, false, false, true),
    Opcode::new(0xE1, "SBC", 2, 6, CPU::instruction_sbc, IndirectX, 0b11000011, false, false, false, true),
    Opcode::new(0xE2, "NOP", 2, 2, CPU::instruction_nop, Immediate, 0b00000000, true, true, false, true),
    Opcode::new(0xE3, "ISB", 2, 8, CPU::instruction_isb, IndirectX, 0b11000011, false, true, false, true),
    Opcode::new(0xE4, "CPX", 2, 3, CPU::instruction_cpx, ZeroPage, 0b10000011, false, false, false, true),
    Opcode::new(0xE5, "SBC", 2, 3, CPU::instruction_sbc, ZeroPage, 0b11000011, false, false, false, true),
    Opcode::new(0xE6, "INC", 2, 5, CPU::instruction_inc, ZeroPage, 0b10000010, false, false, false, true),
    Opcode::new(0xE7, "ISB", 2, 5, CPU::instruction_isb, ZeroPage, 0b11000011, false, true, false, true),
    Opcode::new(0xE8, "INX", 1, 2, CPU::instruction_inx, Implied, 0b10000010, false, false, false, true),
    Opcode::new(0xE9, "SBC", 2, 2, CPU::instruction_sbc, Immediate, 0b11000011, false, false, false, true),
    Opcode::new(0xEA, "NOP", 1, 2, CPU::instruction_nop, Implied, 0b00000000, false, false, false, true),
    Opcode::new(0xEB, "SBC", 2, 2, CPU::instruction_sbc, Immediate, 0b11000011, false, true, false, true),
    Opcode::new(0xEC, "CPX", 3, 4, CPU::instruction_cpx, Absolute, 0b10000011, false, false, false, true),
    Opcode::new(0xED, "SBC", 3, 4, CPU::instruction_sbc, Absolute, 0b11000011, false, false, false, true),
    Opcode::new(0xEE, "INC", 3, 6, CPU::instruction_inc, Absolute, 0b10000010, false, false, false, true),
    Opcode::new(0xEF, "ISB", 3, 6, CPU::instruction_isb, Absolute, 0b11000011, false, true, false, true),
    Opcode::new(0xF0, "BEQ", 2, 2, CPU::instruction_beq, Relative, 0b00000000, false, false, false, true),
    Opcode::new(0xF1, "SBC", 2, 5, CPU::instruction_sbc, IndirectY, 0b11000011, false, false, false, false),
    Opcode::new(0xF2, "JAM", 1, 2, CPU::instruction_jam, Implied, 0b00000000, true, true, false, true),
    Opcode::new(0xF3, "ISB", 2, 8, CPU::instruction_isb, IndirectY, 0b11000011, false, true, false, true),
    Opcode::new(0xF4, "NOP", 2, 4, CPU::instruction_nop, ZeroPageX, 0b00000000, false, true, false, true),
    Opcode::new(0xF5, "SBC", 2, 4, CPU::instruction_sbc, ZeroPageX, 0b11000011, false, false, false, true),
    Opcode::new(0xF6, "INC", 2, 6, CPU::instruction_inc, ZeroPageX, 0b10000010, false, false, false, true),
    Opcode::new(0xF7, "ISB", 2, 6, CPU::instruction_isb, ZeroPageX, 0b11000011, false, true, false, true),
    Opcode::new(0xF8, "SED", 1, 2, CPU::instruction_sed, Implied, 0b00001000, false, false, false, true),
    Opcode::new(0xF9, "SBC", 3, 4, CPU::instruction_sbc, AbsoluteY, 0b11000011, false, false, false, false),
    Opcode::new(0xFA, "NOP", 1, 2, CPU::instruction_nop, Implied, 0b00000000, false, true, false, true),
    Opcode::new(0xFB, "ISB", 3, 7, CPU::instruction_isb, AbsoluteY, 0b11000011, false, true, true, true),
    Opcode::new(0xFC, "NOP", 3, 4, CPU::instruction_nop, AbsoluteX, 0b00000000, false, true, false, false),
    Opcode::new(0xFD, "SBC", 3, 4, CPU::instruction_sbc, AbsoluteX, 0b11000011, false, false, false, false),
    Opcode::new(0xFE, "INC", 3, 7, CPU::instruction_inc, AbsoluteX, 0b10000010, false, false, false, true),
    Opcode::new(0xFF, "ISB", 3, 7, CPU::instruction_isb, AbsoluteX, 0b11000011, false, true, false, true),
  ];
  pub static ref OPCODE_MAP: HashMap<u8, &'static Opcode> = {
    let mut map = HashMap::new();
    for opcode in &*OPCODE_VECTOR {
      map.insert(opcode.code, opcode);
    }
    map
  };
  /// Fast array-based opcode lookup table for O(1) access without hashing.
  /// Index directly by opcode byte (0-255) for maximum performance.
  pub static ref OPCODE_TABLE: [Option<&'static Opcode>; 256] = {
    let mut table: [Option<&'static Opcode>; 256] = [None; 256];
    for opcode in &*OPCODE_VECTOR {
      table[opcode.code as usize] = Some(opcode);
    }
    table
  };
  pub static ref OPCODE_MNEMONICS: HashSet<&'static str> = {
    let mut set = HashSet::new();
    for opcode in &*OPCODE_VECTOR {
      set.insert(opcode.mnemonic);
    }
    set
  };
  pub static ref INSTRUCTION_OPCODE_MAP: HashMap<&'static str, Vec<&'static Opcode>> = {
    let mut map = HashMap::new();
    for opcode in &*OPCODE_VECTOR {
      map.insert(opcode.mnemonic, Vec::new());
    }
    for opcode in &*OPCODE_VECTOR {
      map.get_mut(opcode.mnemonic).unwrap().push(opcode);
    }
    map
  };
  pub static ref INSTRUCTION_MODE_OPCODE_MAP: HashMap<&'static str, HashMap<AddressingMode, &'static Opcode>> = {
    let mut map = HashMap::new();
    for opcode in &*OPCODE_VECTOR {
      map.insert(opcode.mnemonic, HashMap::new());
    }
    // Insert unofficial opcodes first...
    for opcode in (&*OPCODE_VECTOR).iter().filter(|opcode| opcode.unofficial) {
      map.get_mut(opcode.mnemonic).unwrap().insert(opcode.mode, opcode);
    }
    // So that official opcodes with the same addressing mode will take priority.
    for opcode in (&*OPCODE_VECTOR).iter().filter(|opcode| !opcode.unofficial) {
      map.get_mut(opcode.mnemonic).unwrap().insert(opcode.mode, opcode);
    }
    map
  };
}

impl fmt::Display for Opcode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}
