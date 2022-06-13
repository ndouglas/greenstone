use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_sec(&mut self, _opcode: &Opcode) -> u8 {
    trace_enter!();
    self.set_carry_flag(true);
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
