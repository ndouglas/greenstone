use crate::nes::bus::Bus;
use crate::nes::simple_memory::SimpleMemory;
use crate::traits::Addressable;

pub mod addressing_modes;
pub use addressing_modes::*;

pub mod opcodes;
pub use opcodes::*;

pub mod status_flags;
pub use status_flags::*;

pub struct CPU<'a> {
    /// A (Accumulator) register.
    pub a: u8,

    /// X register.
    pub x: u8,

    /// Y register.
    pub y: u8,

    /// Status register.
    pub status: u8,

    /// Stack pointer.
    pub stack_pointer: u8,

    /// Program counter.
    pub program_counter: u16,

    /// Addressable memory.
    pub addressable: Box<dyn Addressable + 'a>,
}

impl<'a> CPU<'a> {
    pub fn new() -> CPU<'a> {
        CPU {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            status: 0x00,
            stack_pointer: 0x00,
            program_counter: 0x0000,
            addressable: Box::new(SimpleMemory::new()),
        }
    }

    pub fn new_with_bus() -> CPU<'a> {
        CPU {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            status: 0x00,
            stack_pointer: 0x00,
            program_counter: 0x0000,
            addressable: Box::new(Bus::new()),
        }
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.read_u8(self.program_counter);
            self.program_counter += 1;
            match opcode {
                // BRK
                0x00 => return,
                // INX
                0xE8 => self.opcode_inx(),
                // LDA
                0xA5 => {
                    self.opcode_lda(&AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }
                0xA9 => {
                    self.opcode_lda(&AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0xAD => {
                    self.opcode_lda(&AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                // TAX
                0xAA => self.opcode_tax(),
                _ => todo!(),
            }
        }
    }

    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.stack_pointer = 0;
        self.status = 0;
        self.program_counter = self.read_u16(0xFFFC);
    }
}

impl Addressable for CPU<'_> {
    fn read_u8(&self, address: u16) -> u8 {
        self.addressable.read_u8(address)
    }

    fn write_u8(&mut self, address: u16, data: u8) {
        self.addressable.write_u8(address, data);
    }

    fn load(&mut self, program: Vec<u8>) {
        self.addressable.load(program)
    }
}
