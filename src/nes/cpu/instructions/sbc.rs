use crate::nes::common::add_u8s;

use super::super::*;

impl CPU {
  #[inline]
  #[named]
  pub fn instruction_sbc(&mut self, opcode: &Opcode) {
    trace_enter!();
    trace_var!(opcode);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    let minuend = self.a;
    trace_u8!(minuend);
    let subtrahend = (self.read_u8(address) as i8).wrapping_neg().wrapping_sub(1) as u8;
    trace_u8!(subtrahend);
    let carry = self.get_carry_flag();
    trace_var!(carry);
    let (answer, set_carry, set_overflow) = add_u8s(minuend, subtrahend, carry);
    trace_u8!(answer);
    trace_var!(set_carry);
    trace_var!(set_overflow);
    self.a = answer;
    self.set_carry_flag(set_carry);
    self.set_overflow_flag(set_overflow);
    self.set_value_flags(answer);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_sbc() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("SBC", Immediate, [2]{a:10, status:1} => []{ a: 8 });
    test_instruction!("SBC", Immediate, [2]{a:10, status:0} => []{ a: 7 });
    test_instruction!("SBC", Immediate, [176]{a:80, status:1} => []{ a: 160, status: 0b11000000 });
    test_instruction!("SBC", ZeroPage,  [0x02, 0x90]{a: 0xFF, status: 1} => []{ a: 0x6F });
    test_instruction!("SBC", ZeroPageX, [0x02, 0x00, 0x90]{x:1, a: 0xFF, status: 1} => []{ a: 0x6F });
    test_instruction!("SBC", Absolute,  [0x04, 0x00, 0x00, 0x90]{a:0xFF, status: 1} => []{ a: 0x6F });
    test_instruction!("SBC", IndirectX, [0x02, 0x00, 0x05, 0x00, 0x90]{x:1, a: 0xFF, status: 1} => []{ a: 0x6F });
    test_instruction!("SBC", IndirectY, [0x02, 0x04, 0x00, 0x00, 0x90]{y:1, a: 0xFF, status: 1} => []{ a: 0x6F });
    test_instruction!("SBC", AbsoluteX, [0x03, 0x00, 0x00, 0x90]{x:1, a: 0xFF, status: 1} => []{ a: 0x6F });
    test_instruction!("SBC", AbsoluteY, [0x03, 0x00, 0x00, 0x90]{y:1, a: 0xFF, status: 1} => []{ a: 0x6F });
  }
}
