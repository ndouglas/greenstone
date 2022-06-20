// A trait for things that listen to interrupts.

pub trait Interruptible {
  fn is_nmi_ready(&self) -> bool;

  fn acknowledge_nmi(&mut self);

  fn is_irq_ready(&self) -> bool;

  fn handle_nmi(&mut self);

  fn handle_irq(&mut self);

  fn handle_break(&mut self);

  fn handle_reset(&mut self);
}
