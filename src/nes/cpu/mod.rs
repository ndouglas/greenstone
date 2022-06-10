pub mod opcodes;
pub use opcodes::*;
pub mod status_flags;
pub use status_flags::*;

pub struct CPU {
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
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            status: 0x00,
            stack_pointer: 0x00,
            program_counter: 0x0000,
        }
    }

    pub fn interpret(&mut self, program: Vec<u8>) {
        self.program_counter = 0;
        loop {
            let opcode = program[self.program_counter as usize];
            self.program_counter += 1;
            match opcode {
                // BRK
                0x00 => return,
                // INX
                0xE8 => self.opcode_inx(),
                // LDA
                0xA9 => self.opcode_lda(program[self.program_counter as usize]),
                // TAX
                0xAA => self.opcode_tax(),
                _ => todo!(),
            }
        }
    }
}
