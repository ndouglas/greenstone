use std::collections::HashMap;
use std::fmt;

use crate::nes::bus::Bus;
use crate::nes::simple_memory::SimpleMemory;
use crate::traits::Addressable;
use crate::traits::Busable;
use crate::traits::Interruptible;

pub mod addressable;
pub use addressable::*;

pub mod addressing_mode;
pub use addressing_mode::*;

pub mod instructions;
pub use instructions::*;

pub mod interruptible;
pub use interruptible::*;

pub mod opcode;
pub use opcode::*;

pub mod stack;
pub use stack::*;

pub mod status;
pub use status::*;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct CPU<'a> {
  pub a: u8,
  pub x: u8,
  pub y: u8,
  pub status: u8,
  pub stack_pointer: u8,
  pub program_counter: u16,
  pub clock_counter: u64,
  #[derivative(Debug = "ignore")]
  pub bus: Box<dyn Busable + 'a>,
}

impl<'a> CPU<'a> {
  #[named]
  pub fn new() -> CPU<'a> {
    CPU {
      a: 0x00,
      x: 0x00,
      y: 0x00,
      status: 0x00,
      stack_pointer: 0xFF,
      program_counter: 0x0000,
      clock_counter: 0,
      bus: Box::new(SimpleMemory::new()),
    }
  }

  #[named]
  pub fn new_with_bus() -> CPU<'a> {
    CPU {
      a: 0x00,
      x: 0x00,
      y: 0x00,
      status: 0x00,
      stack_pointer: 0xFF,
      program_counter: 0x0000,
      clock_counter: 0,
      bus: Box::new(Bus::new()),
    }
  }

  #[named]
  pub fn interpret(&mut self, program: Vec<u8>, start: u16) {
    trace_enter!();
    self.load(program, start);
    self.handle_reset();
    self.run();
    trace_exit!();
  }

  #[named]
  pub fn run(&mut self) {
    trace_enter!();
    self.run_with_callback(|_| {});
    trace_exit!();
  }

  #[named]
  pub fn run_with_callback<F>(&mut self, mut callback: F)
  where
    F: FnMut(&mut CPU),
  {
    trace_enter!();
    loop {
      callback(self);
      self.clock();
    }
    trace_exit!();
  }

  #[named]
  pub fn clock(&mut self) {
    trace_enter!();
    self.process_instruction();
    trace_exit!();
  }

  #[named]
  pub fn process_instruction(&mut self) {
    trace_enter!();
    if self.is_nmi_ready() {
      self.acknowledge_nmi();
      self.handle_nmi();
    } else if self.is_irq_ready() && !self.get_interrupt_disable_flag() {
      self.handle_irq();
    }
    let opcode = self.dequeue_instruction();
    trace_var!(opcode);
    self.execute_instruction(opcode);
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn dequeue_instruction(&mut self) -> &'static Opcode {
    trace_enter!();
    let code = self.get_next_u8();
    debug!("Processing next instruction @ {:#06X}): {}", (self.program_counter - 1), format_u8!(code));
    trace_u8!(code);
    let ref opcodes: HashMap<u8, &'static Opcode> = *OPCODE_MAP;
    let result = opcodes.get(&code).expect(&format!("Opcode {:#04X} is not recognized", code));
    trace_result!(result);
    result
  }

  #[named]
  #[inline]
  fn execute_instruction(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_var!(opcode);
    match opcode.code {
      // Illegal Opcodes
      0xEB => {}
      // General instructions
      _ => match opcode.mnemonic {
        // Legal Opcodes
        "ADC" => self.instruction_adc(&opcode),
        "AND" => self.instruction_and(&opcode),
        "ASL" => self.instruction_asl(&opcode),
        "BCC" => self.instruction_bcc(&opcode),
        "BCS" => self.instruction_bcs(&opcode),
        "BEQ" => self.instruction_beq(&opcode),
        "BIT" => self.instruction_bit(&opcode),
        "BMI" => self.instruction_bmi(&opcode),
        "BNE" => self.instruction_bne(&opcode),
        "BPL" => self.instruction_bpl(&opcode),
        "BRK" => self.instruction_brk(&opcode),
        "BVC" => self.instruction_bvc(&opcode),
        "BVS" => self.instruction_bvs(&opcode),
        "CLC" => self.instruction_clc(&opcode),
        "CLD" => self.instruction_cld(&opcode),
        "CLI" => self.instruction_cli(&opcode),
        "CLV" => self.instruction_clv(&opcode),
        "CMP" => self.instruction_cmp(&opcode),
        "CPX" => self.instruction_cpx(&opcode),
        "CPY" => self.instruction_cpy(&opcode),
        "DEC" => self.instruction_dec(&opcode),
        "DEX" => self.instruction_dex(&opcode),
        "DEY" => self.instruction_dey(&opcode),
        "EOR" => self.instruction_eor(&opcode),
        "INC" => self.instruction_inc(&opcode),
        "INX" => self.instruction_inx(&opcode),
        "INY" => self.instruction_iny(&opcode),
        "JMP" => self.instruction_jmp(&opcode),
        "JSR" => self.instruction_jsr(&opcode),
        "LDA" => self.instruction_lda(&opcode),
        "LDX" => self.instruction_ldx(&opcode),
        "LDY" => self.instruction_ldy(&opcode),
        "LSR" => self.instruction_lsr(&opcode),
        "NOP" => self.instruction_nop(&opcode),
        "ORA" => self.instruction_ora(&opcode),
        "PHA" => self.instruction_pha(&opcode),
        "PHP" => self.instruction_php(&opcode),
        "PLA" => self.instruction_pla(&opcode),
        "PLP" => self.instruction_plp(&opcode),
        "ROL" => self.instruction_rol(&opcode),
        "ROR" => self.instruction_ror(&opcode),
        "RTI" => self.instruction_rti(&opcode),
        "RTS" => self.instruction_rts(&opcode),
        "SBC" => self.instruction_sbc(&opcode),
        "SEC" => self.instruction_sec(&opcode),
        "SED" => self.instruction_sed(&opcode),
        "SEI" => self.instruction_sei(&opcode),
        "STA" => self.instruction_sta(&opcode),
        "STX" => self.instruction_stx(&opcode),
        "STY" => self.instruction_sty(&opcode),
        "TAX" => self.instruction_tax(&opcode),
        "TAY" => self.instruction_tay(&opcode),
        "TSX" => self.instruction_tsx(&opcode),
        "TXA" => self.instruction_txa(&opcode),
        "TXS" => self.instruction_txs(&opcode),
        "TYA" => self.instruction_tya(&opcode),
        // Illegal Opcodes
        "DCP" => self.instruction_dcp(&opcode),
        _ => todo!(),
      },
    }
  }

  #[named]
  #[inline]
  pub fn increment_program_counter(&mut self) {
    trace_enter!();
    self.program_counter = self.program_counter.wrapping_add(1);
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn get_next_u8(&mut self) -> u8 {
    trace_enter!();
    let start_pc = self.program_counter;
    trace_u16!(self.program_counter);
    self.increment_program_counter();
    let result = self.read_u8(start_pc);
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  pub fn get_next_u16(&mut self) -> u16 {
    trace_enter!();
    let start_pc = self.program_counter;
    trace_u16!(self.program_counter);
    self.increment_program_counter();
    self.increment_program_counter();
    let result = self.read_u16(start_pc);
    trace_u16!(result);
    trace_exit!();
    result
  }

}

impl fmt::Display for CPU<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}
