#![allow(unused_macros)]

#[macro_use]
extern crate derivative;
#[macro_use]
extern crate function_name;
pub use ::function_name::named;
extern crate iced;
use iced::Application;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

#[macro_use]
pub mod macros;
pub use macros::*;
pub mod nes;
pub use nes::*;
pub mod traits;
pub use traits::*;
pub mod ui;
pub use ui::*;

#[cfg(test)]
mod test {
  use super::*;

  #[named]
  pub fn init() {
    let _ = pretty_env_logger::env_logger::builder().is_test(true).try_init();
    std::env::set_var("RUST_BACKTRACE", "1");
  }
}
