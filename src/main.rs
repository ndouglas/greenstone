#[macro_use]
extern crate log;
extern crate pretty_env_logger;

pub mod nes;

fn main() {
    pretty_env_logger::init();
    trace!("main()");
}
