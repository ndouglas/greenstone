use super::*;
use crate::traits::Busable;

impl Busable for Bus {
  fn set_controller1(&mut self, state: u8) {
    Bus::set_controller1(self, state);
  }

  fn set_controller2(&mut self, state: u8) {
    Bus::set_controller2(self, state);
  }

  fn press_button1(&mut self, button: u8) {
    Bus::press_button1(self, button);
  }

  fn release_button1(&mut self, button: u8) {
    Bus::release_button1(self, button);
  }

  fn press_button2(&mut self, button: u8) {
    self.controller2_state |= button;
    if self.controller_strobe {
      self.controller2_shift = self.controller2_state;
    }
  }

  fn release_button2(&mut self, button: u8) {
    self.controller2_state &= !button;
    if self.controller_strobe {
      self.controller2_shift = self.controller2_state;
    }
  }

  fn get_ppu_scanline(&self) -> u16 {
    Bus::get_ppu_scanline(self)
  }

  fn get_ppu_dot(&self) -> u16 {
    Bus::get_ppu_dot(self)
  }
}
