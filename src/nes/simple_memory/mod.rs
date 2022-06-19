use crate::traits::Addressable;

const START_ADDRESS: usize = 0x0000;
const PROGRAM_CONTROL_ADDRESS: usize = 0xFFFC;
const MAX_ADDRESS: usize = 0xFFFF;

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

impl Addressable for SimpleMemory {
  #[named]
  fn read_u8(&mut self, address: u16) -> u8 {
    trace_enter!();
    let result = self.memory[address as usize];
    trace_u8!(result);
    trace_exit!();
    result
  }

  #[named]
  fn write_u8(&mut self, address: u16, data: u8) {
    trace_enter!();
    self.memory[address as usize] = data;
    trace_exit!();
  }

  #[named]
  fn load(&mut self, program: Vec<u8>, start: u16) {
    trace_enter!();
    let start_address = start as usize;
    self.memory[start_address..(start_address + program.len())].copy_from_slice(&program[..]);
    self.write_u16(PROGRAM_CONTROL_ADDRESS.try_into().unwrap(), start);
    trace_exit!();
  }

  #[named]
  fn tick(&mut self) {
    trace_enter!();
    trace_exit!();
  }
}
