const MAX_ADDRESS: usize = 0xFFFF;

pub mod addressable;
pub use addressable::*;

pub mod busable;
pub use busable::*;

pub mod interruptible;
pub use interruptible::*;

pub struct SimpleBus {
  memory: [u8; (MAX_ADDRESS + 1)],
}

impl SimpleBus {
  #[named]
  pub fn new() -> SimpleBus {
    SimpleBus {
      memory: [0; (MAX_ADDRESS + 1)],
    }
  }
}
