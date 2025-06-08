mod control;
pub use control::ControlRegister;
pub use control::ControlFlags;
pub use control::GENERATE_NMI_FLAG;

mod mask;
pub use mask::MaskRegister;
pub use mask::MaskFlags;

mod status;
pub use status::StatusRegister;
pub use status::StatusFlags;
