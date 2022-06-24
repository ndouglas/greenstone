use super::CPU;

pub const NEGATIVE_FLAG: u8 = 0b1000_0000;
pub const OVERFLOW_FLAG: u8 = 0b0100_0000;
pub const UNUSED_FLAG: u8 = 0b0010_0000;
pub const BREAK_FLAG: u8 = 0b0001_0000;
pub const DECIMAL_FLAG: u8 = 0b0000_1000;
pub const INTERRUPT_DISABLE_FLAG: u8 = 0b0000_0100;
pub const ZERO_FLAG: u8 = 0b0000_0010;
pub const CARRY_FLAG: u8 = 0b0000_0001;

#[repr(u8)]
#[derive(Debug)]
pub enum StatusFlags {
  Negative = NEGATIVE_FLAG,
  Overflow = OVERFLOW_FLAG,
  Unused = UNUSED_FLAG,
  Break = BREAK_FLAG,
  Decimal = DECIMAL_FLAG,
  InterruptDisable = INTERRUPT_DISABLE_FLAG,
  Zero = ZERO_FLAG,
  Carry = CARRY_FLAG,
}

impl CPU {
  #[inline]
  #[named]
  pub fn set_status_flag(&mut self, flag: StatusFlags, value: bool) {
    trace_enter!();
    trace_var!(flag);
    trace_var!(value);
    if value {
      self.status = self.status | (flag as u8);
    } else {
      self.status = self.status & !(flag as u8);
    }
    trace!("{}", format_cpu_status_register!(self.status));
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_status_flag(&self, flag: StatusFlags) -> bool {
    trace_enter!();
    let result = self.status & (flag as u8) > 0;
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_negative_flag(&mut self, value: bool) {
    trace_enter!();
    self.set_status_flag(StatusFlags::Negative, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_negative_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_status_flag(StatusFlags::Negative);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_overflow_flag(&mut self, value: bool) {
    trace_enter!();
    self.set_status_flag(StatusFlags::Overflow, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_overflow_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_status_flag(StatusFlags::Overflow);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_unused_flag(&mut self, value: bool) {
    trace_enter!();
    self.set_status_flag(StatusFlags::Unused, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_unused_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_status_flag(StatusFlags::Unused);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_break_flag(&mut self, value: bool) {
    trace_enter!();
    self.set_status_flag(StatusFlags::Break, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_break_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_status_flag(StatusFlags::Break);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_decimal_flag(&mut self, value: bool) {
    trace_enter!();
    self.set_status_flag(StatusFlags::Decimal, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_decimal_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_status_flag(StatusFlags::Decimal);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_interrupt_disable_flag(&mut self, value: bool) {
    trace_enter!();
    self.set_status_flag(StatusFlags::InterruptDisable, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_interrupt_disable_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_status_flag(StatusFlags::InterruptDisable);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_zero_flag(&mut self, value: bool) {
    trace_enter!();
    self.set_status_flag(StatusFlags::Zero, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_zero_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_status_flag(StatusFlags::Zero);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_carry_flag(&mut self, value: bool) {
    trace_enter!();
    self.set_status_flag(StatusFlags::Carry, value);
    trace_exit!();
  }

  #[inline]
  #[named]
  pub fn get_carry_flag(&self) -> bool {
    trace_enter!();
    let result = self.get_status_flag(StatusFlags::Carry);
    trace_result!(result);
    result
  }

  #[inline]
  #[named]
  pub fn set_value_flags(&mut self, value: u8) {
    trace_enter!();
    self.set_zero_flag(value == 0);
    self.set_negative_flag(value & NEGATIVE_FLAG == NEGATIVE_FLAG);
    trace_exit!();
  }
}
