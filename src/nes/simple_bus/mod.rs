const MAX_ADDRESS: usize = 0xFFFF;

mod addressable;

mod busable;

mod interruptible;

pub struct SimpleBus {
  memory: [u8; MAX_ADDRESS + 1],
}

impl SimpleBus {
  #[named]
  pub fn new() -> SimpleBus {
    SimpleBus {
      memory: [0; MAX_ADDRESS + 1],
    }
  }
}

impl Default for SimpleBus {
  fn default() -> Self {
    Self::new()
  }
}
