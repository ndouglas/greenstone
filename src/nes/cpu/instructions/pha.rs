use super::super::*;

impl CPU {
  //
  // PHA, PHP (from 6502_cpu.txt)
  //
  // #  address R/W description
  //--- ------- --- -----------------------------------------------
  // 1    PC     R  fetch opcode, increment PC
  // 2    PC     R  read next instruction byte (and throw it away)
  // 3  $0100,S  W  push register on stack, decrement S
  //
  #[inline]
  #[named]
  pub fn instruction_pha(&mut self, _opcode: &Opcode) {
    trace_enter!();
    debug!("Ticking (reading and discarding the next byte)...");
    self.tick();
    self.push_u8(self.a);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_pha() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("PHA", Implied, []{} => []{}, |_cpu: &mut CPU, _opcode: &Opcode| {});
  }
}
