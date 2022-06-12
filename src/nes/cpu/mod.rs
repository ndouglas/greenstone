use std::collections::HashMap;

use crate::nes::bus::Bus;
use crate::nes::simple_memory::SimpleMemory;
use crate::traits::Addressable;

pub mod addressing_mode;
pub use addressing_mode::*;

pub mod instructions;
pub use instructions::*;

pub mod opcode;
pub use opcode::*;

pub mod status_flags;
pub use status_flags::*;

pub struct CPU<'a> {
  pub a: u8,
  pub x: u8,
  pub y: u8,
  pub status: u8,
  pub stack_pointer: u8,
  pub program_counter: u16,
  pub clock_counter: u32,
  pub cycles: u8,
  pub halt: bool,
  pub addressable: Box<dyn Addressable + 'a>,
}

impl<'a> CPU<'a> {
  pub fn new() -> CPU<'a> {
    CPU {
      a: 0x00,
      x: 0x00,
      y: 0x00,
      status: 0x00,
      stack_pointer: 0x00,
      program_counter: 0x0000,
      clock_counter: 0,
      cycles: 0x00,
      halt: false,
      addressable: Box::new(SimpleMemory::new()),
    }
  }

  pub fn new_with_bus() -> CPU<'a> {
    CPU {
      a: 0x00,
      x: 0x00,
      y: 0x00,
      status: 0x00,
      stack_pointer: 0x00,
      program_counter: 0x0000,
      clock_counter: 0,
      cycles: 0x00,
      halt: false,
      addressable: Box::new(Bus::new()),
    }
  }

  pub fn interpret(&mut self, program: Vec<u8>) {
    self.load(program);
    self.reset();
    self.run()
  }

  pub fn run(&mut self) {
    loop {
      self.clock();
      if self.halt {
        return;
      }
    }
  }

  pub fn clock(&mut self) {
    if self.cycles == 0 {
      self.dequeue_instruction();
    }
    self.clock_counter += 1;
    self.cycles -= 1;
  }

  pub fn dequeue_instruction(&mut self) {
    let ref opcodes: HashMap<u8, &'static Opcode> = *OPCODE_MAP;
    let code = self.read_u8(self.program_counter);
    self.program_counter += 1;
    let pc_state = self.program_counter;
    let opcode = opcodes.get(&code).expect(&format!("Opcode {:x} is not recognized", code));
    let opcode_length = opcode.length;
    let mut opcode_cycles = opcode.cycles;
    let extra_cycles = match code {
      // ADC
      0x61 | 0x65 | 0x69 | 0x6D | 0x71 | 0x75 | 0x79 | 0x7D => self.instruction_adc(&opcode.mode),
      // BRK
      0x00 => self.instruction_brk(&opcode.mode),
      // INX
      0xE8 => self.instruction_inx(&opcode.mode),
      // LDA
      0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => self.instruction_lda(&opcode.mode),
      // STA
      0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => self.instruction_sta(&opcode.mode),
      // TAY
      0xA8 => self.instruction_tay(&opcode.mode),
      // TAX
      0xAA => self.instruction_tax(&opcode.mode),
      _ => todo!(),
    };
    opcode_cycles += (extra_cycles && opcode.extra_cycle) as u8;
    if pc_state == self.program_counter {
      self.program_counter += (opcode_length - 1) as u16;
    }
    self.cycles += opcode_cycles;
  }

  //  Opcode::new(0x61, "ADC", 2, 6, AddressingMode::IndirectX, false, false, false, false),
  //  Opcode::new(0x65, "ADC", 2, 3, AddressingMode::ZeroPage, false, false, false, false),
  //  Opcode::new(0x69, "ADC", 2, 2, AddressingMode::Immediate, false, false, false, false),
  //  Opcode::new(0x6D, "ADC", 3, 4, AddressingMode::Absolute, false, false, false, false),
  //  Opcode::new(0x71, "ADC", 2, 5, AddressingMode::IndirectY, false, false, false, true),
  //  Opcode::new(0x75, "ADC", 2, 4, AddressingMode::ZeroPageX, false, false, false, false),
  //  Opcode::new(0x79, "ADC", 3, 4, AddressingMode::AbsoluteY, false, false, false, true),
  //  Opcode::new(0x7D, "ADC", 3, 4, AddressingMode::AbsoluteX, false, false, false, true),

  pub fn reset(&mut self) {
    self.a = 0x00;
    self.x = 0x00;
    self.y = 0x00;
    self.stack_pointer = 0x00;
    self.status = 0x00;
    self.clock_counter = 0;
    self.cycles = 0;
    self.halt = false;
    self.program_counter = self.read_u16(0xFFFC);
  }
}

impl Addressable for CPU<'_> {
  fn read_u8(&self, address: u16) -> u8 {
    self.addressable.read_u8(address)
  }

  fn write_u8(&mut self, address: u16, data: u8) {
    self.addressable.write_u8(address, data);
  }

  fn load(&mut self, program: Vec<u8>) {
    self.addressable.load(program)
  }
}
