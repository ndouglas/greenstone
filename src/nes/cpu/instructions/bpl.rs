use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_bpl(&mut self, opcode: &Opcode) {
    trace_enter!();
    self.branch_on_condition(opcode, !self.get_negative_flag());
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_bpl() {
    init();
    // test_instruction!("BPL", Relative,  [0x10]{status: 0b10000000} => []{program_counter: 2, clock_counter: 2});
    // test_instruction!("BPL", Relative,  []{x: 127} => []{x: 128, clock_counter: 3, status: 0b10000000});
  }

//   #[test]
// fn test_bpl() {
//     let cpu = test_op!("bpl", NoMode, [10]{p: 0b10000000} => []{pc: 2});
//     assert_eq!(cpu.bus.cycles, 2);
// 
//     let cpu = test_op!("bpl", NoMode, [10]{p: 0} => []{pc: 12});
//     assert_eq!(cpu.bus.cycles, 3);
// 
//     // Test page boundary cross
//     let mut cpu = build_cpu!([0]);
//     cpu.pc = 0x00FE;
//     cpu.bus.ram[0x00FE] = 1;
//     cpu.bpl();
//     assert!(cross(0x00FF, 1));
//     assert_eq!(cpu.pc, 0x0100);
//     assert_eq!(cpu.bus.cycles, 3); // Because we call bpl directly, it's only 3 cycles
// }
}
