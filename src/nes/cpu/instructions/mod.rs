// ADC: Add to Accumulator With Carry
pub mod adc;
pub use adc::*;

// BRK: Break
pub mod brk;
pub use brk::*;

// INX: Increment X
pub mod inx;
pub use inx::*;

// LDA: Load A
pub mod lda;
pub use lda::*;

// STA: Store A
pub mod sta;
pub use sta::*;

// TAX: Transfer A -> X
pub mod tax;
pub use tax::*;

// TAY: Transfer A -> Y
pub mod tay;
pub use tay::*;
