use crate::traits::Addressable;
use crate::traits::Interruptible;

pub trait Busable: Addressable + Interruptible {
  /// Set the complete button state for controller 1.
  fn set_controller1(&mut self, _state: u8) {}

  /// Set the complete button state for controller 2.
  fn set_controller2(&mut self, _state: u8) {}

  /// Press a button on controller 1.
  fn press_button1(&mut self, _button: u8) {}

  /// Release a button on controller 1.
  fn release_button1(&mut self, _button: u8) {}

  /// Press a button on controller 2.
  fn press_button2(&mut self, _button: u8) {}

  /// Release a button on controller 2.
  fn release_button2(&mut self, _button: u8) {}
}
