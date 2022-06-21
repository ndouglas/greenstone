use std::collections::HashMap;
use std::fmt;

use crate::nes::simple_bus::SimpleBus;
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
pub struct CPU {
  pub a: u8,
  pub x: u8,
  pub y: u8,
  pub status: u8,
  pub stack_pointer: u8,
  pub program_counter: u16,
  pub clock_counter: u64,
  #[derivative(Debug = "ignore")]
  pub bus: Box<dyn Busable>,
}

impl CPU {
  #[named]
  pub fn new() -> CPU {
    CPU {
      a: 0x00,
      x: 0x00,
      y: 0x00,
      status: 0x00,
      stack_pointer: 0xFF,
      program_counter: 0x0000,
      clock_counter: 0,
      bus: Box::new(SimpleBus::new()),
    }
  }

  #[named]
  pub fn new_with_bus(bus: Box<dyn Busable>) -> CPU {
    CPU {
      a: 0x00,
      x: 0x00,
      y: 0x00,
      status: 0x00,
      stack_pointer: 0xFF,
      program_counter: 0x0000,
      clock_counter: 0,
      bus,
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
    (opcode.function)(self, &opcode);
    trace_exit!();
  }

  #[named]
  #[inline]
  pub fn increment_program_counter(&mut self) {
    trace_enter!();
    self.program_counter = self.program_counter.wrapping_add(1);
    debug!("Incremented program counter to {}.", format_u16!(self.program_counter));
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

impl fmt::Display for CPU {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}
