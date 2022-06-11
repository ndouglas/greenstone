// A trait for addressable things: buses, memory, etc.
pub trait Addressable {
  fn read_u8(&self, address: u16) -> u8;

  fn write_u8(&mut self, address: u16, data: u8);

  fn read_u16(&self, address: u16) -> u16 {
    let lo = self.read_u8(address) as u16;
    let hi = self.read_u8(address + 1) as u16;
    let result = (hi << 8) | lo;
    result
  }

  fn write_u16(&mut self, address: u16, data: u16) {
    let hi = (data >> 8) as u8;
    let lo = (data & 0xFF) as u8;
    self.write_u8(address, lo);
    self.write_u8(address + 1, hi);
  }

  fn load(&mut self, program: Vec<u8>);
}
