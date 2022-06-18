use super::super::*;
use crate::traits::addressable::Addressable;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Interrupt {
  NonMaskable,
  Reset,
  Request,
  Break,
}

impl CPU<'_> {
  #[named]
  pub fn reset(&mut self) {
    self.a = 0x00;
    self.x = 0x00;
    self.y = 0x00;
    self.stack_pointer = 0x00;
    self.status = 0x00;
    self.clock_counter = 0;
    self.halt = false;
    debug!("Ticking twice (reading initial state for program counter)...");
    self.program_counter = self.read_u16(0xFFFC);
  }
}
