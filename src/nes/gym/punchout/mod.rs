//! Reservoir Punch-Out!!: brains that read the screen learn to fight.
//!
//! See `docs/superpowers/specs/2026-10-21-punchout-reservoir-design.md`.

pub mod brains;
pub mod dagger;
pub mod fight;
pub mod gate;
pub mod moves;
pub mod ram;
pub mod start;
pub mod teacher;

pub const PUNCH_OUT: &str = "roms2/mike_tysons_punch_out.nes";
