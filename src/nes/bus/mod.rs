const PROGRAM_CONTROL_ADDRESS: usize = 0xFFFC;
const MAX_ADDRESS: usize = 0xFFFF;

pub mod addressable;
pub use addressable::*;

pub mod busable;
pub use busable::*;

pub mod interruptible;
pub use interruptible::*;

pub struct Bus {
  memory: [u8; MAX_ADDRESS + 1],
}

impl Bus {
  pub fn new() -> Bus {
    Bus { memory: [0; MAX_ADDRESS + 1] }
  }
}
