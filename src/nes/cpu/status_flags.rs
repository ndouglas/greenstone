use super::CPU;

pub enum StatusFlags {
  Negative = 0b1000_0000,
  Overflow = 0b0100_0000,
  NoEffect5 = 0b0010_0000,
  NoEffect4 = 0b0001_0000,
  Decimal = 0b0000_1000,
  InterruptDisable = 0b0000_0100,
  Zero = 0b0000_0010,
  Carry = 0b0000_0001,
}

impl CPU {

  #[inline]
  pub fn set_status_flag(&mut self, flag: StatusFlags, value: bool) {
      if value {
          self.status = self.status | (flag as u8);
      } else {
          self.status = self.status & !(flag as u8);
      }
  }

  #[inline]
  pub fn set_negative_flag(&mut self, value: bool) {
      self.set_status_flag(StatusFlags::Negative, value);
  }

  #[inline]
  pub fn set_overflow_flag(&mut self, value: bool) {
      self.set_status_flag(StatusFlags::Overflow, value);
  }

  #[inline]
  pub fn set_no_effect_5_flag(&mut self, value: bool) {
      self.set_status_flag(StatusFlags::NoEffect5, value);
  }

  #[inline]
  pub fn set_no_effect_4_flag(&mut self, value: bool) {
      self.set_status_flag(StatusFlags::NoEffect4, value);
  }

  #[inline]
  pub fn set_decimal_flag(&mut self, value: bool) {
      self.set_status_flag(StatusFlags::Decimal, value);
  }

  #[inline]
  pub fn set_interrupt_disable_flag(&mut self, value: bool) {
      self.set_status_flag(StatusFlags::InterruptDisable, value);
  }

  #[inline]
  pub fn set_zero_flag(&mut self, value: bool) {
      self.set_status_flag(StatusFlags::Zero, value);
  }

  #[inline]
  pub fn set_carry_flag(&mut self, value: bool) {
      self.set_status_flag(StatusFlags::Carry, value);
  }

}
