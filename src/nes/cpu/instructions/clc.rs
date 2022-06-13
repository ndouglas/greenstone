use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_clc(&mut self, _opcode: &Opcode) -> u8 {
    trace_enter!();
    self.set_carry_flag(false);
    trace_var!(self.get_carry_flag());
    let result = 0;
    trace_result!(result);
    result
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_clc_implied_clear_carry_flag() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0x18, //            CLC         ; Clear carry flag.
      0x00, //            BRK         ;
    ]);
    assert!(cpu.status & CARRY_FLAG == 0, "should clear the carry flag.");
  }
}
