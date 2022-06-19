use std::collections::HashMap;
use std::fmt;

use crate::nes::bus::Bus;
use crate::nes::simple_memory::SimpleMemory;
use crate::traits::Addressable;

pub mod addressing_mode;
pub use addressing_mode::*;

pub mod instructions;
pub use instructions::*;

pub mod interrupt;
pub use interrupt::*;

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
  pub bus: Box<dyn Addressable + 'a>,
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
  pub fn interpret(&mut self, program: Vec<u8>) {
    trace_enter!();
    self.load(program);
    self.reset();
    self.run();
    trace_exit!();
  }

  #[named]
  pub fn run(&mut self) {
    trace_enter!();
    loop {
      self.clock();
    }
  }

  #[named]
  pub fn clock(&mut self) {
    trace_enter!();
    self.process_instruction();
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn dequeue_instruction(&mut self) -> &'static Opcode {
    trace_enter!();
    let code = self.read_u8(self.program_counter);
    debug!("Processing next instruction @ {:#06X}): {}", self.program_counter, format_u8!(code));
    trace_u8!(code);
    let ref opcodes: HashMap<u8, &'static Opcode> = *OPCODE_MAP;
    self.program_counter = self.program_counter.wrapping_add(1);
    let result = opcodes.get(&code).expect(&format!("Opcode {:#04X} is not recognized", code));
    trace_result!(result);
    result
  }

  #[named]
  pub fn process_instruction(&mut self) {
    trace_enter!();
    if self.is_nmi_ready() {
      self.acknowledge_nmi();
      self.nmi();
    }
    else if self.is_irq_ready() && !self.get_interrupt_disable_flag() {
      self.irq();
    }
    let opcode = self.dequeue_instruction();
    trace_var!(opcode);
    let pc_state = self.program_counter;
    self.execute_instruction(opcode);
    if pc_state == self.program_counter {
      let addend = opcode.length.wrapping_sub(1) as u16;
      self.program_counter = self.program_counter.wrapping_add(addend);
    }
    trace_u16!(self.program_counter);
    trace_exit!();
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
        _ => todo!(),
      },
    }
  }

  #[named]
  fn unclocked_read_u8(&mut self, address: u16) -> u8 {
    trace_enter!();
    let result = self.bus.read_u8(address);
    trace_result!(result);
    result
  }

  #[named]
  fn unclocked_write_u8(&mut self, address: u16, data: u8) {
    trace_enter!();
    self.bus.write_u8(address, data);
    trace_exit!();
  }

  #[named]
  #[allow(dead_code)]
  fn unclocked_read_u16(&mut self, address: u16) -> u16 {
    trace_enter!();
    let result = u16::from_le_bytes([self.unclocked_read_u8(address), self.unclocked_read_u8(address.wrapping_add(1))]);
    trace_u16!(result);
    trace_exit!();
    result
  }

  #[named]
  #[allow(dead_code)]
  fn unclocked_write_u16(&mut self, address: u16, data: u16) {
    trace_enter!();
    let hi = (data >> 8) as u8;
    let lo = (data & 0xFF) as u8;
    self.unclocked_write_u8(address, lo);
    self.unclocked_write_u8(address.wrapping_add(1), hi);
    trace_exit!();
  }
}

impl Addressable for CPU<'_> {
  #[named]
  fn read_u8(&mut self, address: u16) -> u8 {
    trace_enter!();
    trace_u16!(address);
    self.tick();
    let result = self.unclocked_read_u8(address);
    trace_result!(result);
    result
  }

  #[named]
  fn write_u8(&mut self, address: u16, data: u8) {
    trace_enter!();
    trace_u16!(address);
    trace_u16!(data);
    self.tick();
    self.unclocked_write_u8(address, data);
    trace_exit!();
  }

  #[named]
  fn load(&mut self, program: Vec<u8>) {
    trace_enter!();
    self.bus.load(program);
    trace_exit!();
  }

  #[named]
  fn tick(&mut self) {
    trace_enter!();
    self.clock_counter = self.clock_counter.wrapping_add(1);
    debug!("Tick {}", self.clock_counter);
    self.bus.tick();
    trace_exit!();
  }
}

impl fmt::Display for CPU<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}
