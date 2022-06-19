use super::super::*;
use crate::traits::addressable::Addressable;

pub const NMI_ADDRESS: u16 = 0xFFFA;
pub const RESET_ADDRESS: u16 = 0xFFFC;
pub const IRQ_ADDRESS: u16 = 0xFFFE;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Interrupt {
  NonMaskable,
  Reset,
  Request,
  Break,
}

impl CPU<'_> {
  #[named]
  #[inline]
  pub fn is_nmi_ready(&self) -> bool {
    false
  }

  #[named]
  #[inline]
  pub fn acknowledge_nmi(&mut self) {}

  #[named]
  #[inline]
  pub fn is_irq_ready(&self) -> bool {
    false
  }

  #[named]
  #[inline]
  pub fn nmi(&mut self) {
    debug!("Ticking twice for NMI interrupt...");
    self.tick();
    self.tick();
    debug!("Ticking twice (pushing program counter to the stack)...");
    self.push_u16(self.program_counter);
    debug!("Ticking (pushing status to stack)...");
    self.push_u8(self.status | UNUSED_FLAG);
    self.set_interrupt_disable_flag(true);
    debug!("Ticking twice (reading NMI address)...");
    self.program_counter = self.read_u16(NMI_ADDRESS);
  }

  #[named]
  #[inline]
  pub fn irq(&mut self) {
    debug!("Ticking twice for IRQ interrupt...");
    self.tick();
    self.tick();
    debug!("Ticking twice (pushing program counter to the stack)...");
    self.push_u16(self.program_counter);
    debug!("Ticking (pushing status to stack)...");
    self.push_u8(self.status | UNUSED_FLAG);
    self.set_interrupt_disable_flag(true);
    debug!("Ticking twice (reading IRQ/Break address)...");
    self.program_counter = self.read_u16(IRQ_ADDRESS);
  }

  #[named]
  #[inline]
  pub fn r#break(&mut self) {
    debug!("Ticking for Break interrupt...");
    self.tick();
    debug!("Ticking twice (pushing program counter to the stack)...");
    self.push_u16(self.program_counter);
    debug!("Ticking (pushing status to stack)...");
    self.push_u8(self.status | UNUSED_FLAG | BREAK_FLAG);
    self.set_interrupt_disable_flag(true);
    debug!("Ticking twice (reading IRQ/Break address)...");
    self.program_counter = self.read_u16(IRQ_ADDRESS);
  }

  #[named]
  #[inline]
  pub fn reset(&mut self) {
    trace_enter!();
    debug!("Ticking five times (reset sequence)...");
    for _ in 0..5 {
      self.tick();
    }
    self.a = 0x00;
    self.x = 0x00;
    self.y = 0x00;
    self.stack_pointer = 0xFF;
    self.status = RESET_STATUS;
    self.clock_counter = 0;
    debug!("Ticking twice (reading initial state for program counter)...");
    self.program_counter = self.read_u16(RESET_ADDRESS);
    trace_exit!();
  }
}
