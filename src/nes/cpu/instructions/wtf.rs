use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_wtf(&mut self, opcode: &Opcode) {
    trace_enter!();
    panic!("Opcode unknown or unimplemented: {}", opcode);
  }
}
