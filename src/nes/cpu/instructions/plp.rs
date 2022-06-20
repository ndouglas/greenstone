use super::super::*;

impl CPU {
  //
  // PLA, PLP (from 6502_cpu.txt)
  //
  // #  address R/W description
  //--- ------- --- -----------------------------------------------
  // 1    PC     R  fetch opcode, increment PC
  // 2    PC     R  read next instruction byte (and throw it away)
  // 3  $0100,S  R  increment S
  // 4  $0100,S  R  pull register from stack
  //
  #[inline]
  #[named]
  pub fn instruction_plp(&mut self, _opcode: &Opcode) {
    trace_enter!();
    debug!("Ticking (reading and discarding the next byte)...");
    self.tick();
    debug!("Ticking (incrementing the stack pointer register)...");
    self.tick();
    let output = self.pop_u8() & !UNUSED_FLAG & !BREAK_FLAG;
    trace_u8!(output);
    self.status = output;
    trace_u8!(self.status);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_plp() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("PLP", Implied, []{} => []{});
  }
}
