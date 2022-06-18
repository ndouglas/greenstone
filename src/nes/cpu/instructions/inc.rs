use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_inc(&mut self, opcode: &Opcode) {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let mode = &opcode.mode;
    trace_var!(mode);
    let address = self.get_operand_address(opcode, mode).unwrap();
    trace_u16!(address);
    let operand = self.read_u8(address);
    trace_u8!(operand);
    let output = operand.wrapping_add(1);
    trace_u8!(output);
    self.tick();
    self.write_u8(address, output);
    self.set_value_flags(output);
    trace_exit!();
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::test::init;

  #[test]
  #[named]
  fn test_inc() {
    init();
    // These test cases are based on Starr Horne's `nes-rust`.
    // See https://github.com/starrhorne/nes-rust/blob/master/src/cpu_test.rs
    test_instruction!("INC", ZeroPage,  [0x02, 255]{} => [0x02, 0]{status: 0b00000010});
    test_instruction!("INC", ZeroPage,  [0x02, 127]{} => [0x02, 128]{status: 0b10000000});
    test_instruction!("INC", ZeroPageX, [0x02, 0x00, 0x02]{x: 1} => [0x02, 0x00, 0x03]{});
    test_instruction!("INC", Absolute,  [0x03, 0x00, 0x02]{} => [0x03, 0x00, 0x03]{});
    test_instruction!("INC", AbsoluteX, [0x03, 0x00, 0x00, 0x02]{x: 1} => [0x03, 0x00, 0x00, 0x03]{});
  }
}
