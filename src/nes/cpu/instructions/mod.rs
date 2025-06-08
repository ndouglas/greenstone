// Common functionality
pub mod common;
pub use common::*;

// ADC: Add to Accumulator With Carry.
mod adc;

// AND: Bitwise AND Accumulator With Data.
mod and;

// ASL: Arithmetic Shift Left
mod asl;

// BCC: Branch if Carry Clear
mod bcc;

// BCS: Branch if Carry Set
mod bcs;

// BEQ: Branch if Equal
mod beq;

// BIT: Test Bits.
mod bit;

// BMI: Branch if Negative
mod bmi;

// BNE: Branch if Not Equal
mod bne;

// BPL: Branch if Positive
mod bpl;

// BRK: Break
mod brk;

// BVC: Branch if Negative
mod bvc;

// BVS: Branch if Overflow Set
mod bvs;

// CLC: Clear Carry
mod clc;

// CLD: Clear Decimal Mode
mod cld;

// CLI: Clear Interrupt Disable
mod cli;

// CLV: Clear Overflow
mod clv;

// CMP: Compare
mod cmp;

// CPX: Compare X
mod cpx;

// CPY: Compare Y
mod cpy;

// DCP: Decrement and Compare (Unofficial)
mod dcp;

// DEC: Decrement
mod dec;

// DEX: Decrement X
mod dex;

// DEY: Decrement Y
mod dey;

// EOR: XOR A
mod eor;

// INC: Increment
mod inc;

// INX: Increment X
mod inx;

// INY: Increment Y
mod iny;

// ISB: Increment and Subtract from A
mod isb;

// JAM: Die in a fire
mod jam;

// JMP: Jump to Address
mod jmp;

// JSR: Jump to Subroutine
mod jsr;

// LAX: Load A and X
mod lax;

// LDA: Load A
mod lda;

// LDX: Load X
mod ldx;

// LDY: Load Y
mod ldy;

// LSR: Logical Shift Right
mod lsr;

// NOP: No Operation
mod nop;

// ORA: OR A
mod ora;

// PHA: Push A onto the Stack
mod pha;

// PHP: Push Status onto the Stack
mod php;

// PLA: Pop A off the Stack
mod pla;

// PLP: Pop Status off the Stack
mod plp;

// RLA: Rotate Left and AND
mod rla;

// ROL: Rotate Left
mod rol;

// ROR: Rotate Right
mod ror;

// RRA: Rotate Right and ADC
mod rra;

// RTI: Return from Interrupt
mod rti;

// RTS: Return from Subroutine
mod rts;

// SAX: Store A & X to M
mod sax;

// SBC: Subtract from Accumulator With Carry.
mod sbc;

// SEC: Set the Carry Flag.
mod sec;

// SED: Set the Decimal Mode Flag.
mod sed;

// SEI: Set the Interrupt Disable Flag.
mod sei;

// SLO: Arithmetic Shift Left + ORA
mod slo;

// SRE: Logical Shift Right + EOR
mod sre;

// STA: Store A.
mod sta;

// STX: Store X.
mod stx;

// STX: Store Y.
mod sty;

// TAX: Transfer A -> X.
mod tax;

// TAY: Transfer A -> Y.
mod tay;

// TSX: Transfer Stack Pointer -> X.
mod tsx;

// TXA: Transfer X -> A.
mod txa;

// TXS: Transfer X -> Stack Pointer.
mod txs;

// TYA: Transfer Y -> A.
mod tya;

// WTF: Unimplemented/Unknown Instruction.
mod wtf;
