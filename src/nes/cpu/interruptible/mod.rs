use super::super::*;
use crate::traits::Addressable;
use crate::traits::Interruptible;

pub const NMI_ADDRESS: u16 = 0xFFFA;
pub const RESET_ADDRESS: u16 = 0xFFFC;
pub const IRQ_ADDRESS: u16 = 0xFFFE;
pub const RESET_STACK: u8 = 0xFD;
pub const RESET_STATUS: u8 = UNUSED_FLAG | INTERRUPT_DISABLE_FLAG;

impl Interruptible for CPU {
  #[named]
  #[inline]
  fn is_nmi_ready(&self) -> bool {
    trace_enter!();
    let result = self.bus.is_nmi_ready();
    trace_var!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  fn acknowledge_nmi(&mut self) {
    trace_enter!();
    self.bus.acknowledge_nmi();
    trace_exit!();
  }

  #[named]
  #[inline]
  fn is_irq_ready(&self) -> bool {
    trace_enter!();
    let result = self.bus.is_irq_ready();
    trace_var!(result);
    trace_exit!();
    result
  }

  #[named]
  #[inline]
  fn handle_nmi(&mut self) {
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
  fn handle_irq(&mut self) {
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
  fn handle_break(&mut self) {
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
  fn handle_reset(&mut self) {
    trace_enter!();
    debug!("Ticking five times (reset sequence)...");
    self.clock_counter = 0;
    for _ in 0..5 {
      self.tick();
    }
    self.a = 0x00;
    self.x = 0x00;
    self.y = 0x00;
    self.stack_pointer = RESET_STACK;
    self.status = RESET_STATUS;
    debug!("Ticking twice (reading initial state for program counter)...");
    self.program_counter = self.read_u16(RESET_ADDRESS);
    trace_exit!();
  }
}
