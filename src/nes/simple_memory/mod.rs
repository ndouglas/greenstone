const START_ADDRESS: usize = 0x0000;
const PROGRAM_CONTROL_ADDRESS: usize = 0xFFFC;
const MAX_ADDRESS: usize = 0xFFFF;

pub mod addressable;
pub use addressable::*;

pub mod busable;
pub use busable::*;

pub mod interruptible;
pub use interruptible::*;

pub struct SimpleMemory {
  memory: [u8; (MAX_ADDRESS + 1)],
}

impl SimpleMemory {
  #[named]
  pub fn new() -> SimpleMemory {
    SimpleMemory {
      memory: [0; (MAX_ADDRESS + 1)],
    }
  }
}
