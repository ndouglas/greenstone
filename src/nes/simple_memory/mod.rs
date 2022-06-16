use crate::traits::Addressable;

const START_ADDRESS: usize = 0x0000;
const PROGRAM_CONTROL_ADDRESS: usize = 0xFFFC;
const MAX_ADDRESS: usize = 0xFFFF;

pub struct SimpleMemory {
  memory: [u8; MAX_ADDRESS],
}

impl SimpleMemory {
  #[named]
  pub fn new() -> SimpleMemory {
    SimpleMemory { memory: [0; MAX_ADDRESS] }
  }
}

impl Addressable for SimpleMemory {
  #[named]
  fn read_u8(&self, address: u16) -> u8 {
    self.memory[address as usize]
  }

  #[named]
  fn write_u8(&mut self, address: u16, data: u8) {
    self.memory[address as usize] = data;
  }

  #[named]
  fn load(&mut self, program: Vec<u8>) {
    self.memory[START_ADDRESS..(START_ADDRESS + program.len())].copy_from_slice(&program[..]);
    self.write_u16(PROGRAM_CONTROL_ADDRESS.try_into().unwrap(), START_ADDRESS.try_into().unwrap());
  }
}
