#![allow(unused_macros)]

#[macro_use]
extern crate derivative;
#[macro_use]
extern crate function_name;
use ::function_name::named;
extern crate iced;
use iced::Application;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

#[macro_use]
pub mod macros;
pub mod nes;
pub mod traits;
pub mod ui;

#[named]
fn main() {
  pretty_env_logger::init();
  trace!("main()");
  ui::UI::run(iced::Settings::default());
}

#[cfg(test)]
mod test {
  use super::*;

  #[named]
  pub fn init() {
    let _ = pretty_env_logger::env_logger::builder().is_test(true).try_init();
    std::env::set_var("RUST_BACKTRACE", "1");
  }
}
