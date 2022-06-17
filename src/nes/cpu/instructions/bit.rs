use super::super::*;

impl CPU<'_> {
  #[inline]
  #[named]
  pub fn instruction_bit(&mut self, opcode: &Opcode) -> u8 {
    trace_enter!();
    let length = opcode.length;
    trace_u8!(length);
    let cycles = opcode.cycles;
    let mode = &opcode.mode;
    trace_var!(mode);
    let (address, additional_cycles) = self.get_operand_address(mode).unwrap();
    trace_u16!(address);
    trace_u8!(additional_cycles);
    let operand = self.read_u8(address);
    trace_u8!(operand);
    self.set_zero_flag(self.a & operand == 0);
    self.set_overflow_flag(operand & OVERFLOW_FLAG != 0);
    self.set_negative_flag(operand & NEGATIVE_FLAG != 0);
    let result = cycles;
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
  fn test_bit() {
    init();
    test_instruction!("BIT", ZeroPage,  [0x02, 0x00]{a: 0x0F} => []{status: 0b00000010});
    test_instruction!("BIT", ZeroPage,  [0x02, 0xF0]{a: 0xFF} => []{status: 0b11000000});
    test_instruction!("BIT", Absolute,  [0x03, 0x00, 0xF0]{a: 0xFF} => []{status: 0b11000000});
  }
}