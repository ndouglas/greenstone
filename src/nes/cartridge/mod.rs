use crate::traits::mappable::MirroringMode;
use crate::traits::Mappable;

pub mod mappable;
pub use mappable::*;

pub mod mapper;
pub use mapper::*;

pub struct Cartridge {
  mapper: Box<dyn Mappable>,
}
