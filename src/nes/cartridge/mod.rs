use crate::traits::mappable::MirroringMode;
use crate::traits::Mappable;

pub mod mappable;
pub use mappable::*;

pub struct Cartridge<'a> {
  mapper: Box<dyn Mappable + 'a>,
}
