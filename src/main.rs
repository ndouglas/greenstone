extern crate iced;
use iced::Application;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

pub mod nes;
pub mod ui;

fn main() {
    pretty_env_logger::init();
    trace!("main()");
    ui::UI::run(iced::Settings::default());
}
