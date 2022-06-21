use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_jam(&mut self, _opcode: &Opcode) {
    trace_enter!();
    panic!("Maybe don't call JAM.");
  }
}
