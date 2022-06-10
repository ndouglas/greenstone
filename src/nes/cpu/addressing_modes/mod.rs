use crate::traits::addressable::Addressable;

use super::CPU;

#[derive(Debug)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
}

impl CPU<'_> {
    pub fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.read_u8(self.program_counter) as u16,
            AddressingMode::Absolute => self.read_u16(self.program_counter),
            AddressingMode::ZeroPageX => {
                let base = self.read_u8(self.program_counter);
                let address = base.wrapping_add(self.x) as u16;
                address
            }
            AddressingMode::ZeroPageY => {
                let base = self.read_u8(self.program_counter);
                let address = base.wrapping_add(self.y) as u16;
                address
            }
            AddressingMode::AbsoluteX => {
                let base = self.read_u16(self.program_counter);
                let address = base.wrapping_add(self.x as u16);
                address
            }
            AddressingMode::AbsoluteY => {
                let base = self.read_u16(self.program_counter);
                let address = base.wrapping_add(self.y as u16);
                address
            }
            AddressingMode::IndirectX => {
                let base = self.read_u8(self.program_counter);
                let pointer: u8 = (base as u8).wrapping_add(self.x);
                let lo = self.read_u8(pointer as u16);
                let hi = self.read_u8(pointer.wrapping_add(1) as u16);
                let address = (hi as u16) << 8 | (lo as u16);
                address
            }
            AddressingMode::IndirectY => {
                let base = self.read_u8(self.program_counter);
                let lo = self.read_u8(base as u16);
                let hi = self.read_u8((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let address = deref_base.wrapping_add(self.y as u16);
                address
            }
        }
    }
}
