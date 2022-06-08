use super::super::CPU;

impl CPU {
  #[inline]
  pub fn opcode_a9(&mut self, param: u8) {
        self.program_counter += 1;
        self.a = param;
        self.set_zero_flag(param == 0);
        self.set_negative_flag(param & 0b1000_0000 != 0);
    }
}
