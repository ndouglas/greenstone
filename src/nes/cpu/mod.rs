use std::collections::HashMap;

use crate::nes::bus::Bus;
use crate::nes::simple_memory::SimpleMemory;
use crate::traits::Addressable;

pub mod addressing_mode;
pub use addressing_mode::*;

pub mod opcode;
pub use opcode::*;

pub mod opcodes;
pub use opcodes::*;

pub mod status_flags;
pub use status_flags::*;

pub struct CPU<'a> {
  pub a: u8,
  pub x: u8,
  pub y: u8,
  pub status: u8,
  pub stack_pointer: u8,
  pub program_counter: u16,
  pub cycle_slack: u8,
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
      cycle_slack: 0x00,
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
      cycle_slack: 0x00,
      addressable: Box::new(Bus::new()),
    }
  }

  pub fn interpret(&mut self, program: Vec<u8>) {
    self.load(program);
    self.reset();
    self.run()
  }

  pub fn run(&mut self) {
    let ref opcodes: HashMap<u8, &'static Opcode> = *OPCODE_MAP;
    loop {
      let code = self.read_u8(self.program_counter);
      self.program_counter += 1;
      let program_counter_state = self.program_counter;
      let opcode = opcodes
        .get(&code)
        .expect(&format!("Opcode {:x} is not recognized", code));
      match code {
        // BRK
        0x00 => return,
        // INX
        0xE8 => self.opcode_inx(),
        // LDA
        0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
          self.opcode_lda(&opcode.mode);
        }
        // STA
        0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => {
          self.opcode_sta(&opcode.mode);
        }
        // TAY
        0xA8 => self.opcode_tay(),
        // TAX
        0xAA => self.opcode_tax(),
        _ => todo!(),
      }
      if program_counter_state == self.program_counter {
        self.program_counter += (opcode.length - 1) as u16;
      }
    }
  }

  pub fn reset(&mut self) {
    self.a = 0;
    self.x = 0;
    self.stack_pointer = 0;
    self.status = 0;
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
