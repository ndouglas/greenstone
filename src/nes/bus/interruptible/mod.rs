use super::*;
use crate::traits::Interruptible;

impl Interruptible for Bus {
  #[named]
  #[inline]
  fn is_nmi_ready(&self) -> bool {
    false
  }

  #[named]
  #[inline]
  fn acknowledge_nmi(&mut self) {}

  #[named]
  #[inline]
  fn is_irq_ready(&self) -> bool {
    false
  }

  #[named]
  #[inline]
  fn handle_nmi(&mut self) {}

  #[named]
  #[inline]
  fn handle_irq(&mut self) {}

  #[named]
  #[inline]
  fn handle_break(&mut self) {}

  #[named]
  #[inline]
  fn handle_reset(&mut self) {}
}
