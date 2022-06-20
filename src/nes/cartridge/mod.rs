use crate::traits::mappable::MirroringMode;
use crate::traits::Mappable;

pub mod data;
pub use data::*;

pub mod header;
pub use header::*;

pub mod mappable;
pub use mappable::*;

pub mod mapper;
pub use mapper::*;

pub mod pager;
pub use pager::*;

pub struct Cartridge {
  mapper: Box<dyn Mappable>,
}
