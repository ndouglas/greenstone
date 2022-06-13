use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_sec(&mut self, _mode: &AddressingMode) -> bool {
    trace_enter!();
    self.set_carry_flag(true);
    trace_var!(self.get_carry_flag());
    trace_result!(false);
    false
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_sec_0x38_implied_set_carry_flag() {
    init();
    let mut cpu = CPU::new();
    cpu.interpret(vec![
      0x38, //            SEC         ; Set carry flag.
      0x00, //            BRK         ;
    ]);
    assert!(cpu.status & CARRY_FLAG == CARRY_FLAG, "should set the carry flag.");
  }

}
