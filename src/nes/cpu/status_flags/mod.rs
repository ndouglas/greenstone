use super::CPU;

pub const NEGATIVE_FLAG: u8 = 0b1000_0000;
pub const OVERFLOW_FLAG: u8 = 0b0100_0000;
pub const NOEFFECT5_FLAG: u8 = 0b0010_0000;
pub const NOEFFECT4_FLAG: u8 = 0b0001_0000;
pub const DECIMAL_FLAG: u8 = 0b0000_1000;
pub const INTERRUPT_DISABLE_FLAG: u8 = 0b0000_0100;
pub const ZERO_FLAG: u8 = 0b0000_0010;
pub const CARRY_FLAG: u8 = 0b0000_0001;

#[repr(u8)]
pub enum StatusFlags {
    Negative = NEGATIVE_FLAG,
    Overflow = OVERFLOW_FLAG,
    NoEffect5 = NOEFFECT5_FLAG,
    NoEffect4 = NOEFFECT4_FLAG,
    Decimal = DECIMAL_FLAG,
    InterruptDisable = INTERRUPT_DISABLE_FLAG,
    Zero = ZERO_FLAG,
    Carry = CARRY_FLAG,
}

impl CPU<'_> {
    #[inline]
    pub fn set_status_flag(&mut self, flag: StatusFlags, value: bool) {
        if value {
            self.status = self.status | (flag as u8);
        } else {
            self.status = self.status & !(flag as u8);
        }
    }

    #[inline]
    pub fn get_status_flag(&self, flag: StatusFlags) -> bool {
        return self.status & (flag as u8) > 0;
    }

    #[inline]
    pub fn set_negative_flag(&mut self, value: bool) {
        self.set_status_flag(StatusFlags::Negative, value);
    }

    #[inline]
    pub fn get_negative_flag(&self) -> bool {
        return self.get_status_flag(StatusFlags::Negative);
    }

    #[inline]
    pub fn set_overflow_flag(&mut self, value: bool) {
        self.set_status_flag(StatusFlags::Overflow, value);
    }

    #[inline]
    pub fn get_overflow_flag(&self) -> bool {
        return self.get_status_flag(StatusFlags::Overflow);
    }

    #[inline]
    pub fn set_no_effect_5_flag(&mut self, value: bool) {
        self.set_status_flag(StatusFlags::NoEffect5, value);
    }

    #[inline]
    pub fn get_no_effect_5_flag(&self) -> bool {
        return self.get_status_flag(StatusFlags::NoEffect5);
    }

    #[inline]
    pub fn set_no_effect_4_flag(&mut self, value: bool) {
        self.set_status_flag(StatusFlags::NoEffect4, value);
    }

    #[inline]
    pub fn get_no_effect_4_flag(&self) -> bool {
        return self.get_status_flag(StatusFlags::NoEffect4);
    }

    #[inline]
    pub fn set_decimal_flag(&mut self, value: bool) {
        self.set_status_flag(StatusFlags::Decimal, value);
    }

    #[inline]
    pub fn get_decimal_flag(&self) -> bool {
        return self.get_status_flag(StatusFlags::Decimal);
    }

    #[inline]
    pub fn set_interrupt_disable_flag(&mut self, value: bool) {
        self.set_status_flag(StatusFlags::InterruptDisable, value);
    }

    #[inline]
    pub fn get_interrupt_disable_flag(&self) -> bool {
        return self.get_status_flag(StatusFlags::InterruptDisable);
    }

    #[inline]
    pub fn set_zero_flag(&mut self, value: bool) {
        self.set_status_flag(StatusFlags::Zero, value);
    }

    #[inline]
    pub fn get_zero_flag(&self) -> bool {
        return self.get_status_flag(StatusFlags::Zero);
    }

    #[inline]
    pub fn set_carry_flag(&mut self, value: bool) {
        self.set_status_flag(StatusFlags::Carry, value);
    }

    #[inline]
    pub fn get_carry_flag(&self) -> bool {
        return self.get_status_flag(StatusFlags::Carry);
    }

    #[inline]
    pub fn set_value_flags(&mut self, value: u8) {
        self.set_zero_flag(value == 0);
        self.set_negative_flag(value & NEGATIVE_FLAG == NEGATIVE_FLAG);
    }
}
