use crate::traits::Addressable;

pub struct Bus {
  memory: [u8; 0xFFFF],
}

impl Bus {
  pub fn new() -> Bus {
    Bus { memory: [0; 0xFFFF] }
  }
}

impl Addressable for Bus {
  #[named]
  fn read_u8(&self, address: u16) -> u8 {
    trace_enter!();
    let result = self.memory[address as usize];
    trace_result!(result);
    result
  }

  #[named]
  fn write_u8(&mut self, address: u16, data: u8) {
    trace_enter!();
    self.memory[address as usize] = data;
    trace_exit!();
  }

  #[named]
  fn load(&mut self, program: Vec<u8>) {
    trace_enter!();
    self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
    self.write_u16(0xFFFC, 0x8000);
    trace_exit!();
  }

  #[named]
  fn tick(&mut self) {}
}
