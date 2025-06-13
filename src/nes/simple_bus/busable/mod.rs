use super::*;
use crate::traits::Busable;

impl Busable for SimpleBus {
  fn as_any(&self) -> &dyn std::any::Any {
    self
  }

  fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
    self
  }
}
