use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_jmp(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    self.program_counter = address;
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_jmp() {
    init();
    test_instruction!("JMP", Absolute, [0x0A, 0x00]{} => []{program_counter: 10});
    test_instruction!("JMP", Indirect, [0x03, 0x00, 0x0A, 0x00]{} => []{program_counter: 10});
  }
}

// test_op!("jmp", Absolute, [10, 0]{} => []{pc: 10});
// test_op!("jmp", Indirect, [3, 0, 10, 0]{} => []{pc: 10});
// 
// // Test page boundary bug
// let mut cpu = build_cpu!([0]);
// cpu.pc = 0;
// cpu.bus.ram[0] = 0xFF;
// cpu.bus.ram[1] = 0x01;
// cpu.bus.ram[0x01FF] = 0x11;
// cpu.bus.ram[0x0100] = 0x22;
// cpu.jmp(Indirect);
// assert_eq!(cpu.pc, 0x2211);
// }
