// Common functionality
pub mod common;
pub use common::*;

// ADC: Add to Accumulator With Carry.
pub mod adc;
pub use adc::*;

// AND: Bitwise AND Accumulator With Data.
pub mod and;
pub use and::*;

// ASL: Arithmetic Shift Left
pub mod asl;
pub use asl::*;

// BIT: Test Bits.
pub mod bit;
pub use bit::*;

// BRK: Break
pub mod brk;
pub use brk::*;

// CLC: Clear Carry
pub mod clc;
pub use clc::*;

// CMP: Compare
pub mod cmp;
pub use cmp::*;

// CPX: Compare X
pub mod cpx;
pub use cpx::*;

// CPY: Compare Y
pub mod cpy;
pub use cpy::*;

// DEC: Decrement
pub mod dec;
pub use dec::*;

// EOR: XOR A
pub mod eor;
pub use eor::*;

// INC: Increment
pub mod inc;
pub use inc::*;

// INX: Increment X
pub mod inx;
pub use inx::*;

// LDA: Load A
pub mod lda;
pub use lda::*;

// LDX: Load X
pub mod ldx;
pub use ldx::*;

// LDY: Load Y
pub mod ldy;
pub use ldy::*;

// LSR: Logical Shift Right
pub mod lsr;
pub use lsr::*;

// ORA: OR A
pub mod ora;
pub use ora::*;

// ROL: Rotate Left
pub mod rol;
pub use rol::*;

// ROR: Rotate Right
pub mod ror;
pub use ror::*;

// SBC: Subtract from Accumulator With Carry.
pub mod sbc;
pub use sbc::*;

// SEC: Set the Carry Flag.
pub mod sec;
pub use sec::*;

// STA: Store A.
pub mod sta;
pub use sta::*;

// STX: Store X.
pub mod stx;
pub use stx::*;

// STX: Store Y.
pub mod sty;
pub use sty::*;

// TAX: Transfer A -> X.
pub mod tax;
pub use tax::*;

// TAY: Transfer A -> Y.
pub mod tay;
pub use tay::*;
