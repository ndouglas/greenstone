use crate::traits::Addressable;

pub struct SimpleMemory {
  memory: [u8; 0xFFFF],
}

impl SimpleMemory {
  #[named]
  pub fn new() -> SimpleMemory {
    SimpleMemory { memory: [0; 0xFFFF] }
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
    self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
    self.write_u16(0xFFFC, 0x8000);
  }
}
