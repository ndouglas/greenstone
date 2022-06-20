use crate::traits::Addressable;
use crate::traits::Interruptible;

pub trait Busable: Addressable + Interruptible {}
