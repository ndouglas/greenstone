use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_jsr(&mut self, _opcode: &Opcode) {
    trace_enter!();
    debug!("Ticking (processing instruction)...");
    self.tick();
    debug!("Ticking (processing instruction)...");
    self.tick();
    let address = self.pop_u16().wrapping_add(1);
    trace_u16!(address);
    self.program_counter = address;
    debug!("Ticking (processing instruction)...");
    self.tick();
    trace_exit!();
  }
}
fn rts(&mut self) {
  self.bus.tick();
  self.bus.tick();
  self.pc = self.pop_word() + 1;
  self.bus.tick();
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_jsr() {
    init();
    test_instruction!("JSR", Absolute, [0x0A, 0x00]{} => []{program_counter: 10});
    test_instruction!("JSR", Indirect, [0x03, 0x00, 0x0A, 0x00]{} => []{program_counter: 10});
  }
}

